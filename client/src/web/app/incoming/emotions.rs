//! The emotions shown over heads.

use super::*;
use crate::net::emotions;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn emotion_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // ZC_EMOTION: someone shows an emotion (the client's handler gives it to the actor as it is).
            emotions::ZC_EMOTION => {
                if let (Some((unit, emotion)), Some(account)) = (emotions::shown(packet), &self.account) {
                    world::emotes(unit, account.account_id, emotion);
                }
            }
            _ => return false,
        }
        true
    }
}
