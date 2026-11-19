//! Logging in, registering and recovering an account, and the questions asked on the way.

use super::*;

pub(super) const LOGIN_PORT: u16 = 6900;
/// The original client's least lengths for a login.
pub(super) const LOGIN_ID_MIN: usize = 4;
pub(super) const LOGIN_PASSWORD_MIN: usize = 6;
/// A login on an account that is in use elsewhere throws the other session out, which takes the servers a
/// moment: the login is tried again this often, this far apart, before the player is told to wait.
pub(super) const LOGIN_RETRIES: u8 = 6;
pub(super) const LOGIN_RETRY_MS: i32 = 1500;
/// The servers' "the last login is not forgotten yet".
pub(super) const STILL_LOGGED_IN: u8 = 8;

/// Sends a form to the account service. Gives back the status and the seconds to wait, if it names any.
pub(super) async fn post_form(path: &'static str, fields: Vec<(&'static str, String)>) -> Result<(u16, Option<u64>), String> {
    let window = web_sys::window().ok_or("no window")?;
    let body = web_sys::UrlSearchParams::new().map_err(js_error)?;
    for (name, value) in &fields {
        body.append(name, value);
    }
    let options = web_sys::RequestInit::new();
    options.set_method("POST");
    options.set_body(&body);
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(path, &options)).await.map_err(js_error)?.dyn_into().map_err(js_error)?;
    let wait = response.headers().get("retry-after").ok().flatten().and_then(|seconds| seconds.parse().ok());
    Ok((response.status(), wait))
}

pub(super) fn valid_id(id: &str) -> bool {
    (4..=23).contains(&id.len()) && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub(super) fn valid_password(password: &str) -> bool {
    (8..=23).contains(&password.len()) && password.bytes().all(|byte| byte.is_ascii_graphic())
}

/// A rough check, to catch slips before the form is sent; the account service decides.
pub(super) fn valid_email(email: &str) -> bool {
    email.len() <= 39
        && !email.contains(' ')
        && email.split_once('@').is_some_and(|(name, domain)| !name.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.'))
}

pub(super) fn minutes(seconds: u64) -> String {
    seconds.div_ceil(60).max(1).to_string()
}

impl App {
    /// OK or Cancel on a question.
    pub(super) fn answer(&mut self, yes: bool) {
        let Some(ask) = self.ask.take() else { return };
        if let Ask::Rebind(to, from, chord) = ask {
            if yes {
                self.rebind(to, from, chord);
            }
            return self.render();
        }
        let Some(socket) = self.socket.as_ref().filter(|_| yes) else { return self.render() };
        match ask {
            Ask::Reserve(char_id) => socket.send(&packets::delete_reserve(char_id)),
            // An account without a date of birth confirms with none; the server says if it wants one.
            Ask::Delete(char_id) => socket.send(&packets::delete_now(char_id, "")),
            Ask::Birthdate(char_id) => socket.send(&packets::delete_now(char_id, &field("birthdate"))),
            Ask::Rebind(..) => {}
        }
        self.sent_birthdate = matches!(ask, Ask::Birthdate(_));
        self.wait = Some(Key::PleaseWait);
        self.render();
    }

    pub(super) fn say(&mut self, key: Key) {
        self.message = Some(key.text(self.lang).to_owned());
        self.wait = None;
        self.render();
    }

    pub(super) fn login(&mut self) {
        let (user, password) = (typed("user"), typed("password"));
        for (text, least) in [(&user, LOGIN_ID_MIN), (&password, LOGIN_PASSWORD_MIN)] {
            if text.len() < least {
                self.message = Some(fill(Key::EnterAtLeast.text(self.lang), &[&least.to_string()]));
                return self.render();
            }
        }
        self.saved_id = if self.save_id { user.clone() } else { String::new() };
        self.relogin = Some((user.clone(), password.clone()));
        self.login_tries = 0;
        self.credentials = Some((user, password));
        self.connect(Server::Login, LOGIN_PORT);
    }

    pub(super) fn request(&mut self, request: Request, path: &'static str, fields: Vec<(&'static str, String)>) {
        self.wait = Some(Key::PleaseWait);
        self.render();
        wasm_bindgen_futures::spawn_local(async move {
            let answer = post_form(path, fields).await;
            dispatch(Action::Answer(request, answer));
        });
    }

    pub(super) fn register(&mut self) {
        let (id, password, email) = (typed("reg-id"), typed("reg-password"), typed("reg-email"));
        let problem = if !valid_id(&id) {
            Some(Key::BadId)
        } else if !valid_password(&password) {
            Some(Key::BadPassword)
        } else if password != typed("reg-repeat") {
            Some(Key::PasswordsDiffer)
        } else if !valid_email(&email) {
            Some(Key::BadEmail)
        } else {
            None
        };
        match problem {
            Some(problem) => self.say(problem),
            None => self.request(Request::Register, "/api/register", vec![("userid", id), ("password", password), ("email", email)]),
        }
    }

    pub(super) fn recover(&mut self) {
        let (id, email) = (typed("rec-id"), typed("rec-email"));
        if !valid_id(&id) {
            return self.say(Key::BadId);
        }
        if !valid_email(&email) {
            return self.say(Key::BadEmail);
        }
        // The letter is written in the language the player reads the client in.
        self.request(Request::Recover, "/api/recover", vec![("userid", id), ("email", email), ("lang", self.lang.code().to_owned())]);
    }

    pub(super) fn reset(&mut self) {
        let Screen::Reset(token) = &self.screen else { return };
        let (token, password) = (token.clone(), typed("new-password"));
        if !valid_password(&password) {
            return self.say(Key::BadPassword);
        }
        if password != typed("new-repeat") {
            return self.say(Key::PasswordsDiffer);
        }
        self.request(Request::Reset, "/api/reset", vec![("token", token), ("password", password)]);
    }

    pub(super) fn answered(&mut self, request: Request, answer: Result<(u16, Option<u64>), String>) {
        // The wait was given up, or the player went elsewhere: the answer is for nobody.
        if self.wait.is_none() || !matches!(self.screen, Screen::Register | Screen::Recover | Screen::Reset(_)) {
            return;
        }
        let (status, wait) = match answer {
            Ok(answer) => answer,
            Err(error) => {
                web_sys::console::error_1(&error.into());
                return self.say(Key::ServiceUnavailable);
            }
        };
        let done = match (request, status) {
            (Request::Register, 201) => Some(Key::AccountCreated),
            (Request::Recover, 200) => Some(Key::RestoreSent),
            (Request::Reset, 200) => Some(Key::PasswordChanged),
            _ => None,
        };
        if let Some(done) = done {
            if matches!(request, Request::Register) {
                self.saved_id = typed("reg-id");
            }
            // A used reset link is taken out of the address bar.
            if matches!(request, Request::Reset) {
                if let Some(Ok(history)) = web_sys::window().map(|window| window.history()) {
                    let _ = history.replace_state_with_url(&JsValue::NULL, "", Some("/"));
                }
            }
            self.screen = Screen::Login;
            return self.say(done);
        }
        match (request, status) {
            (_, 429) => {
                let wait = minutes(wait.unwrap_or(3600));
                self.message = Some(fill(Key::TooManyAttempts.text(self.lang), &[&wait]));
                self.wait = None;
                self.render();
            }
            (Request::Register, 409) => self.say(Key::IdTaken),
            (Request::Reset, 400) => self.say(Key::LinkInvalid),
            (_, 400) => self.say(Key::CheckForm),
            _ => self.say(Key::ServiceUnavailable),
        }
    }

    /// The moment to try a login again, if nothing else has happened since (the generation is the same).
    pub(super) fn login_retried(&mut self, generation: u32) {
        if let Some(credentials) = self.relogin.clone().filter(|_| generation == self.generation) {
            self.login_tries += 1;
            self.credentials = Some(credentials);
            self.connect(Server::Login, LOGIN_PORT);
        }
    }
}
