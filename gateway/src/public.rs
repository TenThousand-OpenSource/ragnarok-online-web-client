//! What the gateway does when it is reached from outside: through a tunnel or a proxy, by people it was not
//! started for.
//!
//! - **The invitation.** With a word set (`--invite-file`), nothing is served to a browser that has not given
//!   it: not the page, not the client's files, not the bridge to the game server. One who has not is shown a
//!   small page that asks for it; the right word leaves a cookie that lets the browser in for a month. Guesses
//!   are counted by address. Behind a tunnel, one who comes to the gateway itself and not through the tunnel
//!   (its owner, on the same machine) is not asked: the tunnel marks everyone who comes through it.
//! - **Who is asking.** Behind a tunnel every request comes from the tunnel's own address, and the limits on
//!   attempts (here, in `accounts` and in `bridge`) would count everyone as one. Told so (`--behind-proxy on`),
//!   the gateway takes the visitor's address from what the proxy says of it. Only then: a visitor who reaches
//!   the gateway directly could write that header themselves.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};

use crate::limiter::Limiter;

/// The cookie a browser that gave the word keeps.
const COOKIE: &str = "roweb_invite";
/// Where the word is sent.
const PATH: &str = "/invite";
/// How long the cookie lasts: thirty days, in seconds.
const KEPT: u64 = 30 * 24 * 60 * 60;

/// The invitation: what the cookie of one who gave the word says, and the count of wrong words by address.
pub struct Invite {
    pass: String,
    guesses: Limiter,
    /// Whether one who comes from this machine itself is let in unasked: so where a proxy stands in front and
    /// says who each of its visitors is (`behind_proxy`), which leaves this machine's own address to its owner.
    owner_unasked: bool,
}

impl Invite {
    /// An invitation by that word: ten wrong words from an address in ten minutes, and it waits.
    pub fn new(word: &str, owner_unasked: bool) -> Self {
        Self { pass: pass(word), guesses: Limiter::new(10, Duration::from_secs(600)), owner_unasked }
    }
}

/// What the cookie says for a word: not the word, so that it cannot be read out of a browser.
fn pass(word: &str) -> String {
    hex::encode(Sha256::digest(format!("roweb invite: {}", word.trim()).as_bytes()))
}

/// The value of a cookie of that name among those a request carries.
fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get_all(header::COOKIE).iter().filter_map(|value| value.to_str().ok()).flat_map(|value| value.split(';')).find_map(|pair| {
        let (key, value) = pair.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}

/// The word in a form sent as `word=…` (the one field of the page below), with what a browser writes in place
/// of other characters put back.
fn word_sent(body: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let sent = text.split('&').find_map(|pair| pair.strip_prefix("word="))?;
    let mut bytes = Vec::with_capacity(sent.len());
    let mut rest = sent.bytes();
    while let Some(byte) = rest.next() {
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let pair = [rest.next()?, rest.next()?];
                bytes.push(u8::from_str_radix(std::str::from_utf8(&pair).ok()?, 16).ok()?);
            }
            other => bytes.push(other),
        }
    }
    String::from_utf8(bytes).ok()
}

/// The page that asks for the word.
fn asking(wrong: bool) -> Response {
    let said = if wrong { "<p>That is not the word.</p>" } else { "" };
    let page = format!(
        r#"<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Ragnarok 2026</title>
<body style="margin:0;height:100vh;display:flex;align-items:center;justify-content:center;background:#1b2140;color:#e9ecf7;font:14px Tahoma,Arial,sans-serif">
<form method="post" action="{PATH}" style="text-align:center"><p>This server is open by invitation.</p>{said}
<input name="word" type="password" autofocus autocomplete="off" placeholder="invitation word" style="padding:6px 8px;border:1px solid #7f93b8;border-radius:3px">
<button style="padding:6px 12px">Enter</button></form>"#
    );
    (StatusCode::UNAUTHORIZED, [(header::CONTENT_TYPE, "text/html; charset=utf-8"), (header::CACHE_CONTROL, "no-store")], page).into_response()
}

/// Lets through what comes from a browser that has given the word, takes the word from one that sends it, and
/// asks the others for it.
pub async fn invited(State(invite): State<Arc<Invite>>, ConnectInfo(peer): ConnectInfo<SocketAddr>, request: Request, next: Next) -> Response {
    if cookie(request.headers(), COOKIE) == Some(invite.pass.as_str()) || (invite.owner_unasked && peer.ip().is_loopback()) {
        return next.run(request).await;
    }
    if request.method() != Method::POST || request.uri().path() != PATH {
        // One turned away from the game server itself is said: the page was open, and the invitation is not
        // with it (a cookie the browser did not send).
        if request.uri().path().starts_with("/ws/") {
            eprintln!("invite: {} turned away from {}: no invitation with the request", peer.ip(), request.uri().path());
        }
        return asking(false);
    }
    let address = peer.ip().to_string();
    let now = Instant::now();
    if invite.guesses.blocked_for(&address, now).is_some() {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let body = axum::body::to_bytes(request.into_body(), 1024).await.unwrap_or_default();
    if word_sent(&body).map(|word| pass(&word)).as_deref() != Some(invite.pass.as_str()) {
        invite.guesses.hit(&address, now);
        return asking(true);
    }
    let kept = format!("{COOKIE}={}; Path=/; Max-Age={KEPT}; HttpOnly; SameSite=Lax", invite.pass);
    (StatusCode::SEE_OTHER, [(header::LOCATION, "/".to_owned()), (header::SET_COOKIE, kept)]).into_response()
}

/// The visitor's address as the proxy in front of the gateway gives it: Cloudflare's own header, or the first
/// of those the request was forwarded for.
fn forwarded(headers: &HeaderMap) -> Option<IpAddr> {
    let said = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    said("cf-connecting-ip").or_else(|| said("x-forwarded-for").and_then(|list| list.split(',').next())).and_then(|address| address.trim().parse().ok())
}

/// Whether a request came through a proxy at all: it carries what one adds.
fn proxied(headers: &HeaderMap) -> bool {
    ["cf-connecting-ip", "cf-ray", "x-forwarded-for", "x-forwarded-proto", "forwarded", "via"].iter().any(|name| headers.contains_key(*name))
}

/// Behind a proxy: the address a request is counted by becomes the visitor's, not the proxy's. One that came
/// through the proxy without a readable address of its visitor is counted by an address that is nobody's, never
/// by this machine's own.
pub async fn behind_proxy(mut request: Request, next: Next) -> Response {
    if proxied(request.headers()) {
        let address = forwarded(request.headers()).filter(|address| !address.is_loopback()).unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
        request.extensions_mut().insert(ConnectInfo(SocketAddr::new(address, 0)));
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(name: &'static str, value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, value.parse().unwrap());
        headers
    }

    #[test]
    fn the_cookie_is_found_among_others() {
        let headers = with("cookie", "theme=dark; roweb_invite=abc123; other=1");
        assert_eq!(cookie(&headers, COOKIE), Some("abc123"));
        assert_eq!(cookie(&with("cookie", "theme=dark"), COOKIE), None);
    }

    #[test]
    fn the_word_is_read_from_the_form_as_it_was_typed() {
        assert_eq!(word_sent(b"word=two+words%21").as_deref(), Some("two words!"));
        assert_eq!(word_sent(b"other=1&word=plain").as_deref(), Some("plain"));
        assert_eq!(word_sent(b"other=1"), None);
        assert_eq!(word_sent(b"word=%zz"), None);
    }

    #[test]
    fn the_pass_is_not_the_word_and_ignores_the_space_around_it() {
        assert_eq!(pass("poring\n"), pass("poring"));
        assert_ne!(pass("poring"), pass("drops"));
        assert!(!pass("poring").contains("poring"));
    }

    #[test]
    fn what_came_through_a_proxy_is_told_from_what_came_straight() {
        assert!(proxied(&with("cf-ray", "8a1b2c3d4e5f-FRA")) && proxied(&with("x-forwarded-proto", "https")));
        assert!(!proxied(&with("cookie", "a=b")));
    }

    #[test]
    fn the_visitor_is_who_the_proxy_says() {
        assert_eq!(forwarded(&with("cf-connecting-ip", "203.0.113.7")), Some("203.0.113.7".parse().unwrap()));
        assert_eq!(forwarded(&with("x-forwarded-for", "198.51.100.4, 10.0.0.1")), Some("198.51.100.4".parse().unwrap()));
        assert_eq!(forwarded(&HeaderMap::new()), None);
    }
}
