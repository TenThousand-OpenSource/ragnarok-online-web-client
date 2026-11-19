//! What the app keeps of the states the character is in.

/// A state the character is in: its number, when it runs out on the page's clock (`now`; none for one without
/// a time), and how long it lasts in all (nought where the server did not say), in milliseconds.
pub(in crate::web) struct Active {
    pub state: u32,
    pub end: Option<u32>,
    pub total: u32,
}

/// The states the character is in (buffs and debuffs), listed at the side of the screen.
#[derive(Default)]
pub(in crate::web) struct Statuses {
    /// Those that are on, in the order they came (the client's `g_activeSkillEffects`).
    pub on: Vec<Active>,
    /// The body state the server last told of: the state shown for it goes when another comes.
    pub(in crate::web::app) body: u16,
    /// The page's timer that writes the times left anew is running.
    pub(in crate::web::app) ticking: bool,
}
