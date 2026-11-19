//! A merchant's shop (vending): opening one's own out of the cart, what is left in it and what was sold, the
//! sign over a vendor's head, a vendor's list, and buying from it.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets.hpp`, `packets_struct.hpp`;
//! `clif_openvendingreq`, `clif_openvending`, `clif_openvending_ack`, `clif_vendingreport`,
//! `clif_showvendingboard`, `clif_closevendingboard`, `clif_vendinglist`, `clif_buyvending` and
//! `clif_parse_OpenVending`, `clif_parse_PurchaseReq2` in `clif.cpp`). The rules of prices are the client's
//! (`UI/Shop/PriceColor.cpp`, `PriceCommission.cpp`, `UIMerchantShopMakeWnd`, `UIMerchantItemPurchaseWnd`).

use super::bytes::{long, text, word};
use super::items::Item;

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

/// The most the client lets a thing cost, the most of one thing a shop sells or a basket takes, and the
/// longest title it lets be typed (`UIMerchantShopMakeWnd`).
pub const MOST_PRICE: u32 = 1_000_000_000;
pub const MOST_AMOUNT: u32 = 30_000;
pub const TITLE: usize = 36;
/// How few and how many kinds of things the client lets a shop have, whatever the server says (the handler
/// of ZC_OPENSTORE).
pub const SLOTS: std::ops::RangeInclusive<u16> = 3..=13;

/// A thing a shop sells: what it costs, and the thing with how many are left (`index` is its place in the
/// vendor's cart, which a purchase names).
#[derive(Debug, Clone, PartialEq)]
pub struct Sold {
    pub price: u32,
    pub item: Item,
}

/// A thing as a shop's lists have it from its 13th byte on, whichever list: `u8 identified`, `u8 damaged`,
/// `u32 card[4]`; the type and the item's number stand before it at 8 and 9.
fn thing(record: &[u8], index: u16, amount: u16, refine: u8) -> Option<Item> {
    Some(Item {
        index,
        id: long(record, 9)?,
        kind: *record.get(8)?,
        amount,
        wearable: 0,
        worn: 0,
        refine,
        identified: *record.get(13)? != 0,
        cards: [long(record, 15)?, long(record, 19)?, long(record, 23)?, long(record, 27)?],
    })
}

/// ZC_OPENSTORE (0x012d, 4 bytes): `u16 slots`, kept within what the client allows.
pub fn slots(packet: &[u8]) -> Option<u16> {
    Some(word(packet, 2)?.clamp(*SLOTS.start(), *SLOTS.end()))
}

/// ZC_PC_PURCHASE_MYITEMLIST (0x0b40): `u16 length`, `u32 account`, then 58 bytes a thing: `u32 price`,
/// `i16 index`, `i16 amount`, `u8 type`, `u32 item`, `u8 identified`, `u8 damaged`, `u32 card[4]`, five
/// options of 5 bytes, `u8 refine`, `u8 grade`.
pub fn mine(packet: &[u8]) -> Option<Vec<Sold>> {
    let length = usize::from(word(packet, 2)?).min(packet.len());
    packet.get(8..length)?.chunks_exact(58).map(|record| Some(Sold { price: long(record, 0)?, item: thing(record, word(record, 4)?, word(record, 6)?, record[56])? })).collect()
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
    let length = usize::from(word(packet, 2)?).min(packet.len());
    let sold = packet
        .get(12..length)?
        .chunks_exact(64)
        .map(|record| {
            let mut item = thing(record, word(record, 6)?, word(record, 4)?, record[62])?;
            item.wearable = long(record, 56)?;
            Some(Sold { price: long(record, 0)?, item })
        })
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

/// What the chat says of a purchase that failed (the client's handler of 0x0135), for a thing called `name`
/// of which `amount` were asked for; `line` reads a line of the client's message table. Nothing for one that
/// went through, nor for a result the client says nothing of.
pub fn purchase_said(result: u8, name: &str, amount: u16, line: impl Fn(usize, &str) -> String) -> Option<String> {
    let failed = |why: usize, otherwise: &str| line(229, "%s Purchase Failed %s").replacen("%s", name, 1).replacen("%s", &line(why, otherwise), 1);
    Some(match result {
        1 => failed(55, "You do not have enough zeny."),
        2 => failed(56, "You are over your Weight Limit."),
        4 => line(240, "Buying %s has failed. It's out of stock. Current Stock %d.").replacen("%s", name, 1).replacen("%d", &amount.to_string(), 1),
        5 => line(281, "You cannot use an NPC shop while in a trade."),
        6 => line(1695, "Item purchase failed due to incorrect shop information."),
        7 => line(1797, "No sales information."),
        _ => return None,
    })
}

/// CZ_REQ_OPENSTORE2 (0x01b2): `u16 length`, `char title[80]`, `u8 open`, then for each thing `u16 index` (its
/// place in the cart), `u16 amount`, `u32 price`. Not opening after all is the same with `open` nought (the
/// client's `MM_REQ_OPENSTORE`, which also sends nought with nothing to sell).
pub fn open(title: &str, open: bool, things: &[(u16, u16, u32)]) -> Vec<u8> {
    let mut packet = vec![0u8; 85];
    packet[..2].copy_from_slice(&0x01b2u16.to_le_bytes());
    packet[2..4].copy_from_slice(&((85 + things.len() * 8) as u16).to_le_bytes());
    // 79 bytes of it at the most, so that it ends in a zero, and never half a character.
    let mut kept = title.len().min(79);
    while !title.is_char_boundary(kept) {
        kept -= 1;
    }
    packet[4..4 + kept].copy_from_slice(&title.as_bytes()[..kept]);
    packet[84] = u8::from(open && !things.is_empty());
    for (index, amount, price) in things {
        packet.extend(index.to_le_bytes());
        packet.extend(amount.to_le_bytes());
        packet.extend(price.to_le_bytes());
    }
    packet
}

/// CZ_REQ_CLOSESTORE (0x012e): one's own shop is closed.
pub fn close() -> Vec<u8> {
    0x012eu16.to_le_bytes().to_vec()
}

/// CZ_REQ_BUY_FROMMC (0x0130, 6 bytes): `u32 account`: what does that vendor sell?
pub fn ask(vendor: u32) -> Vec<u8> {
    [&0x0130u16.to_le_bytes()[..], &vendor.to_le_bytes()].concat()
}

/// CZ_PC_PURCHASE_ITEMLIST_FROMMC2 (0x0801): `u16 length`, `u32 account`, `u32 shop`, then for each thing
/// `u16 amount`, `u16 index`.
pub fn buy(vendor: u32, shop: u32, wanted: &[(u16, u16)]) -> Vec<u8> {
    let mut packet = 0x0801u16.to_le_bytes().to_vec();
    packet.extend(((12 + wanted.len() * 4) as u16).to_le_bytes());
    packet.extend(vendor.to_le_bytes());
    packet.extend(shop.to_le_bytes());
    for (index, amount) in wanted {
        packet.extend(amount.to_le_bytes());
        packet.extend(index.to_le_bytes());
    }
    packet
}

/// The colour the client writes a price in, and the colour of the rim it gives its figures where it gives
/// one (`PriceColor`, `UIWindow::DrawAmount`): by how many figures the price has.
pub fn price_colour(price: u32) -> (&'static str, Option<&'static str>) {
    match price {
        1_000_000_000.. => ("#ff007f", Some("#ff0000")),
        100_000_000.. => ("#c8c864", Some("#000000")),
        10_000_000.. => ("#ff0000", None),
        1_000_000.. => ("#00ff00", Some("#000000")),
        100_000.. => ("#0000ff", None),
        10_000.. => ("#ff1eff", None),
        1_000.. => ("#ffff00", Some("#ff0000")),
        100.. => ("#00ffff", Some("#0000ff")),
        10.. => ("#c800c8", Some("#0000ff")),
        _ => ("#00ffff", Some("#000000")),
    }
}

/// What a vendor is left of a price once the server has kept its share, as the window of a shop being made
/// writes it under the price (`PriceCommission`: a twentieth of a price over ten million).
pub fn after_commission(price: u32) -> u32 {
    if price > 10_000_000 { price - (u64::from(price) * 5 / 100) as u32 } else { price }
}

/// A price as it was typed into the window of a shop being made: its figures with the commas between them
/// left out; none for anything that is not a number (the client's line 602), and never more than the most.
pub fn price_typed(typed: &str) -> Option<u32> {
    let figures: String = typed.trim().chars().filter(|figure| *figure != ',').collect();
    if figures.is_empty() || !figures.bytes().all(|figure| figure.is_ascii_digit()) {
        return None;
    }
    Some(figures.parse::<u64>().map_or(MOST_PRICE, |price| price.min(u64::from(MOST_PRICE)) as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet_length;

    /// A thing of a list from its eighth byte on: Jellopy (909), identified, with a card in its first slot.
    fn record(head: &[u8], length: usize) -> Vec<u8> {
        let mut record = head.to_vec();
        record.push(3);
        record.extend(909u32.to_le_bytes());
        record.extend([1, 0]);
        record.extend(4001u32.to_le_bytes());
        record.resize(length, 0);
        record
    }

    #[test]
    fn reads_ones_own_shop() {
        let mut packet = vec![0x40, 0x0b, 66, 0, 1, 0, 0, 0];
        let mut thing = record(&[0x10, 0x27, 0, 0, 5, 0, 20, 0], 58);
        thing[56] = 7;
        packet.extend(thing);
        let sold = mine(&packet).unwrap();
        assert_eq!((sold[0].price, sold[0].item.index, sold[0].item.amount, sold[0].item.id, sold[0].item.refine), (10_000, 5, 20, 909, 7));
        assert_eq!((sold[0].item.identified, sold[0].item.cards[0]), (true, 4001));
    }

    #[test]
    fn reads_a_vendors_list() {
        let mut packet = vec![0x3d, 0x0b, 76, 0, 9, 0, 0, 0, 0x2a, 0, 0, 0];
        let mut thing = record(&[0xe8, 3, 0, 0, 12, 0, 6, 0], 64);
        thing[56] = 2;
        thing[62] = 4;
        packet.extend(thing);
        let list = listed(&packet).unwrap();
        assert_eq!((list.vendor, list.shop, list.sold.len()), (9, 42, 1));
        let sold = &list.sold[0];
        assert_eq!((sold.price, sold.item.amount, sold.item.index, sold.item.id, sold.item.wearable, sold.item.refine), (1000, 12, 6, 909, 2, 4));
    }

    #[test]
    fn reads_a_sign_a_sale_and_the_answers() {
        let mut packet = vec![0x31, 0x01, 7, 0, 0, 0];
        packet.extend(b"Cheap potions");
        packet.resize(86, 0);
        assert_eq!(sign(&packet), Some((7, "Cheap potions".to_owned())));
        assert_eq!(sign_gone(&[0x32, 0x01, 7, 0, 0, 0]), Some(7));
        let mut sold = vec![0xe5, 0x09, 5, 0, 3, 0];
        sold.resize(18, 0);
        assert_eq!(sale(&sold), Some((5, 3)));
        assert_eq!([opened(&[0x28, 0x0a, 0]), opened(&[0x28, 0x0a, 1]), opened(&[0x28, 0x0a, 3]), opened(&[0x28, 0x0a, 2])], [Some(Opened::Open), Some(Opened::Failed), Some(Opened::NotHere), None]);
        assert_eq!(slots(&[0x2d, 0x01, 12, 0]), Some(12));
        assert_eq!(slots(&[0x2d, 0x01, 1, 0]), Some(3));
        assert_eq!(purchase_result(&[0x35, 0x01, 5, 0, 3, 0, 4]), Some((5, 3, 4)));
    }

    #[test]
    fn a_purchase_that_failed_is_told_of() {
        let line = |_: usize, otherwise: &str| otherwise.to_owned();
        assert_eq!(purchase_said(0, "Jellopy", 3, line), None);
        assert_eq!(purchase_said(1, "Jellopy", 3, line).as_deref(), Some("Jellopy Purchase Failed You do not have enough zeny."));
        assert_eq!(purchase_said(4, "Jellopy", 3, line).as_deref(), Some("Buying Jellopy has failed. It's out of stock. Current Stock 3."));
    }

    #[test]
    fn the_requests_are_laid_out_as_the_server_reads_them() {
        let opening = open("Cheap", true, &[(5, 20, 10_000)]);
        assert_eq!(opening.len(), 93);
        assert_eq!((&opening[..4], &opening[4..10], opening[84]), (&[0xb2, 0x01, 93, 0][..], &b"Cheap\0"[..], 1));
        assert_eq!(opening[85..], [5, 0, 20, 0, 0x10, 0x27, 0, 0]);
        // Not opening, and nothing to sell, say nought.
        assert_eq!((open("", false, &[(5, 20, 1)])[84], open("x", true, &[])[84]), (0, 0));
        assert_eq!(open(&"я".repeat(60), true, &[(1, 1, 1)])[80..84], [0xd1, 0x8f, 0, 0]);
        assert_eq!(buy(9, 42, &[(6, 2)]), [0x01, 0x08, 16, 0, 9, 0, 0, 0, 42, 0, 0, 0, 2, 0, 6, 0]);
        for request in [close(), ask(1)] {
            let id = u16::from_le_bytes([request[0], request[1]]);
            assert_eq!(packet_length(id), Some(request.len() as i32), "{id:#06x}");
        }
        for (id, length) in [(0x01b2, -1), (0x0801, -1), (ZC_OPENSTORE, 4), (ZC_STORE_ENTRY, 86), (ZC_DISAPPEAR_ENTRY, 6), (ZC_PC_PURCHASE_RESULT_FROMMC, 7), (ZC_DELETEITEM_FROM_MCSTORE, 18), (ZC_ACK_OPENSTORE2, 3), (ZC_PC_PURCHASE_ITEMLIST_FROMMC, -1), (ZC_PC_PURCHASE_MYITEMLIST, -1)] {
            assert_eq!(packet_length(id), Some(length), "{id:#06x}");
        }
    }

    #[test]
    fn prices_have_their_colours_their_share_and_their_reading() {
        assert_eq!(price_colour(5).0, "#00ffff");
        assert_eq!(price_colour(10_000_000), ("#ff0000", None));
        assert_eq!(price_colour(999), ("#00ffff", Some("#0000ff")));
        assert_eq!((after_commission(10_000_000), after_commission(20_000_000)), (10_000_000, 19_000_000));
        assert_eq!([price_typed("1,500"), price_typed(" 0 "), price_typed("12a"), price_typed(""), price_typed("99999999999")], [Some(1500), Some(0), None, None, Some(MOST_PRICE)]);
    }
}
