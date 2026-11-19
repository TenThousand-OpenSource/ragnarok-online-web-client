//! The guard on the login server's port: failed logins counted per address and per account name, and whoever
//! runs out of attempts turned away until the oldest ones are old enough. The game server sees every player
//! coming from the gateway's address, so only the gateway can tell who is guessing passwords.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ws::{CloseFrame, Message};
use futures_util::SinkExt;

use crate::limiter::Limiter;

/// The WebSocket close code for "too many failed logins"; the reason is how many seconds to wait.
pub const TOO_MANY_ATTEMPTS: u16 = 4029;

pub(super) const CA_LOGIN: [u8; 2] = [0x64, 0x00];
const AC_ACCEPT_LOGIN: [u8; 2] = [0xc4, 0x0a];
const AC_REFUSE_LOGIN: [u8; 2] = [0x3e, 0x08];

/// Stops password guessing on the login server's port: failed logins are counted per address and per account
/// name, and whoever runs out of attempts is turned away until the oldest ones are old enough.
pub struct LoginGuard {
    pub port: u16,
    pub(super) by_address: Limiter,
    pub(super) by_account: Limiter,
}

impl LoginGuard {
    /// Ten failures per address and thirty per account name in ten minutes. An account's limit is the wider one:
    /// it only has to stop guessing spread over many addresses, and a tight one would let anyone lock a player
    /// out by failing on purpose.
    pub fn new(port: u16) -> Self {
        Self::with_limits(port, 10, 30, Duration::from_secs(600))
    }

    pub fn with_limits(port: u16, per_address: usize, per_account: usize, window: Duration) -> Self {
        Self { port, by_address: Limiter::new(per_address, window), by_account: Limiter::new(per_account, window) }
    }

    /// Gives an account its attempts back: its password was just reset by its owner.
    pub fn forgive_account(&self, name: &str) {
        self.by_account.clear(&name.to_ascii_lowercase());
    }
}

/// One login connection as the guard sees it.
pub(super) struct Watch {
    pub(super) guard: Arc<LoginGuard>,
    pub(super) address: String,
    /// The account name of the login request, once it has passed.
    pub(super) account: Mutex<Option<String>>,
}

pub(super) async fn turn_away<S>(socket: &mut S, wait: Duration)
where
    S: SinkExt<Message> + Unpin,
{
    let frame = CloseFrame { code: TOO_MANY_ATTEMPTS, reason: wait.as_secs().max(1).to_string().into() };
    let _ = socket.send(Message::Close(Some(frame))).await;
}

/// The account name in a login request (CA_LOGIN: `u16 id, u32 version, char[24] name, ...`), lowercased as the
/// login server compares it.
pub(super) fn login_name(request: &[u8]) -> String {
    let name = &request[6..30];
    let end = name.iter().position(|&byte| byte == 0).unwrap_or(name.len());
    String::from_utf8_lossy(&name[..end]).to_ascii_lowercase()
}

/// Counts the login server's answer once enough of it has arrived. Returns whether it is done with it.
pub(super) fn judge(watch: &Watch, answer: &[u8]) -> bool {
    if answer.len() < 2 {
        return false;
    }
    let account = watch.account.lock().unwrap().clone();
    let now = Instant::now();
    if answer[..2] == AC_ACCEPT_LOGIN {
        watch.guard.by_address.clear(&watch.address);
        if let Some(account) = &account {
            watch.guard.by_account.clear(account);
        }
    } else if answer[..2] == AC_REFUSE_LOGIN {
        // `u32 error`: 0 is an unknown account and 1 a wrong password. Other refusals (a ban, an expired account)
        // are not guesses.
        if answer.len() < 6 {
            return false;
        }
        if answer[2..6] == [0, 0, 0, 0] || answer[2..6] == [1, 0, 0, 0] {
            watch.guard.by_address.hit(&watch.address, now);
            if let Some(account) = &account {
                watch.guard.by_account.hit(account, now);
            }
        }
    }
    true
}
