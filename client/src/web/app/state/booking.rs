//! What the app keeps of the old party booking: the ads, the list's window with what is picked in it, the
//! window of registering, one's own ad and the window its jobs are changed in.

use std::collections::BTreeSet;

use crate::booking::{Board, Sought};
use crate::press::booking::Choice;

/// One's own ad while it is registered (the client's `UISeekPartyWnd`, and `registeredTick` of its session).
pub(in crate::web) struct OwnAd {
    /// When the server said it was registered, by the page's clock.
    pub since: u32,
    /// Which of the window's moving pictures it shows (picked as it opens).
    pub kind: u32,
    /// The level it asks for (the client's `m_partyBookingLevelLimit`).
    pub level: i16,
    /// The window is made small.
    pub small: bool,
}

/// The old party booking (the client's `PartyBookingMgr` and its four windows).
pub(in crate::web) struct Booking {
    /// The ads as the client keeps them.
    pub board: Board,
    /// The list's window is open, and made small.
    pub list: bool,
    pub small: bool,
    /// What is typed and picked over the list: the level, the region and its map, the group of jobs and the
    /// job of it (each by its place in its box), and the box "Notice".
    pub level: String,
    pub region: usize,
    pub map: usize,
    pub group: usize,
    pub job: usize,
    pub alarm: bool,
    /// What the list last sought (the window's `m_filterLevel`…), and the title that says so.
    pub sought: Sought,
    pub title: Option<String>,
    /// The row picked, by the number of its ad.
    pub picked: Option<u32>,
    /// The box whose list is dropped now, of either window.
    pub dropped: Option<Choice>,
    /// The help is open.
    pub help: bool,
    /// The window of registering is open, and made small; what is typed, picked and ticked in it (the regions
    /// counted without "all maps"); and the group of jobs it shows.
    pub register: bool,
    pub register_small: bool,
    pub own_level: String,
    pub own_region: usize,
    pub own_map: usize,
    pub own_group: usize,
    pub ticked: BTreeSet<i16>,
    /// The ad last sent to be registered: its level and its jobs (the window's `m_registration`).
    pub sent: Option<(i16, [i16; crate::net::booking::JOBS])>,
    /// One's own ad, while the server has it.
    pub own: Option<OwnAd>,
    /// The jobs of one's ad, each sought or not (the client's `g_partyBookingJobs`); and while the window they
    /// are changed in is open, how they are ticked in it.
    pub jobs: Vec<(i16, bool)>,
    pub changing: Option<Vec<(i16, bool)>>,
}

impl Default for Booking {
    fn default() -> Self {
        Self {
            board: Board::default(),
            list: false,
            small: false,
            level: String::new(),
            region: 0,
            map: 0,
            group: 0,
            job: 0,
            alarm: false,
            sought: Sought::NOTHING_YET,
            title: None,
            picked: None,
            dropped: None,
            help: false,
            register: false,
            register_small: false,
            own_level: String::new(),
            own_region: 0,
            own_map: 0,
            own_group: 0,
            ticked: BTreeSet::new(),
            sent: None,
            own: None,
            jobs: Vec::new(),
            changing: None,
        }
    }
}
