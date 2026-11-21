//! The Game Settings window.

use super::*;

impl App {
    /// A press of this system (`crate::press::settings`).
    pub(super) fn settings_pressed(&mut self, press: crate::press::settings::Press) {
        use crate::press::settings::{Press, Sound};
        match press {
            // From the window of game options, and the window's own close buttons: it opens, or closes if it
            // was open (the client's `ID_OPEN_GAME_SETTINGS`).
            Press::Open => {
                self.settings.open = !self.settings.open;
                self.render();
            }
            Press::Tab(tab) => {
                self.settings.tab = tab;
                self.redraw(&[&wnd::SETTINGS]);
            }
            Press::Switch(option) => {
                self.switch_option(&option);
                self.aura_hidden_said(&option);
                self.redraw(&[&wnd::SETTINGS]);
            }
            // A row that does something at a press (the client's `CUIOptionItem::RunFunction`).
            Press::Run(option) => {
                if option == "TT_EX_SIT" {
                    // Sits down or stands up, whichever it is not doing (the client's `ToggleSitStand`).
                    self.command(if self.hero.sitting { "/stand" } else { "/sit" });
                } else if let Some(command) = Self::command_of(&option) {
                    self.typed_command(command);
                }
            }
            Press::Off(sound) => {
                match sound {
                    Sound::Music => self.settings.music_off = !self.settings.music_off,
                    Sound::Effects => self.settings.effects_off = !self.settings.effects_off,
                }
                self.sound_levels();
                self.redraw(&[&wnd::SETTINGS]);
            }
            Press::Reset => {
                self.reset_settings();
                self.redraw(&[&wnd::SETTINGS]);
            }
        }
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
