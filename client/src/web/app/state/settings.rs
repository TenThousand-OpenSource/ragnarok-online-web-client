//! What the app keeps of the Game Settings and of the Shortcut Settings.

use super::*;

/// The Game Settings window, and what the player has set in it.
#[derive(Default)]
pub(in crate::web) struct Settings {
    /// The window is open, and which of its five pages.
    pub open: bool,
    pub tab: usize,
    /// The options the player has switched otherwise than they start, by the client's name: kept in the browser.
    pub set: HashMap<String, bool>,
    /// How loud the music and the effects are (0 to 100), and whether each is off: kept in the browser.
    pub music: u8,
    pub effects: u8,
    pub music_off: bool,
    pub effects_off: bool,
}

/// The keys: what the player has bound otherwise than the client does, and the window that shows and sets them.
#[derive(Default)]
pub(in crate::web) struct Hotkeys {
    /// The Shortcut Settings window is open; which of its tabs, which page of it, and the row picked in it.
    pub open: bool,
    pub tab: usize,
    pub page: usize,
    pub row: Option<usize>,
    /// What the player has set, by table and place: kept in the browser.
    pub own: HashMap<(usize, usize), Chord>,
    /// What was set in the window since it was opened, until OK keeps it or Cancel drops it.
    pub edited: HashMap<(usize, usize), Chord>,
    /// The keyboard is a Mac's: some keys are missing on it, and Alt is called Option.
    pub mac: bool,
    /// What the window says of the last key pressed in it, when it would not take it.
    pub notice: Option<String>,
    /// The Hotkey Guide is open (the client's `WID_HOTKEYGUIDEWND`).
    pub guide: bool,
}
