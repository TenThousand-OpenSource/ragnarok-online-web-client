use super::*;
use crate::vector::length;

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-3)
}

#[test]
fn a_delay_is_counted_down_by_sixteen_a_call() {
    // 200 takes 13 calls, 400 takes 25: the frame after is the first.
    let late = Carried::new(Way::Stays).delayed(200);
    assert_eq!((late.lived(12), late.lived(13), late.lived(14)), (None, Some(0), Some(1)));
    assert_eq!(Carried::new(Way::Stays).delayed(400).lived(25), Some(0));
    assert_eq!(Carried::new(Way::Stays).lived(0), Some(0));
    // Moved on twice a tick, it waits half as long and shows every other frame.
    let twice = Carried::new(Way::Stays).delayed(200).paced(2);
    assert_eq!((twice.lived(5), twice.lived(6), twice.lived(7)), (None, Some(0), Some(2)));
    assert_eq!(Carried::new(Way::Stays).paced(2).lived(0), Some(1));
}

#[test]
fn a_file_is_over_after_its_last_frame_unless_it_loops() {
    let once = Carried::new(Way::Stays);
    assert_eq!((once.frame(10, 10), once.frame(11, 10)), (Some(10), None));
    let again = once.looped();
    assert_eq!((again.frame(10, 10), again.frame(11, 10), again.frame(25, 10)), (Some(10), Some(0), Some(3)));
}

#[test]
fn a_cannon_ball_flies_in_ten_steps() {
    // 100 to the east: it starts 4 along the way and 10 up, and goes a tenth of the way a step.
    let (from, to) = ([0.0, 2.0, 0.0], [100.0, 2.0, 0.0]);
    assert!(close(steps(from, to, 0).unwrap(), [4.0, 12.0, 0.0]));
    assert!(close(steps(from, to, 10).unwrap(), [104.0, 12.0, 0.0]));
    assert_eq!(steps(from, to, 11), None);
    // A way of 20 is flown twice as fast: steps of 4, and none after the fifth.
    let near = [0.0, 2.0, 20.0];
    assert!(close(steps(from, near, 5).unwrap(), [0.0, 12.0, 24.0]));
    assert_eq!(steps(from, near, 6), None);
    assert_eq!(steps(from, from, 0), None);
}

/// The ball's picture points down the screen, and a turn is clockwise: with the camera looking north (a
/// yaw of 0) one flying east is turned three quarters (to point right) and one flying north a half (up);
/// looking east (90), the one flying east is turned a half.
#[test]
fn a_cannon_ball_is_turned_by_its_way_and_the_cameras_yaw() {
    let from = [0.0; 3];
    assert!((steps_turn(from, [50.0, 0.0, 0.0], 0.0) - 270.0).abs() < 1e-3);
    assert!((steps_turn(from, [0.0, 0.0, 50.0], 0.0) - 180.0).abs() < 1e-3);
    assert!((steps_turn(from, [50.0, 0.0, 0.0], 90.0) - 180.0).abs() < 1e-3);
    // South with the camera looking north: 270 - (0 + 270) = 0; west: 270 - 180.
    assert!(steps_turn(from, [0.0, 0.0, -50.0], 0.0).abs() < 1e-3);
    assert!((steps_turn(from, [-50.0, 0.0, 0.0], 0.0) - 90.0).abs() < 1e-3);
    // A yaw under nothing that brings the sum under nothing is brought round first.
    assert!((steps_turn(from, [50.0, 0.0, 0.0], -30.0) - 300.0).abs() < 1e-3);
    let [x, y] = turned([0.0, 1.0], steps_turn(from, [50.0, 0.0, 0.0], 0.0));
    assert!((x - 1.0).abs() < 1e-3 && y.abs() < 1e-3);
}

#[test]
fn a_storm_cannon_flies_on_and_fades_once_past() {
    let (from, to) = ([0.0; 3], [17.5, 0.0, 0.0]);
    // 1.7 a tick, whole until it is farther than the other: the tenth tick is at 17, not past.
    let (at, alpha) = straight(from, to, 9).unwrap();
    assert!(close(at, [17.0, 0.0, 0.0]) && alpha == 1.0);
    let (at, alpha) = straight(from, to, 10).unwrap();
    assert!(close(at, [18.7, 0.0, 0.0]) && (alpha - 245.0 / 255.0).abs() < 1e-6);
    // 26 ticks past it nothing is left of 255.
    assert!(straight(from, to, 34).is_some_and(|(_, alpha)| (alpha - 5.0 / 255.0).abs() < 1e-6));
    assert_eq!(straight(from, to, 35), None);
    assert_eq!(straight(from, from, 0), None);
}

#[test]
fn what_is_thrown_flies_a_bow_and_lands() {
    let (raised, to) = ([0.0, 10.0, 0.0], [10.0, 0.0, 0.0]);
    // Half way through its time (no tick is, the rule is the same): a quarter of each end and half the
    // third point, which stands as far from the end as the start does, the way turned by 240 degrees.
    let third = [10.0 + 10.0 * -0.5, -10.0, 10.0 * 0.8660254];
    let first = bowed(raised, to, true, 0).unwrap();
    let (near, between, far) = (0.07f32 * 0.07, 2.0 * 0.93 * 0.07, 0.93f32 * 0.93);
    assert!(close(first, std::array::from_fn(|axis| between * third[axis] + far * raised[axis] + near * to[axis])));
    // To the left the bow goes the other way round.
    assert!((bowed(raised, to, false, 0).unwrap()[2] + first[2]).abs() < 1e-3);
    // It is nearly there at the fourteenth tick and has landed at the fifteenth.
    let last = bowed(raised, to, true, BOW_LANDS - 1).unwrap();
    assert!(length([last[0] - to[0], last[1] - to[1], last[2] - to[2]]) < 1.0);
    assert_eq!(bowed(raised, to, true, BOW_LANDS), None);
}

#[test]
fn the_angle_on_the_screen_goes_round_clockwise_from_the_right() {
    let from = [100.0, 100.0];
    assert!(screen_angle(from, [150.0, 100.0]).abs() < 1e-3);
    assert!((screen_angle(from, [100.0, 150.0]) - 90.0).abs() < 1e-3);
    assert!((screen_angle(from, [50.0, 100.0]) - 180.0).abs() < 1e-3);
    assert!((screen_angle(from, [100.0, 50.0]) - 270.0).abs() < 1e-3);
    assert_eq!(screen_angle(from, from), 0.0);
    // What points right, turned by it, points along the way.
    let [x, y] = turned([1.0, 0.0], screen_angle(from, [130.0, 60.0]));
    assert!((x - 0.6).abs() < 1e-3 && (y + 0.8).abs() < 1e-3, "{x} {y}");
    // No angle, and one under nothing, turn nothing.
    assert_eq!(turned([3.0, 4.0], 0.0), [3.0, 4.0]);
    assert_eq!(turned([3.0, 4.0], -90.0), [3.0, 4.0]);
}
