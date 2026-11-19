use super::*;

use crate::prim::testing::{RIGHT, UP, close};

/// A primitive of a kind of this file with one part.
fn with_part(kind: Tei, part: Part) -> Prim {
    let mut prim = Prim::new(Shape::Tei(kind), "picture");
    prim.duration = 1000;
    prim.parts.push(part);
    prim
}

#[test]
fn the_lots_are_those_of_the_clients_rand() {
    // Microsoft's `rand` after `srand(1)` gives 41, 18467, 6334.
    let mut part = Part { seed: 1, ..Part::default() };
    assert_eq!([part.lot(32768), part.lot(32768), part.lot(32768)], [41, 18467, 6334]);
}

#[test]
fn a_rectangle_turns_the_clients_down_into_up_and_shows_the_low_byte_of_its_alpha() {
    let rect = Rect::new("picture", 260, [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0], [6.0, -7.0, 8.0], [9.0, 0.0, 1.0]], (0.25, 0.5), [1, 2, 3], true);
    assert_eq!(rect.alpha, 4.0);
    assert_eq!(rect.corners[0], ([0.0, -1.0, 2.0], [0.25, 1.0]));
    assert_eq!(rect.corners[1], ([3.0, -4.0, 5.0], [0.5, 1.0]));
    assert_eq!(rect.corners[2], ([6.0, 7.0, 8.0], [0.5, 0.0]));
    assert_eq!(rect.corners[3], ([9.0, 0.0, 1.0], [0.25, 0.0]));
    assert_eq!((rect.corners[4], rect.corners[5]), (rect.corners[2], rect.corners[0]));
    // A piece of a band shows its 21st of the picture in the colour of the primitive's size.
    let piece = Rect::piece("picture", 20, 100, [[0.0; 3]; 4], 10.0);
    assert!(close(piece.corners[0].1[0], 20.0 / 21.0) && close(piece.corners[1].1[0], 1.0));
    assert_eq!((piece.colour, piece.adds), ([255, 89, 182], true));
    assert_eq!(sizes::by_size(0.0), ([255; 3], false));
    assert_eq!(sizes::by_size(5.0), ([255; 3], true));
}

#[test]
fn a_beam_of_true_sight_waits_comes_up_and_reaches_out() {
    // As `TexturedQuadSpin` launches one of its kind 3, and one of its kind 4 that waits 24 ticks.
    let mut heights = [0.0; 21];
    (heights[0], heights[4]) = (-12.0, 3.0);
    let mut beam = with_part(Tei::FanBeam, Part { out: 2.0, rise: 90, heights, ..Part::default() });
    assert!(beam.rects(RIGHT, UP).is_empty());
    for _ in 0..10 {
        beam.tick(1000);
    }
    assert_eq!(beam.parts[0].alpha, 60);
    assert!(close(beam.parts[0].out, 32.0));
    let rects = beam.rects(RIGHT, UP);
    assert_eq!(rects.len(), 2);
    assert_eq!((rects[0].adds, rects[0].colour, rects[1].adds, rects[1].colour), (true, [250; 3], false, [250; 3]));
    // Untipped, it lies level 12 over the place and points north, its two last corners at the middle.
    assert_eq!(rects[0].corners[2].0, [0.0, 12.0, 0.0]);
    assert!(close(rects[0].corners[0].0[1], 12.0) && close(rects[0].corners[0].0[2], 32.0 * 88.0_f32.to_radians().sin()));
    assert!(rects[0].corners[0].0[0] > 0.0 && rects[0].corners[1].0[0] < 0.0);
    // It holds a tick and then goes down by 1 a tick.
    beam.tick(1000);
    beam.tick(1000);
    assert_eq!(beam.parts[0].alpha, 59);

    heights[4] = 4.0;
    let mut late = with_part(Tei::FanBeam, Part { process: -24, out: 2.0, heights, ..Part::default() });
    for _ in 0..24 {
        late.tick(1000);
    }
    assert_eq!((late.parts[0].alpha, late.parts[0].out), (0, 2.0));
    late.tick(1000);
    assert_eq!(late.parts[0].alpha, 20);
    assert!(close(late.parts[0].out, 2.3));
    assert_eq!(late.rects(RIGHT, UP)[1].colour, [250, 250, 0]);
}

#[test]
fn a_streak_reaches_from_its_near_end_along_its_way_in_five_sides_twice() {
    let mut heights = [0.0; 21];
    (heights[0], heights[4]) = (8.0, 2.0);
    let mut streak = with_part(
        Tei::PentaRing,
        Part { turned: 90, out: 50.0, tall: 0.05, heights, at: [[1.0, 0.0, 2.0], [1.0, 0.0, 2.0], [0.0; 3], [0.0; 3]], ..Part::default() }
    );
    for _ in 0..5 {
        streak.tick(1000);
    }
    assert_eq!(streak.parts[0].alpha, 10);
    assert!(close(streak.parts[0].at[1][0], 1.0) && close(streak.parts[0].at[1][2], 52.0));
    let rects = streak.rects(RIGHT, UP);
    assert_eq!(rects.len(), 10);
    assert!(!rects[0].adds && rects[5].adds && rects.iter().all(|rect| rect.colour == [255; 3]));
    // Its first corner is 0.05 to the side of the near end (across a way north: west), 8 over the place.
    let (at, _) = rects[0].corners[0];
    assert!(close(at[0], 1.0 - 0.05) && close(at[1], 8.0) && close(at[2], 2.0));
    // The outer one is a quarter thicker.
    assert!(close(rects[5].corners[0].0[0], 1.0 - 0.3));
    for _ in 0..10 {
        streak.tick(1000);
    }
    assert!(streak.parts[0].alpha == 0 && streak.rects(RIGHT, UP).is_empty());
}

#[test]
fn a_wind_grows_to_a_third_of_a_turn_and_fades_as_it_widens() {
    // As `Cloud11Burst` launches one of its first kind.
    let mut wind = with_part(Tei::Wind, Part { round: 30, tall: 5.0, out: 4.5, rise: 90, alpha_top: 1, heights: [1.0; 21], ..Part::default() });
    wind.size = 5.0;
    for _ in 0..11 {
        wind.tick(1000);
    }
    assert_eq!((wind.parts[0].alpha, wind.parts[0].round, wind.parts[0].turned), (110, 63, 55));
    // 63 degrees in steps of 3: 21 edges, 20 pieces, each from 5 to 6 over the ground, 4.5 out.
    let rects = wind.rects(RIGHT, UP);
    assert_eq!(rects.len(), 20);
    let ((foot, _), (top, _)) = (rects[0].corners[0], rects[0].corners[3]);
    assert!(close(foot[1], 5.0) && close(top[1], 6.0) && close(foot[0].hypot(foot[2]), 4.5));
    assert_eq!((rects[0].colour, rects[0].adds), ([255; 3], true));
    for _ in 0..30 {
        wind.tick(1000);
    }
    assert_eq!((wind.parts[0].round, wind.parts[0].alpha), (120, 110 - 2 * 21));
    assert!(close(wind.parts[0].out, 4.5 + 2.1));
    // 120 degrees in steps of 5: 24 edges, the pieces past the twentieth showing the twentieth's part.
    let rects = wind.rects(RIGHT, UP);
    assert_eq!(rects.len(), 23);
    assert!(close(rects[22].corners[0].1[0], 19.0 / 21.0));
}

#[test]
fn a_band_of_feathers_goes_round_and_one_of_a_burst_widens() {
    let mut heights = [0.0; 21];
    (heights[0], heights[1]) = (-14.0, 4.0);
    let mut ring = with_part(Tei::SphereWind, Part { alpha: 100, round: 180, turned: 358, tall: 4.0, out: 14.0, heights, ..Part::default() });
    ring.size = 4.0;
    ring.tick(1000);
    assert_eq!((ring.parts[0].turned, ring.parts[0].alpha), (2, 100));
    // Half a turn in steps of 9: 21 edges, 20 pieces; untipped, the band stands 4 over and under 14 up.
    let rects = ring.rects(RIGHT, UP);
    assert_eq!(rects.len(), 20);
    assert!(close(rects[0].corners[0].0[1], 10.0) && close(rects[0].corners[3].0[1], 18.0));
    assert_eq!(rects[0].colour, [100, 100, 255]);

    let mut burst = with_part(Tei::SphereWind2, Part { round: 180, tall: 2.0, out: 5.0, heights, seed: 1, ..Part::default() });
    for _ in 0..25 {
        burst.tick(1000);
    }
    assert_eq!(burst.parts[0].alpha, 50);
    assert!(close(burst.parts[0].out, 5.0 * 1.01_f32.powi(25)));
    assert!((25..=50).contains(&burst.parts[0].rise));
}
