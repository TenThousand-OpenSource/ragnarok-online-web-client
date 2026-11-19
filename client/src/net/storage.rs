//! The Kafra's storage and a guild's, which the server opens, fills and closes with the same packets (rAthena,
//! `clif_storagelist`, `clif_updatestorageamount`, `clif_storageitemadded`, `clif_storageitemremoved`,
//! `clif_storageclose`, `clif_parse_MoveToKafra` and its neighbours in `src/map/clif.cpp`, at PACKETVER
//! 20260219). The rules of the client's window of it (`UIItemStoreWnd`: which tab a thing is under, the orders
//! of its lists) are in `crate::storage`.
//!
//! A storage's list comes as the inventory's does (`net::inventory`: ZC_INVENTORY_START with the storage's name,
//! then the lists of the kind `STORAGE`); the server tells a guild's storage as that kind too, under the name
//! "Guild Storage".

use super::bytes::{Writer, u16_at, u32_at};
use super::inventory::{Amount, Identified, Item, Layout};

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

/// The item of ZC_ADD_ITEM_TO_STORE, counted from the packet's start.
const STORED: Layout = Layout {
    index: Some(2), id: 8, kind: 12, amount: Amount::Long(4),
    wearable: None, worn: None, refine: Some(56), identified: Identified::Byte(13), cards: 15,
};

/// What ZC_ADD_ITEM_TO_STORE puts into the storage.
pub fn stored(packet: &[u8]) -> Option<Item> {
    if packet.len() < 58 {
        return None;
    }
    Item::read(packet, &STORED)
}

/// `(index, amount)` of ZC_DELETE_ITEM_FROM_STORE.
pub fn taken(packet: &[u8]) -> Option<(u16, u32)> {
    (packet.len() >= 8).then(|| (u16_at(packet, 2), u32_at(packet, 4)))
}

/// A request that moves so many of the thing kept at that place from one of the inventory, the cart and the
/// storage to another, 8 bytes: `u16 index, u32 amount` (the cart's own two are in `net::cart`).
pub(super) fn moved(id: u16, index: u16, amount: u32) -> Vec<u8> {
    Writer::new(id).u16(index).u32(amount).done()
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
    Writer::new(0x0193).done()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::assert_lengths;

    #[test]
    fn the_requests_have_the_lengths_the_server_expects() {
        assert_eq!(to_storage(3, 7), [0x64, 0x03, 3, 0, 7, 0, 0, 0]);
        assert_eq!(from_storage(3, 7), [0x65, 0x03, 3, 0, 7, 0, 0, 0]);
        assert_lengths(&[to_storage(1, 1), from_storage(1, 1), cart_to_storage(1, 1), storage_to_cart(1, 1), close_storage()]);
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
}
