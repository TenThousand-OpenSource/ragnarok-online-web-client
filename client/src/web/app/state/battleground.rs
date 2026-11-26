//! What the app keeps of the battlegrounds: the list's window, the queue one waits in, the question whether
//! to enter, and what a battle's map has told.

use std::collections::HashMap;

use crate::press::battleground::Way;

/// The battlegrounds (the client's `CEntryQueueMgr` and what its session keeps of a battle).
#[derive(Default)]
pub(in crate::web) struct Battleground {
    /// The window of the list is open, and its help.
    pub open: bool,
    pub help: bool,
    /// The battleground picked in the list, by its number: the one applied for and waited for too (the
    /// client's `current`). None until the list was first opened, and after a queue was left.
    pub current: Option<u32>,
    /// One waits in a queue (the client's `ready`).
    pub ready: bool,
    /// The window of the queue is open, and what the server last told for it: the battleground's name, how
    /// many are needed, one's place.
    pub standby: bool,
    pub waiting: Option<(String, u32, i32)>,
    /// The battle is ready and the window that asks whether to enter is open: since when, by the page's clock.
    pub entry: Option<u32>,
    /// The names the question came with, given back with the answer (the client's `queueName` and
    /// `characterName`).
    pub queue: String,
    pub lobby: String,
    /// The question before applying: which way.
    pub asks: Option<Way>,
    /// The player's own team, as the server last named it (the client's `playerTeam`, which it never forgets).
    pub team: u8,
    /// The score of the two teams on this map; none until the server told it (the client's -1 and -1).
    pub score: Option<(i32, i32)>,
    /// Where the others of the team are on this map, by account.
    pub places: HashMap<u32, (u16, u16)>,
}
