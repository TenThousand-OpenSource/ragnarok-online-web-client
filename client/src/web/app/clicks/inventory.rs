//! The windows of what is carried: the inventory, the equipment, an item's description.

use super::*;

impl App {
    /// A press of this system (`crate::press::inventory`).
    pub(super) fn inventory_pressed(&mut self, press: crate::press::inventory::Press) {
        use crate::press::inventory::Press;
        match press {
            Press::Window => {
                self.bag.open = !self.bag.open;
                self.render();
            }
            Press::Tab(tab) => {
                self.bag.tab = [Tab::Usable, Tab::Equipment, Tab::Other, Tab::Favourite][tab.min(3)];
                self.refresh_items();
            }
            // The button does something only on the tab "Fav" (`UIItemWnd::SendMsg`, `ID_SORT_ITEMS`).
            Press::Align => {
                if self.bag.tab == Tab::Favourite {
                    self.bag.aligned = !self.bag.aligned;
                    self.redraw(&[&wnd::INVENTORY]);
                }
            }
            Press::EquipmentPage(page) => {
                self.bag.costumes = page == 1;
                self.redraw(&[&wnd::EQUIPMENT]);
            }
            Press::IdentifyPick(row) => {
                self.bag.identify_picked = Some(row);
                self.render();
            }
            // CZ_REQ_ITEMIDENTIFY (0x0178): `u16 index`. OK asks for the picked item (and does nothing while none
            // is picked); Cancel tells the server -1, so it lets go of the magnifier's use.
            Press::IdentifyOk | Press::IdentifyCancel => {
                let picked = self.bag.identify.as_ref().zip(self.bag.identify_picked).and_then(|(items, picked)| items.get(picked).copied());
                let index = match (&press, picked) {
                    (Press::IdentifyOk, Some(index)) => index,
                    (Press::IdentifyOk, None) => return,
                    _ => 0xffff,
                };
                self.send(&net::inventory::identify(index));
                self.bag.identify = None;
                self.render();
            }
            // The box that asks how many to drop is a form, and its OK sends it (`data-submit="throw"`): the
            // press itself does nothing.
            Press::ThrowOk => {}
            Press::ThrowCancel => {
                self.bag.throwing = None;
                self.redraw(&[&wnd::THROW]);
            }
            // CZ_UNINSTALLATION (0x0b35): `u8 what`. The Equipment window's "off" button.
            // With a cart the client's button names the cart (`UIEquipWnd::UpdateCharacterView`); the server
            // takes a mount or a falcon away first all the same, and a cart when there is neither
            // (rAthena, `clif_parse_RemoveOption`).
            Press::MountOff => self.send_on_map(&net::inventory::uninstall(
                if self.pushes_cart() { net::inventory::CART_OFF } else { crate::mounts::off(self.hero.state).0 }
            )),
            Press::Equipment => {
                self.bag.equipment_open = !self.bag.equipment_open;
                self.render();
            }
            Press::InfoClose => {
                self.bag.info = None;
                self.refresh_items();
            }
            // The illustration of the card whose description is open (`ID_CARDITEM`); pressed again for the
            // card it shows already, it closes, as the client's does.
            Press::InfoCard => {
                let card = self.bag.info.map(|(id, _)| id).filter(|id| data::card_picture(*id).is_some());
                self.bag.illust = if self.bag.illust == card { None } else { card };
                self.redraw(&[&wnd::CARD_ILLUST]);
            }
            Press::IllustClose => {
                self.bag.illust = None;
                self.redraw(&[&wnd::CARD_ILLUST]);
            }
        }
    }

    /// Whether the character pushes a cart: the server tells it as a state (`net::cart::PUSHES_CART`).
    pub(in crate::web) fn pushes_cart(&self) -> bool {
        self.statuses.on.iter().any(|active| active.state == u32::from(crate::net::cart::PUSHES_CART))
    }

    /// A thing of the inventory was dragged onto its tabs (`UIItemWnd::DragAndDrop`): let go on "Fav" it is
    /// kept under that tab, on any other it is taken from under it. The server answers with
    /// ZC_INVENTORY_TAB.
    pub(in crate::web::app) fn tab_let_go(&mut self, dragged: Drag, tab: usize) {
        let Drag::Item(id) = dragged else { return };
        // The thing of that kind that is on the open tab: under "Fav" one kept there, elsewhere one that is not.
        let kept = self.bag.tab == Tab::Favourite;
        let index = self.bag.items
            .iter()
            .find(|item| item.id == id && item.worn == 0 && self.bag.favourites.contains(&item.index) == kept)
            .map(|item| item.index);
        if let Some(index) = index {
            self.send_on_map(&net::inventory::move_to_tab(index, tab == 3));
        }
    }

    /// A thing of the inventory was let go over the Equipment window (`UIEquipWnd::DragAndDrop`): it is asked
    /// to be put on, or readied if it is ammunition (`crate::inventory::worn_by_drop`); anything else, and a
    /// thing taken out of the window itself, only ends the drag.
    pub(in crate::web::app) fn worn_let_go(&mut self, dragged: Drag) {
        let Drag::Item(id) = dragged else { return };
        let Some(item) = self.bag.first_unworn(id).filter(|item| crate::inventory::worn_by_drop(item.kind, item.identified, item.wearable)) else { return };
        self.send_on_map(&net::inventory::wear(item.index, item.wearable));
    }

    /// An item carried is used, by a double click or from the hotkey bar: it is used, taken off or put on.
    pub(in crate::web::app) fn use_item(&mut self, index: u16) {
        let (Some(socket), Some(account)) = (&self.socket, &self.account) else { return };
        let Some(item) = self.bag.at(index) else { return };
        if item.usable() {
            socket.send(&net::inventory::use_item(item.index, account.account_id));
        } else if item.worn != 0 {
            socket.send(&net::inventory::take_off(item.index));
        // A pet's accessory (the server's item kind 8) goes nowhere on the character, and is asked to be
        // worn all the same: the server puts it on the pet. The client tells such a thing by its number
        // (its Lua `IsPetAccessory`, not read here) and sends the same request (`MM_WEAR_EQUIP`).
        } else if item.wearable != 0 || item.kind == 8 {
            socket.send(&net::inventory::wear(item.index, item.wearable));
        }
    }

    /// An item was pressed with the right button: what is known of it shows.
    pub(in crate::web::app) fn item_info(&mut self, id: u32, identified: bool) {
        self.bag.info = Some((id, identified));
        // With the item, the cards set into it where it is carried: its window draws them in its slots.
        let cards: Vec<u32> = self.bag.items.iter().filter(|item| item.id == id).flat_map(|item| crate::item_names::Named::from(item).cards_set()).collect();
        data::want_items(std::iter::once(id).chain(cards), || dispatch(Action::ItemsKnown));
        // A book has its two buttons there, whose pictures are read.
        if data::is_book(id) {
            self.book_pictures_wanted();
        }
        self.refresh_items();
    }

    /// An item was dragged out of its window and let go over the world: it is dropped.
    pub(in crate::web::app) fn thrown(&mut self, dragged: Drag) {
        // The first of that item carried without being worn. One is dropped at once; of more, the
        // original asks how many.
        let Drag::Item(id) = dragged else { return };
        let Some(item) = self.bag.first_unworn(id) else { return };
        if item.amount <= 1 {
            self.send(&net::inventory::throw(item.index, 1));
        } else {
            self.bag.throwing = Some((item.index, item.amount));
            self.redraw(&[&wnd::THROW]);
        }
    }

    /// The box that asks how many to drop was sent: that many are dropped.
    pub(in crate::web::app) fn throw_counted(&mut self) {
        if let (Some((index, most)), Some(socket)) = (self.bag.throwing.take(), &self.socket) {
            if let Some(count) = Self::counted(&wnd::THROW, most) {
                socket.send(&net::inventory::throw(index, count));
            }
        }
        self.redraw(&[&wnd::THROW]);
    }
}
