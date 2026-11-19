//! A deal between two players: asking for one, what each side puts into it, closing one's side, the trade
//! itself and calling it off. The layouts are rAthena's for PACKETVER 20260219 (`clif_trade*` in `clif.cpp`,
//! `packets.hpp`, `packets_struct.hpp`).

use super::bytes::{text, u32_at};

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

/// The most zeny one side puts into a deal (the client's `CUIExchangeUI::OnProcess`).
pub const MOST_ZENY: u32 = 1_000_000_000;

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
    Some((text(&packet[2..26]), u16::from_le_bytes([packet[30], packet[31]])))
}

/// ZC_ACK_EXCHANGE_ITEM (0x01f5, 9 bytes): `u8 result` (`answer`), `u32 character`, `u16 level`: the answer, and
/// the other's level.
pub fn answered(packet: &[u8]) -> Option<(u8, u16)> {
    let packet = packet.get(..9)?;
    Some((packet[2], u16::from_le_bytes([packet[7], packet[8]])))
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
        Put::Item { id, kind: packet[6], amount, identified: packet[11] != 0, refine: packet[60], cards: [u32_at(packet, 13), u32_at(packet, 17), u32_at(packet, 21), u32_at(packet, 25)] }
    })
}

/// ZC_ACK_ADD_EXCHANGE_ITEM (0x00ea, 5 bytes): `u16 index`, `u8 result` (`added`): where in the inventory the
/// thing is (0 for zeny), and whether it was taken.
pub fn add_answered(packet: &[u8]) -> Option<(u16, u8)> {
    let packet = packet.get(..5)?;
    Some((u16::from_le_bytes([packet[2], packet[3]]), packet[4]))
}

/// CZ_REQ_EXCHANGE_ITEM (0x00e4, 6 bytes): `u32 account`: that player is asked for a deal.
pub fn request(account: u32) -> Vec<u8> {
    [&[0xe4, 0x00][..], &account.to_le_bytes()].concat()
}

/// CZ_ACK_EXCHANGE_ITEM (0x00e6, 3 bytes): `u8 answer`: `answer::ACCEPT` or `answer::CANCEL`.
pub fn reply(answer: u8) -> Vec<u8> {
    vec![0xe6, 0x00, answer]
}

/// CZ_ADD_EXCHANGE_ITEM (0x00e8, 8 bytes): `u16 index`, `u32 amount`: so many of the thing carried there go into
/// the deal; index 0 is zeny, and the amount all the zeny put in.
pub fn add(index: u16, amount: u32) -> Vec<u8> {
    [&[0xe8, 0x00][..], &index.to_le_bytes(), &amount.to_le_bytes()].concat()
}

/// CZ_CONCLUDE_EXCHANGE_ITEM (0x00eb): the player's side is closed.
pub fn conclude() -> Vec<u8> {
    vec![0xeb, 0x00]
}

/// CZ_CANCEL_EXCHANGE_ITEM (0x00ed): the deal is called off.
pub fn cancel() -> Vec<u8> {
    vec![0xed, 0x00]
}

/// CZ_EXEC_EXCHANGE_ITEM (0x00ef): the trade, once both sides are closed.
pub fn trade() -> Vec<u8> {
    vec![0xef, 0x00]
}

/// The zeny typed into the deal's window, as the client reads it (`CUIExchangeUI::SubmitZeny`): digits, with
/// commas between them or not; more than the character has is all it has; none for anything else.
pub fn zeny_typed(text: &str, has: u32) -> Option<u32> {
    let digits: String = text.trim().chars().filter(|letter| *letter != ',').collect();
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let amount = digits.parse::<u64>().unwrap_or(u64::MAX);
    Some(if amount > u64::from(has) { has } else { amount as u32 }.min(MOST_ZENY))
}

/// The colour an amount of zeny is written in, by how large it is (the first colour of the client's
/// `PriceColor`, which it keeps blue first: written here as the page wants it).
pub fn zeny_colour(amount: u32) -> &'static str {
    match amount {
        1_000_000_000.. => "#ff007f",
        100_000_000.. => "#c8c864",
        10_000_000.. => "#ff0000",
        1_000_000.. => "#00ff00",
        100_000.. => "#0000ff",
        10_000.. => "#ff1eff",
        1_000.. => "#ffff00",
        100.. => "#00ffff",
        10.. => "#c800c8",
        _ => "#00ffff",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeny_is_coloured_by_how_much_it_is() {
        assert_eq!((zeny_colour(0), zeny_colour(99), zeny_colour(5_000), zeny_colour(10_000_000), zeny_colour(999_999)), ("#00ffff", "#c800c8", "#ffff00", "#ff0000", "#0000ff"));
    }

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
    fn zeny_is_digits_with_or_without_commas() {
        assert_eq!(zeny_typed("1,500", 2000), Some(1500));
        assert_eq!(zeny_typed(" 0 ", 2000), Some(0));
        assert_eq!(zeny_typed("5000", 2000), Some(2000));
        assert_eq!(zeny_typed("2,000,000,000", u32::MAX), Some(MOST_ZENY));
        assert_eq!(zeny_typed("12a", 2000), None);
        assert_eq!(zeny_typed("", 2000), None);
        assert_eq!(zeny_typed("-5", 2000), None);
    }
}
