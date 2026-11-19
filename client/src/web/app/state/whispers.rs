//! What the app keeps of the talks with one player each.

/// A talk with one player, in a window of its own (the client's `UIWhisperWnd`).
pub(in crate::web) struct Talk {
    /// Whom it is with (`m_target`), and what the window is called (`m_name`: the name, and for a friend or
    /// one of the guild what they are to the player).
    pub with: String,
    pub title: String,
    /// What was said, oldest first: each line and its colour.
    pub lines: Vec<(String, &'static str)>,
    /// Where the window was opened, on the stage.
    pub at: (i32, i32),
}

/// The talks with one player each, and what the player has set for them.
pub(in crate::web) struct Whispers {
    /// The windows open, each in its place of the list of windows (`hud/windows/whispers.rs`): a place is
    /// empty until a talk takes it.
    pub talks: [Option<Talk>; crate::whispers::WINDOWS],
    /// A whisper opens a window by itself: from one who is no friend, from a friend; and the chime sounds
    /// when it does (the client's `m_autoOpenWhisperWindow`, `m_autoOpenFriendWhisperWindow`,
    /// `m_whisperOpenSound`, each on until switched off in the Friend Setup window). Kept in the browser.
    pub strangers: bool,
    pub friends: bool,
    pub sound: bool,
    /// Whom the last whisper went to and what it said, until the server answers (`lastWhisperName`,
    /// `lastWhisper`).
    pub last: (String, String),
    /// The menu of a talk's window, opened with the right button: where on the stage, and which window.
    pub menu: Option<((i32, i32), usize)>,
}

impl Default for Whispers {
    fn default() -> Self {
        Self { talks: Default::default(), strangers: true, friends: true, sound: true, last: Default::default(), menu: None }
    }
}

impl Whispers {
    /// The place of the talk with that player, if there is one.
    pub fn with(&self, name: &str) -> Option<usize> {
        self.talks.iter().position(|talk| talk.as_ref().is_some_and(|talk| talk.with == name))
    }
}
