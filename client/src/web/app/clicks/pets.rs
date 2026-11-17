//! Taming a monster, hatching an egg, and the pet window.

use super::*;
use crate::net::pets;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn pet_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            // A press on the slot machine starts its reels and asks the server for the catch.
            "taming-shoot" => {
                if let (Some(taming), Some(socket)) = (&mut self.pets.taming, &self.socket) {
                    if !taming.started {
                        (taming.state, taming.wanted, taming.started, taming.tick) = (1, 7, true, 0);
                        socket.send(&[&0x019fu16.to_le_bytes()[..], &taming.target.to_le_bytes()].concat());
                    }
                }
            }
            "egg-pick" => {
                self.pets.egg_picked = arg.parse().unwrap_or(0);
                self.render();
            }
            // CZ_SELECT_PETEGG (0x01a7): `u16 index`. The picked egg is hatched.
            "egg-ok" | "egg-cancel" => {
                if let (true, Some(socket), Some(index)) = (what == "egg-ok", &self.socket, self.pets.eggs.as_ref().and_then(|eggs| eggs.get(self.pets.egg_picked))) {
                    socket.send(&[0x01a7u16.to_le_bytes(), index.to_le_bytes()].concat());
                }
                self.pets.eggs = None;
                self.render();
            }
            "pet" => {
                (self.pets.open, self.pets.menu, self.pets.asks) = (!self.pets.open && self.pets.pet.is_some(), false, 0);
                self.render();
            }
            "pet-menu" => {
                self.pets.menu = !self.pets.menu;
                self.refresh_pet();
            }
            // The menu of the pet window: feeding is asked about first, the rest is sent as it is.
            "pet-do" => {
                self.pets.menu = false;
                let order = match arg {
                    "feed" => {
                        self.pets.asks = 1;
                        None
                    }
                    "perform" => Some(pets::order::PERFORM),
                    "unequip" => Some(pets::order::UNEQUIP),
                    _ => Some(pets::order::TO_EGG),
                };
                if let (Some(order), Some(socket)) = (order, &self.socket) {
                    socket.send(&pets::command(order));
                }
                self.refresh_pet();
            }
            // A new name: 23 letters at most, and asked about before it is sent.
            "pet-rename" => {
                let name = field("pet-name").trim().to_owned();
                if name.len() >= 24 {
                    self.chat_line(crate::i18n::tr("Your pet name must be 23 characters or less.").to_owned(), "#ff8484");
                } else if !name.is_empty() {
                    (self.pets.asks, self.pets.named) = (2, name);
                    self.refresh_pet();
                }
            }
            "pet-yes" | "pet-no" => {
                if let (true, Some(socket)) = (what == "pet-yes", &self.socket) {
                    if self.pets.asks == 1 {
                        socket.send(&pets::command(pets::order::FEED));
                    } else if let Some(pet) = &mut self.pets.pet {
                        socket.send(&pets::rename(&self.pets.named));
                        pet.name = self.pets.named.clone();
                    }
                }
                self.pets.asks = 0;
                self.refresh_pet();
            }
            _ => return false,
        }
        true
    }
}
