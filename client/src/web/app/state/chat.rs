//! What the app keeps of the chat window.

/// The chat window.
#[derive(Default)]
pub(in crate::web) struct Chat {
    /// Its lines, oldest first, each with its colour and its kind (`said`).
    pub lines: Vec<(String, String, u8)>,
    /// How many rows it shows (0 leaves only its input line), and how many it had before its log was hidden.
    pub rows: usize,
    pub rows_kept: usize,
    /// Which of its tabs is open, and whether the window is locked where it is.
    pub tab: usize,
    pub locked: bool,
    /// Its tabs as the player has them (`crate::chat_tabs`), those taken out into windows of their own among
    /// them: kept in the browser. Empty until the player changes one: then they are as the client starts them.
    pub tabs: Vec<crate::chat_tabs::Tab>,
    /// A line with a link to an item came in whose name may not be known yet.
    pub linked: bool,
    /// The window is put away (Alt+F10 shows and hides it).
    pub hidden: bool,
    /// The tab whose window of what it lists is open (`UIBattleMsgOptionWnd`).
    pub options: Option<usize>,
}
