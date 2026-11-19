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
mod tests {
    use super::*;

    #[test]
    fn no_two_walks_are_asked_for_within_350_milliseconds() {
        assert!(!may_ask(1_350.0, 1_000.0));
        assert!(may_ask(1_351.0, 1_000.0));
    }

    /// East twice, then north-east, at 150 a cell from the time 1000.
    fn route() -> Route {
        Route::new(1_000.0, (10.0, 10.0), &[(10, 10), (11, 10), (12, 10), (13, 11)], 150).unwrap()
    }

    #[test]
    fn a_straight_step_takes_the_speed_and_a_diagonal_one_1414_thousandths_of_it() {
        assert_eq!(route().reached, [1_000.0, 1_150.0, 1_300.0, 1_512.0]);
        assert_eq!((route().ends(), route().goal()), (1_512.0, (13, 11)));
        assert_eq!(Route::new(0.0, (10.0, 10.0), &[(10, 10)], 150), None);
    }

    #[test]
    fn the_first_step_takes_as_long_as_it_is_far_from_where_the_walker_sets_out() {
        // Set out a quarter of a cell towards the next: three quarters of a step left.
        let route = Route::new(0.0, (10.25, 10.0), &[(10, 10), (11, 10), (12, 10)], 200).unwrap();
        assert_eq!(route.reached, [0.0, 150.0, 350.0]);
        // A first step that is diagonal is as long as the diagonal is.
        let route = Route::new(0.0, (10.0, 10.0), &[(10, 10), (11, 11)], 100).unwrap();
        assert_eq!(route.reached, [0.0, 141.0]);
    }

    #[test]
    fn a_way_over_by_now_is_walked_from_now_at_ten_thirteenths() {
        let cells = [(10, 10), (11, 10), (12, 10)];
        // Still under way: as told.
        assert_eq!(Route::found(1_000.0, 1_299.0, (10.0, 10.0), &cells, 150).unwrap().reached, [1_000.0, 1_150.0, 1_300.0]);
        // Over: from now, 150 * 10 / 13 = 115 a cell.
        assert_eq!(Route::found(1_000.0, 1_300.0, (10.0, 10.0), &cells, 150).unwrap().reached, [1_300.0, 1_415.0, 1_530.0]);
    }

    #[test]
    fn a_walker_on_its_way_is_drawn_along_it_by_the_servers_time() {
        let route = route();
        // Frame by frame from the start: half the first step, then into the second.
        let step = route.step((10.0, 10.0), 1_000.0, 1_075.0);
        assert_eq!((step.at, step.facing, step.arrived), ((10.5, 10.0), Some(6), false));
        let step = route.step(step.at, 1_075.0, 1_150.0);
        assert_eq!(step.at, (11.0, 10.0));
        let step = route.step(step.at, 1_150.0, 1_406.0);
        assert!(!step.arrived && step.facing == Some(5));
        // Past its end it stands on the last cell.
        let step = route.step(step.at, 1_406.0, 1_512.0);
        assert_eq!((step.at, step.arrived), ((13.0, 11.0), true));
    }

    #[test]
    fn a_walk_told_of_late_is_caught_up_with_without_a_leap() {
        let route = route();
        // Told of when the walker is already in its second step, and drawn at the start still: it goes
        // towards the end of that step, and is there on time.
        let step = route.step((10.0, 10.0), 1_200.0, 1_250.0);
        assert_eq!((step.at, step.arrived), ((11.0, 10.0), false));
        let step = route.step(step.at, 1_250.0, 1_299.0);
        assert!((step.at.0 - 11.98).abs() < 1e-4 && step.at.1 == 10.0);
    }

    #[test]
    fn a_walk_yet_to_begin_is_begun_slowly() {
        // The clock is 72 behind: a walk told of at once begins that much later. The walker sets out at once
        // and is at the second cell on time.
        let step = route().step((10.0, 10.0), 850.0, 950.0);
        assert!((step.at.0 - (10.0 + 1.0 / 3.0)).abs() < 1e-5 && step.at.1 == 10.0 && !step.arrived);
    }

    #[test]
    fn a_clock_that_went_back_holds_the_walker_for_a_frame() {
        let step = route().step((10.5, 10.0), 1_200.0, 1_100.0);
        assert_eq!((step.at, step.facing, step.arrived), ((10.5, 10.0), None, false));
    }

    #[test]
    fn a_walker_faces_the_nearest_of_the_eight_ways() {
        let ways = [(0.0, -1.0), (-1.0, -1.0), (-1.0, 0.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (1.0, -1.0)];
        for (way, (dx, dy)) in ways.into_iter().enumerate() {
            assert_eq!(usize::from(facing(dx, dy)), way);
            assert_eq!(facing(dx, dy), crate::map::direction(dx as i32, dy as i32));
        }
        // Mostly east, a little north: east.
        assert_eq!(facing(1.0, 0.3), 6);
        assert_eq!(facing(0.1, -1.0), 0);
    }

    #[test]
    fn one_told_to_stand_on_a_cell_is_put_on_it_unless_it_is_drawn_in_it() {
        assert_eq!(stopped((10.6, 12.2), (13, 12)), (13.0, 12.0));
        assert_eq!(stopped((12.6, 12.2), (13, 12)), (12.6, 12.2));
    }
}
