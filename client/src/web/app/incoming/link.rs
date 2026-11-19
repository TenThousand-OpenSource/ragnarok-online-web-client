//! What the servers say of the connection itself: that the map server is still there, and why a server ends
//! the session.

use super::*;
use crate::net::link;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. ZC_PING_LIVE is taken and
    /// nothing done with it, as in the client: that something came is what the watch goes by
    /// (`actions/link.rs`).
    pub(super) fn link_packet(&mut self, id: u16, _packet: &[u8]) -> bool {
        id == link::ZC_PING_LIVE
    }

    /// Why a server ended the session, as the client words it: a line of its message table, in the game with
    /// the reason's number after it in brackets (`CGameMode::ReceiveNotifyBan`), on the way in alone
    /// (`CLoginMode::ReceiveNotifyBan`). Until the table is read, this client's own words for it.
    pub(super) fn ban_words(&self, reason: u8) -> String {
        let (line, bracketed) = link::ban_line(reason, self.server == Server::Map);
        match data::message(line) {
            Some(words) if bracketed => format!("{}({reason})", crate::i18n::tr(&words)),
            Some(words) => crate::i18n::tr(&words).to_owned(),
            None => fill(Key::DisconnectedCode.text(self.lang), &[&reason.to_string()]),
        }
    }
}
