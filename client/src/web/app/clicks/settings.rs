//! The Game Settings window.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn settings_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            // From the window of game options, and the window's own close buttons: it opens, or closes if it
            // was open (the client's `ID_OPEN_GAME_SETTINGS`).
            "settings" => {
                self.settings.open = !self.settings.open;
                self.render();
            }
            "settings-tab" => {
                self.settings.tab = arg.parse().unwrap_or(0);
                self.refresh_settings();
            }
            "settings-switch" => {
                self.switch_option(arg);
                self.refresh_settings();
            }
            // A row that does something at a press (the client's `CUIOptionItem::RunFunction`).
            "settings-run" => {
                if arg == "TT_EX_SIT" {
                    // Sits down or stands up, whichever it is not doing (the client's `ToggleSitStand`).
                    self.command(if self.hero.sitting { "/stand" } else { "/sit" });
                } else if let Some(command) = Self::command_of(arg) {
                    self.command(command);
                }
            }
            "settings-off" => {
                match arg {
                    "music" => self.settings.music_off = !self.settings.music_off,
                    _ => self.settings.effects_off = !self.settings.effects_off,
                }
                self.sound_levels();
                self.refresh_settings();
            }
            "settings-reset" => {
                self.reset_settings();
                self.refresh_settings();
            }
            _ => return false,
        }
        true
    }

    /// A slider of the window was moved: how loud the music or the effects are, 0 to 100.
    pub(in crate::web::app) fn settings_slide(&mut self, which: &str, level: u8) {
        match which {
            "settings-music" => self.settings.music = level.min(100),
            "settings-effects" => self.settings.effects = level.min(100),
            _ => return,
        }
        self.sound_levels();
    }
}
