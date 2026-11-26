//! What the app keeps of the replays: the recorder and what it records, the recordings kept in the browser,
//! and the one being played.

use std::rc::Rc;

use crate::replay::{File, Header, Options};

/// A recording under way (the client's `Replay` in its state 2).
pub(in crate::web) struct Recording {
    /// When it began, by the page's clock.
    pub started: u32,
    /// The name it will be kept under.
    pub name: String,
    pub header: Header,
    pub who: crate::replay::Who,
    /// The packets that came before it began, and those recorded, each with the milliseconds since it began.
    pub before: Vec<Vec<u8>>,
    pub packets: Vec<(u32, Vec<u8>)>,
}

/// A recording being played (the client's `Replay` in its states 6 and 7).
pub(in crate::web) struct Playing {
    pub name: String,
    pub file: Rc<File>,
    /// How many of its packets were handed to the game.
    pub next: usize,
    /// How far into it the player is, in milliseconds, and when that was last counted, by the page's clock.
    pub elapsed: f64,
    pub counted: f64,
    pub speed: f32,
    pub held: bool,
    /// It has run to its end (the client's state 6).
    pub over: bool,
}

/// The replays.
pub(in crate::web) struct Replay {
    /// The recorder is open, and in which of its four forms: 0 ready, 1 recording, 2 its options, 3 asking for
    /// a name (the client's `mode`).
    pub open: bool,
    pub mode: u8,
    /// What is recorded; whether a name is asked for (the client's `pathEnabled`) and whether a name there is
    /// already is refused (`rejectExistingName`).
    pub options: Options,
    pub name_asked: bool,
    pub checked: bool,
    /// The name typed, kept while the recorder is drawn anew.
    pub name: String,
    /// Every packet of the map server since the game was entered, and how many bytes they are: what a
    /// recording begins from. Past `BEFORE_MOST` no more is kept.
    pub before: Vec<Vec<u8>>,
    pub before_bytes: usize,
    pub recording: Option<Recording>,
    /// The recordings the browser keeps, each its name and its bytes: read when the list is first opened.
    pub files: Vec<(String, Rc<Vec<u8>>)>,
    pub files_read: bool,
    /// The list of them is open (on the login screen), and the one picked in it.
    pub list: bool,
    pub picked: Option<usize>,
    pub playing: Option<Playing>,
}

/// The most that is kept of what came before a recording.
pub(in crate::web) const BEFORE_MOST: usize = 32 << 20;

impl Default for Replay {
    /// As the client begins: every option set, a name asked for, a name there is already refused.
    fn default() -> Self {
        Self {
            open: false,
            mode: 0,
            options: [1; 30],
            name_asked: true,
            checked: true,
            name: String::new(),
            before: Vec::new(),
            before_bytes: 0,
            recording: None,
            files: Vec::new(),
            files_read: false,
            list: false,
            picked: None,
            playing: None,
        }
    }
}
