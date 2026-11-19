//! Double clicks on things, and things dragged out onto the world.

use super::*;

impl App {
    /// Something was double-clicked: a row of a shop goes into the basket or out of it, a stack into the storage
    /// or out of it; anything else is used.
    pub(super) fn used(&mut self, what: Use) {
        match what {
            Use::ShopAdd(_) | Use::ShopRemove(_) => self.shop_used(what),
            Use::Put(_) | Use::Take(_) => self.storage_used(what),
            // A double click on a friend opens the talk with that friend (`clicks/friends.rs`).
            Use::Friend(row) => self.friend_talk(row),
            // A double click on a vendor's sign asks what the shop sells (`clicks/vending.rs`).
            Use::Sign(_) => self.vending_used(what),
            what => self.use_it(what),
        }
    }

    /// Something was dragged out of its window and let go over a window that takes things: a shop's two, or
    /// the storage's.
    pub(super) fn let_go(&mut self, dragged: Drag, onto: Drop) {
        match onto {
            Drop::Shop | Drop::Basket => self.shop_let_go(dragged, onto),
            Drop::Storage => self.storage_let_go(dragged),
            Drop::Cart => self.cart_let_go(dragged),
            Drop::Stall | Drop::Stock | Drop::Vendor | Drop::Wants => self.vending_let_go(dragged, onto),
            Drop::InventoryTab(tab) => self.tab_let_go(dragged, tab),
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
