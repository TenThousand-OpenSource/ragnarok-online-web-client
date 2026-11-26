//! What the app keeps of the probabilities.

/// The probabilities: the question of the menu's icon (`UIMenuIconWnd::SendMsg`, `MENUICON_PROBABILITY`).
#[derive(Default)]
pub(in crate::web) struct Probability {
    /// The icon's question is being asked.
    pub asking: bool,
}
