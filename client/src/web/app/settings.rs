//! What the player has set in the Game Settings window, and its keeping.

use super::*;

/// Where the browser keeps it: the options switched (`name=1`), and the sound (`music,effects,music off,effects off`).
const OPTIONS_KEPT: &str = "roweb.options";
const SOUND_KEPT: &str = "roweb.levels";
/// This project's own option, kept with the client's: a far press is asked of the server once, as the Windows
/// client does, instead of being walked in stretches (`world::Options::far_walk`). The stretches are what a
/// player has until this is switched on with `/farwalk` — the owner's choice: a place in sight is walked to
/// when the way round is there, however far.
pub(super) const CLIENT_WALK: &str = "ROWEB_CLIENT_WALK";

impl App {
    /// Reads what the browser keeps.
    pub(super) fn load_settings(&mut self) {
        self.settings.set = crate::kept::pairs(&stored(OPTIONS_KEPT).unwrap_or_default(), '=').map(|(name, on)| (name.to_owned(), on == "1")).collect();
        let levels: Vec<u8> = crate::kept::numbers(&stored(SOUND_KEPT).unwrap_or_default(), ',');
        (self.settings.music, self.settings.effects, self.settings.music_off, self.settings.effects_off) = match levels[..] {
            [music, effects, music_off, effects_off] => (music.min(100), effects.min(100), music_off != 0, effects_off != 0),
            _ => (100, 100, false, false),
        };
        self.sound_levels();
    }

    /// Whether an option of the client is on (by its name, `TT_FIXED_CAMERA_ON_OFF`): as the player set it,
    /// or as the client's table starts it.
    pub(in crate::web) fn option(&self, id: &str) -> bool {
        self.settings.set.get(id).copied().unwrap_or_else(|| Self::option_starts(id))
    }

    /// How an option starts: as the client's table of the Game Settings has it, or, for one that has a
    /// command and no row there, as `commands/beside.rs` has it.
    fn option_starts(id: &str) -> bool {
        Self::starts_beside(id).unwrap_or_else(|| data::option_starts(id))
    }

    /// Switches an option, keeps it, and has what it changes follow.
    pub(super) fn switch_option(&mut self, id: &str) {
        let on = !self.option(id);
        // One that is as it starts again is not kept: it follows the client's table.
        if on == Self::option_starts(id) {
            self.settings.set.remove(id);
        } else {
            self.settings.set.insert(id.to_owned(), on);
        }
        store(OPTIONS_KEPT, crate::kept::paired(self.settings.set.iter().map(|(name, on)| (name, u8::from(*on))), '=').as_deref());
        self.apply_options();
        // What the server is told of (`CZ_CONFIG`): whether the guild's Emergency Call may take the character
        // (`/call`, the setting 1), and whether the pet's window opens by itself when the pet is unhappy (the
        // setting 6).
        match id {
            "TT_BLOCK_CALL_ON_OFF" => self.send_on_map(&crate::net::config::set(1, on)),
            "TT_PETINFO_ALERT_ONOFF" => self.send_on_map(&crate::net::config::set(6, on)),
            _ => {}
        }
        // The minimap's buttons show or go as their option says.
        if id == "TT_SHOW_MINIMAP_BUTTON_ONOFF" {
            self.render();
        }
    }

    /// Tells the world of the options it goes by.
    pub(in crate::web) fn apply_options(&self) {
        self.vending_signs_told();
        world::camera_fixed(self.option("TT_FIXED_CAMERA_ON_OFF"));
        world::options(world::Options {
            no_ctrl: self.option("TT_NOCTRL_ON_OFF"),
            quick_right: self.option("TT_QUICKSPELL_ON_OFF"),
            quick_wheel: self.option("TT_QUICKSPELL2_ON_OFF"),
            effects: self.option("TT_EFFECT_ON_OFF"),
            misses: self.option("TT_MISS_EFFECT_ON_OFF"),
            monster_life: self.option("TT_MONSTERHP_ON_OFF"),
            hide_players: self.option("TT_HIDE_PLAYER_ON_OFF"),
            zoom_out: self.option("TT_EXPAND_SIGHT_ON_OFF"),
            far_walk: !self.option(CLIENT_WALK),
            quake: self.option("TT_QUAKE_ON_OFF"),
            name_above: self.option("TT_NAME_ABOVE_HEAD_ON_OFF"),
            emblems: self.option("TT_SHOW_EMBLEM_ON_OFF"),
            // (The client keeps these two the other way round from their rows: `crate::aura::Options`.)
            auras: self.option("TT_HIDE_AURA_ON_OFF"),
            aura_full: !self.option("TT_FULL_AURA_ON_OFF"),
        });
        // The fog of maps shows or not (`/fog`).
        world::fog(self.option("TT_FOG_ON_OFF"));
        // A window being moved docks to the others and to the screen's edges (`/window`).
        snap_windows(self.option("TT_WINDOW_SNAP_ON_OFF"));
    }

    /// The sound's levels as set, told to what plays, and kept.
    pub(super) fn sound_levels(&self) {
        let level = |level: u8, off: bool| if off { 0.0 } else { f64::from(level) / 100.0 };
        sound::levels(level(self.settings.music, self.settings.music_off), level(self.settings.effects, self.settings.effects_off));
        store(
            SOUND_KEPT,
            crate::kept::numbered(
                [self.settings.music, self.settings.effects, u8::from(self.settings.music_off), u8::from(self.settings.effects_off)],
                ','
            )
                .as_deref()
        );
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
