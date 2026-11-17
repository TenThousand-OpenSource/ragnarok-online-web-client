//! What the character is: its numbers (life, stats, experience) and the skills it knows.

use super::*;
use crate::net::pets;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn status_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // 0x019b: someone gains a level (0 base, 1 job).
            0x019b => {
                let own = self.account.as_ref().map(|account| account.account_id);
                if npc::npc(id, packet) == own {
                    sound::effect("levelup.wav", 1.0);
                    self.pet_feels(pets::moment::LEVEL_UP, None);
                }
            }
            packets::ZC_PAR_CHANGE | packets::ZC_LONGPAR_CHANGE => {
                if let Some((name, value)) = packets::parameter(packet) {
                    if name == packets::PARAMETER_SPEED {
                        world::speed(value);
                    }
                    self.hero.params.insert(name, i64::from(value));
                    self.refresh_values();
                    // Under a quarter of its life left, the player's pet may tell of it: half the time, and
                    // not more than once in five seconds (the client, on `SP_HP`).
                    let most = self.hero.params.get(&packets::parameter::MAX_HP).copied().unwrap_or(0);
                    let time = js_sys::Date::now();
                    if name == packets::parameter::HP && most > 0 && i64::from(value) * 100 / most < 25 && self.pets.pet.is_some() && js_sys::Math::random() < 0.5 && time - self.pets.warned_at >= 5000.0 {
                        self.pets.warned_at = time;
                        self.pet_feels(pets::moment::DANGER, None);
                    }
                }
            }
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
            items::ZC_SKILLINFO_LIST => {
                self.skills.known = items::skills(packet);
                self.refresh_items();
            }
            items::ZC_ADD_SKILL => {
                if let Some(added) = items::skill_added(packet) {
                    self.skills.known.retain(|skill| skill.id != added.id);
                    self.skills.known.push(added);
                    self.refresh_items();
                }
            }
            items::ZC_SKILLINFO_UPDATE => {
                if let Some((id, level, sp, range, raisable)) = items::skill_raised(packet) {
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
}
