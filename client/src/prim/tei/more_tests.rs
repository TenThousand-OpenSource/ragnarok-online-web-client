use super::*;

use crate::prim::testing::{RIGHT, UP, close};

/// A primitive of a kind of `tei.rs` with these parts.
fn with_parts(kind: Tei, parts: Vec<Part>) -> Prim {
    let mut prim = Prim::new(Shape::Tei(kind), "picture");
    prim.duration = 1000;
    prim.parts = parts;
    prim
}

#[test]
fn a_square_of_meteor_assault_waits_slides_faster_and_is_over() {
    // As `PurpleSlash` launches one, to slide east.
    let mut heights = [0.0; 21];
    heights[1] = 0.8;
    let mut square = with_parts(
        Tei::FloorSquare,
        vec![Part { process: -2, out: 20.0, heights, at: [[0.0, -8.0, 0.0], [0.0; 3], [0.0; 3], [0.0; 3]], ..Part::default() }]
    );
    square.tick(1000);
    square.tick(1000);
    assert!(square.parts[0].at[0][0] == 0.0 && square.rects(RIGHT, UP).is_empty());
    // Its first tick over nothing it only slides; from the next it comes up.
    square.tick(1000);
    assert!(close(square.parts[0].at[0][0], 0.8) && square.parts[0].alpha == 0);
    square.tick(1000);
    assert_eq!(square.parts[0].alpha, 10);
    assert!(close(square.parts[0].at[0][0], 0.8 + 0.8 * 1.02));
    let rects = square.rects(RIGHT, UP);
    assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (1, [200, 200, 255], false));
    // Level, 8 over the ground, its first corner 20 from its middle to the north-east.
    let (at, _) = rects[0].corners[0];
    assert!(close(at[1], 8.0) && close(at[0] - square.parts[0].at[0][0], 20.0 * 45.0_f32.to_radians().cos()) && at[2] > 14.0);
    for _ in 0..40 {
        square.tick(1000);
    }
    assert!(!square.parts[0].live && square.rects(RIGHT, UP).is_empty());
}

#[test]
fn a_square_of_the_kind_86_numbered_2_is_green() {
    // As `SonicWave(2)` launches one: `flag1[1]` of 2.
    let mut flags = [0; 21];
    flags[1] = 2;
    let square = with_parts(Tei::FloorSquare, vec![Part { process: 5, alpha: 40, out: 20.0, flags, ..Part::default() }]);
    let rects = square.rects(RIGHT, UP);
    assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (1, [0, 255, 0], false, 40.0));
}

#[test]
fn the_squares_of_a_whirl_stand_one_under_the_other_narrowing() {
    // As `TexturedMark` launches them for Storm Kick.
    let mut stack = Part { tall: 40.0, out: 2.0, seed: 7, ..Part::default() };
    (stack.heights[0], stack.heights[1], stack.flags[0], stack.flags[4]) = (50.0, 5.0, 10, 0);
    let mut whirl = with_parts(Tei::StackedRings, vec![stack]);
    for _ in 0..40 {
        whirl.tick(1000);
    }
    assert_eq!(whirl.parts[0].alpha, 40);
    assert!((120..=200).contains(&whirl.parts[0].turned));
    let rects = whirl.rects(RIGHT, UP);
    assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (10, [50, 50, 255], true));
    // The first square 50 over the ground (two corners 1 higher), 40 to a corner; the second 5 lower and 4 less.
    let reach = |rect: &Rect, corner: usize| rect.corners[corner].0[0].hypot(rect.corners[corner].0[2]);
    assert!(close(rects[0].corners[0].0[1], 51.0) && close(rects[0].corners[2].0[1], 50.0) && close(reach(&rects[0], 0), 40.0));
    assert!(close(rects[1].corners[2].0[1], 45.0) && close(reach(&rects[1], 0), 40.0 - 2.0 * 2.0));
    // It holds to its sixtieth tick and then goes.
    for _ in 0..30 {
        whirl.tick(1000);
    }
    assert_eq!(whirl.parts[0].alpha, 30);
}

#[test]
fn the_sparks_of_cart_termination_fly_their_ways_and_fade_after_25_ticks() {
    let mut thrown = Part { tall: 1.0, out: 2.5, ..Part::default() };
    // One straight up, one level to the north, one level to the east.
    (thrown.heights[2], thrown.heights[0]) = (90.0, 0.0);
    (thrown.heights[6], thrown.heights[4]) = (0.0, 0.0);
    (thrown.heights[10], thrown.heights[8]) = (0.0, 90.0);
    (thrown.heights[12], thrown.heights[13], thrown.heights[14], thrown.heights[15]) = (230.0, 230.0, 230.0, 2.0);
    let mut sparks = with_parts(Tei::Spread, vec![thrown]);
    for _ in 0..10 {
        sparks.tick(1000);
    }
    let part = &sparks.parts[0];
    assert!(close(part.at[2][1], -10.0) && close(part.at[0][2], 10.0) && close(part.at[3][0], 10.0));
    let rects = sparks.rects(RIGHT, UP);
    // The one that went up shows whole, facing the screen about a place 10 up; of the level ones the corner
    // looked at is under the level of the feet, and they are left out.
    assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (1, [255, 160, 160], true, 230.0));
    let middle: [f32; 3] = std::array::from_fn(|axis| (rects[0].corners[0].0[axis] + rects[0].corners[2].0[axis]) / 2.0);
    assert!(close(middle[0], 0.0) && close(middle[1], 10.0) && close(middle[2], 0.0));
    assert!(rects[0].corners.iter().all(|(at, _)| close(at[2], 0.0)));
    for _ in 0..20 {
        sparks.tick(1000);
    }
    assert!(close(sparks.parts[0].heights[12], 230.0 - 5.0 * 6.0) && close(sparks.parts[0].tall, 0.9_f32.powi(5)));
}

#[test]
fn a_shield_flies_through_its_place_with_three_fainter_ones_behind() {
    let mut head = Part { alpha: 25, out: 5.0, turned: 0, ..Part::default() };
    head.heights[1] = 2.5;
    head.at[0] = [-37.5, -8.0, 0.0];
    let mut shields = with_parts(Tei::ShieldBoomerang3, vec![head, Part::default(), Part::default(), Part::default()]);
    for _ in 0..3 {
        shields.tick(1000);
    }
    assert_eq!((shields.parts[0].alpha, shields.parts[0].round), (160, 315));
    assert!(close(shields.parts[0].at[0][0], -30.0));
    // The second took the first's look as it was before this tick's own step, 150 fainter.
    assert_eq!((shields.parts[1].alpha, shields.parts[1].round), (0, 330));
    for _ in 0..12 {
        shields.tick(1000);
    }
    assert!(close(shields.parts[0].at[0][0], 0.0));
    // It came up to 250 over five ticks and went down by 10 at the fifteenth.
    assert_eq!(shields.parts[0].alpha, 240);
    assert!(shields.parts[1].alpha > shields.parts[2].alpha && shields.parts[2].alpha > shields.parts[3].alpha);
    // A square facing the screen, mixed in and white, 8 over the ground.
    let rects = shields.rects(RIGHT, UP);
    assert_eq!((rects[0].colour, rects[0].adds), ([255; 3], false));
    let middle = (rects[0].corners[0].0[1] + rects[0].corners[2].0[1]) / 2.0;
    assert!(close(middle, 8.0));
}

#[test]
fn a_spark_leaps_out_in_a_bow_with_its_tail_behind() {
    // As `WhiteSparkStreak` launches one of its kind 3, to leap east.
    let mut spark = Part { alpha: 200, alpha_top: 3, tall: 0.07, out: 9.0, rise: 0, ..Part::default() };
    spark.at[1] = [1.0, 4.0, 0.0];
    let mut sparks = with_parts(Tei::HitLine3, vec![spark]);
    for _ in 0..15 {
        sparks.tick(1000);
    }
    // Its head is a quarter of a turn along: 9 out and 9 up; the places behind it are 6 degrees apart.
    assert_eq!(sparks.parts[0].heights[..5], [90.0, 90.0, 84.0, 78.0, 72.0]);
    let rects = sparks.rects(RIGHT, UP);
    // Four pieces, each a grey tube of three sides and a blue one about it.
    assert_eq!(rects.len(), 24);
    assert_eq!((rects[0].colour, rects[0].alpha, rects[3].colour, rects[3].alpha), ([105; 3], 200.0, [0, 0, 255], 100.0));
    assert_eq!((rects[6].colour, rects[6].alpha), ([135; 3], 190.0));
    let (at, _) = rects[6].corners[0];
    assert!((at[0] - 9.0).abs() < 0.1 && (at[1] - 14.0).abs() < 0.1);
    assert!(rects.iter().all(|rect| rect.adds));
    for _ in 0..30 {
        sparks.tick(1000);
    }
    assert_eq!((sparks.parts[0].heights[0], sparks.parts[0].alpha), (180.0, 150));
}
