//! Getting somewhere (the client's `CNavigationManager::StartNavigation`, `Navigate`, `RequestRoute`,
//! `RouteToSavedTarget`, `CollectStartLinks`, `RouteFromPlayer`, `CheckArrival`, `ResumeRoute` and
//! `ShowRouteResult`): each returns the client's number for how it ended, which `texts::result_line` puts
//! into words.

use super::super::{Ground, Here, route};
use super::*;

/// Whether a copy of a map and a map are the same map (`IsSameInstanceMap`).
fn same_instance(here: &str, name: &str) -> bool {
    crate::maps::original(here) == crate::maps::original(name)
}

impl Guide {
    /// Says how an attempt ended (`ShowRouteResult` with mode 1); with a way out to go to on this map (5),
    /// also what to do there.
    pub fn say(&mut self, atlas: &Atlas, result: i32) {
        self.said.push(Line::Result(result));
        let Some(link) = self.node.and_then(|node| self.nodes.get(node)).map(|link| &atlas.links[*link]).filter(|_| result == 5) else { return };
        match link.kind {
            201..=204 => {
                self.said.push(Line::TalkTo(link.name.clone()));
                let to = &atlas.maps[link.to];
                self.said.push(Line::MoveTo(format!(" {}[ {} ] ", to.title, to.name)));
            }
            205 => {
                self.said.push(Line::TalkTo(link.name.clone()));
                self.said.push(Line::Airship);
            }
            _ => self.said.push(Line::Portal),
        }
    }

    /// A navigation asked for from outside the window (`StartNavigation`: a link in a text, `/navi`, the
    /// server): to a place of a map (kind 0, with its cell), a map (1), the NPC of a map on that cell (2) or
    /// its monster of that job (3: `id`). `mode` says which ways out may be taken (1 those for zeny, 2
    /// scrolls, 4 the airship) and `hide` keeps the window away. `there` is the ground of the map sought, for
    /// a place on another map. False when it came to nothing, which the chat is told.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        atlas: &Atlas,
        (name, kind, mode, hide): (&str, i32, u8, bool),
        (x, y, id): (i32, i32, i32),
        here: Here,
        ground: &dyn Ground,
        there: Option<&dyn Ground>,
    ) -> bool {
        if !(0..=4).contains(&kind) || name.is_empty() {
            return false;
        }
        if here.map.contains('@') && !same_instance(here.map, name) {
            self.say(atlas, -90);
            return false;
        }
        let Some(map) = atlas.find(name) else { return false };
        let mut text = atlas.maps[map].name.clone();
        let what = match kind {
            1 => What::Map(map),
            0 => What::Place(map),
            2 | 3 => {
                let found = if kind == 2 { atlas.npc_at(map, x, y).map(What::Npc) } else { atlas.mob_of(map, id).map(What::Mob) };
                let Some(what) = found else {
                    self.say(atlas, -94);
                    return false;
                };
                text = format!("{text} {}", atlas.name_of(what));
                what
            }
            _ => return false,
        };
        (self.current, self.saved) = (Some(Target::of(what)), Some(Target::of(what)));
        (self.sought, self.hidden) = (text, hide);
        self.allowed = Allowed { zeny: mode & 1 != 0, scrolls: mode & 2 != 0, airship: mode & 4 != 0, side: self.allowed.side };
        if self.search(atlas) {
            (self.group, self.member) = (Some(0), Some(0));
        }
        let result = self.navigate(atlas, (x, y), here, ground, there);
        self.say(atlas, result);
        if result < 0 {
            return false;
        }
        self.display = Display::Way;
        if self.window {
            self.hidden = false;
        }
        if !self.hidden {
            self.window = true;
        }
        true
    }

    /// The window's button that finds the way to the target looked at (`NAVI_BUTTON_FIND`).
    pub fn find(&mut self, atlas: &Atlas, here: Here, ground: &dyn Ground, there: Option<&dyn Ground>) {
        let mut result = self.save_current();
        if result == 0 {
            let cell = self.saved.as_ref().map_or((-1, -1), |saved| (saved.x, saved.y));
            result = self.navigate(atlas, cell, here, ground, there);
            if result >= 0 {
                self.display = Display::Way;
                self.look_at_node(self.node);
            }
        }
        self.say(atlas, result);
    }

    /// The map whose ground the way to the target needs and `there` must be: that of a place on a map other
    /// than the one the player is on.
    pub fn ground_needed<'atlas>(&self, atlas: &'atlas Atlas, target: Option<&Target>, here: Here) -> Option<&'atlas str> {
        match target?.what {
            What::Place(map) if atlas.find(here.map) != Some(map) => Some(atlas.maps[map].name.as_str()),
            _ => None,
        }
    }

    /// Finds the way to the target gone to (`Navigate`): 11 when one stands by it already; over the map one
    /// is on where it is here and can be walked to; else across the maps.
    pub fn navigate(&mut self, atlas: &Atlas, (x, y): (i32, i32), here: Here, ground: &dyn Ground, there: Option<&dyn Ground>) -> i32 {
        let near = match self.saved.as_ref().map(|saved| saved.what) {
            Some(What::Npc(npc)) => Some((atlas.npcs[npc].map, (atlas.npcs[npc].x, atlas.npcs[npc].y))),
            Some(What::Place(map)) => Some((map, (x, y))),
            _ => None,
        };
        if let Some((map, (to_x, to_y))) = near {
            let by = (to_x - here.cell.0).abs() <= 1 && (to_y - here.cell.1).abs() <= 1;
            if atlas.find(here.map) == Some(map) && by {
                return 11;
            }
        }
        if self.request(atlas, (x, y), here, ground) == -99 {
            // (The ground of the map sought is that of this one when the place is on it and cannot be
            // walked to from here.)
            let on_this = self.saved.as_ref().is_some_and(|saved| atlas.find(here.map) == Some(atlas.map_of(saved.what)));
            if !self.route_to_saved(atlas, (x, y), if on_this { Some(ground) } else { there }) {
                return -97;
            }
            let Some(start) = self.start_links(atlas, here, ground) else { return -98 };
            let direct = !matches!(self.saved.as_ref().map(|saved| saved.what), Some(What::Npc(_) | What::Place(_)));
            let to = self.goal.map_or(usize::MAX, |(map, _)| map);
            match route::find(atlas, &start, &self.end, to, direct, self.allowed) {
                Some((cost, nodes)) => (self.cost, self.nodes) = (cost, nodes),
                None => {
                    self.cost = 0;
                    return -96;
                }
            }
        }
        self.from_player(atlas, here, ground)
    }

    /// The way over the map one is on, kept when there is one (`BuildRoute`).
    fn build(&mut self, from: (i32, i32), to: (i32, i32), ground: &dyn Ground) -> bool {
        self.way = ground.way(from, to).unwrap_or_default();
        !self.way.is_empty()
    }

    /// The way to a target on the map one is on (`RequestRoute`): 9 when it is found; 2 and 3 for a monster
    /// and a map that are here; -99 when the way has to be looked for across the maps.
    fn request(&mut self, atlas: &Atlas, (x, y): (i32, i32), here: Here, ground: &dyn Ground) -> i32 {
        self.node = None;
        let Some(on) = atlas.find(here.map) else { return -93 };
        let (map, cell) = match self.saved.as_ref().map(|saved| saved.what) {
            Some(What::Mob(mob)) => return if atlas.mobs[mob].map == on { 2 } else { -99 },
            Some(What::Map(map)) => return if map == on { 3 } else { -99 },
            Some(What::Place(_)) if x < 0 || y < 0 => return -97,
            Some(What::Place(map)) => (map, (x, y)),
            Some(What::Npc(npc)) => (atlas.npcs[npc].map, (atlas.npcs[npc].x, atlas.npcs[npc].y)),
            Some(What::Node(_)) | None => return -99,
        };
        if map != on || !self.build(here.cell, cell, ground) {
            return -99;
        }
        self.goal = Some((map, cell));
        (self.shown, self.arriving, self.at_goal) = (true, true, true);
        self.mode = Mode::Going;
        9
    }

    /// Where the route across the maps is to end (`RouteToSavedTarget`): the map sought, and the ways into it
    /// that will do. For an NPC those its table names; for a map or a monster every way in; for a place every
    /// way in from which the place can be walked to, which asks for the ground of that map.
    fn route_to_saved(&mut self, atlas: &Atlas, (x, y): (i32, i32), there: Option<&dyn Ground>) -> bool {
        self.end.clear();
        let Some(saved) = &self.saved else { return false };
        let (map, cell) = match saved.what {
            What::Npc(npc) => {
                self.end = atlas.npcs[npc].ways.clone();
                (atlas.npcs[npc].map, (atlas.npcs[npc].x, atlas.npcs[npc].y))
            }
            What::Mob(_) | What::Map(_) => {
                let map = atlas.map_of(saved.what);
                self.end = atlas.ways_into(map).into_iter().map(|link| (link, 0)).collect();
                (map, (-1, -1))
            }
            What::Place(map) => {
                let Some(there) = there.filter(|there| there.walkable(x, y)) else { return false };
                for link in atlas.ways_into(map) {
                    let arrives = (atlas.links[link].to_x, atlas.links[link].to_y);
                    if arrives == (x, y) {
                        self.end.push((link, 1));
                    } else if let Some(way) = there.way(arrives, (x, y)) {
                        self.end.push((link, way.len() as i32));
                    }
                }
                (map, (x, y))
            }
            What::Node(_) => return false,
        };
        if self.end.is_empty() {
            return false;
        }
        self.goal = Some((map, cell));
        true
    }

    /// The ways out of the map one is on that can be walked to, each with how far it is (`CollectStartLinks`).
    fn start_links(&mut self, atlas: &Atlas, here: Here, ground: &dyn Ground) -> Option<Vec<(usize, i32)>> {
        let Some(on) = atlas.find(here.map) else {
            self.said.push(Line::Unsupported);
            return None;
        };
        let mut start = Vec::new();
        for link in &atlas.maps[on].links {
            let stands = (atlas.links[*link].x, atlas.links[*link].y);
            if stands == here.cell {
                start.push((*link, 1));
            } else if let Some(way) = ground.way(here.cell, stands) {
                start.push((*link, way.len() as i32));
            }
        }
        (!start.is_empty()).then_some(start)
    }

    /// What is to be done on the map one is on now (`RouteFromPlayer`), as the navigation begins and on every
    /// map entered while it lasts: on the map sought, the way to the place or the NPC (9, 7), or the end of the
    /// navigation to a map or a monster (6, 8); on a map of the route, the way to its way out (5).
    pub fn from_player(&mut self, atlas: &Atlas, here: Here, ground: &dyn Ground) -> i32 {
        (self.shown, self.arriving, self.at_goal) = (false, false, false);
        let Some(on) = atlas.find(here.map) else { return -93 };
        let mut result = -96;
        if let Some((_, cell)) = self.goal.filter(|(map, _)| *map == on) {
            let kind = self.saved.as_ref().map_or(-1, |saved| saved.what.kind());
            if kind != 2 && kind != 0 {
                self.cancel();
                return match kind {
                    3 => 8,
                    1 => 6,
                    _ => -96,
                };
            }
            if here.cell == cell {
                self.cancel();
                return 11;
            }
            if self.build(here.cell, cell, ground) {
                self.nodes.clear();
                self.cost = (here.cell.0 - cell.0).abs() + (here.cell.1 - cell.1).abs();
                (self.node, self.current) = (None, self.saved.clone());
                (self.shown, self.arriving, self.at_goal) = (true, true, true);
                self.mode = Mode::Going;
                return if kind == 0 { 9 } else { 7 };
            }
        }
        if self.nodes.is_empty() {
            return result;
        }
        let Some(node) = self.node_here(atlas, on, here.cell, ground) else { return result };
        self.node = Some(node);
        self.look_at_node(Some(node));
        self.shown = true;
        result = 5;
        self.mode = Mode::Going;
        result
    }

    /// The way out of the route to go to on this map, with the way to it kept: the one after the way out one
    /// has just come through, else the route's first if it is here, else any of this map that can be walked to.
    fn node_here(&mut self, atlas: &Atlas, on: usize, cell: (i32, i32), ground: &dyn Ground) -> Option<usize> {
        let cells: Vec<(i32, i32)> = self.nodes.iter().map(|link| (atlas.links[*link].x, atlas.links[*link].y)).collect();
        let stands = |node: usize| cells[node];
        let mut others = Vec::new();
        for node in (1..self.nodes.len()).rev() {
            if atlas.links[self.nodes[node]].map != on {
                continue;
            }
            let before = &atlas.links[self.nodes[node - 1]];
            if (before.to_x, before.to_y) != cell {
                others.push(node);
            } else if let Some(way) = ground.way(cell, stands(node)) {
                self.way = way;
                return Some(node);
            }
        }
        if atlas.links[self.nodes[0]].map == on {
            let first = stands(0);
            self.way = if cell == first { vec![cell] } else { ground.way(cell, first).unwrap_or_default() };
            if !self.way.is_empty() {
                return Some(0);
            }
        }
        for node in others {
            let to = stands(node);
            if self.build(cell, to, ground) {
                return Some(node);
            }
        }
        None
    }

    /// The character was sent walking to a cell (`CheckArrival`, as the server answers a walk): within five
    /// cells of where the navigation ends, it is over.
    pub fn arrival(&mut self, to: (i32, i32)) -> bool {
        let near = |(_, cell): (usize, (i32, i32))| (cell.0 - to.0).abs() <= 5 && (cell.1 - to.1).abs() <= 5;
        if self.mode == Mode::Going && self.arriving && self.goal.is_some_and(near) {
            self.cancel();
            return true;
        }
        false
    }

    /// With none of the marks of the way within ten cells of the character, the way is found anew from where
    /// it stands to the same end (`DrawRoute`). True when the way changed.
    pub fn strayed(&mut self, cell: (i32, i32), ground: &dyn Ground) -> bool {
        let Some(end) = self.way.last().copied().filter(|_| self.shown) else { return false };
        let by = |at: &(i32, i32)| (at.0 - cell.0).abs() < 10 && (at.1 - cell.1).abs() < 10;
        if by(&end) || self.way[..self.way.len() - 1].iter().step_by(2).any(by) {
            return false;
        }
        self.build(cell, end, ground);
        true
    }

    /// A map was entered (`ResumeRoute`): on the way, what is to be done here is found and told.
    pub fn resume(&mut self, atlas: &Atlas, here: Here, ground: &dyn Ground) {
        match self.mode {
            Mode::Sought => self.window = true,
            Mode::Going => {
                let result = self.from_player(atlas, here, ground);
                self.say(atlas, result);
                if !self.hidden {
                    self.window = true;
                }
            }
            Mode::Idle => {}
        }
    }
}
