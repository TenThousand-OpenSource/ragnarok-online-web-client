//! The WebSocket-to-TCP bridge, driven the way a browser drives it: a real WebSocket client on one side, a real TCP
//! server on the other. What it relays, what it refuses, and how each side learns that the other left. The login
//! guard is in `login_guard.rs`, the tests against a real rAthena in `live_rathena.rs`; what they share, in
//! `common/mod.rs`.

mod common;

use common::*;
use futures_util::{SinkExt, StreamExt};
use roweb_gateway::bridge::{self, Bridge};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::Message;

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
    let gateway = serve(listener, bridge::router(Bridge {
        host: "127.0.0.1".to_owned(),
        ports: vec![port],
        login_guard: None,
        connections_per_minute: 3,
        lag: Default::default()
    }));

    let mut open_sockets = Vec::new();
    for _ in 0..3 {
        open_sockets.push(open(&gateway, port).await);
        within(server.accept()).await.unwrap();
    }
    assert_eq!(refusal(format!("ws://{gateway}/ws/{port}")).await, 429);
    // The connections made before the limit go on working.
    open_sockets[0].send(Message::Binary(vec![1].into())).await.unwrap();
}
