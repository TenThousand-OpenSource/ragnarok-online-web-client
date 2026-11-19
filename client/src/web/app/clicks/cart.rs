//! The cart's window and what goes into the cart and out of it as the client moves it
//! (`UIMerchantItemWnd::DragAndDrop` and its `UM_DROP`, `UIItemWnd::DragAndDrop`, `UIItemStoreWnd::DragAndDrop`,
//! the states 6, 7 and 8 of `UIItemDropCntWnd`), the Equipment window's button for it, and the choosing
//! of its look (`UISelectCartWnd`).

use super::*;
use crate::net::cart;
use crate::net::storage;
use crate::shops::{Asked, count_asked};

/// The spirit the client wants one to have who uses Change Cart.
const CHANGE_CART_SP: i64 = 40;

impl App {
    /// A press of this system (`crate::press::cart`).
    pub(super) fn cart_pressed(&mut self, press: crate::press::cart::Press) {
        use crate::press::cart::Press;
        match press {
            // The window opens only for one who pushes a cart (`UIBasicInfoWnd`, at its command 247).
            Press::Window => {
                self.cart.open = !self.cart.open && self.cart.kind > 0;
                self.redraw(&[&wnd::CART]);
            }
            // The answer to the server's offer names what the offer named; the skill's window asks plainly
            // (`MM_SELECT_CART`). The window closes either way.
            Press::Look(kind) => {
                if let Some(looks) = self.cart.looks.take() {
                    self.send_on_map(&if looks.identity != 0 { cart::select(looks.identity, kind) } else { cart::change(kind) });
                }
                self.redraw(&[&wnd::CART_LOOKS]);
            }
            Press::LooksClose => {
                self.cart.looks = None;
                self.redraw(&[&wnd::CART_LOOKS]);
            }
            // The box that asks how many is a form, and its OK sends it.
            Press::CountOk => {}
        }
    }

    /// The cart's key (`HK_CARTWND_ONOFF`, Alt+W in the client's table).
    pub(in crate::web::app) fn cart_key(&mut self) -> bool {
        self.pressed(crate::press::cart::Press::Window);
        true
    }

    /// The skill Change Cart is used (`MM_USE_SKILL` for `MC_CHANGECART`): with too little spirit, or no
    /// cart, the client says so in red and does not use it (`false`); otherwise it opens the window of the
    /// looks the character's level has, and the skill is used all the same.
    pub(in crate::web::app) fn cart_change_used(&mut self) -> bool {
        let has = |name: u16| self.hero.params.get(&name).copied().unwrap_or(0);
        if has(packets::parameter::SP) < CHANGE_CART_SP {
            self.chat_line(Self::words(202, "Insufficient SP"), RED);
            return false;
        }
        if self.cart.kind == 0 {
            self.chat_line(Self::words(660, "You must have a cart."), RED);
            return false;
        }
        self.cart_looks_offered(0, crate::cart::by_level(has(packets::parameter::BASE_LEVEL).max(0) as u32));
        true
    }

    /// The window of the looks opens with those kinds; the first picture of each kind's sprite is read for it.
    pub(in crate::web::app) fn cart_looks_offered(&mut self, identity: u32, kinds: Vec<u8>) {
        for (place, kind) in kinds.iter().copied().enumerate() {
            let Some((sprite, _)) = crate::cart::offered(kind, place) else { continue };
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok((picture, _, origin)) = figure::frame_picture(&sprite, 0, 0).await {
                    dispatch(Action::CartLookRead(kind, picture, origin));
                }
            });
        }
        self.cart.looks = Some(Looks { identity, kinds, ..Looks::default() });
        self.redraw(&[&wnd::CART_LOOKS]);
    }

    /// The picture of a look was read.
    pub(in crate::web::app) fn cart_look_read(&mut self, kind: u8, picture: String, origin: (f32, f32)) {
        if let Some(looks) = &mut self.cart.looks {
            looks.pictures.insert(kind, (picture, origin));
            self.redraw(&[&wnd::CART_LOOKS]);
        }
    }

    /// Something was dragged out of its window and let go over the cart's: a thing carried goes in, or one of
    /// the storage, one thing at once, and of more the client asks how many.
    pub(in crate::web::app) fn cart_let_go(&mut self, dragged: Drag) {
        match dragged {
            // The first of that item carried without being worn, as where one is dropped on the ground.
            Drag::Item(id) => {
                if let Some((index, amount)) = self.bag.first_unworn(id).map(|item| (item.index, item.amount)) {
                    self.cart_move(Carting::In(index), amount);
                }
            }
            Drag::Stored(index) => {
                if let Some(amount) = self.storage.open
                    .as_ref()
                    .and_then(|stored| stored.items.iter().find(|item| item.index == index))
                    .map(|item| item.amount)
                {
                    self.cart_move(Carting::FromStorage(index), amount);
                }
            }
            _ => {}
        }
    }

    /// A thing of the cart was dragged out and let go over the inventory (`out`), or over the storage: it
    /// goes there, one thing at once, and of more the client asks how many.
    pub(in crate::web::app) fn cart_let_out(&mut self, index: u16, out: bool) {
        let Some(amount) = self.cart.items.iter().find(|item| item.index == index).map(|item| item.amount) else { return };
        self.cart_move(if out { Carting::Out(index) } else { Carting::ToStorage(index) }, amount);
    }

    /// Nothing leaves the cart or comes into it without one, nor while a shop is being made out of it (the
    /// client's cart lets nothing be taken up then).
    fn cart_move(&mut self, carting: Carting, amount: u16) {
        if self.cart.kind == 0 || amount == 0 || self.vending.making.is_some() {
            return;
        }
        if amount == 1 {
            return self.cart_send(carting, 1);
        }
        self.cart.counting = Some((carting, amount));
        self.redraw(&[&wnd::CART_COUNT]);
    }

    fn cart_send(&mut self, carting: Carting, amount: u32) {
        self.send_on_map(&match carting {
            Carting::In(index) => cart::put(index, amount),
            Carting::Out(index) => cart::take(index, amount),
            Carting::FromStorage(index) => storage::storage_to_cart(index, amount),
            Carting::ToStorage(index) => storage::cart_to_storage(index, amount),
        });
    }

    /// The number of the box was given: going into the cart or the storage, a number under one counts as one
    /// (the box's states 8 and 7); coming out into the inventory, it counts for nothing (state 6); and never
    /// more than there are.
    pub(in crate::web::app) fn cart_counted(&mut self) {
        let Some((carting, most)) = self.cart.counting.take() else { return };
        let asked = if matches!(carting, Carting::Out(_)) { Asked::Moving } else { Asked::Storing };
        if let Some(amount) = count_asked(&field(&wnd::CART_COUNT.field()), u32::from(most), asked) {
            self.cart_send(carting, amount);
        }
        self.redraw(&[&wnd::CART_COUNT]);
    }
}
