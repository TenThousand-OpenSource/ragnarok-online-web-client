//! What the server tells of a hired mercenary: everything about it, a number of it that changed, its skills.

use super::*;
use crate::net::mercenary;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn mercenary_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Everything about it (the client's `Zc_Mer_Init`): its skills are told anew after.
            mercenary::ZC_MER_INIT => {
                if let Some(info) = mercenary::init(packet) {
                    if self.mercenary.unit() != info.unit {
                        self.mercenary.marked = None;
                    }
                    (self.mercenary.info, self.mercenary.skills) = (Some(info), Vec::new());
                    self.ai_wakes();
                    self.redraw(&[&wnd::MERCENARY, &wnd::MERCENARY_SKILLS]);
                }
            }
            mercenary::ZC_MER_PAR_CHANGE => {
                if let (Some((parameter, value)), Some(info), true) = (mercenary::parameter_changed(packet), &mut self.mercenary.info, self.mercenary.out) {
                    info.change(parameter, value);
                    self.redraw(&[&wnd::MERCENARY]);
                }
            }
            // Its skills, laid out as the character's own; and one of them changed.
            mercenary::ZC_MER_SKILLINFO_LIST => {
                self.mercenary.skills = items::skills(packet);
                self.redraw(&[&wnd::MERCENARY_SKILLS]);
            }
            mercenary::ZC_MER_SKILLINFO_UPDATE => {
                if let Some((id, level, sp, range, _)) = items::skill_raised(packet) {
                    if let Some(skill) = self.mercenary.skills.iter_mut().find(|skill| skill.id == id) {
                        (skill.level, skill.sp, skill.range) = (level, sp, range);
                    }
                    self.redraw(&[&wnd::MERCENARY_SKILLS, &wnd::HOTKEYS]);
                }
            }
            _ => return false,
        }
        true
    }
}
