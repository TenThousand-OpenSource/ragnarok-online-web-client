//! The tests of the walls of the kinds 30, 31 and 149: how each comes up and goes, widens and turns, and
//! where and with which picture the changing ones are drawn.

use super::*;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
const UP: [f32; 3] = [0.0, 1.0, 0.0];

fn walls(kind: Walls, duration: i32, slots: Vec<Slot>) -> Prim {
    let mut prim = Prim::new(Shape::Walls(kind), "one");
    (prim.duration, prim.slots, prim.pictures) = (duration, slots, &["one", "two", "three"]);
    prim
}

#[test]
fn a_wall_of_a_spell_waits_begins_anew_and_widens() {
    // As a short casting launches them: 4.1 out, nothing showing, the second wall five ticks late.
    let mut late = Slot::wall(270, 4.1, 80, 19.0, 15);
    late.process = -5;
    let mut prim = walls(Walls::Spell, 60, vec![Slot::wall(180, 4.1, 80, 20.0, 15), late]);
    // Its first tick the first wall is too far out, has nothing to lose and begins anew.
    assert!(prim.tick(60));
    assert!(close(prim.slots[0].out, 3.37) && prim.slots[0].rise == 74 && prim.slots[0].alpha == 0);
    assert!(close(prim.slots[1].out, 4.1) && prim.slots[1].rise == 80);
    // While nearer than 4 it comes up by 10 a tick, leaning a degree more each.
    for _ in 0..5 {
        prim.tick(60);
    }
    assert_eq!((prim.slots[0].alpha, prim.slots[0].rise), (50, 69));
    assert!(close(prim.slots[0].out, 3.72));
    // The second wall has only just begun.
    assert_eq!((prim.slots[1].process, prim.slots[1].alpha), (1, 0));
    assert!(close(prim.slots[1].out, 3.37));
    // An edge in the middle of the first wall sways about the wall's height by three tenths of it at most.
    let middle = prim.slots[0].heights[10];
    assert!(close(middle, 20.0 + 6.0 * sine(6.0)));
    assert!(close(prim.slots[0].heights[0], 20.0));
    // With 30 ticks or fewer left it does not begin anew: it stays gone.
    let mut last = walls(Walls::Spell, 20, vec![Slot::wall(0, 4.1, 80, 20.0, 15)]);
    last.tick(20);
    assert!(close(last.slots[0].out, 4.17) && last.slots[0].alpha == 0);
}

#[test]
fn a_spinning_wall_comes_up_over_ten_ticks_and_goes_by_its_number() {
    let mut prim = walls(Walls::Spin, 300, vec![Slot::wall(0, 1.0, 80, 25.0, 3), Slot::wall(90, 2.0, 80, 25.0, 4), Slot::wall(350, 3.0, 42, 25.0, 2)]);
    for _ in 0..10 {
        prim.tick(300);
    }
    assert_eq!(prim.slots.iter().map(|slot| slot.alpha).collect::<Vec<_>>(), [200, 200, 200]);
    assert_eq!((prim.slots[0].turned, prim.slots[2].turned), (50, 40));
    assert!(close(prim.slots[0].out, 4.0) && close(prim.slots[1].out, 5.0));
    // It leans a degree more a tick, down to 40.
    assert_eq!((prim.slots[0].rise, prim.slots[2].rise), (70, 40));
    prim.tick(300);
    assert_eq!(prim.slots.iter().map(|slot| slot.alpha).collect::<Vec<_>>(), [195, 195, 192]);
    // Its edges stand nothing at the ends and highest in the middle.
    let heights = prim.slots[0].heights;
    assert!(close(heights[0], 0.0) && close(heights[20], 0.0));
    assert!(close(heights[10], 25.0 + 25.0 * 0.15 * sine(11.0)));
    for _ in 0..60 {
        prim.tick(300);
    }
    assert!(prim.slots.iter().all(|slot| slot.alpha == 0));
    assert!(prim.lit(RIGHT, UP).iter().all(|lit| lit.alpha == 0.0));
}

#[test]
fn the_walls_of_a_primitive_whose_accel_is_99_spin_slower_and_lean_further() {
    // As `TexturedCrossLevel` launches its first wall, numbered 0.
    let mut prim = walls(Walls::SlowSpin, 300, vec![Slot::wall(0, 1.0, 80, 25.0, 0)]);
    for _ in 0..10 {
        prim.tick(300);
    }
    assert_eq!((prim.slots[0].alpha, prim.slots[0].turned, prim.slots[0].rise), (200, 30, 70));
    assert!(close(prim.slots[0].out, 2.0));
    for _ in 0..50 {
        prim.tick(300);
    }
    // It leans down to 30 degrees, and one numbered 0 goes down by 2 a tick.
    assert_eq!((prim.slots[0].rise, prim.slots[0].alpha), (30, 100));
    assert_eq!(prim.lit(RIGHT, UP).len(), 20);
}

#[test]
fn a_waving_wall_comes_up_to_a_hundred_and_fades_with_its_primitive() {
    let mut prim = walls(Walls::Wave { gain: 1 }, 120, vec![Slot::wall(180, 4.3, 75, 23.0, 3)]);
    for _ in 0..29 {
        prim.tick(120);
    }
    assert_eq!((prim.slots[0].alpha, prim.slots[0].turned), (87, 267));
    for _ in 0..45 {
        prim.tick(120);
    }
    assert_eq!(prim.slots[0].alpha, 87);
    // From its 75th tick, 46 before its primitive's last, it goes down by 2.
    prim.tick(120);
    assert_eq!((prim.slots[0].process, prim.slots[0].alpha), (75, 85));
    for _ in 0..45 {
        prim.tick(120);
    }
    assert_eq!(prim.slots[0].alpha, 0);
}

#[test]
fn a_waving_wall_is_drawn_with_the_picture_of_its_tick_out_by_the_sine_of_its_turn() {
    let mut slot = Slot::wall(90, 4.0, 90, 10.0, 3);
    (slot.process, slot.alpha, slot.heights) = (4, 60, [10.0; 21]);
    let first = walls(Walls::Wave { gain: 1 }, 120, vec![slot.clone()]);
    let lit = first.lit(RIGHT, UP);
    assert_eq!(lit.len(), 20);
    // The fifth tick shows the second of three pictures again.
    assert!(lit.iter().all(|lit| lit.picture == "two" && lit.alpha == 60.0));
    // Begun a quarter round, it stands one further out: its first edge is 5 north, upright and 10 tall.
    let (foot, _) = lit[0].corners[0];
    assert!(close(foot[0], 0.0) && close(foot[1], 0.0) && close(foot[2], 5.0));
    let (top, _) = lit[0].corners[3];
    assert!(close(top[0], 0.0) && close(top[1], 10.0) && close(top[2], 5.0));
    // The second drawing is as much brighter as it is told, 255 at most.
    let second = walls(Walls::Wave { gain: 3 }, 120, vec![slot.clone()]);
    assert!(second.lit(RIGHT, UP).iter().all(|lit| lit.alpha == 180.0));
    slot.alpha = 100;
    let second = walls(Walls::Wave { gain: 3 }, 120, vec![slot]);
    assert!(second.lit(RIGHT, UP).iter().all(|lit| lit.alpha == 255.0));
}

#[test]
fn a_square_waits_widens_comes_up_and_goes_and_lies_level() {
    let mut part = Slot::wall(0, 10.0, 0, 0.0, 0);
    (part.process, part.at) = (-2, [0.0, 5.0, 0.0]);
    let mut prim = walls(Walls::Square { quick: false }, 200, vec![part.clone()]);
    prim.tick(200);
    prim.tick(200);
    assert!(close(prim.slots[0].out, 10.0) && prim.slots[0].alpha == 0);
    prim.tick(200);
    assert!(close(prim.slots[0].out, 10.7) && prim.slots[0].alpha == 2);
    for _ in 0..10 {
        prim.tick(200);
    }
    assert_eq!(prim.slots[0].alpha, 19);
    // The quick one comes up by 6.
    let mut quick = walls(Walls::Square { quick: true }, 200, vec![part.clone()]);
    for _ in 0..12 {
        quick.tick(200);
    }
    assert_eq!(quick.slots[0].alpha, 60);
    // Its corners are nine tenths of its reach to the east, north, west and south, 5 over the middle.
    (part.out, part.alpha) = (10.0, 20);
    let lit = walls(Walls::Square { quick: false }, 200, vec![part]).lit(RIGHT, UP);
    assert_eq!(lit.len(), 1);
    let at = |index: usize| lit[0].corners[index].0;
    assert!(close(at(0)[0], 9.0) && close(at(0)[1], 5.0) && close(at(0)[2], 0.0));
    assert!(close(at(1)[0], 0.0) && close(at(1)[2], 9.0));
    assert!(close(at(2)[0], -9.0) && close(at(3)[2], -9.0));
    assert_eq!((lit[0].corners[0].1, lit[0].corners[2].1), ([0.0, 1.0], [1.0, 0.0]));
}
