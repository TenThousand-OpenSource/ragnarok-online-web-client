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
//!
//! Here: what the three share (the configuration, the limits, the answers) and registering. What an ID, a
//! password and an e-mail address may be is in `accounts/rules.rs`; recovering and resetting a password, with
//! the letter and its token, in `accounts/recover.rs`; the tests in `accounts/tests.rs`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Form, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Router, extract::DefaultBodyLimit};
use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use serde::Deserialize;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::bridge::{LoginGuard, address_key, same_origin};
use crate::limiter::Limiter;

mod recover;
mod rules;

#[cfg(test)]
mod tests;

pub use recover::reset_letter;
pub use rules::{valid_email, valid_id, valid_password};

const HOUR: Duration = Duration::from_secs(3600);

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
        .route("/api/recover", post(recover::recover))
        .route("/api/reset", post(recover::reset))
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
