//! A merchant's shop (vending): opening one's own out of the cart, what is left in it and what was sold, the
//! sign over a vendor's head, a vendor's list, and buying from it.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets.hpp`, `packets_struct.hpp`;
//! `clif_openvendingreq`, `clif_openvending`, `clif_openvending_ack`, `clif_vendingreport`,
//! `clif_showvendingboard`, `clif_closevendingboard`, `clif_vendinglist`, `clif_buyvending` and
//! `clif_parse_OpenVending`, `clif_parse_PurchaseReq2` in `clif.cpp`). The rules of prices, which are the
//! client's and no packets, are in `crate::vending`; what the client sends is in `vending/requests.rs`.

use super::bytes::{body, long, text, word};
use super::inventory::{Amount, Identified, Item, Layout};

use crate::vending::SLOTS;

/// ZC_OPENSTORE: the skill Vending was used: a shop of so many kinds of things may be made.
pub const ZC_OPENSTORE: u16 = 0x012d;
/// ZC_STORE_ENTRY and ZC_DISAPPEAR_ENTRY: a vendor's sign is put up, and taken down.
pub const ZC_STORE_ENTRY: u16 = 0x0131;
pub const ZC_DISAPPEAR_ENTRY: u16 = 0x0132;
/// ZC_PC_PURCHASE_RESULT_FROMMC: what became of buying a thing.
pub const ZC_PC_PURCHASE_RESULT_FROMMC: u16 = 0x0135;
/// ZC_DELETEITEM_FROM_MCSTORE: so many of a thing of one's own shop were sold.
pub const ZC_DELETEITEM_FROM_MCSTORE: u16 = 0x09e5;
/// ZC_ACK_OPENSTORE2: whether the shop was opened.
pub const ZC_ACK_OPENSTORE2: u16 = 0x0a28;
/// ZC_PC_PURCHASE_ITEMLIST_FROMMC: what a vendor sells.
pub const ZC_PC_PURCHASE_ITEMLIST_FROMMC: u16 = 0x0b3d;
/// ZC_PC_PURCHASE_MYITEMLIST: what one's own shop sells.
pub const ZC_PC_PURCHASE_MYITEMLIST: u16 = 0x0b40;

/// A thing a shop sells: what it costs, and the thing with how many are left (`index` is its place in the
/// vendor's cart, which a purchase names).
#[derive(Debug, Clone, PartialEq)]
pub struct Sold {
    pub price: u32,
    pub item: Item,
}

/// A thing of ZC_PC_PURCHASE_MYITEMLIST and one of ZC_PC_PURCHASE_ITEMLIST_FROMMC, each counted from its
/// price. From its eighth byte on a thing is the same in both lists: `u8 type`, `u32 item`, `u8 identified`,
/// `u8 damaged`, `u32 card[4]`.
const MINE: Layout =
    Layout {
        index: Some(4), id: 9, kind: 8, amount: Amount::Word(6), wearable: None, worn: None,
        refine: Some(56), identified: Identified::Byte(13), cards: 15,
    };
const LISTED: Layout =
    Layout {
        index: Some(6), id: 9, kind: 8, amount: Amount::Word(4), wearable: Some(56), worn: None,
        refine: Some(62), identified: Identified::Byte(13), cards: 15,
    };

/// ZC_OPENSTORE (0x012d, 4 bytes): `u16 slots`, kept within what the client allows.
pub fn slots(packet: &[u8]) -> Option<u16> {
    Some(word(packet, 2)?.clamp(*SLOTS.start(), *SLOTS.end()))
}

/// ZC_PC_PURCHASE_MYITEMLIST (0x0b40): `u16 length`, `u32 account`, then 58 bytes a thing: `u32 price`,
/// `i16 index`, `i16 amount`, `u8 type`, `u32 item`, `u8 identified`, `u8 damaged`, `u32 card[4]`, five
/// options of 5 bytes, `u8 refine`, `u8 grade`.
pub fn mine(packet: &[u8]) -> Option<Vec<Sold>> {
    body(packet, 8)?.chunks_exact(58).map(|record| Some(Sold { price: long(record, 0)?, item: Item::read(record, &MINE)? })).collect()
}

/// A vendor's list: whose it is, the number the purchase must give back, and what is sold.
#[derive(Debug, Clone, PartialEq)]
pub struct Listed {
    pub vendor: u32,
    pub shop: u32,
    pub sold: Vec<Sold>,
}

/// ZC_PC_PURCHASE_ITEMLIST_FROMMC (0x0b3d): `u16 length`, `u32 account`, `u32 shop`, then 64 bytes a thing:
/// `u32 price`, `u16 amount`, `i16 index`, `u8 type`, `u32 item`, `u8 identified`, `u8 damaged`,
/// `u32 card[4]`, five options of 5 bytes, `u32 location`, `u16 view`, `u8 refine`, `u8 grade`.
pub fn listed(packet: &[u8]) -> Option<Listed> {
    let sold = body(packet, 12)?
        .chunks_exact(64)
        .map(|record| Some(Sold { price: long(record, 0)?, item: Item::read(record, &LISTED)? }))
        .collect::<Option<Vec<Sold>>>()?;
    Some(Listed { vendor: long(packet, 4)?, shop: long(packet, 8)?, sold })
}

/// ZC_STORE_ENTRY (0x0131, 86 bytes): `u32 account`, `char title[80]`.
pub fn sign(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 2)?, text(packet.get(6..86)?)))
}

/// ZC_DISAPPEAR_ENTRY (0x0132, 6 bytes): `u32 account`.
pub fn sign_gone(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// ZC_DELETEITEM_FROM_MCSTORE (0x09e5, 18 bytes): `u16 index`, `u16 amount`, `u32 buyer`, `u32 date`,
/// `i32 zeny`: so many of the thing at that place of the cart were sold. Gives back the place and how many.
pub fn sale(packet: &[u8]) -> Option<(u16, u16)> {
    packet.get(..18)?;
    Some((word(packet, 2)?, word(packet, 4)?))
}

/// What ZC_ACK_OPENSTORE2 (0x0a28, 3 bytes: `u8 result`) says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opened {
    Open,
    /// The client's line 2639.
    Failed,
    /// Not at this place: the client's line 3229.
    NotHere,
}

pub fn opened(packet: &[u8]) -> Option<Opened> {
    match *packet.get(2)? {
        0 => Some(Opened::Open),
        1 => Some(Opened::Failed),
        3 => Some(Opened::NotHere),
        _ => None,
    }
}

/// ZC_PC_PURCHASE_RESULT_FROMMC (0x0135, 7 bytes): `u16 index`, `u16 amount`, `u8 result`.
pub fn purchase_result(packet: &[u8]) -> Option<(u16, u16, u8)> {
    Some((word(packet, 2)?, word(packet, 4)?, *packet.get(6)?))
}

mod requests;
pub use requests::*;

#[cfg(test)]
mod tests;
