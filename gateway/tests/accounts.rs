//! Registration and password recovery against the real services of `server/docker-compose.yml`: rAthena's
//! database, its login server, and the development mailbox. Skipped by default:
//! `cargo test -p roweb-gateway -- --ignored`.
//!
//! Every test makes accounts named `rwt<random>` and removes them again.

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Request, StatusCode, header};
use futures_util::{SinkExt, StreamExt};
use roweb_gateway::accounts::{self, Limits};
use roweb_gateway::bridge::{self, Bridge};
use sqlx::mysql::MySqlPool;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

const DATABASE: &str = "mysql://ragnarok:ragnarok@127.0.0.1:3306/ragnarok";
const MAILBOX: &str = "127.0.0.1:8025";
const LOGIN_PORT: u16 = 6900;

fn api(limits: Limits) -> Router {
    let config = accounts::Config {
        database_url: DATABASE.into(),
        smtp_url: "smtp://127.0.0.1:1025".into(),
        mail_from: "Test <no-reply@localhost>".into(),
        public_url: "http://game.test/".into(),
    };
    accounts::router(config, limits, None).unwrap().layer(MockConnectInfo(SocketAddr::from(([203, 0, 113, 7], 40000))))
}

fn generous() -> Limits {
    Limits { register: 100, recover: 100, recover_per_account: 100, reset: 100 }
}

/// A name no other test or run uses.
fn unique() -> String {
    let mut random = [0u8; 5];
    getrandom::fill(&mut random).unwrap();
    format!("rwt{}", hex::encode(random))
}

async fn post(api: &Router, path: &str, form: &[(&str, &str)]) -> (StatusCode, String) {
    let body: Vec<String> = form.iter().map(|(name, value)| format!("{name}={}", urlencode(value))).collect();
    let request = Request::post(path).header(header::CONTENT_TYPE, "application/x-www-form-urlencoded").body(Body::from(body.join("&"))).unwrap();
    let response = api.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let text = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
    (status, String::from_utf8_lossy(&text).into_owned())
}

fn urlencode(value: &str) -> String {
    value.bytes().map(|byte| if byte.is_ascii_alphanumeric() { char::from(byte).to_string() } else { format!("%{byte:02X}") }).collect()
}

async fn register(api: &Router, userid: &str, password: &str, email: &str) -> (StatusCode, String) {
    post(api, "/api/register", &[("userid", userid), ("password", password), ("email", email)]).await
}

async fn database() -> MySqlPool {
    MySqlPool::connect(DATABASE).await.expect("the database of server/docker-compose.yml")
}

async fn remove_account(userid: &str) {
    let db = database().await;
    sqlx::query("DELETE FROM `roweb_password_reset` WHERE `account_id` IN (SELECT `account_id` FROM `login` WHERE `userid` = ?)")
        .bind(userid)
        .execute(&db).await
        .ok();
    sqlx::query("DELETE FROM `login` WHERE `userid` = ?").bind(userid).execute(&db).await.unwrap();
}

/// Whether rAthena's login server takes the password, asked through the bridge as the client does.
async fn can_log_in(userid: &str, password: &str) -> bool {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = bridge::router(Bridge {
        host: "127.0.0.1".into(),
        ports: vec![LOGIN_PORT],
        login_guard: None,
        connections_per_minute: 10_000,
        lag: Default::default()
    });
    tokio::spawn(async move { axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap() });
    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws/{LOGIN_PORT}")).await.unwrap();

    let mut request = vec![0u8; 55];
    request[..2].copy_from_slice(&0x0064u16.to_le_bytes());
    request[6..6 + userid.len()].copy_from_slice(userid.as_bytes());
    request[30..30 + password.len()].copy_from_slice(password.as_bytes());
    socket.send(Message::Binary(request.into())).await.unwrap();
    loop {
        match tokio::time::timeout(Duration::from_secs(5), socket.next()).await.expect("the login server did not answer") {
            Some(Ok(Message::Binary(reply))) => return reply.starts_with(&[0xc4, 0x0a]),
            Some(Ok(_)) => {}
            other => panic!("{other:?}"),
        }
    }
}

/// A GET to the development mailbox's API.
async fn mailbox(path: &str) -> serde_json::Value {
    let mut stream = TcpStream::connect(MAILBOX).await.expect("the mailbox of server/docker-compose.yml");
    stream.write_all(format!("GET {path} HTTP/1.0\r\nHost: {MAILBOX}\r\n\r\n").as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    let body = response.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    serde_json::from_str(body).unwrap_or_else(|error| panic!("{path}: {error}: {response}"))
}

/// The texts of the letters sent to an address, waiting a moment for `expected` of them.
async fn letters_to(email: &str, expected: usize) -> Vec<String> {
    let mut ids = Vec::new();
    for _ in 0..30 {
        let found = mailbox(&format!("/api/v1/search?query=to:{email}")).await;
        ids = found["messages"]
            .as_array()
            .map(|messages| messages.iter().filter_map(|message| message["ID"].as_str().map(str::to_owned)).collect())
            .unwrap_or_default();
        if ids.len() >= expected {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let mut texts = Vec::new();
    for id in ids {
        texts.push(mailbox(&format!("/api/v1/message/{id}")).await["Text"].as_str().unwrap_or_default().to_owned());
    }
    texts
}

fn token_in(letter: &str) -> String {
    let start = letter.find("?reset=").expect("a reset link") + 7;
    letter[start..].chars().take_while(char::is_ascii_hexdigit).collect()
}

#[tokio::test]
#[ignore = "needs the local rAthena, database and mailbox"]
async fn live_a_new_account_can_log_in() {
    let (api, id) = (api(generous()), unique());
    let email = format!("{id}@example.test");

    let (status, text) = register(&api, &id, "first-Pass1", &email).await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert!(can_log_in(&id, "first-Pass1").await);
    assert!(!can_log_in(&id, "another-Pass").await);

    // The password is not stored as it was typed.
    let (stored,): (String,) = sqlx::query_as("SELECT `user_pass` FROM `login` WHERE `userid` = ?").bind(&id).fetch_one(&database().await).await.unwrap();
    assert_eq!(stored.len(), 32);
    assert!(!stored.contains("first-Pass1"));

    // The ID is taken now, in any case of letters, and the first account is untouched.
    assert_eq!(register(&api, &id, "second-Pass2", &email).await.0, StatusCode::CONFLICT);
    assert_eq!(register(&api, &id.to_uppercase(), "second-Pass2", &email).await.0, StatusCode::CONFLICT);
    assert!(can_log_in(&id, "first-Pass1").await);
    remove_account(&id).await;
}

#[tokio::test]
#[ignore = "needs the local rAthena, database and mailbox"]
async fn live_registration_refuses_bad_input() {
    let api = api(generous());
    let id = unique();
    for (userid, password, email) in [
        ("ab", "good-Pass1", "a@example.test"),
        ("bad name", "good-Pass1", "a@example.test"),
        (id.as_str(), "short", "a@example.test"),
        (id.as_str(), "good-Pass1", "not-an-address"),
        ("x' OR '1'='1", "good-Pass1", "a@example.test"),
    ] {
        let (status, _) = post(&api, "/api/register", &[("userid", userid), ("password", password), ("email", email)]).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{userid} / {password} / {email}");
    }
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM `login` WHERE `userid` = ?").bind(&id).fetch_one(&database().await).await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
#[ignore = "needs the local rAthena, database and mailbox"]
async fn live_a_lost_password_is_reset_by_mail() {
    let (api, id) = (api(generous()), unique());
    let email = format!("{id}@example.test");
    assert_eq!(register(&api, &id, "lost-Pass1", &email).await.0, StatusCode::CREATED);

    // A wrong e-mail address and an unknown ID get the same answer as the right pair, and no letter.
    let right = post(&api, "/api/recover", &[("userid", &id), ("email", &email)]).await;
    let wrong_email = post(&api, "/api/recover", &[("userid", &id), ("email", "someone@example.test")]).await;
    let unknown = post(&api, "/api/recover", &[("userid", "nobody_here"), ("email", &email)]).await;
    assert_eq!(right.0, StatusCode::OK);
    assert_eq!(right, wrong_email);
    assert_eq!(right, unknown);
    let letters = letters_to(&email, 1).await;
    assert_eq!(letters.len(), 1, "{letters:?}");
    assert!(letters_to("someone@example.test", 0).await.is_empty());

    // The link leads to the configured address, and the token is nowhere in the database.
    let token = token_in(&letters[0]);
    assert_eq!(token.len(), 64);
    assert!(letters[0].contains(&format!("http://game.test/?reset={token}")), "{}", letters[0]);
    let (stored,): (Vec<u8>,) = sqlx::query_as("SELECT `token_hash` FROM `roweb_password_reset` LIMIT 1").fetch_one(&database().await).await.unwrap();
    assert_ne!(hex::encode(stored), token);

    // A wrong token and a weak password change nothing.
    assert_eq!(post(&api, "/api/reset", &[("token", &"0".repeat(64)), ("password", "new-Pass22")]).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(post(&api, "/api/reset", &[("token", &token), ("password", "weak")]).await.0, StatusCode::BAD_REQUEST);
    assert!(can_log_in(&id, "lost-Pass1").await);

    let (status, text) = post(&api, "/api/reset", &[("token", &token), ("password", "new-Pass22")]).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    assert!(can_log_in(&id, "new-Pass22").await);
    assert!(!can_log_in(&id, "lost-Pass1").await);

    // The link works once.
    assert_eq!(post(&api, "/api/reset", &[("token", &token), ("password", "third-Pass3")]).await.0, StatusCode::BAD_REQUEST);
    assert!(can_log_in(&id, "new-Pass22").await);
    remove_account(&id).await;
}

#[tokio::test]
#[ignore = "needs the local rAthena, database and mailbox"]
async fn live_a_newer_link_replaces_the_older_and_links_expire() {
    let (api, id) = (api(generous()), unique());
    let email = format!("{id}@example.test");
    assert_eq!(register(&api, &id, "lost-Pass1", &email).await.0, StatusCode::CREATED);

    post(&api, "/api/recover", &[("userid", &id), ("email", &email)]).await;
    let first = token_in(&letters_to(&email, 1).await[0]);
    post(&api, "/api/recover", &[("userid", &id), ("email", &email)]).await;
    let letters = letters_to(&email, 2).await;
    assert_eq!(letters.len(), 2);
    let second = letters.iter().map(|letter| token_in(letter)).find(|token| *token != first).expect("a second token");
    assert_eq!(post(&api, "/api/reset", &[("token", &first), ("password", "new-Pass22")]).await.0, StatusCode::BAD_REQUEST);

    // The second link, once its time has passed.
    sqlx::query("UPDATE `roweb_password_reset` SET `expires_at` = DATE_SUB(NOW(), INTERVAL 1 MINUTE) WHERE \
        `account_id` = (SELECT `account_id` FROM `login` WHERE `userid` = ?)")
        .bind(&id)
        .execute(&database().await)
        .await
        .unwrap();
    assert_eq!(post(&api, "/api/reset", &[("token", &second), ("password", "new-Pass22")]).await.0, StatusCode::BAD_REQUEST);
    assert!(can_log_in(&id, "lost-Pass1").await);
    remove_account(&id).await;
}

#[tokio::test]
#[ignore = "needs the local rAthena, database and mailbox"]
async fn live_attempts_are_limited() {
    let api = api(Limits { register: 2, recover: 2, recover_per_account: 1, reset: 2 });
    let (first, second, third) = (unique(), unique(), unique());

    // Registration: two an hour from this address; a form that fails validation is not counted.
    assert_eq!(register(&api, "x", "good-Pass1", "a@example.test").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(register(&api, &first, "good-Pass1", &format!("{first}@example.test")).await.0, StatusCode::CREATED);
    assert_eq!(register(&api, &second, "good-Pass1", &format!("{second}@example.test")).await.0, StatusCode::CREATED);
    let (status, text) = register(&api, &third, "good-Pass1", &format!("{third}@example.test")).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(text.contains("60 minutes"), "{text}");
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM `login` WHERE `userid` = ?").bind(&third).fetch_one(&database().await).await.unwrap();
    assert_eq!(count, 0);

    // Recovery: an account gets one letter an hour, though the second request is answered the same.
    let email = format!("{first}@example.test");
    assert_eq!(post(&api, "/api/recover", &[("userid", &first), ("email", &email)]).await.0, StatusCode::OK);
    assert_eq!(letters_to(&email, 1).await.len(), 1);
    assert_eq!(post(&api, "/api/recover", &[("userid", &first), ("email", &email)]).await.0, StatusCode::OK);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(letters_to(&email, 1).await.len(), 1);
    // And an address two requests an hour, whatever the account.
    assert_eq!(post(&api, "/api/recover", &[("userid", &second), ("email", &format!("{second}@example.test"))]).await.0, StatusCode::TOO_MANY_REQUESTS);

    // Reset: two tries an hour, so tokens cannot be guessed at speed.
    for _ in 0..2 {
        assert_eq!(post(&api, "/api/reset", &[("token", &"a".repeat(64)), ("password", "new-Pass22")]).await.0, StatusCode::BAD_REQUEST);
    }
    assert_eq!(post(&api, "/api/reset", &[("token", &"a".repeat(64)), ("password", "new-Pass22")]).await.0, StatusCode::TOO_MANY_REQUESTS);

    remove_account(&first).await;
    remove_account(&second).await;
}

/// A form sent by a page of another website is refused, whatever it holds.
#[tokio::test]
#[ignore = "needs the local rAthena, database and mailbox"]
async fn live_other_websites_cannot_use_the_forms() {
    let api = api(generous());
    for path in ["/api/register", "/api/recover", "/api/reset"] {
        let request = Request::post(path)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::HOST, "game.test")
            .header(header::ORIGIN, "https://evil.example")
            .body(Body::from("userid=abcd&password=good-Pass1&email=a%40example.test&token=x"))
            .unwrap();
        assert_eq!(api.clone().oneshot(request).await.unwrap().status(), StatusCode::FORBIDDEN, "{path}");
    }
}
