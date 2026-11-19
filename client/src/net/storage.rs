//! The Kafra's storage and a guild's, which the server opens, fills and closes with the same packets (rAthena,
//! `clif_storagelist`, `clif_updatestorageamount`, `clif_storageitemadded`, `clif_storageitemremoved`,
//! `clif_storageclose`, `clif_parse_MoveToKafra` and its neighbours in `src/map/clif.cpp`, at PACKETVER
//! 20260219), and the rules of the client's window of it (`UIItemStoreWnd`): which tab a thing is under.
//!
//! A storage's list comes as the inventory's does (`net::items`: ZC_INVENTORY_START with the storage's name,
//! then the lists of the kind `STORAGE`); the server tells a guild's storage as that kind too, under the name
//! "Guild Storage".

use super::bytes::{u16_at, u32_at};
use super::items::Item;

/// ZC_NOTIFY_STOREITEM_COUNTINFO (0x00f2): `u16 kinds, u16 most`: the storage is open, and how full it is.
pub const ZC_NOTIFY_STOREITEM_COUNTINFO: u16 = 0x00f2;
/// ZC_ADD_ITEM_TO_STORE (0x0b44): `i16 index, i32 amount, u32 id, u8 type, u8 identified, u8 damaged,
/// u32[4] cards, 25 bytes of options, u8 refine, u8 grade`.
pub const ZC_ADD_ITEM_TO_STORE: u16 = 0x0b44;
/// ZC_DELETE_ITEM_FROM_STORE (0x00f6): `u16 index, u32 amount`.
pub const ZC_DELETE_ITEM_FROM_STORE: u16 = 0x00f6;
/// ZC_CLOSE_STORE (0x00f8): the storage is closed.
pub const ZC_CLOSE_STORE: u16 = 0x00f8;
/// Which list a ZC_INVENTORY packet is of: the storage (`INVTYPE_STORAGE`).
pub const STORAGE: u8 = 2;

/// The name ZC_INVENTORY_START (0x0b08: `u16 length, u8 kind`, then the name) gives the list it begins: the
/// client writes it in the title of the storage's window (`CGameMode::ReceiveInventoryStart`), when it is not
/// empty.
pub fn list_name(packet: &[u8]) -> Option<String> {
    let name = super::bytes::text(packet.get(5..)?);
    (!name.is_empty()).then_some(name)
}

/// `(kinds, most)` of ZC_NOTIFY_STOREITEM_COUNTINFO.
pub fn storage_count(packet: &[u8]) -> Option<(u16, u16)> {
    (packet.len() >= 6).then(|| (u16_at(packet, 2), u16_at(packet, 4)))
}

/// What ZC_ADD_ITEM_TO_STORE puts into the storage.
pub fn stored(packet: &[u8]) -> Option<Item> {
    (packet.len() >= 58).then(|| Item {
        index: u16_at(packet, 2),
        amount: u32_at(packet, 4).min(u32::from(u16::MAX)) as u16,
        id: u32_at(packet, 8),
        kind: packet[12],
        identified: packet[13] != 0,
        cards: [0, 1, 2, 3].map(|slot| u32_at(packet, 15 + slot * 4)),
        refine: packet[56],
        wearable: 0,
        worn: 0,
    })
}

/// `(index, amount)` of ZC_DELETE_ITEM_FROM_STORE.
pub fn taken(packet: &[u8]) -> Option<(u16, u32)> {
    (packet.len() >= 8).then(|| (u16_at(packet, 2), u32_at(packet, 4)))
}

fn moved(id: u16, index: u16, amount: u32) -> Vec<u8> {
    let mut packet = id.to_le_bytes().to_vec();
    packet.extend(index.to_le_bytes());
    packet.extend(amount.to_le_bytes());
    packet
}

/// CZ_MOVE_ITEM_FROM_BODY_TO_STORE (0x0364): `u16 index, u32 amount`: so many of a thing carried go into the
/// storage that is open, the Kafra's or the guild's.
pub fn to_storage(index: u16, amount: u32) -> Vec<u8> {
    moved(0x0364, index, amount)
}

/// CZ_MOVE_ITEM_FROM_STORE_TO_BODY (0x0365): `u16 index, u32 amount`: so many of a thing in the storage come
/// out of it into the inventory.
pub fn from_storage(index: u16, amount: u32) -> Vec<u8> {
    moved(0x0365, index, amount)
}

/// CZ_MOVE_ITEM_FROM_CART_TO_STORE (0x0129): `u16 index, u32 amount`: so many of a thing in the cart (its
/// place there) go into the storage.
pub fn cart_to_storage(index: u16, amount: u32) -> Vec<u8> {
    moved(0x0129, index, amount)
}

/// CZ_MOVE_ITEM_FROM_STORE_TO_CART (0x0128): `u16 index, u32 amount`: so many of a thing in the storage go
/// into the cart.
pub fn storage_to_cart(index: u16, amount: u32) -> Vec<u8> {
    moved(0x0128, index, amount)
}

/// CZ_CLOSE_STORE (0x0193).
pub fn close_storage() -> Vec<u8> {
    0x0193u16.to_le_bytes().to_vec()
}

/// How many tabs the storage's window has: things used, of the cash shop, armour, weapons, costumes,
/// ammunition, cards, and the rest.
pub const TABS: usize = 8;

/// The line of the client's message table each tab is called by (what its button says when pointed at, and
/// the title of the window that shows the tab alone: `UIItemStoreWnd::OnCreate`, `UIWindowMgr`), with the
/// English of it.
pub const TAB_LINES: [(usize, &str); TABS] = [(1465, "Use"), (1466, "Cash"), (1467, "Armors"), (1468, "Weapons"), (3151, "Costumes"), (1469, "Ammo"), (1470, "Card"), (1471, "Etc")];

/// The places a costume is worn in (the server's `EQP_COSTUME_HEAD_TOP`, `_MID`, `_LOW` and `_GARMENT`).
pub const COSTUME_PLACES: u32 = 0x400 | 0x800 | 0x1000 | 0x2000;

/// Which tab a thing is under (`UIItemStoreWnd::SendMsg`, `UM_REFRESHITEM`), by its type and whether it is a
/// costume: `None` for a type the window lists nowhere. The client asks its item table whether a thing is a
/// costume; here it is one that is worn in a costume's place, which the server tells with the thing.
pub fn tab_of(item: &Item) -> Option<usize> {
    if item.wearable & COSTUME_PLACES != 0 {
        return Some(4);
    }
    match item.kind {
        0..=2 => Some(0),
        18 => Some(1),
        4 | 11..=14 => Some(2),
        5 | 8 | 9 | 15 => Some(3),
        10 | 16 | 17 | 19 => Some(5),
        6 => Some(6),
        3 | 7 => Some(7),
        _ => None,
    }
}

/// How the storage's lists are ordered (the window's box of three choices): as the server told them, or by
/// name, forwards or backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sorted {
    #[default]
    AsTold,
    Ascending,
    Descending,
}

impl Sorted {
    pub const ALL: [Sorted; 3] = [Sorted::AsTold, Sorted::Ascending, Sorted::Descending];

    /// The line of the client's message table the choice is called by, with the English the table has.
    pub fn line(self) -> (usize, &'static str) {
        match self {
            Sorted::AsTold => (3276, "Base"),
            Sorted::Ascending => (3274, "Upgrade"),
            Sorted::Descending => (3275, "Downgrade"),
        }
    }

    /// Puts names (each with what it names) into this order.
    pub fn order<Thing>(self, things: &mut [(String, Thing)]) {
        match self {
            Sorted::AsTold => {}
            Sorted::Ascending => things.sort_by(|one, other| one.0.cmp(&other.0)),
            Sorted::Descending => things.sort_by(|one, other| other.0.cmp(&one.0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thing(kind: u8, wearable: u32) -> Item {
        Item { index: 1, id: 501, kind, amount: 1, wearable, worn: 0, refine: 0, identified: true, cards: [0; 4] }
    }

    #[test]
    fn the_requests_have_the_lengths_the_server_expects() {
        assert_eq!(to_storage(3, 7), [0x64, 0x03, 3, 0, 7, 0, 0, 0]);
        assert_eq!(from_storage(3, 7), [0x65, 0x03, 3, 0, 7, 0, 0, 0]);
        for packet in [to_storage(1, 1), from_storage(1, 1), cart_to_storage(1, 1), storage_to_cart(1, 1), close_storage()] {
            let id = u16::from_le_bytes([packet[0], packet[1]]);
            assert_eq!(crate::net::packet_length(id), Some(packet.len() as i32), "{id:#06x}");
        }
        for (id, length) in [(ZC_ADD_ITEM_TO_STORE, 58), (ZC_DELETE_ITEM_FROM_STORE, 8), (ZC_CLOSE_STORE, 2), (ZC_NOTIFY_STOREITEM_COUNTINFO, 6)] {
            assert_eq!(crate::net::packet_length(id), Some(length), "{id:#06x}");
        }
    }

    #[test]
    fn what_the_server_says_of_the_storage_is_read() {
        assert_eq!(storage_count(&[0xf2, 0, 5, 0, 0x58, 0x02]), Some((5, 600)));
        assert_eq!(taken(&[0xf6, 0, 4, 0, 9, 0, 0, 0]), Some((4, 9)));
        let mut added = vec![0u8; 58];
        added[2] = 6;
        added[4] = 30;
        added[8..12].copy_from_slice(&909u32.to_le_bytes());
        added[12] = 3;
        added[13] = 1;
        let item = stored(&added).unwrap();
        assert_eq!((item.index, item.amount, item.id, item.kind, item.identified), (6, 30, 909, 3, true));
        let mut start = vec![0x08, 0x0b, 19, 0, 2];
        start.extend(b"Guild Storage\0");
        assert_eq!(list_name(&start).as_deref(), Some("Guild Storage"));
        assert_eq!(list_name(&[0x08, 0x0b, 6, 0, 0, 0]), None);
    }

    #[test]
    fn a_thing_is_under_the_tab_of_its_kind() {
        let tabs: Vec<Option<usize>> = [0, 2, 18, 4, 12, 5, 8, 15, 10, 16, 6, 3, 7, 20].iter().map(|kind| tab_of(&thing(*kind, 0))).collect();
        assert_eq!(tabs, [Some(0), Some(0), Some(1), Some(2), Some(2), Some(3), Some(3), Some(3), Some(5), Some(5), Some(6), Some(7), Some(7), None]);
        // A costume is under its own tab, whatever it is worn as.
        assert_eq!(tab_of(&thing(4, 0x400)), Some(4));
        assert_eq!(tab_of(&thing(4, 0x100)), Some(2));
    }

    #[test]
    fn the_lists_are_ordered_by_name() {
        let mut things = vec![("Jellopy".to_owned(), 1), ("Apple".to_owned(), 2), ("Red Potion".to_owned(), 3)];
        Sorted::AsTold.order(&mut things);
        assert_eq!(things[0].1, 1);
        Sorted::Ascending.order(&mut things);
        assert_eq!([things[0].1, things[1].1, things[2].1], [2, 1, 3]);
        Sorted::Descending.order(&mut things);
        assert_eq!([things[0].1, things[1].1, things[2].1], [3, 1, 2]);
    }
}
