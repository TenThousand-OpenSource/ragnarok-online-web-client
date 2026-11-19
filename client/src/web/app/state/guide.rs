//! What the app keeps of the Adventure Guide.

use super::*;

/// The Adventure Guide's window: whether it is open, which of its tabs, the monster picked in its list, and the
/// monsters' pictures made so far (and those asked for), by the monster's number.
#[derive(Default)]
pub(in crate::web) struct GuideWnd {
    pub open: bool,
    pub tab: usize,
    /// The kind picked under the tab (the tab's own smaller tabs), where it has kinds: its number.
    pub kind: Option<u32>,
    /// The open tab's kinds are folded away: a press on the tab that is open already does that, and another
    /// brings them back (the client's `CUITab::SetSelected`).
    pub folded: bool,
    pub picked: Option<String>,
    pub pictures: HashMap<u32, (String, (usize, usize), (f32, f32))>,
    pub asked: std::collections::HashSet<u32>,
}
