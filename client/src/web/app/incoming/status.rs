//! What the character is: its numbers (life, stats, experience) and the skills it knows.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn status_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_NOTIFY_EFFECT => self.level_gained(packet),
            // ZC_USER_COUNT (0x00c2): `u32 count`: how many play on the server. It is shown by the chat's tabs.
            crate::net::commands::ZC_USER_COUNT => {
                if let Some(count) = crate::net::commands::users(packet) {
                    self.online = Some(count);
                    if let Some(shown) = element("online-count") {
                        shown.set_text_content(self.online.map(|count| count.to_string()).as_deref());
                    } else {
                        self.render();
                    }
                }
            }
            packets::ZC_PAR_CHANGE | packets::ZC_LONGPAR_CHANGE => self.value_changed(packet),
            packets::ZC_LONGLONGPAR_CHANGE => {
                if let Some((name, value)) = packets::long_parameter(packet) {
                    self.hero.params.insert(name, value);
                    self.refresh_values();
                }
            }
            packets::ZC_STATUS => {
                self.hero.params.extend(packets::status(packet));
                self.refresh_values();
            }
            packets::ZC_STATUS_CHANGE => {
                if let Some((name, cost)) = packets::stat_cost(packet) {
                    self.hero.params.insert(name, i64::from(cost));
                    self.refresh_values();
                }
            }
            packets::ZC_COUPLESTATUS => {
                if let Some((name, base, bonus)) = packets::stat(packet) {
                    self.hero.params.insert(name, i64::from(base));
                    if let Some(slot) = name.checked_sub(packets::parameter::STR).and_then(|stat| self.hero.stat_bonus.get_mut(usize::from(stat))) {
                        *slot = bonus;
                    }
                    self.refresh_values();
                }
            }
            net::inventory::ZC_SKILLINFO_LIST => {
                self.skills.known = net::inventory::skills(packet);
                self.refresh_items();
            }
            net::inventory::ZC_ADD_SKILL => {
                if let Some(added) = net::inventory::skill_added(packet) {
                    self.skills.known.retain(|skill| skill.id != added.id);
                    self.skills.known.push(added);
                    self.refresh_items();
                }
            }
            net::inventory::ZC_SKILLINFO_UPDATE => {
                if let Some((id, level, sp, range, raisable)) = net::inventory::skill_raised(packet) {
                    if let Some(skill) = self.skills.known.iter_mut().find(|skill| skill.id == id) {
                        (skill.level, skill.sp, skill.range, skill.raisable) = (level, sp, range, raisable);
                    }
                    self.refresh_items();
                }
            }
            _ => return false,
        }
        true
    }

    /// ZC_NOTIFY_EFFECT (0x019b): `u32 who, u32 kind`: something to show on someone, as the client's
    /// `ReceivePartyMemberEffect` shows it: the angel of a base level gained (0: `EF_ANGEL`, `angel.str`), the
    /// light of a job level (1: `EF_JOBLVUP`, `JobLvUP.str`), a refining that failed (2: `EF_REFINEFAIL`) or
    /// went well (3: `EF_REFINEOK`), a potion brewed (5: `EF_PHARMACY_OK`) or spoilt (6:
    /// `EF_PHARMACY_FAIL`), the level of a super novice (7: `EF_ANGEL2`; 8: `EF_JOBLVUP50` for its job
    /// level) and of a taekwon (9: `EF_ANGEL3`). Kind 4 (the game is over) only sets a switch in the client.
    fn level_gained(&mut self, packet: &[u8]) {
        let own = self.account.as_ref().map(|account| account.account_id);
        let (who, kind) = packets::effect_on(packet);
        let on = who.filter(|who| Some(*who) != own);
        match kind {
            0 => world::effect("angel.str", on),
            1 => world::effect("joblvup.str", on),
            _ => {
                if let (Some(shown), Some(_)) = (crate::skills::notified_effect(kind), who) {
                    world::burst(shown, on, None, 0.0);
                }
            }
        }
        // A level gained by the player's own character is heard, and the pet may tell of it.
        if who == own && who.is_some() && matches!(kind, 0 | 1 | 7 | 8 | 9) {
            sound::effect("levelup.wav", 1.0);
            self.pet_feels(crate::pets::moment::LEVEL_UP, None);
        }
    }

    fn value_changed(&mut self, packet: &[u8]) {
        if let Some((name, value)) = packets::parameter(packet) {
            if name == packets::PARAMETER_SPEED {
                world::speed(value);
            }
            // How heavy the character was before, for what the chat says of the weight (`incoming/inventory.rs`).
            let weighed = |app: &Self, name: u16| app.hero.params.get(&name).copied().unwrap_or(0);
            let heavy = crate::inventory::weight::percent(weighed(self, packets::parameter::WEIGHT), weighed(self, packets::parameter::MAX_WEIGHT));
            let was = weighed(self, name);
            self.hero.params.insert(name, i64::from(value));
            if name == packets::parameter::WEIGHT || name == packets::parameter::MAX_WEIGHT {
                self.weight_told(name, was, heavy);
            }
            self.refresh_values();
            // The skill window writes the points left to give and marks what they can raise (the client's
            // `UINewSkillListWnd` is told `m_skillPoints` anew): it follows them while it is open.
            if name == packets::parameter::SKILL_POINTS && was != i64::from(value) {
                self.redraw(&[&wnd::SKILLS]);
            }
            // Under a quarter of its life left, the player's pet may tell of it: half the time, and
            // not more than once in five seconds (the client, on `SP_HP`).
            let most = self.hero.params.get(&packets::parameter::MAX_HP).copied().unwrap_or(0);
            let time = js_sys::Date::now();
            if name == packets::parameter::HP && most > 0 && i64::from(value) * 100 / most < 25 && self.pets.pet.is_some() && js_sys::Math::random() < 0.5 && time - self.pets.warned_at >= 5000.0 {
                self.pets.warned_at = time;
                self.pet_feels(crate::pets::moment::DANGER, None);
            }
        }
    }
}
