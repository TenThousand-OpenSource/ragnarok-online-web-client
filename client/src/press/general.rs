//! The presses that belong to no one window: the language, the sound, a message's OK, the minimap's buttons, the
//! button that folds the menu away, and the game's menu (`UIEscOptionWnd`).

use super::Step;
use crate::i18n::Lang;

presses! {
    /// What can be pressed here.
    Press {
        /// A language is chosen.
        Language(Lang) = "lang",
        /// The speaker: the sound on and off.
        Sound = "sound",
        /// A message is read.
        Ok = "ok",
        /// The minimap's plus and minus.
        MapZoom(Step) = "map-zoom",
        /// The minimap's button for the marks of who stands where.
        MapMarks = "map-marks",
        /// The button under Basic Information that folds the menu away and brings it back.
        Menu = "menu",
        /// The menu's icon of the game's options: the game's menu opens, or closes if it was open.
        Options = "options",
        /// The game's menu: back to the game.
        EscClose = "esc-close",
        /// The game's menu of one who lies dead: to the save point (it asks first).
        EscSave = "esc-save",
        /// The game's menu of one who lies dead with a Token of Siegfried: to rise where they lie.
        EscRise = "esc-rise",
        /// The question about the save point: no, and yes.
        EscNo = "esc-no",
        EscYes = "esc-yes",
    }
}
