//! What the app keeps of the skills: those known, the one aimed, the hotkey bar.

use super::*;

/// What the character knows, and what is on its hotkey bar.
#[derive(Default)]
pub(in crate::web) struct Skills {
    pub known: Vec<Skill>,
    /// The Skill window is open, and which of its pages.
    pub open: bool,
    pub tab: usize,
    /// The window shows its list of rows, not the tree (the client's `m_simplifiedSkillList`, its small
    /// button), and the row of it that is marked.
    pub mini: bool,
    pub picked: Option<usize>,
    /// The levels planned in the window and not sent yet, a skill for each level, in the order they are
    /// sent in on Apply (the client's `m_plannedSkillLevels`; `crate::skills::raises`), and whether Apply
    /// has asked its question.
    pub plan: Vec<u16>,
    pub asks: bool,
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
    /// A job level was gained and the skill window not opened since: the button that tells of it shows (the
    /// client's `WID_NOTIFYJOBLEVELUPWND`), and the menu's icon of the skills wears its mark.
    pub level_told: bool,
    /// A Talkie Box or Graffiti waits for its words: the skill, its level and the cell it was aimed at (the
    /// client's `UITalkboxTrapInputWnd`, `UIGraffiStrboxWnd`).
    pub writing: Option<(u16, u16, (u16, u16))>,
}
