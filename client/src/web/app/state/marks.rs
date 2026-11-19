//! What the app keeps of the marks on the minimap that the server tells of.

use std::collections::HashMap;

/// Where the members of the guild and the boss monster are on the map the character is on.
#[derive(Default)]
pub(in crate::web) struct Marks {
    /// The visit of a map these are of (`world::visit`): on another map they mean nothing.
    pub visit: u32,
    /// The cell of each member of the guild on this map, by account (the client's `guildPosList`).
    pub guild: HashMap<u32, (u16, u16)>,
    /// The cell of the boss monster, while the server has told one (the client's `bossMonsterX`, `bossMonsterY`).
    pub boss: Option<(u16, u16)>,
}
