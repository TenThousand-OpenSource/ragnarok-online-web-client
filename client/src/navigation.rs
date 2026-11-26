//! The navigation as rules (the client's `CNavigationManager`, `CNaviData` and the window `UINavigationV4Wnd`,
//! rodecomp Source/Contents/Navigation and Source/UI/Window/UIWindow.cpp): what the client knows of the maps
//! it guides across (`atlas`), what a search finds and how it is listed (`search`), the way from map to map
//! through their ways out (`route`), what is sought and where the guide stands now (`guide`), and what it
//! writes (`texts`).
//!
//! The tables are the client's own (`navi_*.lub`, given by the gateway's `tables/navigation.rs`); nothing of
//! them is asked of the server, and the way is found here, as the client finds it.

pub mod atlas;
pub mod guide;
pub mod route;
pub mod search;
pub mod texts;

#[cfg(test)]
mod tests;

pub use atlas::Atlas;
pub use guide::Guide;

/// What the navigation is pointed at (the client's `NavigationTarget::object` with its `type`): a place on a
/// map (0), a map (1), an NPC (2), a kind of monster of a map (3), or a way out on the route found (6). Each
/// by its place in the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum What {
    Place(usize),
    Map(usize),
    Npc(usize),
    Mob(usize),
    Node(usize),
}

impl What {
    /// The client's number of the kind.
    pub fn kind(self) -> i32 {
        match self {
            What::Place(_) => 0,
            What::Map(_) => 1,
            What::Npc(_) => 2,
            What::Mob(_) => 3,
            What::Node(_) => 6,
        }
    }
}

/// A target (`NavigationTarget`): what it is, the two numbers that go with it (a place's cell, an NPC's cell, a
/// monster's level and what it is; -1000 for a monster listed under its map, -1 for nothing), and the name it
/// is ordered and found by.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub what: What,
    pub x: i32,
    pub y: i32,
    pub name: String,
}

impl Target {
    /// A target with no numbers and no name (`NavigationTarget(type, object)`).
    pub fn of(what: What) -> Self {
        Self { what, x: -1, y: -1, name: String::new() }
    }

    /// A monster listed under its map, not by its own name (`CNaviInfo::SearchMap`: both numbers -1000).
    pub fn under_its_map(&self) -> bool {
        self.x == -1000 && self.y == -1000
    }
}

/// The ground of a map as the navigation asks it (the client's `CPathFinder` with `useNodePool`, which looks
/// at as many cells as the map has): the way from one cell to another, both ends in it, or none; and whether a
/// cell can be walked on.
pub trait Ground {
    fn way(&self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>>;
    fn walkable(&self, x: i32, y: i32) -> bool;
}

impl Ground for crate::map::Gat {
    fn way(&self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let path = self.whole_path(from, to);
        (!path.is_empty()).then(|| std::iter::once(from).chain(path).collect())
    }

    fn walkable(&self, x: i32, y: i32) -> bool {
        crate::map::Gat::walkable(self, x, y)
    }
}

/// Where the player's character is: the map (as the server names it, without its ending) and the cell.
#[derive(Debug, Clone, Copy)]
pub struct Here<'map> {
    pub map: &'map str,
    pub cell: (i32, i32),
}
