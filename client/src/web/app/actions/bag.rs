//! Double clicks on things, and things dragged out onto the world.

use super::*;

impl App {
    /// Something was double-clicked: a row of a shop goes into the basket or out of it, a stack into the storage
    /// or out of it; anything else is used.
    pub(super) fn used(&mut self, what: Use) {
        match (what, &self.socket) {
            // With the storage open, a double click moves the whole of a stack into it or out of it.
            // A double click in the shop's list puts a thing aside; one on what was put aside takes it back.
            (Use::ShopAdd(row), _) => {
                // How many: what the line under the list says, or one bought and all of a stack sold.
                let typed = field("shop-amount").trim().parse::<u32>().ok().filter(|amount| *amount > 0);
                let most = match &self.deal.shop {
                    Some(Shop::Buy(offers)) if row < offers.len() => 30000,
                    Some(Shop::Sell(bids)) => bids.get(row).and_then(|(index, _)| self.bag.items.iter().find(|item| item.index == *index)).map_or(0, |item| u32::from(item.amount)),
                    _ => 0,
                };
                let buying = matches!(self.deal.shop, Some(Shop::Buy(_)));
                let aside = self.deal.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| u32::from(*amount));
                let amount = typed.unwrap_or(if buying { 1 } else { most }).min(most - aside.min(most)) as u16;
                if amount > 0 {
                    match self.deal.basket.iter_mut().find(|(kept, _)| *kept == row) {
                        Some((_, kept)) => *kept += amount,
                        None => self.deal.basket.push((row, amount)),
                    }
                }
                self.redraw(&[&wnd::TRADE]);
            }
            (Use::ShopRemove(row), _) => {
                self.deal.basket.retain(|(kept, _)| *kept != row);
                self.redraw(&[&wnd::TRADE]);
            }
            (Use::Put(index), Some(socket)) => {
                if let Some(item) = self.bag.items.iter().find(|item| item.index == index && item.worn == 0) {
                    socket.send(&trade::to_storage(item.index, u32::from(item.amount)));
                }
            }
            (Use::Take(index), Some(socket)) => {
                if let Some(item) = self.deal.storage.as_ref().and_then(|storage| storage.items.iter().find(|item| item.index == index)) {
                    socket.send(&trade::from_storage(item.index, u32::from(item.amount)));
                }
            }
            // With no server to tell, nothing goes into the storage or out of it.
            (Use::Put(_) | Use::Take(_), None) => {}
            (what, _) => self.use_it(what),
        }
    }

    /// An item was dragged out of its window and let go over the world: it is dropped.
    pub(super) fn thrown(&mut self, dragged: Drag) {
        // The first of that item carried without being worn. One is dropped at once; of more, the
        // original asks how many.
        let Drag::Item(id) = dragged else { return };
        let Some(item) = self.bag.items.iter().find(|item| item.id == id && item.worn == 0) else { return };
        if item.amount <= 1 {
            self.send(&items::throw(item.index, 1));
        } else {
            self.bag.throwing = Some((item.index, item.amount));
            self.redraw(&[&wnd::THROW]);
        }
    }
}
