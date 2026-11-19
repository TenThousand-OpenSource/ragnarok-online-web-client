//! A deal between two players: asking for one, what each side puts into it, closing one's side, the trade
//! itself and calling it off. The layouts are rAthena's for PACKETVER 20260219 (`clif_trade*` in `clif.cpp`,
//! `packets.hpp`, `packets_struct.hpp`). The rules of the zeny put into one are no packets: they are in
//! `crate::exchange`.

use super::bytes::{Writer, text, u16_at, u32_at};
use super::items::cards;

/// The rules of a deal are named here as before.
pub use crate::exchange::{MOST_ZENY, zeny_colour, zeny_typed};

/// ZC_REQ_EXCHANGE_ITEM (0x01f4): someone asks for a deal.
pub const ZC_REQ_EXCHANGE_ITEM: u16 = 0x01f4;
/// ZC_ACK_EXCHANGE_ITEM (0x01f5): what became of a request for a deal (`answer`).
pub const ZC_ACK_EXCHANGE_ITEM: u16 = 0x01f5;
/// ZC_ADD_EXCHANGE_ITEM (0x0b42): the other side put something in.
pub const ZC_ADD_EXCHANGE_ITEM: u16 = 0x0b42;
/// ZC_ACK_ADD_EXCHANGE_ITEM (0x00ea): whether what the player put in was taken (`added`).
pub const ZC_ACK_ADD_EXCHANGE_ITEM: u16 = 0x00ea;
/// ZC_CONCLUDE_EXCHANGE_ITEM (0x00ec, 3 bytes): `u8 who`: a side is closed, the player's own (0) or the other (1).
pub const ZC_CONCLUDE_EXCHANGE_ITEM: u16 = 0x00ec;
/// ZC_CANCEL_EXCHANGE_ITEM (0x00ee): the deal is called off.
pub const ZC_CANCEL_EXCHANGE_ITEM: u16 = 0x00ee;
/// ZC_EXEC_EXCHANGE_ITEM (0x00f0, 3 bytes): `u8 failed`: the deal is done, or failed.
pub const ZC_EXEC_EXCHANGE_ITEM: u16 = 0x00f0;
/// ZC_EXCHANGEITEM_UNDO (0x00f1): what the player put in is the player's again.
pub const ZC_EXCHANGEITEM_UNDO: u16 = 0x00f1;

/// What ZC_ACK_EXCHANGE_ITEM answers, and what CZ_ACK_EXCHANGE_ITEM is answered with (rAthena's
/// `e_ack_trade_response`).
pub mod answer {
    pub const TOO_FAR: u8 = 0;
    pub const NO_SUCH_CHARACTER: u8 = 1;
    /// The other is in a deal already.
    pub const FAILED: u8 = 2;
    pub const ACCEPT: u8 = 3;
    pub const CANCEL: u8 = 4;
    pub const BUSY: u8 = 5;
}

/// What ZC_ACK_ADD_EXCHANGE_ITEM answers (rAthena's `e_exitem_add_result`).
pub mod added {
    pub const DONE: u8 = 0;
    pub const OVERWEIGHT: u8 = 1;
    /// The deal is no more.
    pub const CLOSED: u8 = 2;
    /// The other cannot carry more kinds of things.
    pub const TOO_MANY: u8 = 3;
    /// The other cannot carry more of this thing.
    pub const TOO_MANY_OF_IT: u8 = 4;
}

/// What the other side put into the deal: so much zeny, or so many of a thing.
#[derive(Debug, Clone, PartialEq)]
pub enum Put {
    Zeny(u32),
    Item { id: u32, kind: u8, amount: u32, identified: bool, refine: u8, cards: [u32; 4] },
}

/// ZC_REQ_EXCHANGE_ITEM (0x01f4, 32 bytes): `char name[24]`, `u32 character`, `u16 level`: who asks, and their
/// level.
pub fn asked(packet: &[u8]) -> Option<(String, u16)> {
    let packet = packet.get(..32)?;
    Some((text(&packet[2..26]), u16_at(packet, 30)))
}

/// ZC_ACK_EXCHANGE_ITEM (0x01f5, 9 bytes): `u8 result` (`answer`), `u32 character`, `u16 level`: the answer, and
/// the other's level.
pub fn answered(packet: &[u8]) -> Option<(u8, u16)> {
    let packet = packet.get(..9)?;
    Some((packet[2], u16_at(packet, 7)))
}

/// ZC_ADD_EXCHANGE_ITEM (0x0b42, 62 bytes): `u32 item`, `u8 type`, `i32 amount`, `u8 identified`, `u8 damaged`,
/// `u32 cards[4]`, 25 bytes of random options, `u32 location`, `u16 look`, `u8 refine`, `u8 grade`. No item
/// means zeny, and the amount is all the zeny the other has put in.
pub fn put(packet: &[u8]) -> Option<Put> {
    let packet = packet.get(..62)?;
    let (id, amount) = (u32_at(packet, 2), u32_at(packet, 7));
    Some(if id == 0 {
        Put::Zeny(amount)
    } else {
        Put::Item { id, kind: packet[6], amount, identified: packet[11] != 0, refine: packet[60], cards: cards(packet, 13) }
    })
}

/// ZC_ACK_ADD_EXCHANGE_ITEM (0x00ea, 5 bytes): `u16 index`, `u8 result` (`added`): where in the inventory the
/// thing is (0 for zeny), and whether it was taken.
pub fn add_answered(packet: &[u8]) -> Option<(u16, u8)> {
    let packet = packet.get(..5)?;
    Some((u16_at(packet, 2), packet[4]))
}

/// ZC_CONCLUDE_EXCHANGE_ITEM (0x00ec, 3 bytes): `u8 who`: the side that is closed, the player's own (0) or the
/// other's (1).
pub fn concluded(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// ZC_EXEC_EXCHANGE_ITEM (0x00f0, 3 bytes): `u8 failed`: nought for a deal that is done, anything else for one
/// that failed.
pub fn executed(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// CZ_REQ_EXCHANGE_ITEM (0x00e4, 6 bytes): `u32 account`: that player is asked for a deal.
pub fn request(account: u32) -> Vec<u8> {
    Writer::new(0x00e4).u32(account).done()
}

/// CZ_ACK_EXCHANGE_ITEM (0x00e6, 3 bytes): `u8 answer`: `answer::ACCEPT` or `answer::CANCEL`.
pub fn reply(answer: u8) -> Vec<u8> {
    Writer::new(0x00e6).u8(answer).done()
}

/// CZ_ADD_EXCHANGE_ITEM (0x00e8, 8 bytes): `u16 index`, `u32 amount`: so many of the thing carried there go into
/// the deal; index 0 is zeny, and the amount all the zeny put in.
pub fn add(index: u16, amount: u32) -> Vec<u8> {
    Writer::new(0x00e8).u16(index).u32(amount).done()
}

/// CZ_CONCLUDE_EXCHANGE_ITEM (0x00eb): the player's side is closed.
pub fn conclude() -> Vec<u8> {
    Writer::new(0x00eb).done()
}

/// CZ_CANCEL_EXCHANGE_ITEM (0x00ed): the deal is called off.
pub fn cancel() -> Vec<u8> {
    Writer::new(0x00ed).done()
}

/// CZ_EXEC_EXCHANGE_ITEM (0x00ef): the trade, once both sides are closed.
pub fn trade() -> Vec<u8> {
    Writer::new(0x00ef).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_who_asks_and_what_is_answered() {
        let mut packet = vec![0xf4, 0x01];
        packet.extend(b"Boris");
        packet.resize(26, 0);
        packet.extend(150_002u32.to_le_bytes());
        packet.extend(99u16.to_le_bytes());
        assert_eq!(asked(&packet), Some(("Boris".into(), 99)));
        assert_eq!(asked(&packet[..31]), None);
        assert_eq!(answered(&[0xf5, 0x01, answer::ACCEPT, 2, 0, 0, 0, 55, 0]), Some((3, 55)));
        assert_eq!(add_answered(&[0xea, 0x00, 4, 0, added::OVERWEIGHT]), Some((4, 1)));
    }

    #[test]
    fn reads_what_the_other_puts_in() {
        let mut packet = vec![0x42, 0x0b];
        packet.extend(1201u32.to_le_bytes());
        packet.push(5);
        packet.extend(1i32.to_le_bytes());
        packet.extend([1, 0]);
        for card in [4001u32, 0, 0, 0] {
            packet.extend(card.to_le_bytes());
        }
        packet.resize(60, 0);
        packet.extend([7, 0]);
        assert_eq!(packet.len(), 62);
        assert_eq!(put(&packet), Some(Put::Item { id: 1201, kind: 5, amount: 1, identified: true, refine: 7, cards: [4001, 0, 0, 0] }));

        let mut zeny = vec![0x42, 0x0b, 0, 0, 0, 0, 0];
        zeny.extend(5000i32.to_le_bytes());
        zeny.resize(62, 0);
        assert_eq!(put(&zeny), Some(Put::Zeny(5000)));
        assert_eq!(put(&zeny[..61]), None);
    }

    #[test]
    fn writes_what_the_client_sends() {
        assert_eq!(request(2_000_002), [0xe4, 0x00, 0x82, 0x84, 0x1e, 0x00]);
        assert_eq!(reply(answer::CANCEL), [0xe6, 0x00, 4]);
        assert_eq!(add(4, 10), [0xe8, 0x00, 4, 0, 10, 0, 0, 0]);
        assert_eq!(add(0, 5000), [0xe8, 0x00, 0, 0, 0x88, 0x13, 0, 0]);
        assert_eq!((conclude(), cancel(), trade()), (vec![0xeb, 0x00], vec![0xed, 0x00], vec![0xef, 0x00]));
    }

    #[test]
    fn reads_which_side_is_closed_and_how_the_deal_ended() {
        assert_eq!((concluded(&[0xec, 0x00, 1]), concluded(&[0xec, 0x00, 0]), concluded(&[0xec, 0x00])), (Some(1), Some(0), None));
        assert_eq!((executed(&[0xf0, 0x00, 0]), executed(&[0xf0, 0x00, 1]), executed(&[0xf0, 0x00])), (Some(0), Some(1), None));
    }
}
