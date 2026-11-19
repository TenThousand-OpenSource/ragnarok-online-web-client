//! The storage's window: its tabs, its order, its search, and what goes into it and out of it as the client
//! moves it (`UIItemStoreWnd::DragAndDrop`, `UIItemWnd::DragAndDrop`, the states 6 and 7 of
//! `UIItemDropCntWnd`).
//!
//! For another window that moves things into the storage or out of it (the cart): the packets are
//! `net::storage::{to_storage, from_storage, cart_to_storage, storage_to_cart}`; a thing of the storage is
//! dragged as `Drag::Stored(its place there)`, and what is let go over the storage's windows arrives in
//! `storage_let_go` (`Drop::Storage`), where a new kind of thing dragged gets its arm.

use super::*;
use crate::net::storage;
use crate::storage::{Sorted, TABS};
use crate::shops::{Asked, count_asked};

impl App {
    /// A press of this system (`crate::press::storage`).
    pub(super) fn storage_pressed(&mut self, press: crate::press::storage::Press) {
        use crate::press::storage::Press;
        match press {
            Press::Tab(tab) => {
                self.storage.tab = tab.min(TABS - 1);
                self.redraw(&[&wnd::STORAGE]);
            }
            Press::Apart(tab) => {
                self.storage.apart ^= 1 << tab.min(TABS - 1);
                self.redraw(&wnd::STORAGE_WINDOWS);
            }
            Press::Sorting => {
                self.storage.sorting = !self.storage.sorting;
                self.redraw(&[&wnd::STORAGE]);
            }
            Press::Sort(choice) => {
                self.storage.sorted = Sorted::ALL[choice.min(2)];
                self.storage.sorting = false;
                self.redraw(&wnd::STORAGE_WINDOWS);
            }
            Press::FoundClose => {
                self.storage.found = None;
                self.redraw(&[&wnd::STORAGE_FOUND]);
            }
            // CZ_CLOSE_STORE: the window goes when the server says the storage is closed.
            Press::Close => self.send(&storage::close_storage()),
            // The search and the box that asks how many are forms, and their buttons send them.
            Press::Search | Press::CountOk => {}
        }
    }

    /// The search was sent (`UIItemStoreWnd::SendMsg`, 365): with words in its field, the window of what was
    /// found opens for them, and the field is emptied.
    pub(in crate::web::app) fn storage_searched(&mut self) {
        let sought = field(&wnd::STORAGE.field());
        if !sought.is_empty() {
            set_field(&wnd::STORAGE.field(), "");
            self.storage.found = Some(sought);
            self.redraw(&[&wnd::STORAGE_FOUND]);
        }
    }

    /// Something was dragged out of its window and let go over the storage: a thing carried goes in, one
    /// thing at once, and of more the client asks how many.
    pub(in crate::web::app) fn storage_let_go(&mut self, dragged: Drag) {
        // A thing of the cart goes in out of the cart (`clicks/cart.rs`).
        if let Drag::Carted(index) = dragged {
            return self.cart_let_out(index, false);
        }
        // The first of that item carried without being worn, as where one is dropped on the ground.
        let Drag::Item(id) = dragged else { return };
        let Some((index, amount)) = self.bag.first_unworn(id).map(|item| (item.index, item.amount)) else { return };
        self.storage_move(Stowing::In(index), amount);
    }

    /// A thing of the storage was dragged out and let go over the inventory: it comes out, one thing at once,
    /// and of more the client asks how many.
    pub(in crate::web::app) fn storage_let_out(&mut self, index: u16) {
        let Some(amount) = self.storage.open.as_ref().and_then(|stored| stored.items.iter().find(|item| item.index == index)).map(|item| item.amount) else { return };
        self.storage_move(Stowing::Out(index), amount);
    }

    /// A double click on a thing carried while the storage is open, or on one in the storage: it goes across
    /// as if it were dragged. This is this client's own: a double click in the client's inventory uses the
    /// thing, and its storage has none (it moves a whole stack with the right button while Alt is held:
    /// `storage_stowed`).
    /// (One in the storage: `storage_let_out`.)
    pub(in crate::web::app) fn storage_put_used(&mut self, index: u16) {
        if let Some(amount) = self.bag.items.iter().find(|item| item.index == index && item.worn == 0).map(|item| item.amount) {
            self.storage_move(Stowing::In(index), amount);
        }
    }

    /// The right button with Alt held on a thing carried, or on one in the storage: the whole stack goes
    /// across without the question (`UIItemWnd::OnRBtnDown`, `UIItemStoreWnd::OnRBtnDown`).
    pub(in crate::web::app) fn storage_stowed(&mut self, what: Use) {
        let whole = match what {
            Use::Put(index) => self.bag.items.iter().find(|item| item.index == index && item.worn == 0).map(|item| (Stowing::In(index), item.amount)),
            Use::Take(index) => self.storage.open.as_ref().and_then(|stored| stored.items.iter().find(|item| item.index == index)).map(|item| (Stowing::Out(index), item.amount)),
            _ => None,
        };
        if let (Some((stowing, amount)), true) = (whole, self.storage.open.is_some()) {
            self.storage_send(stowing, u32::from(amount));
        }
    }

    fn storage_move(&mut self, stowing: Stowing, amount: u16) {
        if self.storage.open.is_none() || amount == 0 {
            return;
        }
        if amount == 1 {
            return self.storage_send(stowing, 1);
        }
        self.storage.counting = Some((stowing, amount));
        self.redraw(&[&wnd::STORAGE_COUNT]);
    }

    fn storage_send(&mut self, stowing: Stowing, amount: u32) {
        match stowing {
            Stowing::In(index) => self.send(&storage::to_storage(index, amount)),
            Stowing::Out(index) => self.send(&storage::from_storage(index, amount)),
        }
    }

    /// The number of the box was given: going in, a number under one counts as one (the box's state 7); coming
    /// out, it counts for nothing (state 6); and never more than there are.
    pub(in crate::web::app) fn storage_counted(&mut self) {
        let Some((stowing, most)) = self.storage.counting.take() else { return };
        let asked = if matches!(stowing, Stowing::In(_)) { Asked::Storing } else { Asked::Moving };
        if let Some(amount) = count_asked(&field(&wnd::STORAGE_COUNT.field()), u32::from(most), asked) {
            self.storage_send(stowing, amount);
        }
        self.redraw(&[&wnd::STORAGE_COUNT]);
    }
}
