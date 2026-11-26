//! What the navigation knows of the world (the client's `CNaviData` as `CNavigationManager::LoadScript` fills
//! it): the maps, on each its NPCs, its kinds of monsters and its ways out, and what walking from one way out
//! to the next, or to an NPC, costs.

use std::collections::HashMap;

use super::What;

/// A map (`CNaviInfo`): its name, what the client calls it, its kind (5001 a field or a town with a picture
/// of its own, 5005 the same, 5003 indoors…), how many cells wide and tall it is, and who and what is on it.
#[derive(Debug, Default, Clone)]
pub struct Map {
    pub name: String,
    pub title: String,
    pub kind: i32,
    pub width: i32,
    pub height: i32,
    pub npcs: Vec<usize>,
    pub mobs: Vec<usize>,
    pub links: Vec<usize>,
}

impl Map {
    /// Whether the search lists the map (`CNaviInfo::IsSearchable`): not one indoors, `gl_in01` excepted.
    pub fn searchable(&self) -> bool {
        self.name == "gl_in01" || !self.name.contains("_in")
    }
}

/// An NPC or a kind of monster of a map (`CNavi_Object`). Of an NPC: its job, its name, its name in the
/// script, its cell, and the ways into its map with what walking from each to it costs. Of a monster: its job
/// in the lower half of `index` and how many there are in the upper, its name, its name in the database, its
/// level as `x` and what it is as `y` (race, size and element, a byte each).
#[derive(Debug, Default, Clone)]
pub struct Spot {
    pub id: i32,
    pub kind: i32,
    pub index: i32,
    pub name: String,
    pub title: String,
    pub map: usize,
    pub x: i32,
    pub y: i32,
    pub ways: Vec<(usize, i32)>,
    /// How one drawn as a player looks (`navi_picknpc`): job, sex, the clothes' colour, hair, its colour, three
    /// things on the head and a robe.
    pub look: Option<[i32; 9]>,
}

impl Spot {
    /// The job (`CNavi_Object::GetIndex`): of a monster (kinds 300 and 301) the lower half of the number.
    pub fn job(&self) -> i32 {
        if self.kind == 300 || self.kind == 301 { self.index & 0xffff } else { self.index }
    }

    /// How many of a monster a map has (`GetFlag`: the upper half of the number).
    pub fn many(&self) -> i32 {
        (self.index >> 16) & 0xffff
    }
}

/// A way out of a map (`CNavi_Link`): a portal (kind 200) or an NPC that takes one elsewhere (201; 204 for
/// zeny, 205 the airship; 202 and 203 for one side only), where it stands, where it leads, and what walking
/// costs from it to the other ways out of its own map (`near`) and from where it leads to the ways out there
/// (`onward`).
#[derive(Debug, Default, Clone)]
pub struct Link {
    pub id: i32,
    pub kind: i32,
    pub index: i32,
    pub name: String,
    pub title: String,
    pub map: usize,
    pub x: i32,
    pub y: i32,
    pub to: usize,
    pub to_x: i32,
    pub to_y: i32,
    pub near: Vec<(usize, i32)>,
    pub onward: Vec<(usize, i32)>,
}

/// A row of the tables of NPCs and of monsters: the map, then the fields of a `Spot`.
pub type SpotRow = (String, i32, i32, i32, String, String, i32, i32);
/// A row of the table of ways out: the map, the fields of a `Link`, the map it leads to and the cell there.
pub type LinkRow = (String, i32, i32, i32, String, String, i32, i32, String, i32, i32);
/// What walking from a way out, or to an NPC, costs: the map, its number, and the passes (a word, a number
/// and the cost: `"P"` and `"E"` with a way out of the same map and of the map led to; for an NPC the map of
/// a way in and its number).
pub type WayRow = (String, i32, Vec<(String, i32, i32)>);

/// The tables as the gateway gives them (`tables/navigation.rs`).
#[derive(Debug, Default, Clone)]
pub struct Rows {
    pub maps: Vec<(String, String, i32, i32, i32)>,
    pub npcs: Vec<SpotRow>,
    pub mobs: Vec<SpotRow>,
    pub links: Vec<LinkRow>,
    pub ways: Vec<WayRow>,
    pub npc_ways: Vec<WayRow>,
    pub picks: Vec<(String, i32, i32, [i32; 9])>,
}

/// Everything the navigation knows.
#[derive(Debug, Default)]
pub struct Atlas {
    pub maps: Vec<Map>,
    pub npcs: Vec<Spot>,
    pub mobs: Vec<Spot>,
    pub links: Vec<Link>,
    by_name: HashMap<String, usize>,
}

impl Atlas {
    /// Puts the tables together as `LoadScript` does: a row whose map is not among the maps is passed by, a way
    /// out is found by its number among those of its map, and a pass that names none is passed by. (The
    /// client's table of scrolls, `Navi_Scroll`, holds nothing in these files and is not read.)
    pub fn new(rows: Rows) -> Self {
        let mut atlas = Self::default();
        for (name, title, kind, width, height) in rows.maps {
            atlas.by_name.entry(name.clone()).or_insert(atlas.maps.len());
            atlas.maps.push(Map { name, title, kind, width, height, ..Map::default() });
        }
        for (monsters, list) in [(false, rows.npcs), (true, rows.mobs)] {
            for (map, id, kind, index, name, title, x, y) in list {
                let Some(map) = atlas.find(&map) else { continue };
                let spot = Spot { id, kind, index, name, title, map, x, y, ..Spot::default() };
                if monsters {
                    atlas.maps[map].mobs.push(atlas.mobs.len());
                    atlas.mobs.push(spot);
                } else {
                    atlas.maps[map].npcs.push(atlas.npcs.len());
                    atlas.npcs.push(spot);
                }
            }
        }
        for (map, id, kind, index, name, title, x, y, to, to_x, to_y) in rows.links {
            let (Some(map), Some(to)) = (atlas.find(&map), atlas.find(&to)) else { continue };
            atlas.maps[map].links.push(atlas.links.len());
            atlas.links.push(Link { id, kind, index, name, title, map, x, y, to, to_x, to_y, ..Link::default() });
        }
        for (map, x, y, look) in rows.picks {
            if let Some(npc) = atlas.find(&map).and_then(|map| atlas.npc_at(map, x, y)) {
                atlas.npcs[npc].look = Some(look);
            }
        }
        atlas.cost(rows.ways, rows.npc_ways);
        atlas
    }

    /// Fills in what walking costs (`queryNavi_Distance_…`, `queryNavi_NpcDistance_…`).
    fn cost(&mut self, ways: Vec<WayRow>, npc_ways: Vec<WayRow>) {
        for (map, id, passes) in ways {
            let Some(link) = self.find(&map).and_then(|map| self.link_of(map, id)) else { continue };
            for (kind, target, cost) in passes {
                match kind.as_str() {
                    "P" => {
                        if let Some(target) = self.link_of(self.links[link].map, target) {
                            self.links[link].near.push((target, cost));
                        }
                    }
                    "E" => {
                        if let Some(target) = self.link_of(self.links[link].to, target) {
                            self.links[link].onward.push((target, cost));
                        }
                    }
                    _ => {}
                }
            }
        }
        for (map, id, passes) in npc_ways {
            let Some(map) = self.find(&map) else { continue };
            let Some(npc) = self.maps[map].npcs.iter().copied().find(|npc| self.npcs[*npc].id == id) else { continue };
            for (from, target, cost) in passes {
                if let Some(target) = self.find(&from).and_then(|from| self.link_of(from, target)) {
                    self.npcs[npc].ways.push((target, cost));
                }
            }
        }
    }

    /// The map of that name (`CNavigationManager::FindObject`), or the one a copy of a map is a copy of
    /// (`ResourceMgr::ResolveMapName`: `crate::maps::original`).
    pub fn find(&self, name: &str) -> Option<usize> {
        let name = crate::maps::bare(name);
        self.by_name.get(name).or_else(|| self.by_name.get(crate::maps::original(name).as_ref())).copied()
    }

    /// The way out of a map that has that number (`CNaviInfo::FindLink` by its number).
    pub fn link_of(&self, map: usize, id: i32) -> Option<usize> {
        self.maps[map].links.iter().copied().find(|link| self.links[*link].id == id)
    }

    /// The NPC of a map that stands on that cell (`CNaviInfo::FindNpcAt`).
    pub fn npc_at(&self, map: usize, x: i32, y: i32) -> Option<usize> {
        self.maps[map].npcs.iter().copied().find(|npc| (self.npcs[*npc].x, self.npcs[*npc].y) == (x, y))
    }

    /// The kind of monster of a map that has that job (`CNaviInfo::FindMobByIndex`).
    pub fn mob_of(&self, map: usize, job: i32) -> Option<usize> {
        self.maps[map].mobs.iter().copied().find(|mob| self.mobs[*mob].job() == job)
    }

    /// The first kind of monster, of all the maps, called so in the database (`CNaviData::FindMobByTitle`).
    pub fn mob_titled(&self, title: &str) -> Option<usize> {
        self.maps.iter().flat_map(|map| map.mobs.iter().copied()).find(|mob| self.mobs[*mob].title == title)
    }

    /// Every way out, of any map, that leads to that one (`CNaviData::CollectLinksTo`): in the order of the
    /// maps and of each map's ways out.
    pub fn ways_into(&self, map: usize) -> Vec<usize> {
        self.maps.iter().flat_map(|from| from.links.iter().copied()).filter(|link| self.links[*link].to == map).collect()
    }

    /// The map a target is on.
    pub fn map_of(&self, what: What) -> usize {
        match what {
            What::Place(map) | What::Map(map) => map,
            What::Npc(npc) => self.npcs[npc].map,
            What::Mob(mob) => self.mobs[mob].map,
            What::Node(link) => self.links[link].map,
        }
    }
}
