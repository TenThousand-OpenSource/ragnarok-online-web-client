//! The packets about what the character carries and knows: items, skills, and the hotkey bar.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets_struct.hpp`, `packets.hpp`).

use super::bytes::{u16_at, u32_at};

pub const ZC_INVENTORY_START: u16 = 0x0b08;
pub const ZC_INVENTORY_ITEMLIST_NORMAL: u16 = 0x0b09;
pub const ZC_INVENTORY_ITEMLIST_EQUIP: u16 = 0x0b39;
pub const ZC_ITEM_PICKUP_ACK: u16 = 0x0b41;
pub const ZC_USE_ITEM_ACK: u16 = 0x01c8;
pub const ZC_DELETE_ITEM_FROM_BODY: u16 = 0x07fa;
pub const ZC_ITEM_THROW_ACK: u16 = 0x00af;
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

/// The tabs of the inventory window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Usable,
    Equipment,
    Other,
    /// "Fav": what the player keeps under it, whatever kind it is of (`net::items::favourites`). No thing is
    /// under it by its kind: `Item::tab` never says it.
    Favourite,
}

/// Where arrows and the like are worn (the client's wear location `0x8000`).
pub const WORN_AS_AMMO: u32 = 0x8000;

impl Item {
    /// Whether the inventory window shows it: what is not worn, and the arrows that are. (The client takes what
    /// is worn out of its list of what is carried, but leaves the arrows in it and only remembers which are
    /// readied: `CSession::equippedArrowIndex`, looked up among `m_inventoryItems`.)
    pub fn in_bag(&self) -> bool {
        self.worn == 0 || self.worn == WORN_AS_AMMO
    }

    /// Which tab the item is under: what is used up, what is worn, and everything else.
    pub fn tab(&self) -> Tab {
        match self.kind {
            0 | 2 | 11 | 18 => Tab::Usable,
            4 | 5 | 8 | 12 => Tab::Equipment,
            _ if self.wearable != 0 && self.kind != 10 => Tab::Equipment,
            _ => Tab::Other,
        }
    }

    pub fn usable(&self) -> bool {
        self.tab() == Tab::Usable
    }

    /// Whether it may be put on the hotkey bar (the client's shortcut window, where a drag ends on it): one that
    /// is identified and is used up (the kinds 0, 1 and 2), shot or thrown (10, 16, 17, 19), or worn (4, 5, 8, 9,
    /// 11 to 15) with a place to wear it.
    pub fn fits_hotkeys(&self) -> bool {
        self.identified && (matches!(self.kind, 0 | 1 | 2 | 10 | 16 | 17 | 19) || (matches!(self.kind, 4 | 5 | 8 | 9 | 11..=15) && self.wearable != 0))
    }
}

fn cards(bytes: &[u8], at: usize) -> [u32; 4] {
    [0, 1, 2, 3].map(|slot| u32_at(bytes, at + slot * 4))
}

/// ZC_INVENTORY_ITEMLIST_NORMAL (0x0b09): `u16 length, u8 inventory`, then 34 bytes an item: `i16 index, u32 id,
/// u8 type, i16 count, u32 worn, u32[4] cards, i32 expires, u8 flags`. Items of other inventories give nothing.
pub fn stackable(packet: &[u8]) -> Vec<Item> {
    stackable_of(packet, OWN_INVENTORY)
}

/// The same list for another inventory: 2 is the storage.
pub fn stackable_of(packet: &[u8], inventory: u8) -> Vec<Item> {
    if packet.get(4) != Some(&inventory) {
        return Vec::new();
    }
    packet[5..]
        .chunks_exact(34)
        .map(|item| Item {
            index: u16_at(item, 0),
            id: u32_at(item, 2),
            kind: item[6],
            amount: u16_at(item, 7),
            // For ammunition (kind 10) the server puts where it can be worn in the field that holds where a
            // thing is worn; whether it is in use, it says apart (ZC_EQUIP_ARROW).
            wearable: if item[6] == 10 { u32_at(item, 9) } else { 0 },
            worn: if item[6] == 10 { 0 } else { u32_at(item, 9) },
            refine: 0,
            identified: item[33] & 1 != 0,
            cards: cards(item, 13),
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
    if packet.get(4) != Some(&inventory) {
        return Vec::new();
    }
    worn_list(&packet[5..])
}

/// The things of such a list, 68 bytes each (rAthena's `EQUIPITEM_INFO`): the inventory's own, and what the
/// server shows of another player's equipment (`net/viewequip.rs`).
pub fn worn_list(items: &[u8]) -> Vec<Item> {
    items
        .chunks_exact(68)
        .map(|item| Item {
            index: u16_at(item, 0),
            id: u32_at(item, 2),
            kind: item[6],
            amount: 1,
            wearable: u32_at(item, 7),
            worn: u32_at(item, 11),
            refine: item[65],
            identified: item[67] & 1 != 0,
            cards: cards(item, 15),
        })
        .collect()
}

/// ZC_ITEMIDENTIFY_LIST (0x0177): `u16 length`, then the place in the inventory of each item not identified.
pub fn unidentified(packet: &[u8]) -> Vec<u16> {
    super::bytes::words_from(packet, 4)
}

/// ZC_ACK_ITEMIDENTIFY (0x0179): `u16 index, u8 result`: the item at that place, and whether it was identified
/// (0) or not (1).
pub fn identified(packet: &[u8]) -> (u16, Option<u8>) {
    (super::bytes::word(packet, 2).unwrap_or(0), packet.get(4).copied())
}

/// ZC_EQUIP_ARROW (0x013c): `u16 index`. The ammunition in use (0: none).
pub fn ammunition(packet: &[u8]) -> Option<u16> {
    super::bytes::word(packet, 2)
}

/// The number typed into the box that asks how many of a thing carried (`UIItemDropCntWnd`), of which there
/// are `most`: as the client's box has it, what is no number, nothing, or more than there is counts for
/// nothing, and the box goes away without a word.
pub fn count_typed(typed: &str, most: u16) -> Option<u16> {
    typed.trim().parse::<u32>().ok().filter(|count| (1..=u32::from(most)).contains(count)).map(|count| count as u16)
}

/// CZ_ITEM_THROW (0x0363): `u16 index, u16 count`: drops that many of a thing carried onto the ground.
pub fn throw(index: u16, count: u16) -> Vec<u8> {
    let mut packet = 0x0363u16.to_le_bytes().to_vec();
    packet.extend(index.to_le_bytes());
    packet.extend(count.to_le_bytes());
    packet
}

/// ZC_ITEM_PICKUP_ACK (0x0b41): an item came into the inventory (`amount` more of it), or could not (`None`).
/// `u16 index, u16 count, u32 id, u8 identified, u8 damaged, u32[4] cards, u32 wearable, u8 type, u8 result, …,
/// u8 refine, u8 grade`.
pub fn gained(packet: &[u8]) -> Option<Item> {
    if packet.len() < 70 || packet[33] != 0 {
        return None;
    }
    Some(Item {
        index: u16_at(packet, 2),
        id: u32_at(packet, 6),
        kind: packet[32],
        amount: u16_at(packet, 4),
        wearable: u32_at(packet, 28),
        worn: 0,
        refine: packet[68],
        identified: packet[10] != 0,
        cards: cards(packet, 12),
    })
}

/// ZC_USE_ITEM_ACK (0x01c8): `i16 index, u32 id, u32 who, i16 left, u8 ok`. Returns the index, the item, who
/// used it and how many are left, when it was used. Everyone near is told of it, not only the one who did.
pub fn used(packet: &[u8]) -> Option<(u16, u32, u32, u16)> {
    (packet.len() >= 15 && packet[14] != 0).then(|| (u16_at(packet, 2), u32_at(packet, 4), u32_at(packet, 8), u16_at(packet, 12)))
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
    [&0x0439u16.to_le_bytes()[..], &index.to_le_bytes(), &account_id.to_le_bytes()].concat()
}

/// CZ_REQ_WEAR_EQUIP (0x0998): `u16 index, u32 where`.
pub fn wear(index: u16, wearable: u32) -> Vec<u8> {
    [&0x0998u16.to_le_bytes()[..], &index.to_le_bytes(), &wearable.to_le_bytes()].concat()
}

/// CZ_REQ_TAKEOFF_EQUIP (0x00ab): `u16 index`.
pub fn take_off(index: u16) -> Vec<u8> {
    [&0x00abu16.to_le_bytes()[..], &index.to_le_bytes()].concat()
}

/// CZ_REQ_ITEMIDENTIFY (0x0178): `u16 index`. Asks to identify the item at that place of the inventory, after a
/// magnifier listed it; 0xffff (the client's -1) says none was picked.
pub fn identify(index: u16) -> Vec<u8> {
    let mut packet = vec![0x78, 0x01];
    packet.extend_from_slice(&index.to_le_bytes());
    packet
}

mod favourites;
mod hotkeys;
mod skills;
mod wearing;
pub mod weight;
pub use {favourites::*, hotkeys::*, skills::*, wearing::*};

#[cfg(test)]
mod tests;
