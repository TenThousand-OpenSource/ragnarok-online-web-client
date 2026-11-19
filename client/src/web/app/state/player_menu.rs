//! What the app keeps of the menu on another player.

/// The menu on another player, opened with the right button: where on the stage, who, and what they are called.
pub(in crate::web) struct PlayerMenu {
    pub at: (i32, i32),
    pub who: u32,
    pub name: String,
    /// The guild the player is of (0 for none: one may be offered).
    pub guild: u32,
}
