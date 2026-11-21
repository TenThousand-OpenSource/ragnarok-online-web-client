//! The packets about what the character carries and knows: items, skills, and the hotkey bar. An item as a
//! packet tells of it is `Item`, read by `Item::read` from wherever a packet has its record (`Layout`). What
//! is a rule of the inventory and no packet (its tabs, what goes on the hotkey bar, a count typed, the lines
//! of weight) is in `crate::inventory`.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets_struct.hpp`, `packets.hpp`).

use super::bytes::{Writer, long, u16_at, u32_at, word};

pub const ZC_INVENTORY_START: u16 = 0x0b08;
/// ZC_INVENTORY_END (0x0b0b, 4 bytes; the client's `ZC_SPLIT_SEND_ITEMLIST_RESULT`): `u8 kind, u8 partial`:
/// the lists of one kind (the inventory, the cart, a storage) are all sent.
pub const ZC_INVENTORY_END: u16 = 0x0b0b;
/// ZC_EXTEND_BODYITEM_SIZE (0x0b18, 4 bytes): `u16 more`: how many places the inventory has over its hundred
/// (`clif_inventory_expansion_info`): the inventory expansion, which is not of the classic game.
pub const ZC_EXTEND_BODYITEM_SIZE: u16 = 0x0b18;
/// ZC_SEND_SWAP_EQUIPITEM_INFO (0x0a9b): the things of the equipment switch, sent as the game is entered:
/// not of the classic game.
pub const ZC_SEND_SWAP_EQUIPITEM_INFO: u16 = 0x0a9b;
pub const ZC_INVENTORY_ITEMLIST_NORMAL: u16 = 0x0b09;
pub const ZC_INVENTORY_ITEMLIST_EQUIP: u16 = 0x0b39;
pub const ZC_ITEM_PICKUP_ACK: u16 = 0x0b41;
pub const ZC_USE_ITEM_ACK: u16 = 0x01c8;
pub const ZC_DELETE_ITEM_FROM_BODY: u16 = 0x07fa;
pub const ZC_ITEM_THROW_ACK: u16 = 0x00af;
pub const ZC_ITEMIDENTIFY_LIST: u16 = 0x0177;
pub const ZC_ACK_ITEMIDENTIFY: u16 = 0x0179;
pub const ZC_EQUIP_ARROW: u16 = 0x013c;
pub const ZC_REQ_WEAR_EQUIP_ACK: u16 = 0x0999;
pub const ZC_REQ_TAKEOFF_EQUIP_ACK: u16 = 0x099a;
pub const ZC_SKILLINFO_LIST: u16 = 0x010f;
pub const ZC_SKILLINFO_UPDATE: u16 = 0x010e;
pub const ZC_ADD_SKILL: u16 = 0x0111;
pub const ZC_SHORTCUT_KEY_LIST: u16 = 0x0b20;

/// The inventory list a packet belongs to: the character's own is 0 (others are the cart and storages).
const OWN_INVENTORY: u8 = 0;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Where the server keeps it: the number every request about it names.
    pub index: u16,
    pub id: u32,
    /// The server's item type: 0 healing, 2 usable, 3 etc, 4 armor, 5 weapon, 6 card, 10 ammunition, 11 usable
    /// with a delay, 18 cash.
    pub kind: u8,
    pub amount: u16,
    /// Where it can be worn (bits, 0 for what cannot be), and where it is worn now (0 when it is not).
    pub wearable: u32,
    pub worn: u32,
    pub refine: u8,
    pub identified: bool,
    /// What is set into its slots.
    pub cards: [u32; 4],
}


/// The four cards of an item, four bytes each from `at`: the one reader of them for every packet that lists items.
pub(super) fn cards(bytes: &[u8], at: usize) -> [u32; 4] {
    [0, 1, 2, 3].map(|slot| u32_at(bytes, at + slot * 4))
}

/// Where a packet has each part of an item's record, counted from where `Item::read` is given its bytes: one
/// for every packet that tells of an item. What a packet does not tell (`None`) is nought.
pub(super) struct Layout {
    pub(super) index: Option<usize>,
    pub(super) id: usize,
    pub(super) kind: usize,
    pub(super) amount: Amount,
    pub(super) wearable: Option<usize>,
    pub(super) worn: Option<usize>,
    pub(super) refine: Option<usize>,
    pub(super) identified: Identified,
    /// The first of the four cards, four bytes each.
    pub(super) cards: usize,
}

/// How a packet tells how many there are of an item.
pub(super) enum Amount {
    /// It does not: there is one.
    One,
    /// A `u16` there.
    Word(usize),
    /// A `u32` there; more than a `u16` holds counts as the most it holds.
    Long(usize),
}

/// How a packet tells that an item is identified.
pub(super) enum Identified {
    /// A byte of its own there, nought for one that is not.
    Byte(usize),
    /// The lowest bit of the byte of flags there.
    Flag(usize),
}

impl Item {
    /// The item a packet has laid out so; none if the bytes end before any part of it.
    pub(super) fn read(bytes: &[u8], layout: &Layout) -> Option<Item> {
        let number = |at: Option<usize>| at.map_or(Some(0), |at| long(bytes, at));
        bytes.get(layout.cards..layout.cards + 16)?;
        Some(Item {
            index: layout.index.map_or(Some(0), |at| word(bytes, at))?,
            id: long(bytes, layout.id)?,
            kind: *bytes.get(layout.kind)?,
            amount: match layout.amount {
                Amount::One => 1,
                Amount::Word(at) => word(bytes, at)?,
                Amount::Long(at) => long(bytes, at)?.min(u32::from(u16::MAX)) as u16,
            },
            wearable: number(layout.wearable)?,
            worn: number(layout.worn)?,
            refine: layout.refine.map_or(Some(0), |at| bytes.get(at).copied())?,
            identified: match layout.identified {
                Identified::Byte(at) => *bytes.get(at)? != 0,
                Identified::Flag(at) => *bytes.get(at)? & 1 != 0,
            },
            cards: cards(bytes, layout.cards),
        })
    }
}

/// Which list a ZC_INVENTORY_START (0x0b08: `u16 length, u8 kind`, then the list's name) begins, and which a
/// list of items that follows it is of: the character's own (0), the cart's (`net::cart::LIST`) or the
/// storage's (`net::storage::STORAGE`).
pub fn list_kind(packet: &[u8]) -> Option<u8> {
    packet.get(4).copied()
}

/// An item of ZC_INVENTORY_ITEMLIST_NORMAL. Where it is worn is read as the packet has it: `stackable_of`
/// sees to the ammunition.
const STACK: Layout = Layout {
    index: Some(0), id: 2, kind: 6, amount: Amount::Word(7),
    wearable: None, worn: Some(9), refine: None, identified: Identified::Flag(33), cards: 13,
};
/// An item of ZC_INVENTORY_ITEMLIST_EQUIP (rAthena's `EQUIPITEM_INFO`).
const WORN: Layout = Layout {
    index: Some(0), id: 2, kind: 6, amount: Amount::One,
    wearable: Some(7), worn: Some(11), refine: Some(65), identified: Identified::Flag(67), cards: 15,
};
/// The item of ZC_ITEM_PICKUP_ACK, counted from the packet's start.
const GAINED: Layout = Layout {
    index: Some(2), id: 6, kind: 32, amount: Amount::Word(4),
    wearable: Some(28), worn: None, refine: Some(68), identified: Identified::Byte(10), cards: 12,
};

/// ZC_INVENTORY_ITEMLIST_NORMAL (0x0b09): `u16 length, u8 inventory`, then 34 bytes an item: `i16 index, u32 id,
/// u8 type, i16 count, u32 worn, u32[4] cards, i32 expires, u8 flags`. Items of other inventories give nothing.
pub fn stackable(packet: &[u8]) -> Vec<Item> {
    stackable_of(packet, OWN_INVENTORY)
}

/// The same list for another inventory: 2 is the storage.
pub fn stackable_of(packet: &[u8], inventory: u8) -> Vec<Item> {
    if list_kind(packet) != Some(inventory) {
        return Vec::new();
    }
    packet[5..]
        .chunks_exact(34)
        .filter_map(|item| Item::read(item, &STACK))
        .map(|item| match item.kind {
            // For ammunition (kind 10) the server puts where it can be worn in the field that holds where a
            // thing is worn; whether it is in use, it says apart (ZC_EQUIP_ARROW).
            10 => Item { wearable: item.worn, worn: 0, ..item },
            _ => item,
        })
        .collect()
}

/// ZC_INVENTORY_ITEMLIST_EQUIP (0x0b39): `u16 length, u8 inventory`, then 68 bytes an item: `i16 index, u32 id,
/// u8 type, u32 wearable, u32 worn, u32[4] cards, i32 expires, u16 bind, u16 look, u8 options, 25 bytes of them,
/// u8 refine, u8 grade, u8 flags`.
pub fn equipment(packet: &[u8]) -> Vec<Item> {
    equipment_of(packet, OWN_INVENTORY)
}

pub fn equipment_of(packet: &[u8], inventory: u8) -> Vec<Item> {
    if list_kind(packet) != Some(inventory) {
        return Vec::new();
    }
    worn_list(&packet[5..])
}

/// The things of such a list, 68 bytes each (rAthena's `EQUIPITEM_INFO`): the inventory's own, and what the
/// server shows of another player's equipment (`net/viewequip.rs`).
pub fn worn_list(items: &[u8]) -> Vec<Item> {
    items.chunks_exact(68).filter_map(|item| Item::read(item, &WORN)).collect()
}

/// ZC_ITEMIDENTIFY_LIST (0x0177): `u16 length`, then the place in the inventory of each item not identified.
pub fn unidentified(packet: &[u8]) -> Vec<u16> {
    super::bytes::words_from(packet, 4)
}

/// ZC_ACK_ITEMIDENTIFY (0x0179): `u16 index, u8 result`: the item at that place, and whether it was identified
/// (0) or not (1).
pub fn identified(packet: &[u8]) -> (u16, Option<u8>) {
    (word(packet, 2).unwrap_or(0), packet.get(4).copied())
}

/// ZC_EQUIP_ARROW (0x013c): `u16 index`. The ammunition in use (0: none).
pub fn ammunition(packet: &[u8]) -> Option<u16> {
    word(packet, 2)
}

/// CZ_ITEM_THROW (0x0363): `u16 index, u16 count`: drops that many of a thing carried onto the ground.
pub fn throw(index: u16, count: u16) -> Vec<u8> {
    Writer::new(0x0363).u16(index).u16(count).done()
}

/// ZC_ITEM_PICKUP_ACK (0x0b41): an item came into the inventory (`amount` more of it), or could not (`None`).
/// `u16 index, u16 count, u32 id, u8 identified, u8 damaged, u32[4] cards, u32 wearable, u8 type, u8 result, …,
/// u8 refine, u8 grade`.
pub fn gained(packet: &[u8]) -> Option<Item> {
    if packet.len() < 70 || packet[33] != 0 {
        return None;
    }
    Item::read(packet, &GAINED)
}

/// An item used, as ZC_USE_ITEM_ACK tells it: where it is carried, which item, who used it and how many are
/// left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Used {
    pub index: u16,
    pub item: u32,
    pub who: u32,
    pub left: u16,
}

/// ZC_USE_ITEM_ACK (0x01c8): `i16 index, u32 id, u32 who, i16 left, u8 ok`: read when the item was used.
/// Everyone near is told of it, not only the one who did.
pub fn used(packet: &[u8]) -> Option<Used> {
    (packet.len() >= 15 && packet[14] != 0)
        .then(|| Used { index: u16_at(packet, 2), item: u32_at(packet, 4), who: u32_at(packet, 8), left: u16_at(packet, 12) })
}

/// How many of an item left the inventory: ZC_DELETE_ITEM_FROM_BODY (0x07fa: `i16 reason, u16 index, i16 count`)
/// and ZC_ITEM_THROW_ACK (0x00af: `u16 index, u16 count`).
pub fn lost(id: u16, packet: &[u8]) -> Option<(u16, u16)> {
    match id {
        ZC_DELETE_ITEM_FROM_BODY if packet.len() >= 8 => Some((u16_at(packet, 4), u16_at(packet, 6))),
        ZC_ITEM_THROW_ACK if packet.len() >= 6 => Some((u16_at(packet, 2), u16_at(packet, 4))),
        _ => None,
    }
}

/// ZC_REQ_WEAR_EQUIP_ACK (0x0999): `u16 index, u32 where, u16 look, u8 result` (0 is success). Returns the index
/// and where it is worn now.
pub fn put_on(packet: &[u8]) -> Option<(u16, u32)> {
    (packet.len() >= 11 && packet[10] == 0).then(|| (u16_at(packet, 2), u32_at(packet, 4)))
}

/// ZC_REQ_TAKEOFF_EQUIP_ACK (0x099a): `u16 index, u32 where, u8 result` (0 is success). Returns the index.
pub fn taken_off(packet: &[u8]) -> Option<u16> {
    (packet.len() >= 9 && packet[8] == 0).then(|| u16_at(packet, 2))
}

/// CZ_USE_ITEM (0x0439): `u16 index, u32 on whom` (the character's account ID for itself).
pub fn use_item(index: u16, account_id: u32) -> Vec<u8> {
    Writer::new(0x0439).u16(index).u32(account_id).done()
}

/// CZ_REQ_WEAR_EQUIP (0x0998): `u16 index, u32 where`.
pub fn wear(index: u16, wearable: u32) -> Vec<u8> {
    Writer::new(0x0998).u16(index).u32(wearable).done()
}

/// CZ_REQ_TAKEOFF_EQUIP (0x00ab): `u16 index`.
pub fn take_off(index: u16) -> Vec<u8> {
    Writer::new(0x00ab).u16(index).done()
}

/// CZ_REQ_ITEMIDENTIFY (0x0178): `u16 index`. Asks to identify the item at that place of the inventory, after a
/// magnifier listed it; 0xffff (the client's -1) says none was picked.
pub fn identify(index: u16) -> Vec<u8> {
    Writer::new(0x0178).u16(index).done()
}

mod favourites;
mod hotkeys;
mod skills;
mod wearing;
pub use {favourites::*, hotkeys::*, skills::*, wearing::*};

#[cfg(test)]
mod tests;
