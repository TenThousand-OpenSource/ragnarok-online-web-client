//! What the tests of the bridge share (`bridge.rs`, `login_guard.rs`, `live_rathena.rs`): a gateway started on a
//! free port, a stand-in game server, and a WebSocket opened and read as a browser does.
// (Each test file is a program of its own and uses a part of this.)
#![allow(dead_code)]

use std::time::Duration;

use futures_util::StreamExt;
use roweb_gateway::bridge::{self, Bridge};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub const TIMEOUT: Duration = Duration::from_secs(5);

/// Starts a gateway that bridges to `host` on `ports`, and returns its address.
pub async fn gateway(host: &str, ports: Vec<u16>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let router = bridge::router(Bridge { host: host.to_owned(), ports, login_guard: None, connections_per_minute: 10_000, lag: Default::default() });
    serve(listener, router)
}

pub fn serve(listener: TcpListener, router: axum::Router) -> String {
    let address = listener.local_addr().unwrap().to_string();
    tokio::spawn(async move { axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap() });
    address
}

/// A stand-in game server: a listener on a free port.
pub async fn game_server() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

pub async fn open(gateway: &str, port: u16) -> Socket {
    let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{gateway}/ws/{port}")).await.unwrap();
    socket
}

/// The HTTP status a refused handshake came back with.
pub async fn refusal(request: impl IntoClientRequest + Unpin) -> u16 {
    match tokio_tungstenite::connect_async(request).await {
        Err(tungstenite::Error::Http(response)) => response.status().as_u16(),
        Ok(_) => panic!("the handshake was accepted"),
        Err(error) => panic!("{error}"),
    }
}

/// The next binary message's bytes.
pub async fn next_bytes(socket: &mut Socket) -> Vec<u8> {
    loop {
        match within(socket.next()).await.expect("the WebSocket ended").unwrap() {
            Message::Binary(bytes) => return bytes.to_vec(),
            Message::Close(frame) => panic!("closed: {frame:?}"),
            _ => {}
        }
    }
}

/// Reads binary messages until `count` bytes have arrived: the bridge may split or join them freely.
pub async fn read_exactly(socket: &mut Socket, count: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    while bytes.len() < count {
        bytes.extend(next_bytes(socket).await);
    }
    assert_eq!(bytes.len(), count, "more bytes arrived than were expected");
    bytes
}

pub async fn within<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(TIMEOUT, future).await.expect("timed out")
}

pub fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

pub fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// CA_LOGIN (0x0064): `u16 id, u32 version, char[24] user, char[24] password, u8 clienttype`.
pub fn login_request(user: &str, password: &str) -> Vec<u8> {
    let mut packet = vec![0u8; 55];
    packet[..2].copy_from_slice(&0x0064u16.to_le_bytes());
    packet[2..6].copy_from_slice(&55u32.to_le_bytes());
    packet[6..6 + user.len()].copy_from_slice(user.as_bytes());
    packet[30..30 + password.len()].copy_from_slice(password.as_bytes());
    packet
}
