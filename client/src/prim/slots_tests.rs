//! The tests of the kinds made of parts: how a mote rises, comes up and goes, how a wall's alpha and edges
//! move, and where the quads of both are.

use super::*;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

fn near(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|axis| close(a[axis], b[axis]))
}

const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
const UP: [f32; 3] = [0.0, 1.0, 0.0];

fn motes(slots: Vec<Slot>) -> Prim {
    let mut prim = Prim::new(Shape::ParticleUp, "one");
    prim.pictures = &["one", "two"];
    prim.slots = slots;
    prim
}

#[test]
fn a_mote_rises_turns_and_comes_up_then_goes() {
    let mut prim = motes(vec![Slot::mote([1.0, 0.0, 2.0], 2.0, 0.5, false)]);
    // Its first tick it only begins: nothing of it shows, so nothing is drawn.
    assert!(prim.tick(100));
    assert_eq!((prim.slots[0].alpha, prim.slots[0].round), (0, 355));
    assert!(near(prim.slots[0].at, [1.0, 0.5, 2.0]));
    assert!(prim.lit(RIGHT, UP).is_empty());
    // Ticks 2 to 10 it comes up by 15: 135.
    for _ in 0..9 {
        prim.tick(100);
    }
    assert_eq!(prim.slots[0].alpha, 135);
    assert!(close(prim.slots[0].at[1], 5.0));
    assert_eq!(prim.lit(RIGHT, UP)[0].alpha, 135.0);
    // Then down by 3 a tick: 45 ticks to nothing, and the tick after it is over.
    for _ in 0..45 {
        prim.tick(100);
    }
    assert!((prim.slots[0].alpha, prim.slots[0].live) == (0, true));
    prim.tick(100);
    assert!(!prim.slots[0].live);
    // The primitive itself lasts its own time, and its alpha is left alone.
    assert_eq!(prim.alpha, 254.0);
    prim.duration = prim.count;
    assert!(!prim.tick(100));
}

#[test]
fn a_mote_that_flashes_shows_its_second_picture_at_two_ticks() {
    let slot = || Slot::mote([0.0; 3], 2.0, 0.0, true);
    let mut prim = motes(vec![slot(), slot(), slot(), slot()]);
    let mut flashed = vec![Vec::new(); 4];
    for tick in 1..=40 {
        prim.tick(100);
        for (index, slot) in prim.slots.iter().enumerate() {
            if slot.flashing {
                flashed[index].push(tick);
            }
        }
    }
    assert_eq!(flashed, [vec![35, 36], vec![28, 29], vec![21, 22], vec![14, 15]]);
    // The picture drawn follows.
    let mut prim = motes(vec![slot()]);
    for _ in 0..35 {
        prim.tick(100);
    }
    assert_eq!(prim.lit(RIGHT, UP)[0].picture, "two");
    prim.tick(100);
    prim.tick(100);
    assert_eq!(prim.lit(RIGHT, UP)[0].picture, "one");
}

#[test]
fn a_mote_is_a_square_facing_the_screen_about_its_middle() {
    let mut slot = Slot::mote([10.0, 5.0, -3.0], 2.0, 0.0, false);
    (slot.alpha, slot.round) = (90, 0);
    let prim = motes(vec![slot]);
    // Looking north: the screen's right is east, its up the world's up.
    let quad = &prim.lit(RIGHT, UP)[0];
    // The corners: to the right, down the screen, to the left, up; the picture's bottom on the first two.
    assert!(near(quad.corners[0].0, [12.0, 5.0, -3.0]) && quad.corners[0].1 == [0.0, 1.0]);
    assert!(near(quad.corners[1].0, [10.0, 3.0, -3.0]) && quad.corners[1].1 == [1.0, 1.0]);
    assert!(near(quad.corners[2].0, [8.0, 5.0, -3.0]) && quad.corners[2].1 == [1.0, 0.0]);
    assert!(near(quad.corners[3].0, [10.0, 7.0, -3.0]) && quad.corners[3].1 == [0.0, 0.0]);
    assert_eq!((quad.corners[4], quad.corners[5]), (quad.corners[2], quad.corners[0]));
    // Seen from another side it lies in that screen's two directions.
    let quad = &prim.lit([0.0, 0.0, -1.0], UP)[0];
    assert!(near(quad.corners[0].0, [10.0, 5.0, -5.0]));
}

fn walls(slots: Vec<Slot>) -> Prim {
    let mut prim = Prim::new(Shape::Casting3d, "wall");
    prim.slots = slots;
    prim
}

#[test]
fn a_wall_comes_up_holds_and_fades() {
    let mut prim = walls(vec![Slot::wall(0, 5.0, 90, 40.0, 30)]);
    let mut alphas = vec![0];
    for _ in 0..70 {
        prim.tick(100);
        alphas.push(prim.slots[0].alpha);
    }
    // By 5 a tick over its first 15 ticks, then as it is until the 30th, from which it goes down by 2.
    assert_eq!((alphas[1], alphas[15], alphas[16], alphas[29]), (5, 75, 75, 75));
    assert_eq!((alphas[30], alphas[31], alphas[66], alphas[67], alphas[70]), (73, 71, 1, 0, 0));
    // One told to hold longer than it takes to come up never passes 180.
    let mut long = walls(vec![Slot::wall(0, 5.0, 90, 40.0, -1)]);
    long.slots[0].alpha = 170;
    for _ in 0..5 {
        long.tick(100);
    }
    assert_eq!(long.slots[0].alpha, 180);
}

#[test]
fn a_walls_edges_swing_about_three_quarters_of_its_height_and_rise_over_ninety_ticks() {
    let wall = || Slot::wall(0, 5.0, 90, 40.0, 30);
    let mut prim = walls(vec![wall(), wall(), wall()]);
    for _ in 0..30 {
        prim.tick(100);
    }
    let sin = |degrees: i32| ((degrees % 360) as f32).to_radians().sin();
    let risen = sin(30);
    // The first part: 34 degrees on from edge to edge, 4 a tick; the second 3 a tick; the others their own.
    for column in [0i32, 1, 7, 20] {
        let edge = |part: usize| prim.slots[part].heights[column as usize];
        assert!(close(edge(0), (10.0 * sin(column * 34 + 120) + 30.0) * risen), "{column}");
        assert!(close(edge(1), (10.0 * sin(column * 34 + 90) + 30.0) * risen), "{column}");
        assert!(close(edge(2), (10.0 * sin((30 + column * 17) * 2) + 30.0) * risen), "{column}");
    }
    // From the 91st tick it stands at its full height.
    for _ in 0..61 {
        prim.tick(100);
    }
    assert!(close(prim.slots[0].heights[0], 10.0 * sin(364) + 30.0));
}

#[test]
fn a_wall_is_twenty_pieces_round_its_middle() {
    let mut slot = Slot::wall(90, 5.0, 90, 40.0, 30);
    (slot.alpha, slot.heights) = (60, [8.0; 21]);
    let mut prim = walls(vec![slot.clone()]);
    prim.put.1 = -2.0;
    let quads = prim.lit(RIGHT, UP);
    assert_eq!(quads.len(), 20);
    assert!(quads.iter().all(|quad| quad.alpha == 60.0 && quad.picture == "wall"));
    // It begins where it is turned to: 90 degrees is north of the middle, which is 2 over the place.
    let first = &quads[0];
    assert!(near(first.corners[0].0, [0.0, 2.0, 5.0]) && first.corners[0].1 == [0.0, 1.0]);
    // Upright, its top is straight over its foot; the piece's far edge is 18 degrees on.
    assert!(near(first.corners[3].0, [0.0, 10.0, 5.0]) && first.corners[3].1 == [0.0, 0.0]);
    let (sin, cos) = 108f32.to_radians().sin_cos();
    assert!(near(first.corners[1].0, [5.0 * cos, 2.0, 5.0 * sin]) && first.corners[1].1 == [1.0 / 21.0, 1.0]);
    assert!(near(first.corners[2].0, [5.0 * cos, 10.0, 5.0 * sin]));
    // The last piece ends where the first began, and the picture has gone round 20 of its 21 parts.
    let last = &quads[19];
    assert!(near(last.corners[1].0, first.corners[0].0) && close(last.corners[1].1[0], 20.0 / 21.0));
    // A wall that leans: at 60 degrees its top is half its height further out and 0.866 of it up.
    slot.rise = 60;
    let leaning = walls(vec![slot]).lit(RIGHT, UP);
    assert!(near(leaning[0].corners[3].0, [0.0, 8.0 * 60f32.to_radians().sin(), 9.0]));
}

#[test]
fn a_band_widens_and_narrows_at_both_rings() {
    // As a ring of Resurrection: both rings close in, slower and slower.
    let mut ring = Prim::new(Shape::Cylinder3d, "");
    (ring.inner, ring.outer) = (7.5, 10.5);
    (ring.inner_speed, ring.inner_accel, ring.outer_speed, ring.outer_accel) = (-0.12, 0.003, -0.12, 0.003);
    ring.tick(100);
    // The speeds change before the rings do.
    assert!(close(ring.inner, 7.5 - 0.117) && close(ring.outer, 10.5 - 0.117));
    ring.tick(100);
    assert!(close(ring.inner, 7.5 - 0.117 - 0.114) && close(ring.outer, 10.5 - 0.117 - 0.114));
}

#[test]
fn a_mote_waits_while_its_count_is_under_one_and_drifts_along_its_way() {
    // As `ThunderSparks` launches one, to wait three ticks and drift north.
    let mut slot = Slot::mote([1.0, 0.0, 2.0], 0.5, 0.25, false);
    (slot.process, slot.drifts, slot.turned) = (-3, 0.05, 90);
    let mut prim = motes(vec![slot]);
    for _ in 0..3 {
        prim.tick(100);
    }
    assert!(near(prim.slots[0].at, [1.0, 0.0, 2.0]) && prim.slots[0].round == 0);
    for _ in 0..4 {
        prim.tick(100);
    }
    assert!(near(prim.slots[0].at, [1.0, 1.0, 2.2]));
    assert_eq!((prim.slots[0].alpha, prim.slots[0].round), (45, 340));
}
