//! What the app keeps of the reputation.

use crate::reputation::Filter;

/// The reputation (the client's `CReputeMgr` and `UIReputeWnd`).
pub(in crate::web) struct Reputation {
    pub open: bool,
    /// The points the server told, by the reputation's number.
    pub points: Vec<(u64, i64)>,
    /// The server has told the points: until then the window says that it is loading
    /// (`CReputeMgr::m_waitingForPoints`).
    pub told: bool,
    /// What the window lists, and the page of it, counted from 1.
    pub filter: Filter,
    pub page: usize,
    /// The group the drop-down shows, by its place among the groups; none for all of them. It stays what it
    /// was while a word sought is listed.
    pub group: Option<usize>,
    /// The drop-down of the groups is open.
    pub choosing: bool,
    /// The reputations a script marked as it opened the window, until the page is left
    /// (`ReputeInfo::updated`).
    pub marked: Vec<u64>,
}

impl Default for Reputation {
    fn default() -> Self {
        Self { open: false, points: Vec::new(), told: false, filter: Filter::All, page: 1, group: None, choosing: false, marked: Vec::new() }
    }
}
