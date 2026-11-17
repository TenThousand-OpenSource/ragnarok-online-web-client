//! What the character is: its numbers (life, stats, experience) and the skills it knows.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn status_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // 0x019b: someone gains a level (0 base, 1 job).
            0x019b => {
                let own = self.account.as_ref().map(|account| account.account_id);
                if npc::npc(id, packet) == own {
                    sound::effect("levelup.wav", 1.0);
                }
            }
            packets::ZC_PAR_CHANGE | packets::ZC_LONGPAR_CHANGE => {
                if let Some((name, value)) = packets::parameter(packet) {
                    if name == packets::PARAMETER_SPEED {
                        world::speed(value);
                    }
                    self.params.insert(name, i64::from(value));
                    self.refresh_values();
                }
            }
            packets::ZC_LONGLONGPAR_CHANGE => {
                if let Some((name, value)) = packets::long_parameter(packet) {
                    self.params.insert(name, value);
                    self.refresh_values();
                }
            }
            packets::ZC_STATUS => {
                self.params.extend(packets::status(packet));
                self.refresh_values();
            }
            packets::ZC_STATUS_CHANGE => {
                if let Some((name, cost)) = packets::stat_cost(packet) {
                    self.params.insert(name, i64::from(cost));
                    self.refresh_values();
                }
            }
            packets::ZC_COUPLESTATUS => {
                if let Some((name, base, bonus)) = packets::stat(packet) {
                    self.params.insert(name, i64::from(base));
                    if let Some(slot) = name.checked_sub(packets::parameter::STR).and_then(|stat| self.stat_bonus.get_mut(usize::from(stat))) {
                        *slot = bonus;
                    }
                    self.refresh_values();
                }
            }
            items::ZC_SKILLINFO_LIST => {
                self.skills = items::skills(packet);
                self.refresh_items();
            }
            items::ZC_ADD_SKILL => {
                if let Some(added) = items::skill_added(packet) {
                    self.skills.retain(|skill| skill.id != added.id);
                    self.skills.push(added);
                    self.refresh_items();
                }
            }
            items::ZC_SKILLINFO_UPDATE => {
                if let Some((id, level, sp, range, raisable)) = items::skill_raised(packet) {
                    if let Some(skill) = self.skills.iter_mut().find(|skill| skill.id == id) {
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
