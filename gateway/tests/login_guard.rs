//! The bridge's login guard, against a stand-in login server: an address or an account that guesses passwords
//! is turned away for a while (`bridge/guard.rs`).

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use common::*;
use futures_util::{SinkExt, StreamExt};
use roweb_gateway::bridge::{self, Bridge, LoginGuard, TOO_MANY_ATTEMPTS};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

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
    serve(listener, bridge::router(Bridge {
        host: "127.0.0.1".to_owned(),
        ports: vec![port],
        login_guard: Some(Arc::new(guard)),
        connections_per_minute: 10_000,
        lag: Default::default()
    }))
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
    let gateway = serve(listener, bridge::router(Bridge {
        host: "127.0.0.1".to_owned(),
        ports: vec![port],
        login_guard: Some(Arc::new(guard)),
        connections_per_minute: 10_000,
        lag: Default::default()
    }));
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
