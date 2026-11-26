//! The cash shop: asking for its window and closing it, the lists of what it offers, buying, and the server's
//! answers. The layouts are rAthena's (`clif_cashshop_open`, `clif_cashshop_list`, `clif_cashshop_result`,
//! `clif_parse_cashshop_buy` in `src/map/clif.cpp`); the client's side is `CGameMode::SendMsg` at
//! `MM_REQUEST_CASH_SHOP` and `MM_GET_ACCOUNT_LIMITED_SALE_LIST`, `CGameMode::ReceiveCashShopBanner`,
//! `ReceiveTypedItemList`, `ReceiveRequestResult` and `UICashShopWnd::SendMsg`.

use super::bytes::{Writer, long, records, short, u32_at, word};
use crate::cashshop::Good;

/// ZC_SE_CASHSHOP_OPEN (0x0b6e, 14 bytes): `u32 cash points`, `u32 free (kafra) points`, `u32 tab`.
pub const ZC_SE_CASHSHOP_OPEN: u16 = 0x0b6e;
/// ZC_ACK_SCHEDULER_CASHITEM (0x08ca): `u16 length`, `i16 count`, `i16 tab`, then `u32 item, u32 price` a
/// thing. (The client's reader takes eight bytes a thing: the picture and the place worn that rAthena's
/// `ENABLE_CASHSHOP_PREVIEW_PATCH` adds are for a client patched to read them.)
pub const ZC_ACK_SCHEDULER_CASHITEM: u16 = 0x08ca;
/// ZC_SE_PC_BUY_CASHITEM_RESULT (0x0849, 16 bytes): `u32 item`, `i16 result`, `u32 cash points`,
/// `u32 free points`.
pub const ZC_SE_PC_BUY_CASHITEM_RESULT: u16 = 0x0849;

/// The cash points, the free points and the tab of a ZC_SE_CASHSHOP_OPEN.
pub fn opened(packet: &[u8]) -> Option<(u32, u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?, long(packet, 10)?))
}

/// The tab and the things of a ZC_ACK_SCHEDULER_CASHITEM: as many as it counts, or as it has room for.
pub fn listed(packet: &[u8]) -> Option<(u16, Vec<Good>)> {
    let (count, tab) = (word(packet, 4)?, word(packet, 6)?);
    let goods = records(packet, 8, 8).take(usize::from(count)).map(|thing| Good { item: u32_at(thing, 0), price: u32_at(thing, 4), tab }).collect();
    Some((tab, goods))
}

/// What the server answers to a purchase.
#[derive(Debug, PartialEq)]
pub struct Bought {
    /// The item it could not sell, if it says which.
    pub item: u32,
    /// 0 when everything was bought (`crate::cashshop::answer`).
    pub result: i16,
    pub cash: u32,
    pub points: u32,
}

/// A ZC_SE_PC_BUY_CASHITEM_RESULT.
pub fn bought(packet: &[u8]) -> Option<Bought> {
    Some(Bought { item: long(packet, 2)?, result: short(packet, 6)?, cash: long(packet, 8)?, points: long(packet, 12)? })
}

/// CZ_SE_CASHSHOP_OPEN (0x0b6d, 6 bytes): `u32 tab`. The menu's icon and the key ask for the window with tab
/// 0; the answer opens it, or closes it if it was open.
pub fn open(tab: u32) -> Vec<u8> {
    Writer::new(0x0b6d).u32(tab).done()
}

/// CZ_SE_CASHSHOP_CLOSE (0x084a, 2 bytes): the window was closed.
pub fn close() -> Vec<u8> {
    Writer::new(0x084a).done()
}

/// CZ_REQ_SCHEDULER_CASHITEM (0x08c9, 2 bytes): the lists of what the shop offers are asked for, as a map is
/// entered while the client has none.
pub fn ask_lists() -> Vec<u8> {
    Writer::new(0x08c9).done()
}

/// CZ_GET_ACCOUNT_LIMTIED_SALE_LIST (0x0b4c, 2 bytes): the sale for one account is asked for, as the window
/// is made and when "Refresh" is pressed.
pub fn ask_limited() -> Vec<u8> {
    Writer::new(0x0b4c).done()
}

/// CZ_SE_PC_BUY_CASHITEM_LIST (0x0848): `u16 length`, `u16 count`, `u32 free points to use`, then
/// `u32 item, u32 amount, u16 tab` a kind.
pub fn buy(points: u32, kinds: &[(u32, u32, u16)]) -> Vec<u8> {
    let mut packet = Writer::sized(0x0848).u16(kinds.len() as u16).u32(points);
    for (item, amount, tab) in kinds {
        packet = packet.u32(*item).u32(*amount).u16(*tab);
    }
    packet.done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing;

    #[test]
    fn reads_the_points_as_the_window_opens() {
        let packet = testing::packet(ZC_SE_CASHSHOP_OPEN, &[&testing::longs(&[5000, 120, 3])]);
        assert_eq!(crate::net::packet_length(ZC_SE_CASHSHOP_OPEN), Some(packet.len() as i32));
        assert_eq!(opened(&packet), Some((5000, 120, 3)));
        assert_eq!(opened(&packet[..13]), None);
    }

    #[test]
    fn reads_a_list_of_what_is_offered() {
        let body = [&2i16.to_le_bytes()[..], &5i16.to_le_bytes(), &testing::longs(&[501, 10, 12210, 250])].concat();
        let packet = testing::sized(ZC_ACK_SCHEDULER_CASHITEM, &body);
        assert_eq!(crate::net::packet_length(ZC_ACK_SCHEDULER_CASHITEM), Some(-1));
        assert_eq!(listed(&packet), Some((5, vec![Good { item: 501, price: 10, tab: 5 }, Good { item: 12210, price: 250, tab: 5 }])));
        // A list that says more than it has gives what it has.
        assert_eq!(listed(&packet[..packet.len() - 3]).map(|(_, goods)| goods.len()), Some(1));
        assert_eq!(listed(&packet[..7]), None);
    }

    #[test]
    fn reads_the_answer_to_a_purchase() {
        let packet = testing::packet(ZC_SE_PC_BUY_CASHITEM_RESULT, &[&501u32.to_le_bytes(), &8i16.to_le_bytes(), &testing::longs(&[4990, 120])]);
        assert_eq!(crate::net::packet_length(ZC_SE_PC_BUY_CASHITEM_RESULT), Some(packet.len() as i32));
        assert_eq!(bought(&packet), Some(Bought { item: 501, result: 8, cash: 4990, points: 120 }));
        assert_eq!(bought(&packet[..15]), None);
    }

    #[test]
    fn what_is_sent_is_as_long_as_the_server_expects() {
        testing::assert_lengths(&[open(0), close(), ask_lists(), ask_limited()]);
        // A purchase is ten bytes and ten more a kind (rAthena's `PACKET_CZ_SE_PC_BUY_CASHITEM_LIST`).
        let packet = buy(7, &[(501, 3, 1), (12210, 1, 0xFFFF)]);
        assert_eq!(packet.len(), 10 + 2 * 10);
        assert_eq!(&packet[..10], [0x48, 0x08, 30, 0, 2, 0, 7, 0, 0, 0]);
        assert_eq!(&packet[20..], [0xb2, 0x2f, 0, 0, 1, 0, 0, 0, 0xff, 0xff]);
    }
}
