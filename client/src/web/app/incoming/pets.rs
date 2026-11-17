//! Taming a monster, hatching its egg, and what becomes of the pet.

use super::*;
use crate::net::pets;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn pet_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // ZC_PETEGG_LIST (0x01a6): `u16 length`, then the place in the inventory of each egg. An incubator was
            // used: the eggs are listed to pick the one to hatch (the client's `UIPetEggListWnd`).
            0x01a6 => {
                self.pets.eggs = Some(packet.get(4..).unwrap_or_default().chunks_exact(2).map(|index| u16::from_le_bytes([index[0], index[1]])).collect());
                self.pets.egg_picked = 0;
                self.render();
            }
            // ZC_START_CAPTURE (0x019e): a taming item was used. The monster to tame is picked with the pointer, as
            // a skill's target is (the client's `Zc_Start_Capture`: reach 5).
            0x019e => {
                self.skills.aimed = Some((TAME, 0));
                world::aim(Some(false), 5);
                // Taming and fighting do not go together: the fight stops as soon as the bait is in hand.
                world::stand_down();
                if let Some(socket) = &self.socket {
                    socket.send(&0x0118u16.to_le_bytes());
                }
            }
            // ZC_TRYCAPTURE_MONSTER (0x01a0): `u8 caught`. The reels slow down and stop on the picture of the
            // catch (the eighth) or of the miss (the fourth).
            0x01a0 => {
                if let (Some(taming), Some(caught)) = (&mut self.pets.taming, packet.get(2)) {
                    (taming.wanted, taming.delay, taming.longest) = (if *caught != 0 { 7 } else { 3 }, 7, 7);
                }
            }
            // ZC_PROPERTY_PET: everything about the player's pet. (A level of 10000 and more tells of a pet still
            // in an egg that is being looked at; the egg's description is not shown here yet.)
            pets::ZC_PROPERTY_PET => {
                if let Some(pet) = pets::property(packet).filter(|pet| pet.level < 10000) {
                    self.pets.pet = Some(pet);
                    self.refresh_pet();
                }
            }
            // ZC_CHANGESTATE_PET: one thing about a pet changed (the client's `ReceivePetChangeState`).
            pets::ZC_CHANGESTATE_PET => {
                let Some((what, unit, value)) = pets::changed(packet) else { return true };
                match (what, &mut self.pets.pet) {
                    (pets::change::OWN, _) => self.pets.unit = unit,
                    (pets::change::INTIMACY, Some(pet)) => pet.intimacy = value,
                    (pets::change::HUNGER, Some(pet)) => pet.hunger = value,
                    (pets::change::ACCESSORY, Some(pet)) if unit == self.pets.unit => pet.accessory = value as u16,
                    // Back in its egg: there is no pet any more, and its window closes.
                    (pets::change::EGG, _) if value == 0 => {
                        (self.pets.pet, self.pets.unit, self.pets.open, self.pets.menu, self.pets.asks) = (None, 0, false, false, 0);
                        self.render();
                    }
                    _ => {}
                }
                self.refresh_pet();
            }
            // ZC_FEED_PET: the pet ate, and so much of its food is left; or there is none of its food (the
            // client's `ReceiveFeedResult`, with its colours).
            pets::ZC_FEED_PET => {
                self.pets.fed = pets::fed(packet);
                if let Some((_, food)) = self.pets.fed {
                    data::want_items(std::iter::once(food), || dispatch(Action::ItemsKnown));
                }
                self.tell_fed();
            }
            _ => return false,
        }
        true
    }

    /// Tells in the chat how feeding the pet went, once the food's name is known.
    pub(in crate::web::app) fn tell_fed(&mut self) {
        let Some((fed, food)) = self.pets.fed else { return };
        let Some((name, _)) = data::item(food) else { return };
        self.pets.fed = None;
        if fed {
            let left: u32 = self.bag.items.iter().filter(|item| item.id == food).map(|item| u32::from(item.amount)).sum();
            self.chat_line(crate::i18n::tr("Your pet has been fed. {} ({}) remaining").replacen("{}", &name, 1).replacen("{}", &left.to_string(), 1), "#64ffff");
        } else {
            self.chat_line(crate::i18n::tr("Pet food [{}] is not available.").replace("{}", &name), "#f51e1e");
        }
    }
}
