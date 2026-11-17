//! Buying from and selling to NPC shops, and the Kafra's storage (rAthena, `clif_npcbuysell`, `clif_buylist`,
//! `clif_selllist`, `clif_storagelist` and their neighbours in `src/map/clif.cpp`, at PACKETVER 20260219).

use super::items::Item;

/// ZC_SELECT_DEALTYPE (0x00c4): `u32 npc`: the shop asks whether to buy or to sell.
pub const ZC_SELECT_DEALTYPE: u16 = 0x00c4;
/// ZC_PC_PURCHASE_ITEMLIST (0x0b77): `u16 length`, then 19 bytes an item: `u32 id, u32 price, u32 discounted,
/// u8 type, u16 look, u32 wearable`.
pub const ZC_PC_PURCHASE_ITEMLIST: u16 = 0x0b77;
/// ZC_PC_SELL_ITEMLIST (0x00c7): `u16 length`, then 10 bytes an item: `u16 index, u32 price, u32 overcharged`.
pub const ZC_PC_SELL_ITEMLIST: u16 = 0x00c7;
/// ZC_PC_PURCHASE_RESULT (0x00ca) and ZC_PC_SELL_RESULT (0x00cb): `u8 result`, 0 for done.
pub const ZC_PC_PURCHASE_RESULT: u16 = 0x00ca;
pub const ZC_PC_SELL_RESULT: u16 = 0x00cb;
/// ZC_NOTIFY_STOREITEM_COUNTINFO (0x00f2): `u16 kinds, u16 most`: the storage is open, and how full it is.
pub const ZC_NOTIFY_STOREITEM_COUNTINFO: u16 = 0x00f2;
/// ZC_ADD_ITEM_TO_STORE (0x0b44): `i16 index, i32 amount, u32 id, u8 type, u8 identified, u8 damaged,
/// u32[4] cards, 25 bytes of options, u8 refine, u8 grade`.
pub const ZC_ADD_ITEM_TO_STORE: u16 = 0x0b44;
/// ZC_DELETE_ITEM_FROM_STORE (0x00f6): `u16 index, u32 amount`.
pub const ZC_DELETE_ITEM_FROM_STORE: u16 = 0x00f6;
/// ZC_CLOSE_STORE (0x00f8): the storage is closed.
pub const ZC_CLOSE_STORE: u16 = 0x00f8;
/// Which list a ZC_INVENTORY packet is of: the storage.
pub const STORAGE: u8 = 2;

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// Something a shop sells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offer {
    pub id: u32,
    /// What it costs this character (the discount counted in).
    pub price: u32,
}

pub fn offers(packet: &[u8]) -> Vec<Offer> {
    packet.get(4..).unwrap_or_default().chunks_exact(19).map(|item| Offer { id: u32_at(item, 0), price: u32_at(item, 8) }).collect()
}

/// What the shop gives for each thing carried that it takes: `(index, price)`, the overcharge counted in.
pub fn bids(packet: &[u8]) -> Vec<(u16, u32)> {
    packet.get(4..).unwrap_or_default().chunks_exact(10).map(|item| (u16_at(item, 0), u32_at(item, 6))).collect()
}

/// CZ_ACK_SELECT_DEALTYPE (0x00c5): `u32 npc, u8 type`: 0 to buy, 1 to sell.
pub fn deal(npc: u32, sell: bool) -> Vec<u8> {
    let mut packet = 0x00c5u16.to_le_bytes().to_vec();
    packet.extend(npc.to_le_bytes());
    packet.push(u8::from(sell));
    packet
}

/// CZ_PC_PURCHASE_ITEMLIST (0x00c8): `u16 length`, then `u16 amount, u32 id` for each thing bought. An empty list
/// leaves the shop.
pub fn buy(wanted: &[(u32, u16)]) -> Vec<u8> {
    let mut packet = 0x00c8u16.to_le_bytes().to_vec();
    packet.extend(((4 + wanted.len() * 6) as u16).to_le_bytes());
    for (id, amount) in wanted {
        packet.extend(amount.to_le_bytes());
        packet.extend(id.to_le_bytes());
    }
    packet
}

/// CZ_PC_SELL_ITEMLIST (0x00c9): `u16 length`, then `u16 index, u16 amount` for each thing sold.
pub fn sell(sold: &[(u16, u16)]) -> Vec<u8> {
    let mut packet = 0x00c9u16.to_le_bytes().to_vec();
    packet.extend(((4 + sold.len() * 4) as u16).to_le_bytes());
    for (index, amount) in sold {
        packet.extend(index.to_le_bytes());
        packet.extend(amount.to_le_bytes());
    }
    packet
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

/// CZ_MOVE_ITEM_FROM_BODY_TO_STORE (0x0364): `u16 index, u32 amount`.
pub fn to_storage(index: u16, amount: u32) -> Vec<u8> {
    moved(0x0364, index, amount)
}

/// CZ_MOVE_ITEM_FROM_STORE_TO_BODY (0x0365): `u16 index, u32 amount`.
pub fn from_storage(index: u16, amount: u32) -> Vec<u8> {
    moved(0x0365, index, amount)
}

/// CZ_CLOSE_STORE (0x0193).
pub fn close_storage() -> Vec<u8> {
    0x0193u16.to_le_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_a_shop_sells_and_takes() {
        let mut selling = vec![0x77, 0x0b, 23, 0];
        selling.extend([0xf5, 1, 0, 0, 50, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(offers(&selling), [Offer { id: 501, price: 45 }]);
        let mut taking = vec![0xc7, 0, 14, 0];
        taking.extend([3, 0, 25, 0, 0, 0, 27, 0, 0, 0]);
        assert_eq!(bids(&taking), [(3, 27)]);
    }

    #[test]
    fn the_requests_have_the_lengths_the_server_expects() {
        assert_eq!(buy(&[(501, 3)]), [0xc8, 0, 10, 0, 3, 0, 0xf5, 1, 0, 0]);
        assert_eq!(sell(&[(3, 2)]), [0xc9, 0, 8, 0, 3, 0, 2, 0]);
        for packet in [deal(1, true), to_storage(1, 1), from_storage(1, 1), close_storage()] {
            let id = u16::from_le_bytes([packet[0], packet[1]]);
            assert_eq!(crate::net::packet_length(id), Some(packet.len() as i32), "{id:#06x}");
        }
        for (id, length) in [(ZC_SELECT_DEALTYPE, 6), (ZC_ADD_ITEM_TO_STORE, 58), (ZC_DELETE_ITEM_FROM_STORE, 8), (ZC_CLOSE_STORE, 2), (ZC_NOTIFY_STOREITEM_COUNTINFO, 6)] {
            assert_eq!(crate::net::packet_length(id), Some(length), "{id:#06x}");
        }
    }
}
