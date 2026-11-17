//! The connection: what comes of the socket, and keeping it alive.

use super::*;

impl App {
    /// Something came of the connection: it is open, bytes arrived, or it closed.
    pub(super) fn socket_event(&mut self, generation: u32, event: SocketEvent) {
        match event {
            SocketEvent::Open => self.connected(),
            SocketEvent::Bytes(bytes) => self.received(generation, &bytes),
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
                socket.send(&packets::tick(now()));
                // Every half a minute, and at once on coming in: how many play (CZ_REQ_USER_COUNT, 0x00c1,
                // what the client's `/who` sends).
                if self.ticks % 3 == 0 {
                    socket.send(&0x00c1u16.to_le_bytes());
                }
                self.ticks = self.ticks.wrapping_add(1);
            }
            _ => {}
        }
    }
}
