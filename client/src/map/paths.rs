//! The paths walked on a map's cells, and the direction of a step. The server's search follows rAthena's
//! `path.cpp`, as the server decides where a character really goes; the client's follows `CPathFinder`.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::Gat;

/// The longest path, in cells, the server walks in one go (rAthena's `MAX_WALKPATH`).
pub const MAX_WALK_PATH: usize = 32;
/// The longest path, in cells, a player may ask the server to walk in one go (rAthena's `max_walk_path`, 17 as
/// on the official servers): a longer one is refused without a word.
pub const MAX_ASKED_PATH: usize = 17;
/// How many cells the Windows client's search for a way looks at before it gives up (the 149 nodes of
/// `CPathFinder::FindOrCreateNode`, rodecomp Source/Contents/Movement/TerrainPathSearch.cpp).
pub const CLIENT_PATH_NODES: usize = 149;

impl Gat {
    /// The path from one cell to another, the start left out and the goal last; empty when there is none within
    /// the server's reach. Steps are to the eight neighbours; a diagonal needs both cells beside it free.
    pub fn path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        let longest = MAX_WALK_PATH;
        if from == to || !self.walkable(to.0, to.1) || !self.walkable(from.0, from.1) {
            return Vec::new();
        }
        const STRAIGHT: u32 = 10;
        const DIAGONAL: u32 = 14;
        let index = |(x, y): (i32, i32)| y as usize * self.width + x as usize;
        // The server's estimate of what is left: straight steps along both axes.
        let estimate = |(x, y): (i32, i32)| STRAIGHT * ((x - to.0).unsigned_abs() + (y - to.1).unsigned_abs());

        let mut cost = vec![u32::MAX; self.width * self.height];
        let mut came_from = vec![usize::MAX; self.width * self.height];
        let mut open = BinaryHeap::new();
        cost[index(from)] = 0;
        // Ordered by cost and estimate, then by the order found, so equal paths come out the same every time.
        let mut found = 0u32;
        open.push(Reverse((estimate(from), found, from)));
        while let Some(Reverse((_, _, cell))) = open.pop() {
            if cell == to {
                let mut path = Vec::new();
                let mut at = index(to);
                while at != index(from) {
                    path.push(((at % self.width) as i32, (at / self.width) as i32));
                    at = came_from[at];
                }
                path.reverse();
                return if path.len() <= longest { path } else { Vec::new() };
            }
            let here = cost[index(cell)];
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let next = (cell.0 + dx, cell.1 + dy);
                if !self.walkable(next.0, next.1) {
                    continue;
                }
                let diagonal = dx != 0 && dy != 0;
                if diagonal && (!self.walkable(cell.0 + dx, cell.1) || !self.walkable(cell.0, cell.1 + dy)) {
                    continue;
                }
                let through = here + if diagonal { DIAGONAL } else { STRAIGHT };
                // Nothing farther than the server walks is worth looking at.
                if through > (longest as u32) * DIAGONAL || through >= cost[index(next)] {
                    continue;
                }
                cost[index(next)] = through;
                came_from[index(next)] = index(cell);
                found += 1;
                open.push(Reverse((through + estimate(next), found, next)));
            }
        }
        Vec::new()
    }

    /// The way the Windows client finds from one cell to another before it asks the server for a walk
    /// (`CPathFinder::FindPath`, rodecomp Source/Contents/Movement/TerrainPathSearch.cpp): the start left out
    /// and the goal last, or nothing. Its search looks at no more than `CLIENT_PATH_NODES` cells and gives up
    /// altogether when it needs one more, so it finds a way round what is near and never one across a map.
    /// The neighbours are tried in the client's order, and what is left is estimated in straight steps.
    pub fn client_path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        if from == to || !self.walkable(to.0, to.1) {
            return Vec::new();
        }
        let estimate = |(x, y): (i32, i32)| 10 * ((x - to.0).unsigned_abs() + (y - to.1).unsigned_abs());
        // Of each cell looked at: what the way to it costs, and the cell before it.
        let mut nodes = HashMap::from([(from, (0u32, from))]);
        let mut open = BinaryHeap::from([Reverse((estimate(from), 0u32, 0u32, from))]);
        let mut found = 0u32;
        while let Some(Reverse((_, _, costed, cell))) = open.pop() {
            if cell == to {
                let mut path = Vec::new();
                let mut at = to;
                while at != from {
                    path.push(at);
                    at = nodes[&at].1;
                }
                path.reverse();
                return path;
            }
            // An older, dearer entry of a cell since reached more cheaply.
            if nodes[&cell].0 != costed {
                continue;
            }
            for (dx, dy) in [(1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1)] {
                let next = (cell.0 + dx, cell.1 + dy);
                let diagonal = dx != 0 && dy != 0;
                if !self.walkable(next.0, next.1) || diagonal && (!self.walkable(cell.0, next.1) || !self.walkable(next.0, cell.1)) {
                    continue;
                }
                let through = costed + if diagonal { 14 } else { 10 };
                match nodes.get(&next) {
                    Some((cost, _)) if *cost <= through => continue,
                    None if nodes.len() >= CLIENT_PATH_NODES => return Vec::new(),
                    _ => {}
                }
                nodes.insert(next, (through, cell));
                found += 1;
                open.push(Reverse((through + estimate(next), found, through, next)));
            }
        }
        Vec::new()
    }

    /// Where a press on `to` sends a character standing on `from`, as the Windows client decides it
    /// (`CGameMode`'s press on the ground, rodecomp Source/Client/Mode/GameMode.cpp, and
    /// `GetClosestValidCell`, Source/Contents/Messenger/ClientMessenger.cpp). To the place itself when it is
    /// walkable, nobody stands on it (`taken`: the client's `IsRegisterCell`) and the client's search finds a
    /// way to it. Else the client goes along the straight line from the place back to the character and takes
    /// the first cell on it or beside it that is walkable and not taken: there, if its search finds a way to
    /// that one, and nowhere if not — it does not look further, and it does not walk on by itself afterwards.
    ///
    /// The client asks for the cell however long the way found is, and this server refuses without a word a
    /// way longer than `MAX_ASKED_PATH`: such a press does nothing, there as here. With `far` — this project's
    /// own switch, off unless the player turns it on — the cell that far along the way found is given instead,
    /// from which the character is sent on (`world/pursuit.rs`, the journey).
    pub fn goal_towards(&self, from: (i32, i32), to: (i32, i32), taken: impl Fn((i32, i32)) -> bool, far: bool) -> Option<(i32, i32)> {
        let free = |cell: (i32, i32)| self.walkable(cell.0, cell.1) && !taken(cell);
        let reached = |cell: (i32, i32)| {
            let way = self.client_path(from, cell);
            if far { way.get(way.len().min(MAX_ASKED_PATH).wrapping_sub(1)).copied() } else { way.last().copied() }
        };
        if let Some(cell) = Some(to).filter(|to| free(*to)).and_then(reached) {
            return Some(cell);
        }
        const BESIDE: [(i32, i32); 9] = [(0, 0), (1, 0), (1, 1), (1, -1), (-1, 0), (-1, 1), (-1, -1), (0, 1), (0, -1)];
        let (dx, dy) = (from.0 - to.0, from.1 - to.1);
        // Along the longer side of the line a cell at a time, the other side by whole division as the client.
        let wide = dx.abs() >= dy.abs();
        let (long, short) = if wide { (dx, dy) } else { (dy, dx) };
        for step in 0..long.abs() {
            let along = step * long.signum();
            let across = along * short / long;
            let on_line = if wide { (to.0 + along, to.1 + across) } else { (to.0 + across, to.1 + along) };
            if let Some(cell) = BESIDE.iter().map(|(x, y)| (on_line.0 + x, on_line.1 + y)).find(|cell| free(*cell)) {
                return reached(cell);
            }
        }
        None
    }
}

/// The direction of a step, as the game numbers them: 0 is south, then clockwise (1 south-west, 2 west, … 7
/// south-east).
pub fn direction(dx: i32, dy: i32) -> u8 {
    match (dx.signum(), dy.signum()) {
        (0, -1) => 0,
        (-1, -1) => 1,
        (-1, 0) => 2,
        (-1, 1) => 3,
        (0, 1) => 4,
        (1, 1) => 5,
        (1, 0) => 6,
        (1, -1) => 7,
        _ => 0,
    }
}
