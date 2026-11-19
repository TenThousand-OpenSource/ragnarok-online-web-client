//! What the app keeps of the skills: those known, the one aimed, the hotkey bar.

use super::*;

/// What the character knows, and what is on its hotkey bar.
#[derive(Default)]
pub(in crate::web) struct Skills {
    pub known: Vec<Skill>,
    /// The Skill window is open, and which of its pages.
    pub open: bool,
    pub tab: usize,
    /// The skill whose description is shown.
    pub info: Option<u16>,
    /// The skill being aimed with the pointer, and its level.
    pub aimed: Option<(u16, u16)>,
    /// The level each skill is used at when it is less than all it has (the arrows of the skill window).
    pub use_level: HashMap<u16, u16>,
    pub hotkeys: Vec<Hotkey>,
    /// The hotkey bar is put away; how many of its rows show beyond the first (none to three).
    pub hotkeys_hidden: bool,
    pub hotkey_rows: u8,
}
