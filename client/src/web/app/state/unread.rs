//! What the app keeps of the packets it does not read.

/// The packets of the map server that no reader took, counted since the page was opened, and whether the
/// list of them is shown (`/packets`: a tool of this project's own, the client has nothing of the kind).
#[derive(Default)]
pub(in crate::web) struct Unread {
    pub tally: crate::unread::Tally,
    pub open: bool,
}
