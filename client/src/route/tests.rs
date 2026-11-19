//! The tests of a walk's route (`route.rs`).

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
