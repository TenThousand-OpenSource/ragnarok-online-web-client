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
            _ => return false,
        }
        true
    }
}
