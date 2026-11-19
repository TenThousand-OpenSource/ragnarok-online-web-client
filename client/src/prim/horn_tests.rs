//! The tests of the spike, and of the primitives that share something with their effect.

use super::*;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

#[test]
fn a_spike_is_pushed_up_and_stops_at_its_tick() {
    // As the ice of Frost Diver is: tipped a quarter turn, which makes its forward the way up, begun 30 under
    // the ground, slowing by twice what would stop it at its last tick, and stopped at its twentieth.
    let mut ice = Prim::new(Shape::QuadHorn3d, "");
    (ice.duration, ice.first_pattern, ice.settles_at) = (40, true, Some(20));
    (ice.put.1, ice.latitude, ice.speed) = (30.0, 90.0, 3.0);
    ice.accel = -(ice.speed / 40.0) * 2.0;
    ice.tick(100);
    assert!(close(ice.moved[1], 2.85) && close(ice.moved[0], 0.0) && close(ice.moved[2], 0.0));
    for _ in 1..=20 {
        ice.tick(100);
    }
    // 21 ticks of a speed that begins at 2.85 and loses 0.15 a tick; then it stands.
    let risen = ice.moved[1];
    assert!(close(risen, (0..21).map(|tick| 2.85 - 0.15 * tick as f32).sum()));
    assert_eq!((ice.speed, ice.accel), (0.0, 0.0));
    ice.tick(100);
    assert!(close(ice.moved[1], risen));
}

#[test]
fn a_spike_is_four_sides_to_a_point() {
    let mut ice = Prim::new(Shape::QuadHorn3d, "");
    (ice.size, ice.height, ice.latitude) = (2.0, 10.0, 90.0);
    let sides = ice.solid([100.0, 0.0, 50.0]);
    assert_eq!(sides.len(), 12);
    // Standing upright its base lies level about its place and its point is its height above.
    for side in sides.chunks_exact(3) {
        assert!(close(side[0].0[1], 0.0) && close(side[2].0[1], 0.0));
        assert!(close((side[0].0[0] - 100.0).abs(), 2.0) && close((side[0].0[2] - 50.0).abs(), 2.0));
        assert!(close(side[1].0[0], 100.0) && close(side[1].0[1], 10.0) && close(side[1].0[2], 50.0));
        assert_eq!((side[0].1[1], side[1].1[1], side[2].1[1]), (1.0, 0.0, 1.0));
    }
    // The picture goes round it a fifth to a side.
    assert!(close(sides[0].1[0], 0.0) && close(sides[2].1[0], 0.2) && close(sides[9].1[0], 0.6) && close(sides[11].1[0], 0.8));
}

#[test]
fn a_crossed_picture_leads_its_effect_and_lands() {
    // As a bolt of ice is: from 60 above, flying backwards along a forward tipped up, so down.
    let mut bolt = Prim::new(Shape::CrossTexture3d, "");
    (bolt.duration, bolt.leads, bolt.lands) = (70, true, true);
    (bolt.put.1, bolt.latitude, bolt.speed) = (-60.0, 90.0, -25.0);
    let mut master = Master::left(100);
    let flat = |_: f32, _: f32| 0.0;
    bolt.tick_on(&mut master, 0, &flat);
    bolt.tick_on(&mut master, 0, &flat);
    assert!(close(master.aim[1], 10.0) && !master.landed && bolt.lands);
    bolt.tick_on(&mut master, 0, &flat);
    // Under the ground: it stands, has 25 ticks left (less the one just lived) and its effect knows.
    assert!(master.landed && !bolt.lands && bolt.speed == 0.0);
    assert_eq!(bolt.count, 70 - 25 + 1);
    assert!(close(master.aim[1], -15.0));
    bolt.tick_on(&mut master, 0, &flat);
    assert!(close(master.aim[1], -15.0));
}

#[test]
fn a_crossed_picture_runs_through_its_pictures() {
    let mut bolt = Prim::new(Shape::CrossTexture3d, "one");
    assert_eq!(bolt.shown(), "one");
    (bolt.pictures, bolt.anim_speed) = (&["a", "b", "c"], 1);
    let shown: Vec<_> = (0..4).map(|_| (bolt.tick(100), bolt.shown()).1).collect();
    assert_eq!(shown, ["b", "c", "a", "b"]);
    // One whose pictures do not start over ends after the last.
    let mut once = Prim::new(Shape::CrossTexture3d, "");
    (once.pictures, once.anim_speed, once.repeats) = (&["a", "b"], 1, false);
    assert!(once.tick(100) && !once.tick(100));
}

#[test]
fn a_sprite_that_seeks_ends_near_the_aim() {
    // As a ball of fire is: turned to fly backwards towards a place 20 to the east, a tenth of the way a tick.
    let mut ball = Prim::new(Shape::Particle3d, "");
    (ball.duration, ball.seeks, ball.longitude, ball.speed) = (20, true, -90.0, -2.0);
    let mut master = Master { left: 100, aim: [20.0, 0.0, 0.0], landed: false };
    let flat = |_: f32, _: f32| 0.0;
    let mut lived = 0;
    while ball.tick_on(&mut master, 1, &flat) {
        lived += 1;
    }
    // After nine ticks it is 18 along, within 3 of the place, and ends on that tick.
    assert_eq!(lived, 8);
    assert!(close(ball.moved[0], 18.0) && close(ball.moved[2], 0.0));
}
