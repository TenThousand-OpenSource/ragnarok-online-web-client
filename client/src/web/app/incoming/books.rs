//! A book an NPC's script has the player read (`crate::net::books`).

use super::*;
use crate::net::books;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn book_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // `CGameMode::ReceiveBookOpen`: the book's window shows that page of it.
            books::ZC_READ_BOOK => {
                if let Some((book, page)) = books::read_book(packet) {
                    self.book_asked(book, Reading::Shown, page as usize);
                }
            }
            _ => return false,
        }
        true
    }
}
