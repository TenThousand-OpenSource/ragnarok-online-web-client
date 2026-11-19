//! Buying from and selling to NPC shops (rAthena, `clif_npcbuysell`, `clif_buylist`, `clif_selllist`,
//! `clif_npc_buy_result`, `clif_npc_sell_result` in `src/map/clif.cpp`, at PACKETVER 20260219), and the rules
//! of the client's two windows of a shop: what may go into the basket and what is typed into the box that asks
//! how many (`UIItemPurchaseWnd::SendMsg`, `UIItemDropCntWnd::SendMsg`).

use super::bytes::{u16_at, u32_at};

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
    packet.get(4..).unwrap_or_default().chunks_exact(19).map(|item| Offer { id: u32_at(item, 0), price: u32_at(item, 4), discounted: u32_at(item, 8), kind: item[12] }).collect()
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
    packet.get(4..).unwrap_or_default().chunks_exact(10).map(|item| Bid { index: u16_at(item, 0), price: u32_at(item, 2), overcharged: u32_at(item, 6) }).collect()
}

/// The most of one item that is bought at a time (`UIItemPurchaseWnd::SendMsg`, `UM_DROP`: 30000).
pub const MOST_BOUGHT: u32 = 30000;

/// Whether a thing is of the kinds the client lets into the basket once and without asking how many: what is
/// worn (`TYPE_ARMOR`, `TYPE_WEAPON`, `TYPE_BOW`, `TYPE_BOTHHAND`, `TYPE_ARMORTM`…`TYPE_ARMORTMB`, `TYPE_GUN`).
pub fn bought_singly(kind: u8) -> bool {
    matches!(kind, 4 | 5 | 8 | 9 | 11..=15)
}

/// How many of `asked` more go into the basket that has `kept` of the item already, and whether the client
/// says its line 1704 (no more than 30000 of one item at a time).
pub fn into_basket(kept: u32, asked: u32) -> (u32, bool) {
    if kept + asked > MOST_BOUGHT { (MOST_BOUGHT.saturating_sub(kept), true) } else { (asked, false) }
}

/// What the box that asks how many is asked for (`UIItemDropCntWnd`'s states).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// How many to buy (state 1): nothing for a number under one, and no more than 30000.
    Buying,
    /// How many of what there is to move: to sell, out of a basket, out of the storage (states 2, 3, 4, 6):
    /// nothing for a number under one, and no more than there are.
    Moving,
    /// How many to put into the storage (state 7): a number under one counts as one, and no more than there are.
    Storing,
}

/// The number typed into the box, as the client reads it (`atoi`: the digits a text begins with, nought for
/// anything else), by what was asked and how many there are.
pub fn count_asked(typed: &str, most: u32, asked: Asked) -> Option<u32> {
    let digits: String = typed.trim().chars().take_while(char::is_ascii_digit).take(9).collect();
    let count = digits.parse::<u32>().unwrap_or(0);
    match asked {
        Asked::Buying => (count > 0).then(|| count.min(MOST_BOUGHT)),
        Asked::Moving => (count > 0 && most > 0).then(|| count.min(most)),
        Asked::Storing => (most > 0).then(|| count.clamp(1, most)),
    }
}

/// How wide the client leaves an item's name in a row of a shop's windows: `usual` (100 in the list and among
/// what is sold, 110 among what is bought), less when two prices are written and the first has more figures.
pub fn name_width(price: u32, differs: bool, usual: i32) -> i32 {
    match price {
        _ if !differs => usual,
        10000.. => 45,
        1000.. => 55,
        100.. => 70,
        10.. => 80,
        _ => usual,
    }
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
    let mut packet = 0x00c5u16.to_le_bytes().to_vec();
    packet.extend(npc.to_le_bytes());
    packet.push(u8::from(sell));
    packet
}

/// CZ_PC_PURCHASE_ITEMLIST (0x00c8): `u16 length`, then `u16 amount, u32 id` for each thing bought.
pub fn buy(wanted: &[(u32, u16)]) -> Vec<u8> {
    let mut packet = 0x00c8u16.to_le_bytes().to_vec();
    packet.extend(((4 + wanted.len() * 6) as u16).to_le_bytes());
    for (id, amount) in wanted {
        packet.extend(amount.to_le_bytes());
        packet.extend(id.to_le_bytes());
    }
    packet
}

/// CZ_NPC_TRADE_QUIT (0x09d4): the windows of an NPC's shop were closed.
pub fn quit() -> Vec<u8> {
    0x09d4u16.to_le_bytes().to_vec()
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
        let packet = deal(1, true);
        assert_eq!(crate::net::packet_length(0x00c5), Some(packet.len() as i32));
        for (id, length) in [(ZC_SELECT_DEALTYPE, 6), (ZC_PC_PURCHASE_RESULT, 3), (ZC_PC_SELL_RESULT, 3)] {
            assert_eq!(crate::net::packet_length(id), Some(length), "{id:#06x}");
        }
    }

    #[test]
    fn the_basket_takes_what_the_client_lets_into_it() {
        assert_eq!(into_basket(0, 5), (5, false));
        assert_eq!(into_basket(29990, 20), (10, true));
        assert_eq!(into_basket(30000, 1), (0, true));
        assert!(bought_singly(5) && bought_singly(4) && bought_singly(15) && !bought_singly(0) && !bought_singly(10) && !bought_singly(3));
    }

    #[test]
    fn the_number_typed_is_read_as_each_state_of_the_box_reads_it() {
        assert_eq!(count_asked("12", 0, Asked::Buying), Some(12));
        assert_eq!(count_asked("99999", 0, Asked::Buying), Some(30000));
        assert_eq!(count_asked("0", 0, Asked::Buying), None);
        assert_eq!(count_asked("x", 0, Asked::Buying), None);
        assert_eq!(count_asked("7", 5, Asked::Moving), Some(5));
        assert_eq!(count_asked(" 3 ", 5, Asked::Moving), Some(3));
        assert_eq!(count_asked("", 5, Asked::Moving), None);
        assert_eq!(count_asked("0", 5, Asked::Storing), Some(1));
        assert_eq!(count_asked("12a", 50, Asked::Storing), Some(12));
        assert_eq!(count_asked("60", 50, Asked::Storing), Some(50));
    }

    #[test]
    fn a_name_is_left_less_room_beside_two_prices() {
        assert_eq!(name_width(12000, false, 100), 100);
        assert_eq!([name_width(12000, true, 100), name_width(1200, true, 100), name_width(120, true, 100), name_width(12, true, 100), name_width(5, true, 110)], [45, 55, 70, 80, 110]);
    }

    #[test]
    fn the_results_have_the_clients_lines() {
        assert_eq!([purchase_line(0), purchase_line(1), purchase_line(2), purchase_line(3), purchase_line(9), purchase_line(4)], [Some((54, false)), Some((55, true)), Some((56, true)), Some((221, true)), Some((1940, true)), None]);
        assert_eq!([sell_line(0), sell_line(1)], [Some((54, false)), Some((57, true))]);
    }
}
