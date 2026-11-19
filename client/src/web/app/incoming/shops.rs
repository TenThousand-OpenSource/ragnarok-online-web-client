//! An NPC's shop: its question, its lists, and what it answers to a deal.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn shops_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            net::shops::ZC_SELECT_DEALTYPE => {
                if let Some(npc) = net::talk::npc(id, packet) {
                    self.shop_opened(Shop::Choose(npc));
                }
            }
            net::shops::ZC_PC_PURCHASE_ITEMLIST => {
                let offers = net::shops::offers(packet);
                data::want_items(offers.iter().map(|offer| offer.id), || dispatch(Action::ItemsKnown));
                self.shop_opened(Shop::Buy(offers));
            }
            net::shops::ZC_PC_SELL_ITEMLIST => self.shop_opened(Shop::Sell(net::shops::bids(packet))),
            net::shops::ZC_PC_PURCHASE_RESULT => self.shop_answer(net::shops::purchase_line(net::shops::result(packet))),
            net::shops::ZC_PC_SELL_RESULT => self.shop_answer(net::shops::sell_line(net::shops::result(packet))),
            _ => return false,
        }
        true
    }

    /// The shop asks, or lists what it sells or takes: the basket is empty again.
    fn shop_opened(&mut self, shop: Shop) {
        self.shops.basket.clear();
        self.shops.counting = None;
        self.shops.shop = Some(shop);
        self.redraw(&[&wnd::TRADE, &wnd::SHOP, &wnd::SHOP_BASKET, &wnd::SHOP_COUNT]);
    }

    /// What the shop answers to a deal (`CGameMode`'s handlers of ZC_PC_PURCHASE_RESULT and
    /// ZC_PC_SELL_RESULT): the client's line for it in the chat, in the colour it has there (0xFFFF00 for a
    /// deal done, which is blue and green; red for one that failed), and the shop's windows closed.
    fn shop_answer(&mut self, line: Option<(usize, bool)>) {
        if let Some((line, failed)) = line {
            let otherwise = match line {
                54 => "The deal has successfully completed.",
                55 => "You do not have enough zeny.",
                56 => "You are over your Weight Limit.",
                57 => "The deal has failed.",
                221 => "Out of the maximum capacity",
                _ => "Amounts are exceeded the possession of the item is not available for purchase.",
            };
            self.chat_line(Self::words(line, otherwise), if failed { RED } else { CYAN });
        }
        if matches!(self.shops.shop, Some(Shop::Buy(_) | Shop::Sell(_))) {
            self.shops.shop = None;
            self.shops.basket.clear();
            self.shops.counting = None;
            self.redraw(&[&wnd::SHOP, &wnd::SHOP_BASKET, &wnd::SHOP_COUNT]);
        }
    }
}
