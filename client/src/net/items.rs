//! The packets about what the character carries and knows: items, skills, and the hotkey bar.
//!
//! Layouts are rAthena's at PACKETVER 20260219 (`src/map/packets_struct.hpp`, `packets.hpp`).

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

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

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
}

impl Item {
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
    packet[5..]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub id: u16,
    /// How it is used: 0 is passive, 1 on an enemy, 2 on a place, 4 on oneself, 16 on a friend, 32 a trap.
    pub target: u32,
    pub level: u16,
    pub sp: u16,
    pub range: u16,
    /// Whether a skill point can raise it now.
    pub raisable: bool,
}

impl Skill {
    pub fn passive(&self) -> bool {
        self.target == 0
    }

    /// Used on the character itself, with no target to choose.
    pub fn on_self(&self) -> bool {
        self.target == 4
    }
}

/// ZC_SKILLINFO_LIST (0x010f): `u16 length`, then 37 bytes a skill: `u16 id, i32 target, u16 level, u16 sp,
/// u16 range, char[24] name, u8 raisable`.
pub fn skills(packet: &[u8]) -> Vec<Skill> {
    packet.get(4..).unwrap_or_default().chunks_exact(37).map(|skill| skill_at(skill, true)).collect()
}

fn skill_at(skill: &[u8], full: bool) -> Skill {
    Skill { id: u16_at(skill, 0), target: u32_at(skill, 2), level: u16_at(skill, 6), sp: u16_at(skill, 8), range: u16_at(skill, 10), raisable: full && skill[36] != 0 }
}

/// ZC_ADD_SKILL (0x0111): one skill, laid out as in the list.
pub fn skill_added(packet: &[u8]) -> Option<Skill> {
    packet.get(2..39).map(|skill| skill_at(skill, true))
}

/// ZC_SKILLINFO_UPDATE (0x010e): `u16 id, u16 level, u16 sp, u16 range, u8 raisable`: a skill went up a level.
pub fn skill_raised(packet: &[u8]) -> Option<(u16, u16, u16, u16, bool)> {
    (packet.len() >= 11).then(|| (u16_at(packet, 2), u16_at(packet, 4), u16_at(packet, 6), u16_at(packet, 8), packet[10] != 0))
}

/// CZ_UPGRADE_SKILLLEVEL (0x0112): `u16 id`: spends a skill point.
pub fn raise_skill(id: u16) -> Vec<u8> {
    [0x0112u16.to_le_bytes(), id.to_le_bytes()].concat()
}

/// CZ_USE_SKILL (0x0438): `u16 level, u16 id, u32 target`.
pub fn use_skill(level: u16, id: u16, target: u32) -> Vec<u8> {
    [&0x0438u16.to_le_bytes()[..], &level.to_le_bytes(), &id.to_le_bytes(), &target.to_le_bytes()].concat()
}

/// What a slot of the hotkey bar holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hotkey {
    pub skill: bool,
    /// The item's or the skill's ID; 0 for an empty slot.
    pub id: u32,
    /// For a skill, the level it is used at.
    pub count: u16,
}

/// How many slots the bar has: four rows of nine, and two more the server keeps.
pub const HOTKEYS: usize = 38;

/// ZC_SHORTCUT_KEY_LIST (0x0b20): `u8 rotate, u16 tab`, then 7 bytes a slot: `u8 skill, u32 id, u16 count`.
pub fn hotkeys(packet: &[u8]) -> Vec<Hotkey> {
    packet.get(5..).unwrap_or_default().chunks_exact(7).take(HOTKEYS).map(|slot| Hotkey { skill: slot[0] != 0, id: u32_at(slot, 1), count: u16_at(slot, 5) }).collect()
}

/// CZ_SHORTCUT_KEY_CHANGE (0x0b21): `u16 tab, u16 slot, u8 skill, u32 id, u16 count`.
pub fn set_hotkey(slot: u16, hotkey: Hotkey) -> Vec<u8> {
    [&0x0b21u16.to_le_bytes()[..], &0u16.to_le_bytes(), &slot.to_le_bytes(), &[u8::from(hotkey.skill)], &hotkey.id.to_le_bytes(), &hotkey.count.to_le_bytes()].concat()
}

/// CZ_REQ_ITEMIDENTIFY (0x0178): `u16 index`. Asks to identify the item at that place of the inventory, after a
/// magnifier listed it; 0xffff (the client's -1) says none was picked.
pub fn identify(index: u16) -> Vec<u8> {
    let mut packet = vec![0x78, 0x01];
    packet.extend_from_slice(&index.to_le_bytes());
    packet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet_length;

    #[test]
    fn requests_have_the_length_the_server_expects() {
        for request in [use_item(2, 2_000_000), wear(3, 2), take_off(3), raise_skill(142), use_skill(1, 142, 2_000_000), set_hotkey(0, Hotkey { skill: true, id: 142, count: 1 })] {
            let id = u16_at(&request, 0);
            assert_eq!(packet_length(id), Some(request.len() as i32), "{id:#06x}");
        }
        // And the fixed-length replies are as long as they are read here.
        for (id, length) in [(ZC_ITEM_PICKUP_ACK, 70), (ZC_USE_ITEM_ACK, 15), (ZC_DELETE_ITEM_FROM_BODY, 8), (ZC_ITEM_THROW_ACK, 6), (ZC_REQ_WEAR_EQUIP_ACK, 11), (ZC_REQ_TAKEOFF_EQUIP_ACK, 9), (ZC_SKILLINFO_UPDATE, 11), (ZC_ADD_SKILL, 39), (ZC_SHORTCUT_KEY_LIST, 5 + 7 * HOTKEYS as i32)] {
            assert_eq!(packet_length(id), Some(length), "{id:#06x}");
        }
    }

    fn stack(index: u16, id: u32, kind: u8, amount: u16) -> Vec<u8> {
        let mut item = vec![0u8; 34];
        item[..2].copy_from_slice(&index.to_le_bytes());
        item[2..6].copy_from_slice(&id.to_le_bytes());
        item[6] = kind;
        item[7..9].copy_from_slice(&amount.to_le_bytes());
        item[33] = 1;
        item
    }

    #[test]
    fn reads_the_stackable_items() {
        let mut packet = vec![0x09, 0x0b, 0, 0, 0];
        packet.extend(stack(2, 501, 0, 30));
        packet.extend(stack(3, 909, 3, 7));
        let items = stackable(&packet);
        assert_eq!(items.len(), 2);
        assert_eq!((items[0].index, items[0].id, items[0].amount, items[0].identified), (2, 501, 30, true));
        assert_eq!((items[0].tab(), items[1].tab()), (Tab::Usable, Tab::Other));
        assert!(items[0].usable() && !items[1].usable());
        // The cart's and the storage's lists are not the character's.
        packet[4] = 1;
        assert!(stackable(&packet).is_empty());
    }

    #[test]
    fn reads_equipment() {
        let mut item = vec![0u8; 68];
        item[..2].copy_from_slice(&4u16.to_le_bytes());
        item[2..6].copy_from_slice(&1201u32.to_le_bytes());
        item[6] = 5;
        item[7..11].copy_from_slice(&2u32.to_le_bytes());
        item[11..15].copy_from_slice(&2u32.to_le_bytes());
        item[15..19].copy_from_slice(&4001u32.to_le_bytes());
        item[65] = 7;
        item[67] = 1;
        let mut packet = vec![0x39, 0x0b, 0, 0, 0];
        packet.extend(&item);
        let items = equipment(&packet);
        assert_eq!(items, [Item { index: 4, id: 1201, kind: 5, amount: 1, wearable: 2, worn: 2, refine: 7, identified: true, cards: [4001, 0, 0, 0] }]);
        assert_eq!(items[0].tab(), Tab::Equipment);
    }

    #[test]
    fn follows_items_coming_and_going() {
        let mut gain = vec![0u8; 70];
        gain[2..4].copy_from_slice(&5u16.to_le_bytes());
        gain[4..6].copy_from_slice(&3u16.to_le_bytes());
        gain[6..10].copy_from_slice(&501u32.to_le_bytes());
        gain[10] = 1;
        let item = gained(&gain).unwrap();
        assert_eq!((item.index, item.id, item.amount, item.identified), (5, 501, 3, true));
        // Result 2 is "too heavy": nothing came.
        gain[33] = 2;
        assert_eq!(gained(&gain), None);

        let mut use_ack = vec![0u8; 15];
        use_ack[2..4].copy_from_slice(&5u16.to_le_bytes());
        use_ack[4..8].copy_from_slice(&501u32.to_le_bytes());
        use_ack[8..12].copy_from_slice(&2000001u32.to_le_bytes());
        use_ack[12..14].copy_from_slice(&2u16.to_le_bytes());
        use_ack[14] = 1;
        assert_eq!(used(&use_ack), Some((5, 501, 2000001, 2)));
        use_ack[14] = 0;
        assert_eq!(used(&use_ack), None);

        assert_eq!(lost(ZC_DELETE_ITEM_FROM_BODY, &[0xfa, 0x07, 0, 0, 5, 0, 1, 0]), Some((5, 1)));
        assert_eq!(lost(ZC_ITEM_THROW_ACK, &[0xaf, 0x00, 5, 0, 2, 0]), Some((5, 2)));
        assert_eq!(put_on(&[0x99, 0x09, 4, 0, 2, 0, 0, 0, 0, 0, 0]), Some((4, 2)));
        assert_eq!(put_on(&[0x99, 0x09, 4, 0, 2, 0, 0, 0, 0, 0, 1]), None);
        assert_eq!(taken_off(&[0x9a, 0x09, 4, 0, 2, 0, 0, 0, 0]), Some(4));
    }

    #[test]
    fn reads_skills() {
        let mut skill = vec![0u8; 37];
        skill[..2].copy_from_slice(&142u16.to_le_bytes());
        skill[2..6].copy_from_slice(&4u32.to_le_bytes());
        skill[6..8].copy_from_slice(&1u16.to_le_bytes());
        skill[8..10].copy_from_slice(&3u16.to_le_bytes());
        skill[10..12].copy_from_slice(&1u16.to_le_bytes());
        skill[36] = 0;
        let mut basic = vec![0u8; 37];
        basic[0] = 1;
        basic[36] = 1;
        let mut packet = vec![0x0f, 0x01, 0, 0];
        packet.extend(&skill);
        packet.extend(&basic);
        let list = skills(&packet);
        assert_eq!(list[0], Skill { id: 142, target: 4, level: 1, sp: 3, range: 1, raisable: false });
        assert!(list[0].on_self() && !list[0].passive());
        assert!(list[1].passive() && list[1].raisable);

        assert_eq!(skill_raised(&[0x0e, 0x01, 1, 0, 2, 0, 0, 0, 1, 0, 1]), Some((1, 2, 0, 1, true)));
        let mut added = vec![0x11, 0x01];
        added.extend(&skill);
        assert_eq!(skill_added(&added).unwrap().id, 142);
    }

    #[test]
    fn reads_and_sets_the_hotkey_bar() {
        let mut packet = vec![0x20, 0x0b, 0, 0, 0];
        packet.extend([1, 142, 0, 0, 0, 1, 0]);
        packet.extend([0, 0xf5, 0x01, 0, 0, 0, 0]);
        packet.resize(5 + 7 * HOTKEYS, 0);
        let bar = hotkeys(&packet);
        assert_eq!(bar.len(), HOTKEYS);
        assert_eq!(bar[0], Hotkey { skill: true, id: 142, count: 1 });
        assert_eq!(bar[1], Hotkey { skill: false, id: 501, count: 0 });
        assert_eq!(bar[2], Hotkey::default());
        assert_eq!(set_hotkey(9, bar[0]), [0x21, 0x0b, 0, 0, 9, 0, 1, 142, 0, 0, 0, 1, 0]);
    }

    #[test]
    fn only_what_is_used_shot_or_worn_goes_on_the_hotkey_bar() {
        let item = |kind: u8, wearable: u32, identified: bool| Item { index: 2, id: 501, kind, amount: 1, wearable, worn: 0, refine: 0, identified, cards: [0; 4] };
        // A potion and an arrow do; loot does not.
        assert!(item(0, 0, true).fits_hotkeys());
        assert!(item(10, 0, true).fits_hotkeys());
        assert!(!item(3, 0, true).fits_hotkeys());
        // A weapon does if it has a place to be worn, and nothing does until it is identified.
        assert!(item(5, 2, true).fits_hotkeys());
        assert!(!item(5, 0, true).fits_hotkeys());
        assert!(!item(5, 2, false).fits_hotkeys());
    }

}
