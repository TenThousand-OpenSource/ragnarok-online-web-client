//! The connection: what comes of the socket, and keeping it alive.

use super::*;
use crate::net::link;

impl App {
    /// Something came of the connection: it is open, bytes arrived, or it closed.
    pub(super) fn socket_event(&mut self, generation: u32, event: SocketEvent) {
        match event {
            // The watch on a silent connection counts from when it opened, and from whatever comes
            // (the client's `m_lastReceiveTick`).
            SocketEvent::Open => {
                self.link.heard_at = js_sys::Date::now();
                self.connected();
            }
            SocketEvent::Bytes(bytes) => {
                self.link.heard_at = js_sys::Date::now();
                self.received(generation, &bytes);
            }
            SocketEvent::Closed(TOO_MANY_ATTEMPTS, reason) => {
                let wait = minutes(reason.parse().unwrap_or(60));
                self.leave(Some(fill(Key::TooManyLogins.text(self.lang), &[&wait])));
            }
            SocketEvent::Closed(..) => {
                let text = if self.server == Server::Login && self.account.is_none() { Key::ConnectFailed } else { Key::Disconnected };
                self.leave(Some(text.text(self.lang).to_owned()));
            }
        }
    }

    /// The timer that keeps the connection alive.
    pub(super) fn keep_alive(&mut self) {
        match (&self.server, &self.socket, &self.account) {
            (Server::Char, Some(socket), Some(account)) => socket.send(&packets::ping(account.account_id)),
            (Server::Map, Some(socket), _) if self.game.is_some() => {
                // CZ_REQUEST_TIME, as the client's `CGameMode::SendSyncRequest`: the answer corrects the clock
                // the world shows walks by (`world::clock_told`).
                socket.send(&packets::tick(now()));
                // Every third time, and at once on coming in: how many play (CZ_REQ_USER_COUNT, 0x00c1,
                // what the client's `/who` sends).
                if self.ticks % 3 == 0 {
                    socket.send(&crate::net::commands::user_count());
                }
                self.ticks = self.ticks.wrapping_add(1);
            }
            _ => {}
        }
    }

    /// The client's watch on the connection to the map server (`CPingLiveSchedule::Process`, every five
    /// seconds): when nothing has come for a minute the connection is closed and the player told (the
    /// client's `UIDisconnectedServerMsgWnd`, which says line 2 of the message table, "Disconnected from
    /// Server."); when nothing was sent for ten seconds, CZ_PING_LIVE is. What is shown for a time (an
    /// announcement across the top of the screen) is looked over at the same beat.
    pub(super) fn watch(&mut self) {
        self.announcements_pass();
        let (Server::Map, Some(socket), true) = (&self.server, &self.socket, self.game.is_some()) else { return };
        match crate::link::watch(js_sys::Date::now(), self.link.heard_at, socket.sent_at()) {
            crate::link::Watch::Close => self.leave(Some(Key::Disconnected.text(self.lang).to_owned())),
            crate::link::Watch::Ping => socket.send(&link::ping_live()),
            crate::link::Watch::Wait => {}
        }
    }
}
