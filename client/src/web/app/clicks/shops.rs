//! An NPC's shop, and the Kafra's storage.

use super::*;

impl App {
    /// A press of this system (`crate::press::shops`).
    pub(super) fn trade_pressed(&mut self, press: crate::press::shops::Press) {
        use crate::press::shops::Press;
        match press {
            Press::Pick(row) => {
                self.deal.picked = Some(row);
                self.redraw(&[&wnd::TRADE]);
            }
            Press::Buy | Press::Sell | Press::Cancel | Press::Ok => {
                let (Some(shop), Some(socket)) = (self.deal.shop.take(), &self.socket) else { return };
                let basket = std::mem::take(&mut self.deal.basket);
                self.deal.picked = None;
                match (press, shop) {
                    (Press::Buy, Shop::Choose(npc)) => socket.send(&trade::deal(npc, false)),
                    (Press::Sell, Shop::Choose(npc)) => socket.send(&trade::deal(npc, true)),
                    (_, Shop::Choose(_)) => {}
                    (Press::Ok, Shop::Buy(offers)) => {
                        let wanted: Vec<(u32, u16)> = basket.iter().filter_map(|(row, amount)| Some((offers.get(*row)?.id, *amount))).collect();
                        socket.send(&trade::buy(&wanted));
                    }
                    (Press::Ok, Shop::Sell(bids)) => {
                        let sold: Vec<(u16, u16)> = basket.iter().filter_map(|(row, amount)| Some((bids.get(*row)?.0, *amount))).collect();
                        socket.send(&trade::sell(&sold));
                    }
                    // Leaving a list tells the server so with an empty one.
                    (_, Shop::Buy(_)) => socket.send(&trade::buy(&[])),
                    (_, Shop::Sell(_)) => socket.send(&trade::sell(&[])),
                }
                self.redraw(&[&wnd::TRADE]);
            }
            Press::StorageTab(tab) => {
                self.deal.storage_tab = tab;
                self.refresh_items();
            }
            Press::StorageClose => {
                self.send(&trade::close_storage());
            }
        }
    }
}
