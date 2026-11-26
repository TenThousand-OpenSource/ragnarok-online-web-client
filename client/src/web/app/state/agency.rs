//! What the app keeps of the Adventurer's Agency: the board, a notice being written, one being read, and
//! those who ask a leader to be let in.

use std::collections::VecDeque;

use crate::agency::Notice;
use crate::net::agency::Asking;

/// A notice being written (the client's `UIRegisterPartyWnd`).
#[derive(Clone, Default)]
pub(in crate::web) struct NoticeDraft {
    /// Whose it is, as the window shows it: the player's own name.
    pub name: String,
    /// The roles wanted, the purpose, and whether the box of purposes is open.
    pub roles: [bool; 4],
    pub purpose: i32,
    pub purposes_open: bool,
    /// What stands in the fields: read off the page before the window is drawn anew.
    pub least: String,
    pub most: String,
    pub memo: String,
    /// The window changes a notice there is (its button is Edit), or puts one up (Register).
    pub changes: bool,
}

/// The Adventurer's Agency (the client's `CAdventurerAgencyMgr`, `UIAdvenPartyBoardWnd` and
/// `UIRequestJoinPartyMgr`).
pub(in crate::web) struct Agency {
    /// The board is open.
    pub open: bool,
    /// The player has a notice up (the client's `registered`), and that notice.
    pub registered: bool,
    pub own: Option<Notice>,
    /// The notices of the page shown, the page and how many there are.
    pub entries: Vec<Notice>,
    pub page: i32,
    pub pages: i32,
    /// The list shown is what a search found.
    pub searching: bool,
    /// A page was asked for and has not come: the arrows and the button that asks anew are dim.
    pub loading: bool,
    /// A list has come since the board opened: only then does the board show whether one may put up a notice.
    pub settled: bool,
    /// Whose notice is being taken down (the client's `masterAccountId`).
    pub stopping: u32,
    /// The search: the roles asked for, the purpose, whether the box of purposes is open, and what stands in
    /// its fields (read off the page before the board is drawn anew).
    pub roles: [bool; 4],
    pub purpose: i32,
    pub purposes_open: bool,
    pub least: String,
    pub most: String,
    pub memo: String,
    /// The notice being written, and the one being read.
    pub writing: Option<NoticeDraft>,
    pub reading: Option<Notice>,
    /// Who asks to be let into the party now, those who wait behind, whether the box that blocks is ticked,
    /// and whose requests are not shown any more.
    pub asking: Option<Asking>,
    pub waiting: VecDeque<Asking>,
    pub block: bool,
    pub blocked: Vec<u32>,
}

impl Default for Agency {
    /// As the client's board begins: the first page of one.
    fn default() -> Self {
        Self {
            open: false,
            registered: false,
            own: None,
            entries: Vec::new(),
            page: 1,
            pages: 1,
            searching: false,
            loading: false,
            settled: false,
            stopping: 0,
            roles: [false; 4],
            purpose: 0,
            purposes_open: false,
            least: String::new(),
            most: String::new(),
            memo: String::new(),
            writing: None,
            reading: None,
            asking: None,
            waiting: VecDeque::new(),
            block: false,
            blocked: Vec::new(),
        }
    }
}
