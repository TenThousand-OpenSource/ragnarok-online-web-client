//! A packet of the map server that none of the readers took: counted for the developer's list (`/packets`).

use super::*;
use crate::net::unread;

impl App {
    /// Counts a packet nobody read, by its number. The first of a number is told in the browser's console, with
    /// what rAthena calls it; the list is drawn anew if it is open.
    pub(super) fn unread_packet(&mut self, id: u16, packet: &[u8]) {
        if self.unread.tally.count(id, packet.len()) {
            web_sys::console::warn_1(&format!("packet not read: {id:#06x} {} ({} bytes)", unread::name(id).unwrap_or("?"), packet.len()).into());
        }
        if self.unread.open {
            self.redraw(&[&wnd::UNREAD]);
        }
    }
}
