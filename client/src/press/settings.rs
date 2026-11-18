//! The presses of the Game Settings window (the client's `CUIGameSettingsUI`): its tabs, the switches and
//! buttons of its options, the boxes that turn the sound off, and Reset (see `crate::press`).

presses! {
    /// What can be pressed here.
    Press {
        /// From the window of game options, and the window's own close buttons: it opens, or closes if it was
        /// open.
        Open = "settings",
        /// A tab of the window, by its place.
        Tab(usize) = "settings-tab",
        /// The switch of an option, by the option's name in the client's `GameSettings.lub`.
        Switch(String) = "settings-switch",
        /// The button of an option that does something at a press, by the option's name.
        Run(String) = "settings-run",
        /// The box that turns one kind of sound off, and on again.
        Off(Sound) = "settings-off",
        /// Reset: every option as the client has it.
        Reset = "settings-reset",
    }
}

/// One of the two kinds of sound the first page sets: the music, or the effects. It is written as the end of
/// its slider's name too (`settings-music`, `settings-effects`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sound {
    Music,
    Effects,
}

impl std::fmt::Display for Sound {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        to.write_str(if *self == Sound::Music { "music" } else { "effects" })
    }
}

impl std::str::FromStr for Sound {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        match text {
            "music" => Ok(Sound::Music),
            "effects" => Ok(Sound::Effects),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [
            Press::Open,
            Press::Tab(4),
            Press::Switch("TT_EFFECT_ON_OFF".to_owned()),
            Press::Run("TT_EX_SIT".to_owned()),
            Press::Off(Sound::Music),
            Press::Off(Sound::Effects),
            Press::Reset,
        ];
        assert_eq!(all.len(), Press::NAMES.len() + 1, "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }

    #[test]
    fn a_sound_that_is_neither_is_no_press() {
        assert_eq!(Press::parse("settings-off", "voice"), None);
    }
}
