//! The skill window, a skill's description, and the hotkey bar.

use super::*;

impl App {
    /// A press of this system (`crate::press::skills`).
    pub(super) fn skill_pressed(&mut self, press: crate::press::skills::Press) {
        use crate::press::Step;
        use crate::press::skills::{LevelStep, Press};
        match press {
            Press::Window => {
                self.skills.open = !self.skills.open;
                self.render();
            }
            Press::Raise(id) => {
                if let Some(socket) = &self.socket {
                    socket.send(&items::raise_skill(id));
                }
            }
            Press::InfoClose => {
                self.skills.info = None;
                self.refresh_items();
            }
            Press::HotkeyRows(step) => {
                self.skills.hotkey_rows = if step == Step::More { (self.skills.hotkey_rows + 1).min(3) } else { self.skills.hotkey_rows.saturating_sub(1) };
                self.render();
            }
            Press::Tab(tab) => {
                self.skills.tab = tab;
                self.render();
            }
            // An arrow by a skill's level: the level it is used at, one down or up, between 1 and what is known
            // (the client's `UINewSkillListWnd`). It is the level it goes onto the hotkey bar with.
            Press::Level(LevelStep { skill: id, step }) => {
                let Some(known) = self.skills.known.iter().find(|skill| skill.id == id).map(|skill| skill.level.max(1)) else { return };
                let used = self.skills.use_level.get(&id).copied().unwrap_or(known);
                let wanted = if step == Step::More { (used + 1).min(known) } else { used.saturating_sub(1).max(1) };
                if wanted == known {
                    self.skills.use_level.remove(&id);
                } else {
                    self.skills.use_level.insert(id, wanted);
                }
                // Kept in the browser for this character, so the levels are the same next time. (Where the
                // client keeps them was not looked for: this is this client's own way.)
                if let Some(zone) = &self.zone {
                    let kept: Vec<String> = self.skills.use_level.iter().map(|(skill, level)| format!("{skill}:{level}")).collect();
                    store(&format!("roweb.skill-levels.{}", zone.char_id), (!kept.is_empty()).then(|| kept.join(";")).as_deref());
                }
                self.refresh_items();
            }
        }
    }
}
