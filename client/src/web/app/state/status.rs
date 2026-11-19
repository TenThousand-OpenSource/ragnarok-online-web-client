//! What the app keeps of the character itself: its values, and the windows that show them.

use super::*;

/// The character itself: its values, and the windows that show them.
#[derive(Default)]
pub(in crate::web) struct Hero {
    /// The character's values as the map server gives them, by parameter number (`packets::parameter`).
    pub params: HashMap<u16, i64>,
    /// What equipment and the like add to each stat: Str, Agi, Vit, Int, Dex, Luk.
    pub stat_bonus: [i32; 6],
    /// The character lies dead, until it is returned to its save point.
    pub dead: bool,
    pub sitting: bool,
    /// The bits of the state the character is in (the server's `option`): what it rides.
    pub state: u32,
    /// Basic Information is in its small form; the Status window is open.
    pub basic_small: bool,
    pub status_open: bool,
}

impl Hero {
    /// The zeny the character has.
    pub fn zeny(&self) -> i64 {
        self.params.get(&packets::parameter::ZENY).copied().unwrap_or(0)
    }
}
