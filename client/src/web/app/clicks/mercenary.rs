//! The mercenary window, what it asks, the list of the mercenary's skills, and the menu on the mercenary.

use super::*;
use crate::net::mercenary;

impl App {
    /// A press of this system (`crate::press::mercenary`).
    pub(super) fn mercenary_pressed(&mut self, press: crate::press::mercenary::Press) {
        use crate::press::mercenary::{Order, Press};
        match press {
            Press::Window => {
                (self.mercenary.open, self.mercenary.asks) = (!self.mercenary.open && self.mercenary.shown(), false);
                self.render();
            }
            Press::Fire => {
                self.mercenary.asks = true;
                self.redraw(&[&wnd::MERCENARY_ASKS]);
            }
            Press::Skills => {
                self.mercenary.skills_open = !self.mercenary.skills_open && self.mercenary.shown();
                self.render();
            }
            // Dismissed (`UIMerInfoWnd::SendMsg`, its `ID_FIRE_MER`): the server is told, and its windows
            // close at once.
            Press::Yes | Press::No => {
                let fired = std::mem::take(&mut self.mercenary.asks) && press == Press::Yes;
                if fired {
                    self.send_on_map(&mercenary::command(mercenary::order::DELETE));
                    (self.mercenary.open, self.mercenary.skills_open, self.mercenary.out, self.mercenary.marked) = (false, false, false, None);
                }
                self.render();
            }
            Press::SkillPick(row) => {
                self.mercenary.skill_picked = Some(row);
                self.redraw(&[&wnd::MERCENARY_SKILLS]);
            }
            Press::SkillUse => {
                if let Some(skill) = self.mercenary.skill_picked.and_then(|row| self.mercenary.skills.get(row)).filter(|skill| skill.level != 0 && skill.sp != 0) {
                    self.use_it(Use::Skill(skill.id));
                }
            }
            Press::SkillLevel(step) => {
                let most = self.mercenary.skills.iter().find(|skill| skill.id == step.skill).map(|skill| skill.level);
                self.companion_skill_level(step, most);
                self.redraw(&[&wnd::MERCENARY_SKILLS]);
            }
            // The menu on the mercenary itself (the client's `MENUID_MER_SHOWINFO`, `MENUID_MER_FOLLOW`).
            Press::Do(order) => {
                self.mercenary.menu_at = None;
                match order {
                    Order::Info => {
                        self.send_on_map(&mercenary::command(mercenary::order::INFO));
                        self.mercenary.open = !self.mercenary.open;
                    }
                    Order::StandBy => self.companion_stands_by(crate::ai::Kind::Mercenary),
                }
                self.render();
            }
        }
    }
}
