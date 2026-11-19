//! A walk as the client shows it: the way cell by cell with the server's time at which each cell is reached,
//! and where the walker is drawn at a time of the server's clock (`crate::server_time`).
//!
//! From the client's `CPathFinder::BuildMovementRoute` (the times), `CGameActor::FindRoute` (a way already
//! over when it is told is walked from now, quicker), `MovementRouteInterpolation::NextPoint` and the `ST_MOVE`
//! arm of `CGameActor::ProcessState` (the place each frame), and the `AM_STOPMOVE` arm of `CGameActor::SendMsg`
//! (the server names the cell someone stands on). Also the waits the client keeps between its own requests
//! to walk.
//!
//! Places are in cells, whole numbers being cell centres; times are milliseconds of the server's clock.

/// A diagonal step takes this many times a straight one (`BuildMovementRoute`).
const DIAGONAL: f32 = 1.414;

/// The way of a walk: the cell set out from, then every cell walked through, and when each is reached.
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    cells: Vec<(i32, i32)>,
    reached: Vec<f64>,
}

/// Where a walker is drawn after a frame: its place, which way it faces if it moved (0 south, then clockwise,
/// as `crate::map::direction`), and whether the walk is over.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub at: (f32, f32),
    pub facing: Option<u8>,
    pub arrived: bool,
}

impl Route {
    /// The way along `cells` (the cell set out from first) begun at `tick`, a straight step taking `speed`
    /// milliseconds (`BuildMovementRoute`). `from` is the exact place set out from: the first step takes as
    /// long as it is far from there to the second cell. A diagonal step takes 1.414 of a straight one, cut to
    /// whole milliseconds. None for a way of no step.
    pub fn new(tick: f64, from: (f32, f32), cells: &[(i32, i32)], speed: i32) -> Option<Self> {
        if cells.len() < 2 {
            return None;
        }
        let (across, along) = (cells[1].0 as f32 - from.0, cells[1].1 as f32 - from.1);
        let first = ((across * across + along * along).sqrt() * speed as f32).trunc();
        let mut reached = vec![tick, tick + f64::from(first)];
        for step in cells[1..].windows(2) {
            let diagonal = step[0].0 != step[1].0 && step[0].1 != step[1].1;
            let takes = if diagonal { (speed as f32 * DIAGONAL) as i32 } else { speed };
            reached.push(reached[reached.len() - 1] + f64::from(takes));
        }
        Some(Self { cells: cells.to_vec(), reached })
    }

    /// The way as it is walked when told of at `now` (`CGameActor::FindRoute`): begun at `tick`; but one that
    /// would be over by now is begun now instead, each step taking 10/13 of its time.
    pub fn found(tick: f64, now: f64, from: (f32, f32), cells: &[(i32, i32)], speed: i32) -> Option<Self> {
        let route = Self::new(tick, from, cells, speed)?;
        if route.ends() > now { Some(route) } else { Self::new(now, from, cells, speed * 10 / 13) }
    }

    /// When the last cell is reached.
    pub fn ends(&self) -> f64 {
        self.reached[self.reached.len() - 1]
    }

    /// The cell walked to.
    pub fn goal(&self) -> (i32, i32) {
        self.cells[self.cells.len() - 1]
    }

    /// The cell being walked to at a time and when it is reached, and whether the walk is still under way
    /// (`NextPoint`): before the walk begins that is its second cell; once it is over, its last.
    fn next(&self, time: f64) -> (bool, f64, (i32, i32)) {
        if self.reached[0] > time {
            return (true, self.reached[1], self.cells[1]);
        }
        match (1..self.cells.len()).find(|&step| time < self.reached[step]) {
            Some(step) => (true, self.reached[step], self.cells[step]),
            None => (false, self.ends(), self.goal()),
        }
    }

    /// Where a walker drawn at `drawn` at the time `last` is drawn at the time `now` (the `ST_MOVE` arm of
    /// `ProcessState`): it goes straight from where it is drawn towards the cell being walked to, as far of the
    /// way as the time gone by is of the time left until that cell is reached. So a walker drawn elsewhere
    /// than the way has it (the walk was told of late, or begins from another cell) comes to the way without
    /// a leap, and is on time at the next cell. Once the walk is over it stands on its last cell.
    pub fn step(&self, drawn: (f32, f32), last: f64, now: f64) -> Step {
        let (under_way, reached, cell) = self.next(now);
        let cell = (cell.0 as f32, cell.1 as f32);
        let (left, gone) = ((reached - last).max(0.0), (now - last).max(0.0));
        let mut at = drawn;
        if left > 0.0 && gone > 0.0 {
            let part = (gone / left) as f32;
            at = (drawn.0 + (cell.0 - drawn.0) * part, drawn.1 + (cell.1 - drawn.1) * part);
        }
        let facing = (at != drawn).then(|| facing(at.0 - drawn.0, at.1 - drawn.1));
        if under_way || now < reached { Step { at, facing, arrived: false } } else { Step { at: cell, facing, arrived: true } }
    }
}

/// Which way someone moving by (`dx`, `dy`) faces, of the eight ways (0 south, then clockwise): the client
/// takes the angle of the move (`ST_MOVE`) and the nearest of the eight to it (`Get8Dir`).
pub fn facing(dx: f32, dy: f32) -> u8 {
    ((-dx).atan2(-dy).to_degrees().rem_euclid(360.0) / 45.0).round() as u8 % 8
}

/// Where someone drawn at `drawn` is drawn once the server names the cell it stands on (ZC_STOPMOVE, the
/// client's `AM_STOPMOVE`): on the middle of that cell; but one drawn within that cell already stays where it
/// is drawn. The client makes a way from where it is drawn to the cell at 2 milliseconds a cell, writes the
/// cell's middle as its place at once, and shows no step of it and no turn (`isNeverAnimation`): within a frame
/// or two the way is over and it stands on the cell. Within the cell there is no way to make (`FindPath`
/// refuses one from a cell to itself), and it is only made to stand.
pub fn stopped(drawn: (f32, f32), cell: (i32, i32)) -> (f32, f32) {
    if (drawn.0.round() as i32, drawn.1.round() as i32) == cell { drawn } else { (cell.0 as f32, cell.1 as f32) }
}

/// The client's own waits before it asks the server for a walk, in milliseconds. No two requests go within
/// 350 of each other, whatever asks (`AM_REQ_MOVE_PRIMITIVE` of `CPlayer::SendMsg`: one that comes sooner is
/// dropped, not kept). While the left button is held on the ground a walk to where the pointer is is asked
/// for 600 after the last (`CGameMode::ProcessGroundClick`). One who runs after a monster to strike it is sent
/// on no sooner than 450 after it last was (`CPlayer::UpdatePendingAction`).
pub const ASKED_APART: f64 = 350.0;
pub const HELD_APART: f64 = 600.0;
pub const CHASE_APART: f64 = 450.0;

/// Whether a walk may be asked for at `time`, the last having been asked for at `last`.
pub fn may_ask(time: f64, last: f64) -> bool {
    time > last + ASKED_APART
}

#[cfg(test)]
mod tests;
