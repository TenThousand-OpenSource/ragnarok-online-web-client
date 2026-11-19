//! A book an NPC's script has the player read (`readbook`): the one packet of it. The layout is rAthena's
//! (`clif_readbook` in `src/map/clif.cpp`); the client's reader is `CGameMode::ReceiveBookOpen`. The client
//! answers nothing: a book is read out of its own files (`crate::books`).

use super::bytes::long;

/// ZC_READ_BOOK (0x0294, 10 bytes): `u32 book, u32 page`. The book is the number of an item, whose name is
/// the book's title and by which its file is named; page 0 asks for the page the bookmark was left at.
pub const ZC_READ_BOOK: u16 = 0x0294;

/// The book and the page of a ZC_READ_BOOK.
pub fn read_book(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_book_and_the_page() {
        let mut packet = vec![0x94, 0x02];
        packet.extend(11010u32.to_le_bytes());
        packet.extend(1u32.to_le_bytes());
        assert_eq!(crate::net::packet_length(ZC_READ_BOOK), Some(packet.len() as i32));
        assert_eq!(read_book(&packet), Some((11010, 1)));
        assert_eq!(read_book(&packet[..8]), None);
    }
}
