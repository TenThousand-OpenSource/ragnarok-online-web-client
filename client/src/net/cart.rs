//! The merchant's cart: what lies in it, how full it is, things moved between it and the inventory, and the choosing of its look.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets.hpp`, `packets_struct.hpp`; `clif_cartlist`,
//! `clif_cart_additem`, `clif_cart_delitem`, `clif_cart_additem_ack`, `clif_clearcart`, `clif_SelectCart` in
//! `clif.cpp`). At this version the cart's list comes as the inventory's does (ZC_INVENTORY_START, 0x0b08, then
//! 0x0b09 and 0x0b39, each with the kind of list `LIST`), and that there is a cart at all, and which, is told
//! by a state (`EFST_ON_PUSH_CART`, whose first value is the cart's kind): the bits of the old effect state
//! are not set.

use super::bytes::{Writer, body, long, word};
use super::inventory::Item;

/// ZC_NOTIFY_CARTITEM_COUNTINFO: how full the cart is.
pub const ZC_NOTIFY_CARTITEM_COUNTINFO: u16 = 0x0121;
/// ZC_DELETE_ITEM_FROM_CART: so many of a thing left the cart.
pub const ZC_DELETE_ITEM_FROM_CART: u16 = 0x0125;
/// ZC_CARTOFF: the cart is gone.
pub const ZC_CARTOFF: u16 = 0x012b;
/// ZC_ACK_ADDITEM_TO_CART: why a thing did not go into the cart.
pub const ZC_ACK_ADDITEM_TO_CART: u16 = 0x012c;
/// ZC_SELECTCART: the looks a cart may be given (the skill Cart Decoration).
pub const ZC_SELECTCART: u16 = 0x097f;
/// ZC_ADD_ITEM_TO_CART: a thing went into the cart.
pub const ZC_ADD_ITEM_TO_CART: u16 = 0x0b45;

/// Which list a ZC_INVENTORY packet is of: the cart (rAthena's `INVTYPE_CART`).
pub const LIST: u8 = 1;
/// The state that says someone pushes a cart (`EFST_ON_PUSH_CART`); its first value is the cart's kind.
pub const PUSHES_CART: u16 = 673;

/// How full the cart is: the kinds of things in it and how many fit, what they weigh and what it bears. The
/// weights are the server's, ten times what the client writes (`cartWeight = curWeight / 10`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Fill {
    pub kinds: u16,
    pub most_kinds: u16,
    pub weight: u32,
    pub most_weight: u32,
}

/// ZC_NOTIFY_CARTITEM_COUNTINFO (0x0121, 14 bytes): `i16 kinds`, `i16 most`, `i32 weight`, `i32 most weight`.
pub fn fill(packet: &[u8]) -> Option<Fill> {
    Some(Fill { kinds: word(packet, 2)?, most_kinds: word(packet, 4)?, weight: long(packet, 6)?, most_weight: long(packet, 10)? })
}

/// ZC_ADD_ITEM_TO_CART (0x0b45, 58 bytes): `i16 index`, `i32 amount`, `u32 item`, `u8 type`, `u8 identified`,
/// `u8 damaged`, `u32 card[4]`, five options of 5 bytes, `u8 refine`, `u8 grade`: the layout of a thing put
/// into the storage (`storage::stored`).
pub fn added(packet: &[u8]) -> Option<Item> {
    super::storage::stored(packet)
}

/// ZC_DELETE_ITEM_FROM_CART (0x0125, 8 bytes): `u16 index`, `i32 amount`.
pub fn removed(packet: &[u8]) -> Option<(u16, u32)> {
    Some((word(packet, 2)?, long(packet, 4)?))
}

/// ZC_ACK_ADDITEM_TO_CART (0x012c, 3 bytes): `u8 result`. The line of the client's message table it says in
/// red (`MSI_OVER_WEIGHT` for 0, `MSI_INSUFFICIENT_OVER_COUNT` for 1), with the words until the table is there.
pub fn refused(packet: &[u8]) -> Option<(usize, &'static str)> {
    match *packet.get(2)? {
        0 => Some((56, "You are over the weight limit.")),
        1 => Some((221, "You cannot carry more than that.")),
        _ => None,
    }
}

/// ZC_SELECTCART (0x097f): `u16 length`, `u32 identity`, then a byte for each look offered. Gives back what
/// the answer must name, and the looks.
pub fn looks(packet: &[u8]) -> Option<(u32, Vec<u8>)> {
    Some((long(packet, 4)?, body(packet, 8)?.to_vec()))
}

/// CZ_MOVE_ITEM_FROM_BODY_TO_CART (0x0126) and CZ_MOVE_ITEM_FROM_CART_TO_BODY (0x0127), 8 bytes: `u16 index`
/// where the thing is kept now, `i32 amount`. (Between the cart and the storage: `net::storage`, which has
/// the one writer of them all.)
pub fn put(index: u16, amount: u32) -> Vec<u8> {
    super::storage::moved(0x0126, index, amount)
}

pub fn take(index: u16, amount: u32) -> Vec<u8> {
    super::storage::moved(0x0127, index, amount)
}

/// CZ_REQ_CHANGECART (0x01af, 4 bytes): `u16 look`: the cart is to look so (the skill Change Cart; the
/// client's `MM_SELECT_CART` with no identity).
pub fn change(look: u8) -> Vec<u8> {
    Writer::new(0x01af).u16(u16::from(look)).done()
}

/// CZ_SELECTCART (0x0980, 7 bytes): `u32 identity`, `u8 look`: the answer to ZC_SELECTCART.
pub fn select(identity: u32, look: u8) -> Vec<u8> {
    Writer::new(0x0980).u32(identity).u8(look).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::assert_lengths;
    use crate::net::packet_length;

    #[test]
    fn reads_how_full_the_cart_is() {
        let mut packet = vec![0x21, 0x01, 3, 0, 100, 0];
        packet.extend(1250u32.to_le_bytes());
        packet.extend(80_000u32.to_le_bytes());
        assert_eq!(fill(&packet), Some(Fill { kinds: 3, most_kinds: 100, weight: 1250, most_weight: 80_000 }));
        assert_eq!(fill(&packet[..13]), None);
    }

    #[test]
    fn reads_a_thing_going_in_and_coming_out() {
        let mut packet = vec![0x45, 0x0b, 4, 0];
        packet.extend(12u32.to_le_bytes());
        packet.extend(501u32.to_le_bytes());
        packet.extend([0, 1, 0]);
        packet.resize(58, 0);
        let item = added(&packet).unwrap();
        assert_eq!((item.index, item.amount, item.id, item.identified), (4, 12, 501, true));
        assert_eq!(removed(&[0x25, 0x01, 4, 0, 5, 0, 0, 0]), Some((4, 5)));
    }

    #[test]
    fn a_refusal_names_its_line() {
        assert_eq!(refused(&[0x2c, 0x01, 0]).map(|(line, _)| line), Some(56));
        assert_eq!(refused(&[0x2c, 0x01, 1]).map(|(line, _)| line), Some(221));
        assert_eq!(refused(&[0x2c, 0x01, 7]), None);
    }

    #[test]
    fn reads_the_looks_offered() {
        let mut packet = vec![0x7f, 0x09, 11, 0];
        packet.extend(2_000_001u32.to_le_bytes());
        packet.extend([10, 11, 12]);
        assert_eq!(looks(&packet), Some((2_000_001, vec![10, 11, 12])));
    }

    #[test]
    fn the_packets_have_the_lengths_the_server_expects() {
        for packet in [put(3, 70_000), take(3, 70_000)] {
            assert_eq!(packet[2..], [3, 0, 0x70, 0x11, 1, 0]);
            assert_eq!(packet_length(u16::from_le_bytes([packet[0], packet[1]])), Some(8));
        }
        assert_eq!((put(1, 1)[0], take(1, 1)[0]), (0x26, 0x27));
        assert_eq!(change(3), [0xaf, 0x01, 3, 0]);
        assert_eq!(select(7, 11), [0x80, 0x09, 7, 0, 0, 0, 11]);
        assert_lengths(&[change(1), select(1, 10)]);
        for (id, length) in [(ZC_NOTIFY_CARTITEM_COUNTINFO, 14), (ZC_DELETE_ITEM_FROM_CART, 8), (ZC_CARTOFF, 2), (ZC_ACK_ADDITEM_TO_CART, 3), (ZC_SELECTCART, -1), (ZC_ADD_ITEM_TO_CART, 58)] {
            assert_eq!(packet_length(id), Some(length), "{id:#06x}");
        }
    }
}
