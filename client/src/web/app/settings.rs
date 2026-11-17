//! What the player has set in the Game Settings window, and its keeping.

use super::*;

/// Where the browser keeps it: the options switched (`name=1`), and the sound (`music,effects,music off,effects off`).
const OPTIONS_KEPT: &str = "roweb.options";
const SOUND_KEPT: &str = "roweb.levels";

impl App {
    /// Reads what the browser keeps.
    pub(super) fn load_settings(&mut self) {
        self.settings.set = stored(OPTIONS_KEPT).unwrap_or_default().split(';').filter_map(|entry| entry.split_once('=')).map(|(name, on)| (name.to_owned(), on == "1")).collect();
        let levels: Vec<u8> = stored(SOUND_KEPT).unwrap_or_default().split(',').filter_map(|number| number.parse().ok()).collect();
        (self.settings.music, self.settings.effects, self.settings.music_off, self.settings.effects_off) = match levels[..] {
            [music, effects, music_off, effects_off] => (music.min(100), effects.min(100), music_off != 0, effects_off != 0),
            _ => (100, 100, false, false),
        };
        self.sound_levels();
    }

    /// Whether an option of the client is on (by its name, `TT_FIXED_CAMERA_ON_OFF`): as the player set it,
    /// or as the client's table starts it.
    pub(in crate::web) fn option(&self, id: &str) -> bool {
        self.settings.set.get(id).copied().unwrap_or_else(|| data::option_starts(id))
    }

    /// Switches an option, keeps it, and has what it changes follow.
    pub(super) fn switch_option(&mut self, id: &str) {
        let on = !self.option(id);
        // One that is as it starts again is not kept: it follows the client's table.
        if on == data::option_starts(id) {
            self.settings.set.remove(id);
        } else {
            self.settings.set.insert(id.to_owned(), on);
        }
        let kept: Vec<String> = self.settings.set.iter().map(|(name, on)| format!("{name}={}", u8::from(*on))).collect();
        store(OPTIONS_KEPT, (!kept.is_empty()).then(|| kept.join(";")).as_deref());
        self.apply_options();
    }

    /// Tells the world of the options it goes by.
    pub(in crate::web) fn apply_options(&self) {
        world::camera_fixed(self.option("TT_FIXED_CAMERA_ON_OFF"));
        world::options(world::Options {
            no_ctrl: self.option("TT_NOCTRL_ON_OFF"),
            quick_right: self.option("TT_QUICKSPELL_ON_OFF"),
            quick_wheel: self.option("TT_QUICKSPELL2_ON_OFF"),
            effects: self.option("TT_EFFECT_ON_OFF"),
            misses: self.option("TT_MISS_EFFECT_ON_OFF"),
            monster_life: self.option("TT_MONSTERHP_ON_OFF"),
            hide_players: self.option("TT_HIDE_PLAYER_ON_OFF"),
        });
    }

    /// The sound's levels as set, told to what plays, and kept.
    pub(super) fn sound_levels(&self) {
        let level = |level: u8, off: bool| if off { 0.0 } else { f64::from(level) / 100.0 };
        sound::levels(level(self.settings.music, self.settings.music_off), level(self.settings.effects, self.settings.effects_off));
        store(SOUND_KEPT, Some(&format!("{},{},{},{}", self.settings.music, self.settings.effects, u8::from(self.settings.music_off), u8::from(self.settings.effects_off))));
    }

    /// Reset in the window: every option as the client's table starts it, the sound at full.
    pub(super) fn reset_settings(&mut self) {
        self.settings.set.clear();
        (self.settings.music, self.settings.effects, self.settings.music_off, self.settings.effects_off) = (100, 100, false, false);
        store(OPTIONS_KEPT, None);
        self.sound_levels();
        self.apply_options();
    }
}
