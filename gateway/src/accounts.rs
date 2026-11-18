//! Accounts, for players who have none or lost their password: the game protocol itself has neither.
//!
//! - `POST /api/register` (`userid`, `password`, `email`): makes an account in rAthena's `login` table. The
//!   table wants a sex for the account; nothing asks the player for one, because with this client each
//!   character has its own, chosen as it is made (`CH_MAKE_CHAR`, 0x0a39), and the account's is used for nothing.
//! - `POST /api/recover` (`userid`, `email`): e-mails a link to set a new password, if the two match an account.
//! - `POST /api/reset` (`token`, `password`): sets the new password with the token from that link.
//!
//! Requests are forms (`application/x-www-form-urlencoded`); answers are a status and a line of plain text for
//! the player. Every endpoint counts attempts per address, and recovery also per account.
//!
//! Passwords are stored as rAthena reads them with `use_MD5_passwords: yes`: an unsalted MD5. That is the best
//! this login server offers, and far from good: the database must be kept as secret as the passwords themselves.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Form, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Router, extract::DefaultBodyLimit};
use lettre::message::{Mailbox, header::ContentType};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::bridge::{LoginGuard, address_key, same_origin};
use crate::limiter::Limiter;

/// How long a reset link works.
const TOKEN_MINUTES: u32 = 30;
const HOUR: Duration = Duration::from_secs(3600);

/// What rAthena's `login` table and login packet hold: 23 characters each, and 39 for the e-mail.
const ID_LENGTH: std::ops::RangeInclusive<usize> = 4..=23;
const PASSWORD_LENGTH: std::ops::RangeInclusive<usize> = 8..=23;
const EMAIL_MAX: usize = 39;

pub struct Config {
    /// `mysql://user:password@host:port/database`, rAthena's login database.
    pub database_url: String,
    /// `smtp://host:port`, or `smtps://user:password@host` for a real mail server.
    pub smtp_url: String,
    pub mail_from: String,
    /// Where players reach the gateway. Reset links start with it; it is never taken from a request.
    pub public_url: String,
}

/// How many attempts an address (and, for recovery, an account) gets per hour.
pub struct Limits {
    pub register: usize,
    pub recover: usize,
    pub recover_per_account: usize,
    pub reset: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { register: 5, recover: 3, recover_per_account: 3, reset: 10 }
    }
}

struct Accounts {
    db: MySqlPool,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    mail_from: Mailbox,
    public_url: String,
    register: Limiter,
    recover: Limiter,
    recover_per_account: Limiter,
    reset: Limiter,
    login_guard: Option<Arc<LoginGuard>>,
}

/// The router must be served with `into_make_service_with_connect_info::<SocketAddr>()`. Nothing connects to the
/// database or the mail server until a request needs them.
pub fn router(config: Config, limits: Limits, login_guard: Option<Arc<LoginGuard>>) -> Result<Router, String> {
    let accounts = Accounts {
        db: MySqlPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(3))
            .connect_lazy(&config.database_url)
            .map_err(|error| format!("database URL: {error}"))?,
        mailer: AsyncSmtpTransport::<Tokio1Executor>::from_url(&config.smtp_url).map_err(|error| format!("SMTP URL: {error}"))?.build(),
        mail_from: config.mail_from.parse().map_err(|error| format!("mail sender {}: {error}", config.mail_from))?,
        public_url: config.public_url.trim_end_matches('/').to_owned(),
        register: Limiter::new(limits.register, HOUR),
        recover: Limiter::new(limits.recover, HOUR),
        recover_per_account: Limiter::new(limits.recover_per_account, HOUR),
        reset: Limiter::new(limits.reset, HOUR),
        login_guard,
    };
    Ok(Router::new()
        .route("/api/register", post(register))
        .route("/api/recover", post(recover))
        .route("/api/reset", post(reset))
        .layer(DefaultBodyLimit::max(4096))
        .with_state(Arc::new(accounts)))
}

/// An answer for the player: a status and one line of text.
type Answer = (StatusCode, String);

fn answer(status: StatusCode, text: &str) -> Answer {
    (status, text.to_owned())
}

fn too_many(wait: Duration) -> Response {
    let minutes = wait.as_secs().div_ceil(60).max(1);
    let text = format!("Too many attempts. Please try again in {minutes} minute{}.", if minutes == 1 { "" } else { "s" });
    (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, wait.as_secs().max(1).to_string())], text).into_response()
}

fn unavailable(what: &str, error: impl std::fmt::Display) -> Response {
    eprintln!("accounts: {what}: {error}");
    answer(StatusCode::SERVICE_UNAVAILABLE, "The account service is not available right now. Please try again later.").into_response()
}

pub fn valid_id(id: &str) -> Result<(), &'static str> {
    if !ID_LENGTH.contains(&id.len()) || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return Err("An ID is 4 to 23 letters, digits and underscores.");
    }
    Ok(())
}

pub fn valid_password(password: &str) -> Result<(), &'static str> {
    // Printable ASCII without spaces: what every keyboard and the game's login packet can carry unchanged.
    if !PASSWORD_LENGTH.contains(&password.len()) || !password.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err("A password is 8 to 23 letters, digits and symbols, without spaces.");
    }
    Ok(())
}

pub fn valid_email(email: &str) -> Result<(), &'static str> {
    let well_formed = email.split_once('@').is_some_and(|(name, domain)| {
        !name.is_empty()
            && !domain.contains('@')
            && domain.split('.').count() >= 2
            && domain.split('.').all(|label| !label.is_empty() && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
            && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b".!#$%&'*+/=?^_`{|}~-".contains(&byte))
    });
    if !well_formed || email.len() > EMAIL_MAX {
        return Err("Please enter a valid e-mail address of at most 39 characters.");
    }
    Ok(())
}

#[derive(Deserialize)]
struct RegisterForm {
    userid: String,
    password: String,
    email: String,
}

async fn register(State(accounts): State<Arc<Accounts>>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Form(form): Form<RegisterForm>) -> Response {
    if !same_origin(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let address = address_key(peer.ip());
    if let Some(wait) = accounts.register.blocked_for(&address, Instant::now()) {
        return too_many(wait);
    }
    if let Err(problem) = valid_id(&form.userid).and(valid_password(&form.password)).and(valid_email(&form.email)) {
        return answer(StatusCode::BAD_REQUEST, problem).into_response();
    }

    // What reaches the database counts as an attempt: both a new account and a taken ID tell the asker something.
    accounts.register.hit(&address, Instant::now());
    // One statement, so two requests for the same ID cannot both pass the check.
    let inserted = sqlx::query(
        "INSERT INTO `login` (`userid`, `user_pass`, `sex`, `email`) \
         SELECT ?, MD5(?), 'M', ? FROM DUAL WHERE NOT EXISTS (SELECT 1 FROM `login` WHERE `userid` = ?)",
    )
    .bind(&form.userid)
    .bind(&form.password)
    .bind(&form.email)
    .bind(&form.userid)
    .execute(&accounts.db)
    .await;
    match inserted {
        Ok(result) if result.rows_affected() == 1 => answer(StatusCode::CREATED, "Your account has been created. You can log in now.").into_response(),
        Ok(_) => answer(StatusCode::CONFLICT, "This ID is already in use. Please choose another one.").into_response(),
        Err(error) => unavailable("register", error),
    }
}

#[derive(Deserialize)]
struct RecoverForm {
    userid: String,
    email: String,
    /// The language to write the letter in: `ru`, or anything else for English.
    #[serde(default)]
    lang: String,
}

async fn recover(State(accounts): State<Arc<Accounts>>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Form(form): Form<RecoverForm>) -> Response {
    if !same_origin(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if let Err(problem) = valid_id(&form.userid).and(valid_email(&form.email)) {
        return answer(StatusCode::BAD_REQUEST, problem).into_response();
    }
    if let Err(wait) = accounts.recover.attempt(&address_key(peer.ip()), Instant::now()) {
        return too_many(wait);
    }
    // The same answer whether or not the account exists, so the form cannot be used to find accounts or their
    // e-mail addresses. The work goes on after the answer, so its duration tells nothing either.
    let accounts = accounts.clone();
    tokio::spawn(async move {
        if let Err(error) = send_reset_link(&accounts, &form.userid, &form.email, &form.lang).await {
            eprintln!("accounts: recover: {error}");
        }
    });
    answer(StatusCode::OK, "If the ID and e-mail address belong to an account, a message with a link to set a new password is on its way.").into_response()
}

async fn send_reset_link(accounts: &Accounts, userid: &str, email: &str, lang: &str) -> Result<(), String> {
    // Not the servers' own account (sex 'S'), which has no owner to write to.
    let found: Option<(u32, String)> = sqlx::query_as("SELECT `account_id`, `email` FROM `login` WHERE `userid` = ? AND `email` = ? AND `sex` <> 'S'")
        .bind(userid)
        .bind(email)
        .fetch_optional(&accounts.db)
        .await
        .map_err(|error| error.to_string())?;
    let Some((account_id, stored_email)) = found else { return Ok(()) };
    // An account gets only so many letters an hour, whoever asks and from wherever.
    if accounts.recover_per_account.attempt(&account_id.to_string(), Instant::now()).is_err() {
        return Ok(());
    }

    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret).map_err(|error| error.to_string())?;
    let token = hex::encode(secret);
    ensure_table(&accounts.db).await?;
    // A new link replaces the account's older ones. Only the token's hash is stored: the database cannot reset.
    sqlx::query("DELETE FROM `roweb_password_reset` WHERE `account_id` = ? OR `expires_at` < NOW()")
        .bind(account_id)
        .execute(&accounts.db)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("INSERT INTO `roweb_password_reset` (`token_hash`, `account_id`, `expires_at`) VALUES (?, ?, DATE_ADD(NOW(), INTERVAL ? MINUTE))")
        .bind(token_hash(&token))
        .bind(account_id)
        .bind(TOKEN_MINUTES)
        .execute(&accounts.db)
        .await
        .map_err(|error| error.to_string())?;

    let (subject, text) = reset_letter(lang, userid, &format!("{}/?reset={token}", accounts.public_url));
    let message = Message::builder()
        .from(accounts.mail_from.clone())
        .to(stored_email.parse().map_err(|error| format!("address of account {account_id}: {error}"))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(text)
        .map_err(|error| error.to_string())?;
    accounts.mailer.send(message).await.map_err(|error| format!("mail: {error}"))?;
    Ok(())
}

/// The subject and text of the letter with a reset link, in the player's language.
pub fn reset_letter(lang: &str, userid: &str, link: &str) -> (&'static str, String) {
    if lang.eq_ignore_ascii_case("ru") {
        (
            "Установка нового пароля",
            format!(
                "Здравствуйте,\n\nкто-то запросил установку нового пароля для игрового аккаунта \"{userid}\".\n\n\
                 Чтобы выбрать новый пароль, откройте эту ссылку в течение {TOKEN_MINUTES} минут:\n\n{link}\n\n\
                 Если это были не вы, просто не отвечайте на письмо: пароль останется прежним.\n"
            ),
        )
    } else {
        (
            "Set a new password",
            format!(
                "Hello,\n\nsomeone asked to set a new password for the game account \"{userid}\".\n\n\
                 To choose a new password, open this link within {TOKEN_MINUTES} minutes:\n\n{link}\n\n\
                 If it was not you, ignore this message: the password stays as it is.\n"
            ),
        )
    }
}

async fn ensure_table(db: &MySqlPool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `roweb_password_reset` (\
           `token_hash` BINARY(32) NOT NULL PRIMARY KEY, \
           `account_id` INT UNSIGNED NOT NULL, \
           `expires_at` DATETIME NOT NULL, \
           KEY `account` (`account_id`))",
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

#[derive(Deserialize)]
struct ResetForm {
    token: String,
    password: String,
}

async fn reset(State(accounts): State<Arc<Accounts>>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Form(form): Form<ResetForm>) -> Response {
    if !same_origin(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if let Err(problem) = valid_password(&form.password) {
        return answer(StatusCode::BAD_REQUEST, problem).into_response();
    }
    if let Err(wait) = accounts.reset.attempt(&address_key(peer.ip()), Instant::now()) {
        return too_many(wait);
    }
    match set_password(&accounts, &form.token, &form.password).await {
        Ok(true) => answer(StatusCode::OK, "Your password has been changed. You can log in now.").into_response(),
        Ok(false) => answer(StatusCode::BAD_REQUEST, "This link is not valid any more. Please ask for a new one.").into_response(),
        Err(error) => unavailable("reset", error),
    }
}

/// Sets the password of the account the token was made for. `false`: no such token, or it is used or too old.
async fn set_password(accounts: &Accounts, token: &str, password: &str) -> Result<bool, String> {
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(false);
    }
    ensure_table(&accounts.db).await?;
    let hash = token_hash(token);
    let found: Option<(u32,)> = sqlx::query_as("SELECT `account_id` FROM `roweb_password_reset` WHERE `token_hash` = ? AND `expires_at` > NOW()")
        .bind(&hash)
        .fetch_optional(&accounts.db)
        .await
        .map_err(|error| error.to_string())?;
    let Some((account_id,)) = found else { return Ok(false) };
    // Whoever deletes the row has the token: of two requests with the same one, only one gets past here.
    let taken = sqlx::query("DELETE FROM `roweb_password_reset` WHERE `token_hash` = ?").bind(&hash).execute(&accounts.db).await.map_err(|error| error.to_string())?;
    if taken.rows_affected() != 1 {
        return Ok(false);
    }
    sqlx::query("UPDATE `login` SET `user_pass` = MD5(?) WHERE `account_id` = ? AND `sex` <> 'S'")
        .bind(password)
        .bind(account_id)
        .execute(&accounts.db)
        .await
        .map_err(|error| error.to_string())?;

    // The owner is back: the account's failed logins, perhaps the very reason for the reset, are forgiven.
    if let Some(guard) = &accounts.login_guard {
        let name: Option<(String,)> =
            sqlx::query_as("SELECT `userid` FROM `login` WHERE `account_id` = ?").bind(account_id).fetch_optional(&accounts.db).await.map_err(|error| error.to_string())?;
        if let Some((name,)) = name {
            guard.forgive_account(&name);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_what_the_login_server_takes() {
        for id in ["abcd", "Player_01", &"x".repeat(23)] {
            assert!(valid_id(id).is_ok(), "{id}");
        }
        for id in ["", "abc", &"x".repeat(24), "with space", "semi;colon", "quote'", "кириллица", "a\0bc"] {
            assert!(valid_id(id).is_err(), "{id:?}");
        }
    }

    #[test]
    fn passwords_fit_the_login_packet() {
        for password in ["12345678", "p@ss-W0rd!", &"x".repeat(23)] {
            assert!(valid_password(password).is_ok(), "{password}");
        }
        for password in ["", "short7!", &"x".repeat(24), "has a space", "tab\there!", "пароль1234", "nul\0inside"] {
            assert!(valid_password(password).is_err(), "{password:?}");
        }
    }

    #[test]
    fn emails_are_well_formed_and_fit_the_column() {
        for email in ["a@b.co", "first.last+tag@mail.example.org", "player_1@example-host.net"] {
            assert!(valid_email(email).is_ok(), "{email}");
        }
        let too_long = format!("{}@example.org", "x".repeat(30));
        for email in ["", "plain", "@example.org", "name@", "name@host", "two@@example.org", "a@b@c.org", "spa ce@example.org", "name@exa mple.org", "name@.org", "name@example.", "new\nline@example.org", &too_long] {
            assert!(valid_email(email).is_err(), "{email:?}");
        }
    }

    #[test]
    fn the_letter_is_in_the_players_language() {
        let (subject, text) = reset_letter("ru", "hero", "http://game.test/?reset=abc");
        assert_eq!(subject, "Установка нового пароля");
        assert!(text.contains("\"hero\"") && text.contains("http://game.test/?reset=abc") && text.contains("30 минут"), "{text}");
        // English for every other language, and for none.
        for lang in ["en", "", "de"] {
            let (subject, text) = reset_letter(lang, "hero", "http://game.test/?reset=abc");
            assert_eq!(subject, "Set a new password");
            assert!(text.contains("http://game.test/?reset=abc") && text.contains("30 minutes"));
        }
    }

    #[test]
    fn says_how_long_to_wait() {
        let response = too_many(Duration::from_secs(61));
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "61");
    }
}
