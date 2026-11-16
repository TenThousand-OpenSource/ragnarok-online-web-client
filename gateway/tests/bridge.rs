//! The WebSocket-to-TCP bridge, driven the way a browser drives it: a real WebSocket client on one side, a real TCP
//! server on the other.
//!
//! `live_rathena_*` tests need the local rAthena of `server/docker-compose.yml` and are skipped by default:
//! `cargo test -p roweb-gateway -- --ignored`.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use roweb_gateway::bridge::{self, Bridge};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const TIMEOUT: Duration = Duration::from_secs(5);

/// Starts a gateway that bridges to `host` on `ports`, and returns its address.
async fn gateway(host: &str, ports: Vec<u16>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let router = bridge::router(Bridge { host: host.to_owned(), ports, login_guard: None, connections_per_minute: 10_000 });
    serve(listener, router)
}

fn serve(listener: TcpListener, router: axum::Router) -> String {
    let address = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move { axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap() });
    address
}

/// A stand-in game server: a listener on a free port.
async fn game_server() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

async fn open(gateway: &str, port: u16) -> Socket {
    let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{gateway}/ws/{port}")).await.unwrap();
    socket
}

/// The HTTP status a refused handshake came back with.
async fn refusal(request: impl IntoClientRequest + Unpin) -> u16 {
    match tokio_tungstenite::connect_async(request).await {
        Err(tungstenite::Error::Http(response)) => response.status().as_u16(),
        Ok(_) => panic!("the handshake was accepted"),
        Err(error) => panic!("{error}"),
    }
}

/// The next binary message's bytes.
async fn next_bytes(socket: &mut Socket) -> Vec<u8> {
    loop {
        match within(socket.next()).await.expect("the WebSocket ended").unwrap() {
            Message::Binary(bytes) => return bytes.to_vec(),
            Message::Close(frame) => panic!("closed: {frame:?}"),
            _ => {}
        }
    }
}

/// Reads binary messages until `count` bytes have arrived: the bridge may split or join them freely.
async fn read_exactly(socket: &mut Socket, count: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    while bytes.len() < count {
        bytes.extend(next_bytes(socket).await);
    }
    assert_eq!(bytes.len(), count, "more bytes arrived than were expected");
    bytes
}

async fn within<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(TIMEOUT, future).await.expect("timed out")
}

#[tokio::test]
async fn relays_bytes_both_ways() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    browser.send(Message::Binary(vec![0x64, 0x00, 1, 2, 3].into())).await.unwrap();
    let mut received = [0u8; 5];
    within(game.read_exact(&mut received)).await.unwrap();
    assert_eq!(received, [0x64, 0x00, 1, 2, 3]);

    game.write_all(&[0xc4, 0x0a, 9, 8]).await.unwrap();
    assert_eq!(read_exactly(&mut browser, 4).await, [0xc4, 0x0a, 9, 8]);
}

/// The game server speaks first on some connections; nothing may wait for the browser to send.
#[tokio::test]
async fn relays_what_the_server_sends_first() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    game.write_all(b"hello").await.unwrap();
    assert_eq!(read_exactly(&mut browser, 5).await, b"hello");
}

/// One packet split over several WebSocket messages, and several packets in one, reach the server as one stream.
#[tokio::test]
async fn message_boundaries_do_not_matter() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    for part in [&b"ab"[..], b"", b"cde", b"f"] {
        browser.send(Message::Binary(part.to_vec().into())).await.unwrap();
    }
    let mut received = [0u8; 6];
    within(game.read_exact(&mut received)).await.unwrap();
    assert_eq!(&received, b"abcdef");
}

/// Text messages and pings are not game data and never reach the server.
#[tokio::test]
async fn only_binary_messages_are_relayed() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    browser.send(Message::Text("not a packet".into())).await.unwrap();
    browser.send(Message::Ping(vec![1, 2].into())).await.unwrap();
    browser.send(Message::Binary(vec![7].into())).await.unwrap();
    let mut received = [0u8; 1];
    within(game.read_exact(&mut received)).await.unwrap();
    assert_eq!(received, [7]);
}

/// A megabyte each way arrives whole and in order (a map's worth of spawn packets at once).
#[tokio::test]
async fn relays_large_streams_intact() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();
    let data: Vec<u8> = (0..1_000_000u32).map(|index| (index * 31 % 251) as u8).collect();

    for chunk in data.chunks(60_000) {
        browser.send(Message::Binary(chunk.to_vec().into())).await.unwrap();
    }
    let mut received = vec![0u8; data.len()];
    within(game.read_exact(&mut received)).await.unwrap();
    assert!(received == data, "browser to server: the stream was altered");

    let expected = data.clone();
    let writer = tokio::spawn(async move {
        game.write_all(&data).await.unwrap();
        game
    });
    assert!(read_exactly(&mut browser, expected.len()).await == expected, "server to browser: the stream was altered");
    drop(within(writer).await.unwrap());
}

#[tokio::test]
async fn two_connections_stay_separate() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut first = open(&gateway, port).await;
    let (mut first_game, _) = within(server.accept()).await.unwrap();
    let mut second = open(&gateway, port).await;
    let (mut second_game, _) = within(server.accept()).await.unwrap();

    first_game.write_all(b"one").await.unwrap();
    second_game.write_all(b"two").await.unwrap();
    assert_eq!(read_exactly(&mut second, 3).await, b"two");
    assert_eq!(read_exactly(&mut first, 3).await, b"one");
}

#[tokio::test]
async fn refuses_ports_that_are_not_listed() {
    let (_server, port) = game_server().await;
    let (_other, other_port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;

    assert_eq!(refusal(format!("ws://{gateway}/ws/{other_port}")).await, 403);
    assert_eq!(refusal(format!("ws://{gateway}/ws/22")).await, 403);
    // Not a port at all.
    assert_eq!(refusal(format!("ws://{gateway}/ws/login")).await, 400);
}

/// A page of another website, open in the player's browser, must not reach the game server through the gateway.
#[tokio::test]
async fn refuses_other_origins() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let request = |origin: &str| {
        let mut request = format!("ws://{gateway}/ws/{port}").into_client_request().unwrap();
        request.headers_mut().insert("Origin", origin.parse().unwrap());
        request
    };

    assert_eq!(refusal(request("http://evil.example")).await, 403);
    assert_eq!(refusal(request("null")).await, 403);

    let (mut browser, _) = tokio_tungstenite::connect_async(request(&format!("http://{gateway}"))).await.unwrap();
    let (mut game, _) = within(server.accept()).await.unwrap();
    game.write_all(b"ok").await.unwrap();
    assert_eq!(read_exactly(&mut browser, 2).await, b"ok");
}

#[tokio::test]
async fn closes_the_websocket_when_the_server_disconnects() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    // What was sent before the disconnect still arrives, then the close.
    game.write_all(b"bye").await.unwrap();
    drop(game);
    assert_eq!(read_exactly(&mut browser, 3).await, b"bye");
    assert!(matches!(within(browser.next()).await, Some(Ok(Message::Close(_)))));
}

#[tokio::test]
async fn disconnects_from_the_server_when_the_browser_leaves() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    browser.send(Message::Binary(b"last".to_vec().into())).await.unwrap();
    browser.close(None).await.unwrap();
    let mut received = Vec::new();
    within(game.read_to_end(&mut received)).await.unwrap();
    assert_eq!(received, b"last");
}

/// A tab that is closed or loses its network sends no close frame at all.
#[tokio::test]
async fn disconnects_from_the_server_when_the_browser_vanishes() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    drop(browser);
    let mut received = Vec::new();
    within(game.read_to_end(&mut received)).await.unwrap();
    assert!(received.is_empty());
}

#[tokio::test]
async fn reports_an_unreachable_server() {
    // A port nothing listens on: taken, then released.
    let (server, port) = game_server().await;
    drop(server);
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;

    match within(browser.next()).await {
        Some(Ok(Message::Close(Some(frame)))) => assert_eq!(frame.code, CloseCode::Error),
        other => panic!("expected a close frame, got {other:?}"),
    }
}

/// Messages over the size limit end the connection instead of being buffered.
#[tokio::test]
async fn refuses_oversized_messages() {
    let (server, port) = game_server().await;
    let gateway = gateway("127.0.0.1", vec![port]).await;
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    browser.send(Message::Binary(vec![0u8; 2 * 1024 * 1024].into())).await.unwrap();
    let mut received = Vec::new();
    within(game.read_to_end(&mut received)).await.unwrap();
    assert!(received.is_empty(), "{} bytes of an oversized message reached the server", received.len());
}

/// The game servers see one address, the gateway's, so it is the gateway that stops a flood of connections.
#[tokio::test]
async fn limits_new_connections_per_address() {
    let (server, port) = game_server().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway = serve(listener, bridge::router(Bridge { host: "127.0.0.1".to_owned(), ports: vec![port], login_guard: None, connections_per_minute: 3 }));

    let mut open_sockets = Vec::new();
    for _ in 0..3 {
        open_sockets.push(open(&gateway, port).await);
        within(server.accept()).await.unwrap();
    }
    assert_eq!(refusal(format!("ws://{gateway}/ws/{port}")).await, 429);
    // The connections made before the limit go on working.
    open_sockets[0].send(Message::Binary(vec![1].into())).await.unwrap();
}

// ---------------------------------------------------------------------------------------------------------------
// The login guard, against a stand-in login server.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use roweb_gateway::bridge::{LoginGuard, TOO_MANY_ATTEMPTS};

#[derive(Debug, PartialEq)]
enum Login {
    Accepted,
    /// The login server's error code.
    Refused(u32),
    /// The gateway did not let the attempt through; seconds to wait.
    TurnedAway(u64),
}

/// A login server that accepts the password "right", bans the account "banned" (error 4), does not know "nobody"
/// (error 0) and calls everything else a wrong password (error 1). Counts the login requests it gets.
async fn login_server() -> (u16, Arc<AtomicUsize>) {
    let (listener, port) = game_server().await;
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = requests.clone();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut request = [0u8; 55];
                if socket.read_exact(&mut request).await.is_err() {
                    return;
                }
                counter.fetch_add(1, Ordering::SeqCst);
                let field = |at: usize| String::from_utf8_lossy(&request[at..at + 24]).trim_end_matches('\0').to_owned();
                let (user, password) = (field(6), field(30));
                let reply = if password == "right" {
                    vec![0xc4, 0x0a, 4, 0]
                } else {
                    let error: u32 = match user.as_str() {
                        "banned" => 4,
                        "nobody" => 0,
                        _ => 1,
                    };
                    let mut refusal = vec![0u8; 26];
                    refusal[..2].copy_from_slice(&[0x3e, 0x08]);
                    refusal[2..6].copy_from_slice(&error.to_le_bytes());
                    refusal
                };
                let _ = socket.write_all(&reply).await;
                // As the real server: the client hangs up.
                let _ = socket.read(&mut request).await;
            });
        }
    });
    (port, requests)
}

async fn guarded_gateway(port: u16, per_address: usize, per_account: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let guard = LoginGuard::with_limits(port, per_address, per_account, Duration::from_secs(600));
    serve(listener, bridge::router(Bridge { host: "127.0.0.1".to_owned(), ports: vec![port], login_guard: Some(Arc::new(guard)), connections_per_minute: 10_000 }))
}

/// One login attempt, with the request sent in pieces of `piece` bytes.
async fn attempt_in_pieces(gateway: &str, port: u16, user: &str, password: &str, piece: usize) -> Login {
    let mut browser = open(gateway, port).await;
    for part in login_request(user, password).chunks(piece) {
        // The gateway may already have closed the socket on an address that is out of attempts.
        if browser.send(Message::Binary(part.to_vec().into())).await.is_err() {
            break;
        }
    }
    let mut reply = Vec::new();
    loop {
        match within(browser.next()).await {
            Some(Ok(Message::Binary(bytes))) => {
                reply.extend(bytes);
                if reply.starts_with(&[0xc4, 0x0a]) {
                    return Login::Accepted;
                }
                if reply.len() >= 6 && reply.starts_with(&[0x3e, 0x08]) {
                    return Login::Refused(u32_at(&reply, 2));
                }
            }
            Some(Ok(Message::Close(Some(frame)))) if u16::from(frame.code) == TOO_MANY_ATTEMPTS => {
                return Login::TurnedAway(frame.reason.as_str().parse().expect("the reason is the seconds to wait"));
            }
            other => panic!("unexpected end of a login attempt: {other:?}"),
        }
    }
}

async fn attempt(gateway: &str, port: u16, user: &str, password: &str) -> Login {
    attempt_in_pieces(gateway, port, user, password, 55).await
}

#[tokio::test]
async fn guard_turns_an_address_away_after_too_many_failures() {
    let (port, requests) = login_server().await;
    let gateway = guarded_gateway(port, 3, 100).await;

    // Different accounts each time: it is the address that is guessing.
    for user in ["alice", "bob", "nobody"] {
        assert!(matches!(attempt(&gateway, port, user, "guess").await, Login::Refused(0 | 1)));
    }
    // Out of attempts: not even the right password gets through, and the login server is not asked at all.
    match attempt(&gateway, port, "alice", "right").await {
        Login::TurnedAway(seconds) => assert!((590..=600).contains(&seconds), "{seconds} seconds"),
        other => panic!("{other:?}"),
    }
    assert_eq!(requests.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn guard_gives_attempts_back_after_a_success() {
    let (port, _) = login_server().await;
    let gateway = guarded_gateway(port, 3, 100).await;

    for _ in 0..2 {
        assert_eq!(attempt(&gateway, port, "alice", "typo").await, Login::Refused(1));
    }
    assert_eq!(attempt(&gateway, port, "alice", "right").await, Login::Accepted);
    // A full set of attempts again.
    for _ in 0..3 {
        assert_eq!(attempt(&gateway, port, "alice", "typo").await, Login::Refused(1));
    }
    assert!(matches!(attempt(&gateway, port, "alice", "right").await, Login::TurnedAway(_)));
}

/// Guessing one account's password from many addresses: here the address limit is out of reach, the account's is not.
#[tokio::test]
async fn guard_locks_an_account_after_too_many_failures() {
    let (port, requests) = login_server().await;
    let gateway = guarded_gateway(port, 100, 2).await;

    assert_eq!(attempt(&gateway, port, "victim", "guess1").await, Login::Refused(1));
    // The login server compares names without case, and so does the guard.
    assert_eq!(attempt(&gateway, port, "VICTIM", "guess2").await, Login::Refused(1));
    assert!(matches!(attempt(&gateway, port, "Victim", "right").await, Login::TurnedAway(_)));
    assert_eq!(requests.load(Ordering::SeqCst), 2, "the request for a locked account reached the login server");

    // Other accounts are not affected.
    assert_eq!(attempt(&gateway, port, "other", "right").await, Login::Accepted);
}

/// The account lock holds however the request is cut into WebSocket messages.
#[tokio::test]
async fn guard_reads_a_login_request_that_comes_in_pieces() {
    let (port, requests) = login_server().await;
    let gateway = guarded_gateway(port, 100, 1).await;

    assert_eq!(attempt_in_pieces(&gateway, port, "victim", "guess", 1).await, Login::Refused(1));
    assert!(matches!(attempt_in_pieces(&gateway, port, "victim", "right", 7).await, Login::TurnedAway(_)));
    assert_eq!(attempt_in_pieces(&gateway, port, "other", "right", 20).await, Login::Accepted);
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

/// A banned account's refusals are not password guesses.
#[tokio::test]
async fn guard_counts_only_failed_guesses() {
    let (port, _) = login_server().await;
    let gateway = guarded_gateway(port, 2, 2).await;

    for _ in 0..5 {
        assert_eq!(attempt(&gateway, port, "banned", "whatever").await, Login::Refused(4));
    }
    assert_eq!(attempt(&gateway, port, "alice", "right").await, Login::Accepted);
}

/// The guard watches the login port only: the other servers' streams pass untouched.
#[tokio::test]
async fn guard_leaves_other_ports_alone() {
    let (server, port) = game_server().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let guard = LoginGuard::with_limits(1, 1, 1, Duration::from_secs(600));
    let gateway = serve(listener, bridge::router(Bridge { host: "127.0.0.1".to_owned(), ports: vec![port], login_guard: Some(Arc::new(guard)), connections_per_minute: 10_000 }));
    let mut browser = open(&gateway, port).await;
    let (mut game, _) = within(server.accept()).await.unwrap();

    // Bytes that would read as refused logins on the login port, and a first message too short to be judged there.
    browser.send(Message::Binary(vec![0x64].into())).await.unwrap();
    let mut received = [0u8; 1];
    within(game.read_exact(&mut received)).await.unwrap();
    for _ in 0..3 {
        game.write_all(&[0x3e, 0x08, 1, 0, 0, 0]).await.unwrap();
        assert_eq!(read_exactly(&mut browser, 6).await, [0x3e, 0x08, 1, 0, 0, 0]);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Against the real rAthena of server/docker-compose.yml (PACKETVER 20260219), with the second seeded account so a
// player logged in as the first one is not disturbed (a login kicks whoever is online on the same account).

const LOGIN_PORT: u16 = 6900;
const CHAR_PORT: u16 = 6121;
const MAP_PORT: u16 = 5121;
const ACCOUNT_ID: u32 = 2_000_001;

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// Reads one variable-length packet (`u16 id, u16 length, ...`) whole.
async fn read_packet(socket: &mut Socket) -> Vec<u8> {
    let mut packet = next_bytes(socket).await;
    while packet.len() < 4 || packet.len() < usize::from(u16_at(&packet, 2)) {
        packet.extend(next_bytes(socket).await);
    }
    packet
}

/// CA_LOGIN (0x0064): `u16 id, u32 version, char[24] user, char[24] password, u8 clienttype`.
fn login_request(user: &str, password: &str) -> Vec<u8> {
    let mut packet = vec![0u8; 55];
    packet[..2].copy_from_slice(&0x0064u16.to_le_bytes());
    packet[2..6].copy_from_slice(&55u32.to_le_bytes());
    packet[6..6 + user.len()].copy_from_slice(user.as_bytes());
    packet[30..30 + password.len()].copy_from_slice(password.as_bytes());
    packet
}

#[tokio::test]
#[ignore = "needs the local rAthena"]
async fn live_rathena_refuses_a_wrong_password() {
    let gateway = gateway("127.0.0.1", vec![LOGIN_PORT]).await;
    let mut login = open(&gateway, LOGIN_PORT).await;
    login.send(Message::Binary(login_request("tester2", "wrong-password").into())).await.unwrap();

    // AC_REFUSE_LOGIN (0x083e): `u16 id, u32 error, char[20] unblock_time`; error 1 is "incorrect password".
    let reply = read_exactly(&mut login, 26).await;
    assert_eq!(u16_at(&reply, 0), 0x083e, "{reply:02x?}");
    assert_eq!(u32_at(&reply, 2), 1);
}

/// The whole way in, as the client goes: login server, then the char server it names, over two bridged sockets.
#[tokio::test]
#[ignore = "needs the local rAthena"]
async fn live_rathena_login_and_char_server() {
    let gateway = gateway("127.0.0.1", vec![LOGIN_PORT, CHAR_PORT, MAP_PORT]).await;

    let mut login = open(&gateway, LOGIN_PORT).await;
    login.send(Message::Binary(login_request("tester2", "tester2").into())).await.unwrap();
    // AC_ACCEPT_LOGIN (0x0ac4): `u16 id, u16 length, u32 login_id1, u32 AID, u32 login_id2, u32 last_ip,
    // char[26] last_login, u8 sex, char[17] token`, then 160 bytes per char server (`u32 ip, u16 port, char[20] name, ...`).
    let accept = read_packet(&mut login).await;
    assert_eq!(u16_at(&accept, 0), 0x0ac4, "{:02x?}", &accept[..accept.len().min(32)]);
    assert_eq!(accept.len(), usize::from(u16_at(&accept, 2)));
    let (login_id1, account_id, login_id2, sex) = (u32_at(&accept, 4), u32_at(&accept, 8), u32_at(&accept, 12), accept[46]);
    assert_eq!(account_id, ACCOUNT_ID);
    let servers = &accept[64..];
    assert_eq!(servers.len() % 160, 0, "char server list of {} bytes", servers.len());
    assert!(!servers.is_empty(), "the login server lists no char server (is char-server connected?)");
    assert_eq!(servers[..4], [127, 0, 0, 1]);
    let char_port = u16_at(servers, 4);
    assert_eq!(char_port, CHAR_PORT);
    drop(login);

    // CH_ENTER (0x0065): `u16 id, u32 AID, u32 login_id1, u32 login_id2, u16 unused, u8 sex`.
    let mut char_server = open(&gateway, char_port).await;
    let mut enter = vec![0u8; 17];
    enter[..2].copy_from_slice(&0x0065u16.to_le_bytes());
    enter[2..6].copy_from_slice(&account_id.to_le_bytes());
    enter[6..10].copy_from_slice(&login_id1.to_le_bytes());
    enter[10..14].copy_from_slice(&login_id2.to_le_bytes());
    enter[16] = sex;
    char_server.send(Message::Binary(enter.into())).await.unwrap();

    // The char server answers with the bare account ID, then its packets (slot info first: HC_ACCEPT_ENTER2, 0x082d).
    let mut reply = Vec::new();
    while reply.len() < 6 {
        reply.extend(next_bytes(&mut char_server).await);
    }
    assert_eq!(u32_at(&reply, 0), ACCOUNT_ID, "{:02x?}", &reply[..reply.len().min(32)]);
    assert_eq!(u16_at(&reply, 4), 0x082d, "{:02x?}", &reply[..reply.len().min(32)]);
}
