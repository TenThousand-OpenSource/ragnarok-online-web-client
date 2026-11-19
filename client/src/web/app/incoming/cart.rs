//! The merchant's cart: what the map server tells of it. What the client does with each packet is in
//! `GameModePackets.cpp` (0x0121, 0x0125, 0x012b, 0x012c, 0x0b45), `ReceiveInventoryStart` and the lists
//! after it, `ApplyStatusChange` for the state that says there is a cart, and `GameModeHandlers.cpp` for
//! ZC_SELECTCART.

use super::*;
use crate::net::cart;
use crate::net::statuses;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. Packets of other systems are
    /// overheard here and left to their own readers: entering the game, where the cart of the character
    /// before is forgotten; the inventory's lists, of which the cart's is one kind; and a state going on or
    /// off, one of which says who pushes which cart.
    pub(super) fn cart_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_ACCEPT_ENTER => {
                self.cart = Cart::default();
                world::carted(None, None);
                return false;
            }
            items::ZC_INVENTORY_START => {
                if items::list_kind(packet) == Some(cart::LIST) {
                    self.cart.items.clear();
                }
                return false;
            }
            items::ZC_INVENTORY_ITEMLIST_NORMAL | items::ZC_INVENTORY_ITEMLIST_EQUIP => {
                let listed = if id == items::ZC_INVENTORY_ITEMLIST_NORMAL { items::stackable_of(packet, cart::LIST) } else { items::equipment_of(packet, cart::LIST) };
                if !listed.is_empty() {
                    self.cart.items.extend(listed);
                    self.cart_changed();
                }
                return false;
            }
            statuses::ZC_MSG_STATE_CHANGE | statuses::ZC_MSG_STATE_CHANGE2 | statuses::ZC_MSG_STATE_CHANGE3 | statuses::ZC_EFST_SET_ENTER2 => {
                if let Some(change) = statuses::changed(packet).filter(|change| change.state == cart::PUSHES_CART) {
                    self.cart_pushed(&change);
                }
                return false;
            }
            cart::ZC_NOTIFY_CARTITEM_COUNTINFO => {
                if let Some(fill) = cart::fill(packet) {
                    self.cart.fill = fill;
                    self.redraw(&[&wnd::CART]);
                }
            }
            cart::ZC_ADD_ITEM_TO_CART => {
                if let Some(added) = cart::added(packet) {
                    match self.cart.items.iter_mut().find(|item| item.index == added.index) {
                        Some(item) => item.amount = item.amount.saturating_add(added.amount),
                        None => self.cart.items.push(added),
                    }
                    self.cart_changed();
                }
            }
            cart::ZC_DELETE_ITEM_FROM_CART => {
                if let Some((index, amount)) = cart::removed(packet) {
                    if let Some(item) = self.cart.items.iter_mut().find(|item| item.index == index) {
                        item.amount = item.amount.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
                    }
                    self.cart.items.retain(|item| item.amount > 0);
                    self.cart_changed();
                }
            }
            // The client says why a thing did not go in, in red.
            cart::ZC_ACK_ADDITEM_TO_CART => {
                if let Some((line, otherwise)) = cart::refused(packet) {
                    self.chat_line(Self::words(line, otherwise), RED);
                }
            }
            // The cart is gone: what was in it is forgotten and its window closed.
            cart::ZC_CARTOFF => {
                self.cart.items.clear();
                self.cart.open = false;
                self.cart.counting = None;
                self.redraw(&wnd::CART_WINDOWS);
            }
            cart::ZC_SELECTCART => {
                if let Some((identity, kinds)) = cart::looks(packet).filter(|(_, kinds)| !kinds.is_empty()) {
                    self.cart_looks_offered(identity, kinds);
                }
            }
            _ => return false,
        }
        true
    }

    /// The state that says someone pushes a cart went on or off (`CGameActor::SendMsg`, `AM_STATUS_CHANGE`
    /// with `EFST_ON_PUSH_CART`): its first value is the cart's kind. The world draws the cart; of the
    /// player's own the app keeps the kind, by which its window opens and the Equipment window has its
    /// buttons.
    fn cart_pushed(&mut self, change: &statuses::Change) {
        let own = change.who == self.own_account();
        let kind = change.on.then_some(change.values[0].clamp(0, 255) as u8);
        world::carted((!own).then_some(change.who), kind);
        if own {
            self.cart.kind = kind.unwrap_or(0);
            if self.cart.kind == 0 {
                self.cart.open = false;
                self.cart.looks = None;
            }
            self.redraw(&[&wnd::CART, &wnd::CART_LOOKS, &wnd::EQUIPMENT]);
        }
    }

    /// What is in the cart changed: the names of its things are asked for, and the windows that show them are
    /// drawn anew.
    pub(in crate::web::app) fn cart_changed(&mut self) {
        data::want_items(self.cart.items.iter().flat_map(|item| std::iter::once(item.id).chain(crate::item_names::Named::from(item).cards_set())), || dispatch(Action::ItemsKnown));
        self.stall_kept();
        self.redraw(&wnd::CART_WINDOWS);
    }
}
