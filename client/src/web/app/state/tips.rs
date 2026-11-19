//! What the app keeps of the Tipbox.

/// The Tipbox (`UITipboxWnd`).
#[derive(Default)]
pub(in crate::web) struct Tips {
    pub open: bool,
    /// The tip shown, by its number, and which of its pages.
    pub tip: u32,
    pub page: usize,
    /// What a search found: the tips' numbers. None while a tip is shown.
    pub found: Option<Vec<u32>>,
    /// The window is not opened as the game is entered (the box "Open on Startup" was emptied): kept in the
    /// browser. And whether it was already opened so, this visit.
    pub not_at_start: bool,
    pub(in crate::web::app) started: bool,
}
