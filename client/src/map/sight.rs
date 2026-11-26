//! From where something is reached on a map's cells: the line of sight as the server judges it, the reach of a
//! blow or a skill, and the way to the nearest cell from which a target is in reach. The first two follow
//! rAthena (`path_search_long` in `path.cpp`, `battle_check_range` in `battle.cpp`), as the server decides
//! whether a blow or a skill asked for happens. The Windows client has none of this: it measures the distance
//! alone (`CPlayer::UpdatePendingAction`, rodecomp Source/Contents/Actors/Player.cpp) and asks; the way into
//! reach is this project's own, part of its far walk (`docs/DEBTS.md`).

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::{Gat, FAR_PATH_MARGIN, MAX_ASKED_ROUND};

/// How far, in cells counted along the longer side, the server lets anything be aimed at (rAthena's
/// `AREA_SIZE`, the `area_size` of its battle settings: 14).
pub const SIGHT: i32 = 14;

impl Gat {
    /// Whether a shot from one cell arrives at another: rAthena's `path_search_long` with `CELL_CHKWALL`,
    /// step for step. The line is walked from the western of the two cells to the eastern, a cell at a time
    /// along its longer side, and no cell on it between the two may be a wall; the two cells themselves are
    /// not looked at, and a cliff (a cell not walked on that is shot over) stops nothing.
    pub fn sees(&self, from: (i32, i32), to: (i32, i32)) -> bool {
        let ((mut x, mut y), end) = if to.0 < from.0 { (to, from) } else { (from, to) };
        let (dx, dy) = (end.0 - x, end.1 - y);
        let weight = if dx > dy.abs() { dx } else { dy.abs() };
        let (mut wx, mut wy) = (0, 0);
        while (x, y) != end {
            wx += dx;
            wy += dy;
            if wx >= weight {
                wx -= weight;
                x += 1;
            }
            if wy >= weight {
                wy -= weight;
                y += 1;
            } else if wy < 0 {
                wy += weight;
                y -= 1;
            }
            if (x, y) != end && self.wall(x, y) {
                return false;
            }
        }
        true
    }

    /// Whether a player on `from` reaches `to` with a blow or a skill of `range` cells, as the server judges
    /// it (`battle_check_range` for a player; with `OFFICIAL_WALKPATH` the blow's own test of the line in
    /// `unit_attack_timer_sub` says the same). The distance is straight across, a tenth of a cell less, in
    /// whole cells (`distance_client`); a cell beside is reached whatever is between; farther than `SIGHT`
    /// nothing is; else the shot must arrive (`sees`). A blow in a hand reaches by the same rule as an arrow.
    pub fn reaches(&self, from: (i32, i32), to: (i32, i32), range: i32) -> bool {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let across = (f64::from(dx * dx + dy * dy).sqrt() - 0.1).max(0.0) as i32;
        let cells = dx.abs().max(dy.abs());
        across <= range.max(0) && (cells < 2 || cells <= SIGHT && self.sees(from, to))
    }

    /// The way from `from` to the nearest cell, by the steps walked, of which `reached` says the target is in
    /// reach from it and on which nobody stands (`taken`); the start left out, and empty when the start is
    /// such a cell. None when there is no such cell near: only cells no farther than `FAR_PATH_MARGIN` from
    /// the rectangle that `from` and `near` (where the target is) span are looked at, as the far walk does
    /// (`near_path`). Steps and their costs are those of the walk: to the eight neighbours, a diagonal with
    /// both cells beside it free.
    pub fn way_into_reach(
        &self,
        from: (i32, i32),
        near: (i32, i32),
        reached: impl Fn((i32, i32)) -> bool,
        taken: impl Fn((i32, i32)) -> bool,
    ) -> Option<Vec<(i32, i32)>> {
        let (west, east) = (from.0.min(near.0) - FAR_PATH_MARGIN, from.0.max(near.0) + FAR_PATH_MARGIN);
        let (south, north) = (from.1.min(near.1) - FAR_PATH_MARGIN, from.1.max(near.1) + FAR_PATH_MARGIN);
        // Of each cell looked at: what the way to it costs, and the cell before it.
        let mut nodes = HashMap::from([(from, (0u32, from))]);
        let mut open = BinaryHeap::from([Reverse((0u32, 0u32, from))]);
        let mut found = 0u32;
        while let Some(Reverse((costed, _, cell))) = open.pop() {
            // An older, dearer entry of a cell since reached more cheaply.
            if nodes[&cell].0 != costed {
                continue;
            }
            if reached(cell) && (cell == from || !taken(cell)) {
                let mut way = Vec::new();
                let mut at = cell;
                while at != from {
                    way.push(at);
                    at = nodes[&at].1;
                }
                way.reverse();
                return Some(way);
            }
            for (dx, dy) in [(1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1)] {
                let next = (cell.0 + dx, cell.1 + dy);
                let diagonal = dx != 0 && dy != 0;
                if !(west..=east).contains(&next.0) || !(south..=north).contains(&next.1) || !self.walkable(next.0, next.1) {
                    continue;
                }
                if diagonal && (!self.walkable(cell.0, next.1) || !self.walkable(next.0, cell.1)) {
                    continue;
                }
                let through = costed + if diagonal { 14 } else { 10 };
                if nodes.get(&next).is_some_and(|(cost, _)| *cost <= through) {
                    continue;
                }
                nodes.insert(next, (through, cell));
                found += 1;
                open.push(Reverse((through, found, next)));
            }
        }
        None
    }
}

/// The cell of a way to ask the server for: its end, or the cell `MAX_ASKED_ROUND` along it when it is longer
/// (a stretch the server walks whatever stands beside it, from which the character is sent on). None of an
/// empty way.
pub fn stretch(way: &[(i32, i32)]) -> Option<(i32, i32)> {
    way.get(way.len().min(MAX_ASKED_ROUND).wrapping_sub(1)).copied()
}

/// What a character sent after something does about it now: an order to strike, to use a skill or to pick up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ask the server for it.
    Ask,
    /// Ask the server to walk to the cell the character is on, and then, at once, for it.
    Halt,
    /// Walk on: the walk under way ends where it is in reach.
    Wait,
    /// Ask for the walk into reach, which takes the place of any walk under way.
    Go,
}

/// What an order asks for now: `here`, whether its target is in reach from the cell the character is on;
/// `walking`, whether the character walks; `bound`, whether the walk under way ends on a cell from which the
/// target is in reach.
///
/// An order takes the place of whatever the character walked for, at once, as the client's does
/// (`AM_PROCEEDATTACK` and the like of `CPlayer::SendMsg` set the pending action, and
/// `CPlayer::UpdatePendingAction` asks for the blow in the same frame, walking or not). A walk that leads
/// elsewhere is ended first (`Halt`), which the client does not do and this server needs: rAthena keeps a
/// blow or a skill asked of one who walks and is not a cell nearer than the reach needs (`unit_attack`,
/// `unit_skilluse_id2`: `stepaction`), tries it again at each cell, and so only when the walk is over, where
/// the target may be out of reach; and a walk asked for after the blow calls the blow off (`unit_walktoxy`:
/// `unit_stop_attack`), so the walk is always asked for first.
pub fn order(here: bool, walking: bool, bound: bool) -> Order {
    match (here, walking, bound) {
        (true, false, _) | (true, true, true) => Order::Ask,
        (true, true, false) => Order::Halt,
        (false, true, true) => Order::Wait,
        (false, _, _) => Order::Go,
    }
}

#[cfg(test)]
mod tests;
