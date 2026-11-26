//! The way from map to map through their ways out (the client's `CNaviData::FindRoute`, rodecomp
//! Source/Contents/Navigation/NavigationGraph.cpp).

use super::Atlas;

/// Which ways out a route may take (the navigation's `allowGates`, `allowWarps` and `routeSide`): those that
/// cost zeny (kind 204, and 202 and 203, each for one side only), and the airship (205). A plain portal (200)
/// and an NPC that asks nothing (201) are always taken.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Allowed {
    pub zeny: bool,
    /// The client's `includeObjects`: its scrolls, of which these files have none.
    pub scrolls: bool,
    pub airship: bool,
    pub side: i32,
}

impl Allowed {
    /// Whether a way out of that kind may be taken.
    fn takes(self, kind: i32) -> bool {
        match kind {
            204 => self.zeny,
            205 => self.airship,
            202 => self.zeny && self.side == 2,
            203 => self.zeny && self.side == 1,
            _ => true,
        }
    }
}

/// The cost the search begins with: a route dearer than this is none (`CNaviData::range`).
const DEAREST: i32 = 50000;
/// How many maps deep the search goes.
const DEEPEST: i32 = 100;

/// One way out the search has reached: the one it came from (by its place among those reached), the way out
/// itself, and what walking to it has cost.
struct Reached {
    from: Option<usize>,
    link: usize,
    cost: i32,
}

/// The route to a map: the ways out to take, in order, and what walking it costs; none when no way leads
/// there. `start` are the ways out of the map one is on with what walking to each costs. With `direct` any
/// way into the map will do (a map or a monster is sought); otherwise only one of `end` — the ways in from
/// which the place sought is reached, each with what walking on from it costs.
///
/// As the client has it, the search goes a map deeper at a time and takes the first way in it comes upon,
/// not the cheapest of all: among routes of equally many maps, the one whose ways out come first.
pub fn find(atlas: &Atlas, start: &[(usize, i32)], end: &[(usize, i32)], to: usize, direct: bool, allowed: Allowed) -> Option<(i32, Vec<usize>)> {
    let mut depths = vec![-1; atlas.links.len()];
    let mut reached: Vec<Reached> = Vec::new();
    for (link, cost) in start {
        if allowed.takes(atlas.links[*link].kind) {
            depths[*link] = 0;
            reached.push(Reached { from: None, link: *link, cost: *cost });
        }
    }
    let (mut first, mut depth) = (0, 1);
    loop {
        let count = reached.len();
        for at in first..count {
            let arrives = atlas.links[reached[at].link].to == to;
            let cost = match (arrives, direct) {
                (false, _) => continue,
                (true, true) => reached[at].cost,
                (true, false) => match end.iter().find(|(link, _)| *link == reached[at].link) {
                    Some((_, onward)) => onward + reached[at].cost,
                    None => continue,
                },
            };
            if cost >= DEAREST {
                // (The client then gives the route of the way out it last held as its best, which at this
                // point is none: no route.)
                return None;
            }
            let mut route = Vec::new();
            let mut step = Some(at);
            while let Some(place) = step {
                route.push(reached[place].link);
                step = reached[place].from;
            }
            route.reverse();
            return Some((cost, route));
        }
        for at in first..count {
            for (next, cost) in &atlas.links[reached[at].link].onward {
                let through = cost + reached[at].cost;
                if depths[*next] == depth {
                    // Reached already at this depth: by the cheaper of the two ways.
                    if let Some(other) = reached[count..].iter_mut().find(|other| other.link == *next) {
                        if other.cost > through {
                            (other.from, other.cost) = (Some(at), through);
                        }
                    }
                } else if depths[*next] == -1 && allowed.takes(atlas.links[*next].kind) {
                    depths[*next] = depth;
                    reached.push(Reached { from: Some(at), link: *next, cost: through });
                }
            }
        }
        first = count;
        depth += 1;
        if first == reached.len() || depth >= DEEPEST {
            return None;
        }
    }
}
