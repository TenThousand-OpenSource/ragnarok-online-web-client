//! The inventory's fourth tab, "Fav": which of the things carried the player keeps under it. The server keeps
//! the mark with the item and tells it in the lists, when a thing is picked up, and when it is moved to the tab
//! or off it (rAthena, `clif_parse_MoveItem`, `clif_favorite_item`, at PACKETVER 20260219).

use super::*;

/// ZC_INVENTORY_TAB (0x0908): `u16 index, u8 off`: a thing is under the tab now (0) or no more (1).
pub const ZC_INVENTORY_TAB: u16 = 0x0908;

/// The places in the inventory of the things of a ZC_INVENTORY_ITEMLIST_NORMAL (0x0b09) that are kept under
/// the tab: bit 1 of an item's last byte (`PlaceETCTab`). Nothing for a list of another inventory.
pub fn favourite_stacks(packet: &[u8]) -> Vec<u16> {
    listed(packet, 34, 33, 2)
}

/// The same of a ZC_INVENTORY_ITEMLIST_EQUIP (0x0b39): bit 2 of an item's last byte.
pub fn favourite_equipment(packet: &[u8]) -> Vec<u16> {
    listed(packet, 68, 67, 4)
}

fn listed(packet: &[u8], size: usize, flags: usize, bit: u8) -> Vec<u16> {
    if packet.get(4) != Some(&OWN_INVENTORY) {
        return Vec::new();
    }
    packet[5..].chunks_exact(size).filter(|item| item[flags] & bit != 0).map(|item| u16_at(item, 0)).collect()
}

/// Whether the thing a ZC_ITEM_PICKUP_ACK (0x0b41) tells of is kept under the tab: its byte 65 (`favorite`).
pub fn favourite_gained(packet: &[u8]) -> bool {
    packet.get(65).is_some_and(|favourite| *favourite != 0)
}

/// `(index, kept)` of ZC_INVENTORY_TAB: the client (`ReceiveInventoryItemUpdate`) keeps the thing under the
/// tab when the byte is 0.
pub fn favourite_moved(packet: &[u8]) -> Option<(u16, bool)> {
    (packet.len() >= 5).then(|| (u16_at(packet, 2), packet[4] == 0))
}

/// CZ_INVENTORY_TAB (0x0907): `u16 index, u8 off`: a thing dragged onto the inventory's tabs. Let go on the
/// "Fav" tab it is kept under it (0), on any other it is taken from under it (1) (`UIItemWnd::DragAndDrop`).
pub fn move_to_tab(index: u16, favourite: bool) -> Vec<u8> {
    [&0x0907u16.to_le_bytes()[..], &index.to_le_bytes(), &[u8::from(!favourite)]].concat()
}

/// The order the things under the tab are put in while its "Align items" button is on (`ItemTypeNameLess`):
/// by this rank of their kind (`ItemTypeSortRank`), then by name.
pub fn sort_rank(kind: u8) -> u8 {
    match kind {
        0..=2 => 0,
        18 => 1,
        4 | 11..=14 => 2,
        5 | 8 | 9 | 15 => 3,
        10 | 16 | 17 | 19 => 4,
        6 => 5,
        3 | 7 => 6,
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet_length;

    #[test]
    fn the_lists_tell_what_is_kept_under_the_tab() {
        let mut stacks = vec![0x09, 0x0b, 0, 0, 0];
        for (index, flags) in [(2u16, 1u8), (3, 3), (4, 2)] {
            let mut item = vec![0u8; 34];
            item[..2].copy_from_slice(&index.to_le_bytes());
            item[33] = flags;
            stacks.extend(item);
        }
        assert_eq!(favourite_stacks(&stacks), [3, 4]);
        let mut worn = vec![0x39, 0x0b, 0, 0, 0];
        for (index, flags) in [(7u16, 1u8), (8, 5), (9, 3)] {
            let mut item = vec![0u8; 68];
            item[..2].copy_from_slice(&index.to_le_bytes());
            item[67] = flags;
            worn.extend(item);
        }
        assert_eq!(favourite_equipment(&worn), [8]);
        // The storage's list says nothing of the inventory.
        worn[4] = 2;
        assert_eq!(favourite_equipment(&worn), []);
    }

    #[test]
    fn the_kinds_are_ranked_as_the_client_orders_them() {
        assert_eq!([0, 18, 4, 5, 10, 6, 3, 20].map(sort_rank), [0, 1, 2, 3, 4, 5, 6, 10]);
    }

    #[test]
    fn a_thing_is_moved_to_the_tab_and_off_it() {
        assert_eq!(move_to_tab(5, true), [0x07, 0x09, 5, 0, 0]);
        assert_eq!(move_to_tab(5, false), [0x07, 0x09, 5, 0, 1]);
        assert_eq!(packet_length(0x0907), Some(5));
        assert_eq!(packet_length(ZC_INVENTORY_TAB), Some(5));
        assert_eq!(favourite_moved(&[0x08, 0x09, 5, 0, 0]), Some((5, true)));
        assert_eq!(favourite_moved(&[0x08, 0x09, 5, 0, 1]), Some((5, false)));
        let mut gained = vec![0u8; 70];
        assert!(!favourite_gained(&gained));
        gained[65] = 1;
        assert!(favourite_gained(&gained));
    }
}
