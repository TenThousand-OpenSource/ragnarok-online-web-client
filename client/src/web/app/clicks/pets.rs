//! Taming a monster, hatching an egg, and the pet window.

use super::*;
use crate::net::pets;

impl App {
    /// A press of this system (`crate::press::pets`).
    pub(super) fn pets_pressed(&mut self, press: crate::press::pets::Press) {
        use crate::press::pets::Press;
        match press {
            // A press on the slot machine starts its reels and asks the server for the catch.
            Press::TamingShoot => {
                if let (Some(taming), Some(socket)) = (&mut self.pets.taming, &self.socket) {
                    if !taming.started {
                        (taming.state, taming.wanted, taming.started, taming.tick) = (1, 7, true, 0);
                        socket.send(&pets::capture(taming.target));
                    }
                }
            }
            Press::EggPick(row) => {
                self.pets.egg_picked = row;
                self.render();
            }
            // The picked egg is hatched.
            Press::EggOk | Press::EggCancel => {
                if let (true, Some(socket), Some(index)) = (press == Press::EggOk, &self.socket, self.pets.eggs.as_ref().and_then(|eggs| eggs.get(self.pets.egg_picked))) {
                    socket.send(&pets::hatch(*index));
                }
                self.pets.eggs = None;
                self.render();
            }
            Press::Window => {
                (self.pets.open, self.pets.menu, self.pets.asks) = (!self.pets.open && self.pets.pet.is_some(), false, 0);
                self.render();
            }
            Press::Menu => {
                self.pets.menu = !self.pets.menu;
                self.redraw(&[&wnd::PET, &wnd::PET_ASKS]);
            }
            Press::Do(order) => self.pet_told(order),
            // A new name: 23 letters at most, and asked about before it is sent.
            Press::Rename => {
                let name = field("pet-name").trim().to_owned();
                if name.len() >= 24 {
                    // The client says so in a message box (`MSI_PETNAME_IN23`).
                    self.message = Some(crate::i18n::tr("Your pet name must be 23 characters or less.").to_owned());
                    self.render();
                } else if !name.is_empty() {
                    (self.pets.asks, self.pets.named) = (2, name);
                    self.redraw(&[&wnd::PET, &wnd::PET_ASKS]);
                }
            }
            Press::Yes | Press::No => {
                if let (true, Some(socket)) = (press == Press::Yes, &self.socket) {
                    if self.pets.asks == 1 {
                        socket.send(&pets::command(pets::order::FEED));
                    } else if let Some(pet) = &mut self.pets.pet {
                        socket.send(&pets::rename(&self.pets.named));
                        pet.name = self.pets.named.clone();
                    }
                }
                self.pets.asks = 0;
                self.redraw(&[&wnd::PET, &wnd::PET_ASKS]);
            }
        }
    }

    /// The menu of the pet window: feeding is asked about first, the rest is sent as it is.
    fn pet_told(&mut self, told: crate::press::pets::Order) {
        use crate::press::pets::Order;
        self.pets.menu = false;
        // From the menu on the pet itself: the menu goes, and its first row opens the pet window.
        if self.pets.menu_at.take().is_some() {
            // Feeding is asked about first, in the box that belongs to the pet window.
            if matches!(told, Order::Info | Order::Feed) {
                self.pets.open = true;
            }
            self.render();
            if told == Order::Info {
                return;
            }
        }
        let order = match told {
            Order::Feed => {
                self.pets.asks = 1;
                None
            }
            Order::Perform => Some(pets::order::PERFORM),
            Order::Unequip => Some(pets::order::UNEQUIP),
            // The pet window's own menu has no row for the window itself; were one pressed there, it would go
            // the way of the last row, as any other word did.
            Order::Info | Order::Egg => Some(pets::order::TO_EGG),
        };
        if let (Some(order), Some(socket)) = (order, &self.socket) {
            socket.send(&pets::command(order));
        }
        self.redraw(&[&wnd::PET, &wnd::PET_ASKS]);
    }

    /// Three seconds after its trick the pet, one time in four, shows how it feels standing about (the
    /// client's `CGameActor::ProcessState`).
    pub(in crate::web::app) fn pet_idles(&mut self) {
        if js_sys::Math::random() < 0.25 {
            self.pet_feels(crate::pets::moment::IDLE, None);
        }
    }
}
