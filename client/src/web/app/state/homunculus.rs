//! What the app keeps of the alchemist's homunculus: what the server told of it, its skills, its window and
//! the list of its skills, what the window asks.

use crate::net::homunculus;
use crate::net::items::Skill;

/// What the homunculus window asks before it acts (the client's message boxes of `UIHomunInfoWnd::SendMsg`).
#[derive(Debug, Clone, PartialEq)]
pub(in crate::web) enum HomunculusAsk {
    /// Whether to feed it.
    Feed,
    /// Whether to give it the name typed.
    Rename(String),
    /// Whether to delete it for good.
    Delete,
}

/// The player's homunculus.
#[derive(Default)]
pub(in crate::web) struct Homunculus {
    /// The unit on the map that is it (the client's `m_homunAid`), 0 for none; whether it is out on the map
    /// now (`homunAlive`), and the monster it is drawn as (`m_homunJob`), both as the world last showed.
    pub unit: u32,
    pub out: bool,
    pub job: u16,
    /// Everything the server told of it.
    pub info: Option<homunculus::Homunculus>,
    pub skills: Vec<Skill>,
    /// Its window is open; the list of its skills is, and which row of it is picked.
    pub open: bool,
    pub skills_open: bool,
    pub skill_picked: Option<usize>,
    pub asks: Option<HomunculusAsk>,
    /// It is fed by itself (the box of its window; the server keeps it and tells it: ZC_CONFIG).
    pub autofeed: bool,
    /// The hunger the chat last told of (the client's `m_homunHungerNotified`).
    pub hungry_told: i32,
    /// The answer to feeding it (whether it ate, and its food), until the chat has told of it: the food's
    /// name may arrive a moment later.
    pub fed: Option<(bool, u32)>,
    /// The menu on the homunculus itself, opened with the right button on it: where on the stage.
    pub menu_at: Option<(i32, i32)>,
    /// The enemy picked for it with Alt and the right button: a second such press on the same one sends it to
    /// fight (the client's `homunEnemyGid` and `attackHomunEnemy`).
    pub marked: Option<u32>,
}

impl Homunculus {
    /// Whether there is a window to show: it is known, and out on the map.
    pub(in crate::web) fn shown(&self) -> bool {
        self.out && self.info.is_some()
    }
}
