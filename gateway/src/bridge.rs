//! `GET /ws/<port>`: a WebSocket bridged to a TCP connection to the game server, because browsers cannot open raw
//! TCP sockets.
//!
//! The game protocol is a byte stream, so WebSocket message boundaries carry no meaning: every binary message is
//! written to the socket as it comes, and whatever the socket yields is sent on as one binary message.
//!
//! Only the configured host and ports are reachable, and only pages served from the gateway's own origin may
//! connect, so this is neither an open proxy nor reachable from other websites open in the player's browser.
//!
//! For looking at the game as a player far away sees it, the bridge can hold back what goes each way for a
//! while (`Lag`): a debug option, off unless asked for, and never for a gateway others play through.
//!
//! On the login server's port the bridge also counts failed logins (`LoginGuard`): the game server sees every
//! player coming from the gateway's address, so only the gateway can tell who is guessing passwords.
//!
//! Here: the route, and the copying of bytes both ways. The guard is in `bridge/guard.rs`, the delay in
//! `bridge/lag.rs`.

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
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

mod guard;
mod lag;

use guard::{CA_LOGIN, Watch, judge, login_name, turn_away};
pub use guard::{LoginGuard, TOO_MANY_ATTEMPTS};
pub use lag::Lag;

/// The largest WebSocket message taken from a browser. A game packet is at most 64 KiB.
const MAX_MESSAGE: usize = 256 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// How many pieces may wait to be passed on, each way, before the side they come from is made to wait.
const WAITING: usize = 1024;

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
    /// The debug delay (`--lag`, `--jitter`); none unless asked for.
    pub lag: Lag,
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
    let lag = bridge.lag;
    upgrade.max_message_size(MAX_MESSAGE).on_upgrade(move |mut socket| async move {
        if let Some(wait) = watch.as_ref().and_then(|watch| watch.guard.by_address.blocked_for(&watch.address, Instant::now())) {
            return turn_away(&mut socket, wait).await;
        }
        match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(&target)).await {
            Ok(Ok(tcp)) => {
                // Game packets are small and time-critical: never wait to batch them.
                let _ = tcp.set_nodelay(true);
                let (from_player, to_player) = pump(socket, tcp, watch, lag).await;
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

/// What waits to be passed on one way: each piece and when it is due.
type Waiting = tokio::sync::mpsc::Sender<(Instant, Vec<u8>)>;

/// Reads what the browser sends and leaves it to be written to the server when it is due. With a `watch`, a
/// login request for an account that is out of attempts is not passed on: it ends with how long the account
/// stays blocked. Ends with nothing when the browser closes or the writing has ended.
async fn from_browser(mut browser: futures_util::stream::SplitStream<WebSocket>, for_server: Waiting, watch: Option<&Watch>, lag: Lag, count: &AtomicUsize) -> Option<Duration> {
    let mut last = None;
    // The start of the stream, held back until it is known whether it may pass.
    let mut held = watch.map(|_| Vec::new());
    while let Some(Ok(message)) = browser.next().await {
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
                count.fetch_add(passed.len(), Ordering::Relaxed);
                let due = lag.due_now(last);
                last = Some(due);
                if for_server.send((due, passed)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            // The protocol is binary; text is ignored, and pings are answered by the WebSocket layer.
            _ => {}
        }
    }
    None
}

/// Reads what the server sends and leaves it to be sent to the browser when it is due. With a `watch`, the
/// server's answer to the login is counted. Ends when the server closes or the sending has ended.
async fn from_server(mut server: tokio::net::tcp::OwnedReadHalf, for_browser: Waiting, watch: Option<&Watch>, lag: Lag, count: &AtomicUsize) {
    let mut last = None;
    let mut buffer = vec![0u8; 16 * 1024];
    // The start of the server's answer, until it has been judged.
    let mut answer = watch.map(|_| Vec::new());
    loop {
        match server.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                if let (Some(start), Some(watch)) = (&mut answer, watch) {
                    start.extend_from_slice(&buffer[..read]);
                    if judge(watch, start) {
                        answer = None;
                    }
                }
                count.fetch_add(read, Ordering::Relaxed);
                let due = lag.due_now(last);
                last = Some(due);
                if for_browser.send((due, buffer[..read].to_vec())).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// Copies bytes both ways until either side closes, then closes the other. What is read one way (`from_browser`,
/// `from_server`) waits its turn to be written until it is due (`Lag`: at once, unless a delay was asked for).
/// Gives back how many bytes went each way, from the player and to the player.
async fn pump(socket: WebSocket, tcp: TcpStream, watch: Option<Watch>, lag: Lag) -> (usize, usize) {
    let (from_player, to_player) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let (mut ws_tx, ws_rx) = socket.split();
    let (tcp_rx, mut tcp_tx) = tcp.into_split();
    let watch = watch.as_ref();
    let (for_server, mut to_write) = tokio::sync::mpsc::channel::<(Instant, Vec<u8>)>(WAITING);
    let (for_browser, mut to_send) = tokio::sync::mpsc::channel::<(Instant, Vec<u8>)>(WAITING);

    let to_server = async {
        while let Some((due, bytes)) = to_write.recv().await {
            tokio::time::sleep_until(tokio::time::Instant::from_std(due)).await;
            if tcp_tx.write_all(&bytes).await.is_err() {
                break;
            }
        }
        // Nothing more is taken: the reading ends when it next has something.
        to_write.close();
    };
    let to_browser = async {
        while let Some((due, bytes)) = to_send.recv().await {
            tokio::time::sleep_until(tokio::time::Instant::from_std(due)).await;
            if ws_tx.send(Message::Binary(bytes.into())).await.is_err() {
                return to_send.close();
            }
        }
        let _ = ws_tx.send(Message::Close(None)).await;
    };
    // A direction is over when its reading has ended and what it read is written, or when its writing failed;
    // a login turned away leaves what waits unwritten. The reading lets go of its end of the queue as it ends.
    let server_way = async {
        tokio::pin!(to_server);
        tokio::select! {
            blocked = from_browser(ws_rx, for_server, watch, lag, &from_player) => {
                if blocked.is_none() {
                    to_server.await;
                }
                blocked
            }
            () = &mut to_server => None,
        }
    };
    let browser_way = async {
        tokio::pin!(to_browser);
        tokio::select! {
            () = from_server(tcp_rx, for_browser, watch, lag, &to_player) => to_browser.await,
            () = &mut to_browser => {}
        }
    };
    // Whichever direction ends first ends the bridge: dropping the halves closes both connections.
    let blocked = tokio::select! {
        blocked = server_way => blocked,
        () = browser_way => None,
    };
    if let Some(wait) = blocked {
        turn_away(&mut ws_tx, wait).await;
    }
    (from_player.into_inner(), to_player.into_inner())
}
