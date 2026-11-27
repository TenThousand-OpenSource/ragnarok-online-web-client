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
    /// The packets recorded, each with the milliseconds since it began: at their head, at nought, what the
    /// client puts there (`crate::replay::snapshot::skills`).
    pub packets: Vec<(u32, Vec<u8>)>,
    /// The sections the Windows client keeps its session and its map in, as they were when it began.
    pub client: Vec<(usize, Vec<u8>)>,
    /// Whether the quest window was open when last looked at: its opening and closing are written down.
    pub quests: bool,
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
    /// It begins from its file's sections and its map does not show yet: its clock begins when it does.
    pub waits: bool,
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
    /// The two packets of the map's kind as they last came (ZC_MAPPROPERTY_R2, ZC_NOTIFY_MAPPROPERTY2): a
    /// recording begins with them (the client's `Replay::CachePacket`).
    pub map_told: [Option<Vec<u8>>; 2],
    pub recording: Option<Recording>,
    /// The recordings the browser keeps, each its name and its bytes: read when the list is first opened.
    pub files: Vec<(String, Rc<Vec<u8>>)>,
    pub files_read: bool,
    /// The list of them is open (on the login screen), and the one picked in it.
    pub list: bool,
    pub picked: Option<usize>,
    pub playing: Option<Playing>,
    /// A press the recording being played was made with is being done again: nothing holds it back.
    pub told: bool,
}

impl Replay {
    /// The options the recording being played was made with; none while none plays.
    pub fn held_by(&self) -> Option<Options> {
        let playing = self.playing.as_ref()?;
        let mut options: Options = [1; 30];
        let kept = &playing.file.header.options;
        options[..kept.len().min(30)].copy_from_slice(&kept[..kept.len().min(30)]);
        Some(options)
    }
}

/// How often the player's clock is looked at, in milliseconds.
pub(in crate::web) const REPLAY_TICK: i32 = 50;

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
            map_told: [None, None],
            recording: None,
            files: Vec::new(),
            files_read: false,
            list: false,
            picked: None,
            playing: None,
            told: false,
        }
    }
}
