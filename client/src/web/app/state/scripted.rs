//! What the app keeps of what an NPC's script shows and plays: the picture over the screen, the marks on the
//! minimap, the bar over the character that is to be answered, the sounds that repeat.

use std::collections::BTreeMap;

use crate::net::scripted::Stand;

/// A mark a script put on the minimap (the client's `miniMapPings`).
pub(in crate::web) struct Mark {
    pub cell: (u32, u32),
    pub colour: [u8; 3],
    /// When it goes by itself, in milliseconds since 1970: a mark of kind 0, shown for a while.
    pub until: Option<f64>,
}

/// A sound a script has play again and again with a wait between (the client's `loopSounds`).
pub(in crate::web) struct Repeated {
    pub name: String,
    /// From whom it sounds (0: from nowhere).
    pub who: u32,
    /// The wait, and when it plays next, in milliseconds.
    pub term: u32,
    pub next: f64,
}

/// What a script shows and plays.
#[derive(Default)]
pub(in crate::web) struct Scripted {
    /// The picture over the screen and where it stands (the client's `UIIllustWnd`).
    pub cutin: Option<(String, Stand)>,
    /// The marks on the minimap, by the number each was given: those of one visit to a map (`world::visit`).
    pub compass: (u32, BTreeMap<u8, Mark>),
    /// When the bar over the player's own character is full, in milliseconds since 1970: the server waits to
    /// be told (the client's `m_playerMessage82Pending`).
    pub progress: Option<f64>,
    pub repeated: Vec<Repeated>,
}
