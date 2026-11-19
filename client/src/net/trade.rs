//! Buying from and selling to NPC shops (rAthena, `clif_npcbuysell`, `clif_buylist`, `clif_selllist`,
//! `clif_npc_buy_result`, `clif_npc_sell_result` in `src/map/clif.cpp`, at PACKETVER 20260219). The rules of
//! the client's two windows of a shop (what may go into the basket and what is typed into the box that asks
//! how many: `UIItemPurchaseWnd::SendMsg`, `UIItemDropCntWnd::SendMsg`) are in `crate::shops`, and named here
//! too for those who look for them under `net::trade`.

use super::bytes::{Writer, records, u16_at, u32_at};

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

/// Something a shop sells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Offer {
    pub id: u32,
    /// What it costs, and what it costs this character (the Discount skill counted in): the client writes
    /// both where they differ.
    pub price: u32,
    pub discounted: u32,
    /// The server's type of item (the client's `TYPE_…`).
    pub kind: u8,
}

pub fn offers(packet: &[u8]) -> Vec<Offer> {
    records(packet, 4, 19).map(|item| Offer { id: u32_at(item, 0), price: u32_at(item, 4), discounted: u32_at(item, 8), kind: item[12] }).collect()
}

/// What a shop gives for a thing carried that it takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bid {
    /// The thing's place in the inventory.
    pub index: u16,
    /// What it is worth, and what this character gets (the Overcharge skill counted in).
    pub price: u32,
    pub overcharged: u32,
}

pub fn bids(packet: &[u8]) -> Vec<Bid> {
    records(packet, 4, 10).map(|item| Bid { index: u16_at(item, 0), price: u32_at(item, 2), overcharged: u32_at(item, 6) }).collect()
}

pub use crate::shops::{Asked, MOST_BOUGHT, bought_singly, count_asked, into_basket, name_width};

/// The `u8 result` of ZC_PC_PURCHASE_RESULT (0x00ca) and of ZC_PC_SELL_RESULT (0x00cb), for `purchase_line`
/// and `sell_line`; nought, which is "done", for a packet too short to have it.
pub fn result(packet: &[u8]) -> u8 {
    packet.get(2).copied().unwrap_or(0)
}

/// The line of the client's message table said for ZC_PC_PURCHASE_RESULT's `u8 result`, and whether in red
/// (else in the chat's yellow): `CGameMode`'s handler says 54 for 0, 55 for 1 (not enough zeny), 56 for 2 (too
/// heavy), 221 for 3 (too many things) and 1940 for 9. Its other arms (10 to 14) are of shops this server's
/// classic rules do not have.
pub fn purchase_line(result: u8) -> Option<(usize, bool)> {
    match result {
        0 => Some((54, false)),
        1 => Some((55, true)),
        2 => Some((56, true)),
        3 => Some((221, true)),
        9 => Some((1940, true)),
        _ => None,
    }
}

/// The same for ZC_PC_SELL_RESULT: 54 for 0, 57 in red for 1.
pub fn sell_line(result: u8) -> Option<(usize, bool)> {
    match result {
        0 => Some((54, false)),
        1 => Some((57, true)),
        _ => None,
    }
}

/// CZ_ACK_SELECT_DEALTYPE (0x00c5): `u32 npc, u8 type`: 0 to buy, 1 to sell.
pub fn deal(npc: u32, sell: bool) -> Vec<u8> {
    Writer::new(0x00c5).u32(npc).u8(u8::from(sell)).done()
}

/// CZ_PC_PURCHASE_ITEMLIST (0x00c8): `u16 length`, then `u16 amount, u32 id` for each thing bought.
pub fn buy(wanted: &[(u32, u16)]) -> Vec<u8> {
    wanted.iter().fold(Writer::sized(0x00c8), |packet, (id, amount)| packet.u16(*amount).u32(*id)).done()
}

/// CZ_NPC_TRADE_QUIT (0x09d4): the windows of an NPC's shop were closed.
pub fn quit() -> Vec<u8> {
    Writer::new(0x09d4).done()
}

/// CZ_PC_SELL_ITEMLIST (0x00c9): `u16 length`, then `u16 index, u16 amount` for each thing sold.
pub fn sell(sold: &[(u16, u16)]) -> Vec<u8> {
    sold.iter().fold(Writer::sized(0x00c9), |packet, (index, amount)| packet.u16(*index).u16(*amount)).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_a_shop_sells_and_takes() {
        let mut selling = vec![0x77, 0x0b, 23, 0];
        selling.extend([0xf5, 1, 0, 0, 50, 0, 0, 0, 45, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0]);
        assert_eq!(offers(&selling), [Offer { id: 501, price: 50, discounted: 45, kind: 5 }]);
        let mut taking = vec![0xc7, 0, 14, 0];
        taking.extend([3, 0, 25, 0, 0, 0, 27, 0, 0, 0]);
        assert_eq!(bids(&taking), [Bid { index: 3, price: 25, overcharged: 27 }]);
    }

    #[test]
    fn the_requests_have_the_lengths_the_server_expects() {
        assert_eq!(buy(&[(501, 3)]), [0xc8, 0, 10, 0, 3, 0, 0xf5, 1, 0, 0]);
        assert_eq!(sell(&[(3, 2)]), [0xc9, 0, 8, 0, 3, 0, 2, 0]);
        crate::net::bytes::testing::assert_lengths(&[deal(1, true)]);
        for (id, length) in [(ZC_SELECT_DEALTYPE, 6), (ZC_PC_PURCHASE_RESULT, 3), (ZC_PC_SELL_RESULT, 3)] {
            assert_eq!(crate::net::packet_length(id), Some(length), "{id:#06x}");
        }
    }

    #[test]
    fn the_results_have_the_clients_lines() {
        assert_eq!([purchase_line(0), purchase_line(1), purchase_line(2), purchase_line(3), purchase_line(9), purchase_line(4)], [Some((54, false)), Some((55, true)), Some((56, true)), Some((221, true)), Some((1940, true)), None]);
        assert_eq!([sell_line(0), sell_line(1)], [Some((54, false)), Some((57, true))]);
        // The result is the packet's third byte, and "done" where it has none.
        assert_eq!((result(&[0xca, 0, 2]), result(&[0xcb, 0, 1]), result(&[0xca, 0])), (2, 1, 0));
    }
}
