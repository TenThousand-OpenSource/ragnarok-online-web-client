//! The bridge's debug delay (`bridge::Lag`, the gateway's `--lag` and `--jitter`): what goes each way is held
//! back, and stays in the order it came.

use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use roweb_gateway::bridge::{self, Bridge, Lag};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn holds_back_what_goes_each_way_and_keeps_its_order() {
    let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = server.local_addr().unwrap().port();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway = listener.local_addr().unwrap().to_string();
    let lag = Lag { fixed: Duration::from_millis(60), jitter: Duration::from_millis(40) };
    let router = bridge::router(Bridge { host: "127.0.0.1".to_owned(), ports: vec![port], login_guard: None, connections_per_minute: 10_000, lag });
    tokio::spawn(async move { axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap() });
    let (mut browser, _) = tokio_tungstenite::connect_async(format!("ws://{gateway}/ws/{port}")).await.unwrap();
    let (mut game, _) = server.accept().await.unwrap();

    // Forty pieces, one after another: each draws its own delay, and none overtakes another.
    let sent = Instant::now();
    for piece in 0..40u8 {
        browser.send(Message::Binary(vec![piece].into())).await.unwrap();
    }
    let mut received = [0u8; 40];
    tokio::time::timeout(Duration::from_secs(5), game.read_exact(&mut received)).await.unwrap().unwrap();
    assert!(sent.elapsed() >= Duration::from_millis(60), "came after {:?}", sent.elapsed());
    assert_eq!(received.to_vec(), (0..40).collect::<Vec<u8>>());

    // And back: the answer takes its time too.
    let answered = Instant::now();
    game.write_all(&[7, 8, 9]).await.unwrap();
    let mut answer = Vec::new();
    while answer.len() < 3 {
        match tokio::time::timeout(Duration::from_secs(5), browser.next()).await.unwrap().unwrap().unwrap() {
            Message::Binary(bytes) => answer.extend(bytes.to_vec()),
            other => panic!("{other:?}"),
        }
    }
    assert!(answered.elapsed() >= Duration::from_millis(60), "came after {:?}", answered.elapsed());
    assert_eq!(answer, [7, 8, 9]);
}
