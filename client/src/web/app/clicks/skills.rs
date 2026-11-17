//! The skill window, a skill's description, and the hotkey bar.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn skill_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "skills" => {
                self.skills.open = !self.skills.open;
                self.render();
            }
            "raise-skill" => {
                if let (Ok(id), Some(socket)) = (arg.parse(), &self.socket) {
                    socket.send(&items::raise_skill(id));
                }
            }
            "skill-info-close" => {
                self.skills.info = None;
                self.refresh_items();
            }
            "hotkey-rows" => {
                self.skills.hotkey_rows = if arg == "+" { (self.skills.hotkey_rows + 1).min(3) } else { self.skills.hotkey_rows.saturating_sub(1) };
                self.render();
            }
            "skill-tab" => {
                self.skills.tab = arg.parse().unwrap_or(0);
                self.render();
            }
            // An arrow by a skill's level: the level it is used at, one down or up, between 1 and what is known
            // (the client's `UINewSkillListWnd`). It is the level it goes onto the hotkey bar with.
            "skill-level" => {
                let Some((id, step)) = arg.split_once(':').and_then(|(id, step)| Some((id.parse::<u16>().ok()?, step))) else { return true };
                let Some(known) = self.skills.known.iter().find(|skill| skill.id == id).map(|skill| skill.level.max(1)) else { return true };
                let used = self.skills.use_level.get(&id).copied().unwrap_or(known);
                let wanted = if step == "+" { (used + 1).min(known) } else { used.saturating_sub(1).max(1) };
                if wanted == known {
                    self.skills.use_level.remove(&id);
                } else {
                    self.skills.use_level.insert(id, wanted);
                }
                self.refresh_items();
            }
            _ => return false,
        }
        true
    }
}
