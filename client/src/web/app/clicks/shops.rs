//! An NPC's shop, and the Kafra's storage.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn trade_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "shop-pick" => {
                self.deal.picked = arg.parse().ok();
                self.refresh_trade();
            }
            "shop-buy" | "shop-sell" | "shop-cancel" | "shop-ok" => {
                let (Some(shop), Some(socket)) = (self.deal.shop.take(), &self.socket) else { return true };
                let basket = std::mem::take(&mut self.deal.basket);
                self.deal.picked = None;
                match (what, shop) {
                    ("shop-buy", Shop::Choose(npc)) => socket.send(&trade::deal(npc, false)),
                    ("shop-sell", Shop::Choose(npc)) => socket.send(&trade::deal(npc, true)),
                    (_, Shop::Choose(_)) => {}
                    ("shop-ok", Shop::Buy(offers)) => {
                        let wanted: Vec<(u32, u16)> = basket.iter().filter_map(|(row, amount)| Some((offers.get(*row)?.id, *amount))).collect();
                        socket.send(&trade::buy(&wanted));
                    }
                    ("shop-ok", Shop::Sell(bids)) => {
                        let sold: Vec<(u16, u16)> = basket.iter().filter_map(|(row, amount)| Some((bids.get(*row)?.0, *amount))).collect();
                        socket.send(&trade::sell(&sold));
                    }
                    // Leaving a list tells the server so with an empty one.
                    (_, Shop::Buy(_)) => socket.send(&trade::buy(&[])),
                    (_, Shop::Sell(_)) => socket.send(&trade::sell(&[])),
                }
                self.refresh_trade();
            }
            "storage-tab" => {
                self.deal.storage_tab = arg.parse().unwrap_or(0);
                self.refresh_items();
            }
            "storage-close" => {
                if let Some(socket) = &self.socket {
                    socket.send(&trade::close_storage());
                }
            }
            _ => return false,
        }
        true
    }
}
