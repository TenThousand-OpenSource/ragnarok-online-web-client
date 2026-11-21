//! What the app keeps of a talk with an NPC.

/// A talk with an NPC, as its window shows it.
#[derive(Default)]
pub(in crate::web) struct Dialog {
    pub npc: u32,
    pub lines: Vec<String>,
    /// The button under the text: "next" (true) or "close" (false).
    pub button: Option<bool>,
    /// What "next" was pressed after: the next line starts the text afresh.
    pub stale: bool,
    /// The choices of a menu, and the one picked.
    pub menu: Vec<String>,
    pub picked: usize,
    /// Something is asked for: words (true) or a number (false).
    pub asks: Option<bool>,
    /// When "next" or the menu's OK was last taken, by the client's clock (`crate::talk::too_soon`).
    pub answered: u32,
}
