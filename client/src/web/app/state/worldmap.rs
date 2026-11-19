//! What the app keeps of the World Map and of the large window of the map one is on.

/// The World Map (`UIRoMapWnd`) and the large map (`UIMiniMapWnd`).
#[derive(Default)]
pub(in crate::web) struct WorldMap {
    /// The World Map is open; which world it shows, once one was chosen or found; the list of worlds is open.
    pub open: bool,
    pub world: Option<usize>,
    pub choosing: bool,
    /// The maps' names and levels are written on them (the box "Monster(Tab)").
    pub labels: bool,
    /// The large window of the map one is on is open.
    pub large: bool,
    /// The minimap: 0 shown, 1 shown fainter, 2 away (Ctrl+Tab runs through them).
    pub minimap: u8,
}
