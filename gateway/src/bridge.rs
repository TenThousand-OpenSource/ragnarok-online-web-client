//! `GET /ws/<port>`: a WebSocket bridged to a TCP connection to the game server, because browsers cannot open raw
//! TCP sockets.
//!
//! The game protocol is a byte stream, so WebSocket message boundaries carry no meaning: every binary message is
//! written to the socket as it comes, and whatever the socket yields is sent on as one binary message.
//!
//! Only the configured host and ports are reachable, and only pages served from the gateway's own origin may
//! connect, so this is neither an open proxy nor reachable from other websites open in the player's browser.
//!
//! On the login server's port the bridge also counts failed logins (`LoginGuard`): the game server sees every
//! player coming from the gateway's address, so only the gateway can tell who is guessing passwords.

use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::limiter::Limiter;

/// The largest WebSocket message taken from a browser. A game packet is at most 64 KiB.
const MAX_MESSAGE: usize = 256 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// The WebSocket close code for "too many failed logins"; the reason is how many seconds to wait.
pub const TOO_MANY_ATTEMPTS: u16 = 4029;

const CA_LOGIN: [u8; 2] = [0x64, 0x00];
const AC_ACCEPT_LOGIN: [u8; 2] = [0xc4, 0x0a];
const AC_REFUSE_LOGIN: [u8; 2] = [0x3e, 0x08];

/// Stops password guessing on the login server's port: failed logins are counted per address and per account
/// name, and whoever runs out of attempts is turned away until the oldest ones are old enough.
pub struct LoginGuard {
    pub port: u16,
    by_address: Limiter,
    by_account: Limiter,
}

impl LoginGuard {
    /// Ten failures per address and thirty per account name in ten minutes. An account's limit is the wider one:
    /// it only has to stop guessing spread over many addresses, and a tight one would let anyone lock a player
    /// out by failing on purpose.
    pub fn new(port: u16) -> Self {
        Self::with_limits(port, 10, 30, Duration::from_secs(600))
    }

    pub fn with_limits(port: u16, per_address: usize, per_account: usize, window: Duration) -> Self {
        Self { port, by_address: Limiter::new(per_address, window), by_account: Limiter::new(per_account, window) }
    }

    /// Gives an account its attempts back: its password was just reset by its owner.
    pub fn forgive_account(&self, name: &str) {
        self.by_account.clear(&name.to_ascii_lowercase());
    }
}

#[derive(Clone)]
pub struct Bridge {
    /// The game server's host, as the gateway reaches it.
    pub host: String,
    /// The login, char and map ports. Nothing else is reachable.
    pub ports: Vec<u16>,
    pub login_guard: Option<Arc<LoginGuard>>,
    /// How many connections one address may open per minute. The game servers cannot limit this themselves: they
    /// see every player coming from the gateway's address. Playing takes three (login, char, map) and one more
    /// for each change of map server.
    pub connections_per_minute: usize,
}

struct Shared {
    bridge: Bridge,
    connections: Limiter,
}

/// The router must be served with `into_make_service_with_connect_info::<SocketAddr>()`: the guard needs the
/// player's address.
pub fn router(bridge: Bridge) -> Router {
    let connections = Limiter::new(bridge.connections_per_minute, Duration::from_secs(60));
    Router::new().route("/ws/{port}", get(connect)).with_state(Arc::new(Shared { bridge, connections }))
}

/// One login connection as the guard sees it.
struct Watch {
    guard: Arc<LoginGuard>,
    address: String,
    /// The account name of the login request, once it has passed.
    account: Mutex<Option<String>>,
}

async fn connect(
    State(shared): State<Arc<Shared>>,
    Path(port): Path<u16>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let bridge = &shared.bridge;
    // Who was let through to the game server and who was not is said, a line each: on a server others reach,
    // that is how a player who cannot get in is told from one who never arrived.
    if !bridge.ports.contains(&port) || !same_origin(&headers) {
        let said = |name: header::HeaderName| headers.get(name).and_then(|value| value.to_str().ok()).unwrap_or("-").to_owned();
        eprintln!("bridge: {} refused on port {port}: origin {} is not host {}", peer.ip(), said(header::ORIGIN), said(header::HOST));
        return StatusCode::FORBIDDEN.into_response();
    }
    if shared.connections.attempt(&address_key(peer.ip()), Instant::now()).is_err() {
        eprintln!("bridge: {} refused on port {port}: too many connections", peer.ip());
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    eprintln!("bridge: {} connects to port {port}", peer.ip());
    let watch = bridge.login_guard.clone().filter(|guard| guard.port == port).map(|guard| Watch { guard, address: address_key(peer.ip()), account: Mutex::new(None) });
    let target = format!("{}:{port}", bridge.host);
    upgrade.max_message_size(MAX_MESSAGE).on_upgrade(move |mut socket| async move {
        if let Some(wait) = watch.as_ref().and_then(|watch| watch.guard.by_address.blocked_for(&watch.address, Instant::now())) {
            return turn_away(&mut socket, wait).await;
        }
        match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await {
            Ok(Ok(tcp)) => {
                // Game packets are small and time-critical: never wait to batch them.
                let _ = tcp.set_nodelay(true);
                let (from_player, to_player) = pump(socket, tcp, watch).await;
                eprintln!("bridge: {} leaves port {port}: {from_player} bytes from the player, {to_player} to the player", peer.ip());
            }
            failed => {
                let reason = match failed {
                    Ok(Err(error)) => error.to_string(),
                    _ => "timed out".to_owned(),
                };
                eprintln!("bridge: {target}: {reason}");
                let frame = CloseFrame { code: close_code::ERROR, reason: "game server unreachable".into() };
                let _ = socket.send(Message::Close(Some(frame))).await;
            }
        }
    })
}

/// Browsers always send `Origin` on a WebSocket handshake. It must name the host the request was sent to; other
/// programs (tests, tools) send none and are let through.
pub(crate) fn same_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN) else { return true };
    let (Ok(origin), Some(Ok(host))) = (origin.to_str(), headers.get(header::HOST).map(|host| host.to_str())) else { return false };
    origin.split_once("://").is_some_and(|(_, origin_host)| origin_host.eq_ignore_ascii_case(host))
}

/// An IPv4 address as it is, also when it arrives mapped into IPv6.
pub(crate) fn address_key(address: IpAddr) -> String {
    address.to_canonical().to_string()
}

async fn turn_away<S>(socket: &mut S, wait: Duration)
where
    S: SinkExt<Message> + Unpin,
{
    let frame = CloseFrame { code: TOO_MANY_ATTEMPTS, reason: wait.as_secs().max(1).to_string().into() };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

/// The account name in a login request (CA_LOGIN: `u16 id, u32 version, char[24] name, ...`), lowercased as the
/// login server compares it.
fn login_name(request: &[u8]) -> String {
    let name = &request[6..30];
    let end = name.iter().position(|&byte| byte == 0).unwrap_or(name.len());
    String::from_utf8_lossy(&name[..end]).to_ascii_lowercase()
}

/// Copies bytes both ways until either side closes, then closes the other. With a `watch`, a login request for an
/// account that is out of attempts is not passed on, and the server's answer is counted. Gives back how many
/// bytes went each way, from the player and to the player.
async fn pump(socket: WebSocket, tcp: TcpStream, watch: Option<Watch>) -> (usize, usize) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (from_player, to_player) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (mut tcp_rx, mut tcp_tx) = tcp.into_split();
    let watch = watch.as_ref();

    // Ends with how long the account stays blocked, if that is why it ended.
    let to_server = async {
        // The start of the stream, held back until it is known whether it may pass.
        let mut held = watch.map(|_| Vec::new());
        while let Some(Ok(message)) = ws_rx.next().await {
            match message {
                Message::Binary(bytes) => {
                    let passed = match (&mut held, watch) {
                        (Some(start), Some(watch)) => {
                            start.extend_from_slice(&bytes);
                            let is_login = start.len() < 2 || start[..2] == CA_LOGIN;
                            if is_login && start.len() < 30 {
                                continue;
                            }
                            if is_login {
                                let name = login_name(start);
                                if let Some(wait) = watch.guard.by_account.blocked_for(&name, Instant::now()) {
                                    return Some(wait);
                                }
                                *watch.account.lock().unwrap() = Some(name);
                            }
                            held.take().unwrap()
                        }
                        _ => bytes.to_vec(),
                    };
                    from_player.fetch_add(passed.len(), Ordering::Relaxed);
                    if tcp_tx.write_all(&passed).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => break,
                // The protocol is binary; text is ignored, and pings are answered by the WebSocket layer.
                _ => {}
            }
        }
        None
    };
    let to_browser = async {
        let mut buffer = vec![0u8; 16 * 1024];
        // The start of the server's answer, until it has been judged.
        let mut answer = watch.map(|_| Vec::new());
        loop {
            match tcp_rx.read(&mut buffer).await {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    if let (Some(start), Some(watch)) = (&mut answer, watch) {
                        start.extend_from_slice(&buffer[..read]);
                        if judge(watch, start) {
                            answer = None;
                        }
                    }
                    to_player.fetch_add(read, Ordering::Relaxed);
                    if ws_tx.send(Message::Binary(buffer[..read].to_vec().into())).await.is_err() {
                        return;
                    }
                }
            }
        }
        let _ = ws_tx.send(Message::Close(None)).await;
    };
    // Whichever direction ends first ends the bridge: dropping the halves closes both connections.
    let blocked = tokio::select! {
        blocked = to_server => blocked,
        () = to_browser => None,
    };
    if let Some(wait) = blocked {
        turn_away(&mut ws_tx, wait).await;
    }
    (from_player.into_inner(), to_player.into_inner())
}

/// Counts the login server's answer once enough of it has arrived. Returns whether it is done with it.
fn judge(watch: &Watch, answer: &[u8]) -> bool {
    if answer.len() < 2 {
        return false;
    }
    let account = watch.account.lock().unwrap().clone();
    let now = Instant::now();
    if answer[..2] == AC_ACCEPT_LOGIN {
        watch.guard.by_address.clear(&watch.address);
        if let Some(account) = &account {
            watch.guard.by_account.clear(account);
        }
    } else if answer[..2] == AC_REFUSE_LOGIN {
        // `u32 error`: 0 is an unknown account and 1 a wrong password. Other refusals (a ban, an expired account)
        // are not guesses.
        if answer.len() < 6 {
            return false;
        }
        if answer[2..6] == [0, 0, 0, 0] || answer[2..6] == [1, 0, 0, 0] {
            watch.guard.by_address.hit(&watch.address, now);
            if let Some(account) = &account {
                watch.guard.by_account.hit(account, now);
            }
        }
    }
    true
}
