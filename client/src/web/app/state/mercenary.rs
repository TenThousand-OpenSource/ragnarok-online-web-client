//! What the app keeps of a hired mercenary: what the server told of it, its skills, its window and the list
//! of its skills.

use crate::net::items::Skill;
use crate::net::mercenary;

/// The player's mercenary.
#[derive(Default)]
pub(in crate::web) struct Mercenary {
    /// Everything the server told of it: the unit that is it among that. Whether it is out on the map now
    /// (the client's `mercenaryAlive`), and the job it is drawn as (`m_mercenaryClass`), both as the world
    /// last showed.
    pub info: Option<mercenary::Mercenary>,
    pub out: bool,
    pub job: u16,
    pub skills: Vec<Skill>,
    /// Its window is open; the list of its skills is, and which row of it is picked; the window asks
    /// whether to dismiss it.
    pub open: bool,
    pub skills_open: bool,
    pub skill_picked: Option<usize>,
    pub asks: bool,
    /// The menu on the mercenary itself, opened with the right button on it: where on the stage.
    pub menu_at: Option<(i32, i32)>,
    /// The enemy picked for it with Alt and the left button: a second such press on the same one sends it to
    /// fight (the client's `mercenaryEnemyGid` and `attackMercenaryEnemy`).
    pub marked: Option<u32>,
}

impl Mercenary {
    /// The unit on the map that is it, 0 for none.
    pub(in crate::web) fn unit(&self) -> u32 {
        self.info.as_ref().map_or(0, |info| info.unit)
    }

    /// Whether there is a window to show: it is known, and out on the map.
    pub(in crate::web) fn shown(&self) -> bool {
        self.out && self.info.is_some()
    }
}
