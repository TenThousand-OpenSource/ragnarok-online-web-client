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
                self.send(&0x0118u16.to_le_bytes());
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
                    // A pet close enough to talk will need its lines.
                    if pets::speaks(pet.intimacy) {
                        data::want_pet_talk();
                    }
                    self.pets.hunger_before = self.pets.pet.as_ref().map_or(pet.hunger, |before| before.hunger);
                    self.pets.pet = Some(pet);
                    self.redraw(&[&wnd::PET, &wnd::PET_ASKS]);
                }
            }
            pets::ZC_CHANGESTATE_PET => self.pet_changed(packet),
            // ZC_FEED_PET: the pet ate, and so much of its food is left; or there is none of its food (the
            // client's `ReceiveFeedResult`, with its colours).
            pets::ZC_FEED_PET => {
                self.pets.fed = pets::fed(packet);
                // Fed, the pet shows and says how it likes it, by how hungry it was (`ReceiveFeedResult`).
                if matches!(self.pets.fed, Some((true, _))) {
                    self.pet_feels(pets::moment::FED, Some(self.pets.hunger_before));
                }
                if let Some((_, food)) = self.pets.fed {
                    data::want_items(std::iter::once(food), || dispatch(Action::ItemsKnown));
                }
                self.tell_fed();
            }
            pets::ZC_PET_ACT => self.pet_acted(packet),
            _ => return false,
        }
        true
    }

    /// The player's pet shows what it feels at a moment and says something of it: its owner's client picks both
    /// and tells the server, which shows them to everyone around (the client's `MM_PET_ACT`). `hunger` is the
    /// hunger to go by, when not the pet's own now. Words are for a loyal pet only.
    pub(in crate::web::app) fn pet_feels(&self, moment: i32, hunger: Option<i32>) {
        let (Some(pet), Some(socket)) = (&self.pets.pet, &self.socket) else { return };
        self.pet_says(moment, hunger);
        if let Some(feeling) = pets::feeling(hunger.unwrap_or(pet.hunger), pet.intimacy, moment) {
            socket.send(&pets::act(feeling));
        }
    }

    /// The words alone.
    pub(in crate::web::app) fn pet_says(&self, moment: i32, hunger: Option<i32>) {
        let (Some(pet), Some(socket), Some(account)) = (&self.pets.pet, &self.socket, &self.account) else { return };
        if pets::speaks(pet.intimacy) {
            socket.send(&pets::act(pets::speech(pet.job, moment, hunger.unwrap_or(pet.hunger), account.sex)));
        }
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

    /// ZC_CHANGESTATE_PET: one thing about a pet changed (the client's `ReceivePetChangeState`).
    fn pet_changed(&mut self, packet: &[u8]) {
        let Some((what, unit, value)) = pets::changed(packet) else { return };
        match (what, &mut self.pets.pet) {
            (pets::change::OWN, _) => (self.pets.unit, self.pets.alerted) = (unit, self.pets.alerted && unit == self.pets.unit),
            (pets::change::INTIMACY, Some(pet)) => {
                pet.intimacy = value;
                // A pet grown awkward has its window opened, once, when the player has the client's Pet
                // Information Window option on (`UpdatePetFriendlyWindow`).
                if value <= 100 && !self.pets.alerted && self.settings.set.get("TT_PETINFO_ALERT_ONOFF").copied().unwrap_or_else(|| data::option_starts("TT_PETINFO_ALERT_ONOFF")) {
                    (self.pets.alerted, self.pets.open) = (true, true);
                    self.render();
                }
            }
            (pets::change::HUNGER, Some(pet)) => (self.pets.hunger_before, pet.hunger) = (pet.hunger, value),
            // Anyone's pet is seen wearing it; the player's own has it noted in its window too.
            (pets::change::ACCESSORY, pet) => {
                world::dressed(unit, value as u16);
                if let (Some(pet), true) = (pet, unit == self.pets.unit) {
                    pet.accessory = value as u16;
                }
            }
            // Anyone's pet, not only the player's own.
            (pets::change::PERFORMANCE, _) => {
                world::performs(unit, value);
                // The player's own pet says something of its trick (the client, as the trick starts), and
                // three seconds on may show how it feels (`PetIdle`).
                if unit == self.pets.unit {
                    let moment = match value {
                        2 => pets::moment::TRICK + 1,
                        3 => pets::moment::TRICK + 2,
                        4 => pets::moment::TRICK_SPECIAL,
                        _ => pets::moment::TRICK,
                    };
                    self.pet_says(moment, None);
                    after(3000, Action::PetIdle);
                }
            }
            // Back in its egg: there is no pet any more, and its window closes.
            (pets::change::EGG, _) if value == 0 => {
                (self.pets.pet, self.pets.unit, self.pets.open, self.pets.menu, self.pets.asks) = (None, 0, false, false, 0);
                self.render();
            }
            _ => {}
        }
        self.redraw(&[&wnd::PET, &wnd::PET_ASKS]);
    }

    /// ZC_PET_ACT: a pet shows an emotion, or says one of its lines (the client's `ReceivePetAct`): the line
    /// stands over it and in the chat, as "name : words", in the client's near-white.
    fn pet_acted(&mut self, packet: &[u8]) {
        let (Some((unit, act)), Some(account)) = (pets::acted(packet), &self.account) else { return };
        let own = account.account_id;
        match act {
            pets::Act::Emotion(emotion) => world::emotes(unit, own, emotion),
            pets::Act::Speech { job, moment, hunger } => {
                data::want_pet_talk();
                let name = world::unit_name(unit).or_else(|| self.pets.pet.as_ref().filter(|_| unit == self.pets.unit).map(|pet| pet.name.clone()));
                if let (Some(name), Some(line)) = (name, data::pet_line(job, hunger, moment)) {
                    let words = format!("{name} : {line}");
                    world::said(unit, own, &words);
                    self.chat_line(words, "#fafafa");
                }
            }
        }
    }
}
