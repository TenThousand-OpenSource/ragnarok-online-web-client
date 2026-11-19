//! A lost password: `POST /api/recover` e-mails a link to set a new one (the letter, and the token in its link,
//! of which the database keeps only the hash), and `POST /api/reset` sets the new password with that token.

use lettre::message::header::ContentType;
use lettre::{AsyncTransport, Message};
use sha2::{Digest, Sha256};

use super::*;

/// How long a reset link works.
const TOKEN_MINUTES: u32 = 30;

#[derive(Deserialize)]
pub(super) struct RecoverForm {
    userid: String,
    email: String,
    /// The language to write the letter in: `ru`, or anything else for English.
    #[serde(default)]
    lang: String,
}

pub(super) async fn recover(
    State(accounts): State<Arc<Accounts>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<RecoverForm>
) -> Response {
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
pub(super) struct ResetForm {
    token: String,
    password: String,
}

pub(super) async fn reset(
    State(accounts): State<Arc<Accounts>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<ResetForm>
) -> Response {
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
    let taken = sqlx::query("DELETE FROM `roweb_password_reset` WHERE `token_hash` = ?").bind(&hash).execute(&accounts.db).await.map_err(
        |error| error.to_string()
    )?;
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
            sqlx::query_as("SELECT `userid` FROM `login` WHERE `account_id` = ?")
                .bind(account_id)
                .fetch_optional(&accounts.db).await
                .map_err(|error| error.to_string())?;
        if let Some((name,)) = name {
            guard.forgive_account(&name);
        }
    }
    Ok(true)
}
