//! What the app keeps of the server's announcements.

/// One announcement shown across the top of the screen (the client's `BroadcastBalloonMessage`).
pub(in crate::web) struct Announced {
    pub lines: Vec<String>,
    /// Its colour, as the page takes it, and the height of its letters.
    pub colour: String,
    pub letters: u16,
    /// When it goes, in milliseconds.
    pub until: f64,
}

/// The announcements shown now, the oldest first (the client's `UIBroadcastBalloon::m_messages`).
#[derive(Default)]
pub(in crate::web) struct Broadcast {
    pub shown: Vec<Announced>,
}
