//! The tests of the primitives that show a sprite: how they fly and fall, and which frame shows.

use super::*;

use crate::prim::testing::close;

fn flat(_: f32, _: f32) -> f32 {
    0.0
}

#[test]
fn a_particle_flies_along_its_forward_and_slows() {
    // As a spark of Detoxify is: tipped a quarter turn, which makes its forward the way up.
    let mut spark = Prim::new(Shape::Particle3d, "");
    (spark.latitude, spark.speed, spark.accel) = (90.0, 1.0, -0.25);
    (spark.size, spark.size_speed, spark.roll_speed) = (2.0, -0.5, 3.0);
    spark.tick(100);
    // Its speed changes before it moves.
    assert!(close(spark.moved[0], 0.0) && close(spark.moved[1], 0.75) && close(spark.moved[2], 0.0));
    assert!(close(spark.size, 1.5) && close(spark.roll, 3.0));
    spark.tick(100);
    assert!(close(spark.moved[1], 1.25) && close(spark.size, 1.0) && close(spark.roll, 6.0));
}

#[test]
fn a_particle_turned_first_and_then_tipped_goes_that_way() {
    // Turned a quarter to the east and tipped 30 degrees: the client's forward is then
    // (sin a, -cos a sin b, cos a cos b), its y pointing down.
    let mut spark = Prim::new(Shape::Particle3d, "");
    (spark.yaw_first, spark.longitude, spark.latitude) = (true, 90.0, 30.0);
    let reach = spark.forward(2.0);
    assert!(close(reach[0], 2.0) && close(reach[1], 0.0) && close(reach[2], 0.0));
    (spark.longitude, spark.latitude) = (0.0, 30.0);
    let reach = spark.forward(2.0);
    assert!(close(reach[0], 0.0) && close(reach[1], 1.0) && close(reach[2], 3f32.sqrt()));
    // Tipped first and then turned, the same figures give another way: the tip is turned with it.
    (spark.yaw_first, spark.longitude) = (false, 90.0);
    let reach = spark.forward(2.0);
    assert!(close(reach[0], 3f32.sqrt()) && close(reach[1], 1.0) && close(reach[2], 0.0));
}

#[test]
fn a_falling_particle_goes_level_and_is_thrown_back_by_the_ground() {
    let mut coin = Prim::new(Shape::ParticleGravity3d, "");
    // Put 2 above the ground, going north at 1 a tick, falling 1.5 a tick.
    coin.put.1 = -2.0;
    (coin.speed, coin.grav_speed) = (1.0, 1.5);
    coin.tick_on(&mut Master::left(100), 1, &flat);
    assert!(close(coin.moved[2], 1.0) && close(coin.moved[1], -1.5));
    // Still over the ground: it falls on.
    assert!(close(coin.grav_speed, 1.5));
    coin.tick_on(&mut Master::left(100), 1, &flat);
    // Under it now: thrown back up at two thirds of its speed.
    assert!(close(coin.moved[1], -3.0) && close(coin.grav_speed, -1.0));
    coin.tick_on(&mut Master::left(100), 1, &flat);
    assert!(close(coin.moved[1], -2.0));
}

#[test]
fn a_falling_particle_never_goes_backwards() {
    let mut coin = Prim::new(Shape::ParticleGravity3d, "");
    (coin.speed, coin.accel) = (0.5, -1.0);
    coin.tick_on(&mut Master::left(100), 1, &flat);
    assert!(close(coin.speed, 0.0) && close(coin.moved[2], 0.0));
}

#[test]
fn a_rising_particle_is_not_thrown_back() {
    let mut coin = Prim::new(Shape::ParticleGravity3d, "");
    // Under the ground but going up: left alone.
    coin.put.1 = 5.0;
    coin.grav_speed = -1.0;
    coin.tick_on(&mut Master::left(100), 1, &flat);
    assert!(close(coin.grav_speed, -1.0) && close(coin.moved[1], 1.0));
}

#[test]
fn the_frames_of_a_sprite_come_every_so_many_ticks_and_start_over() {
    let mut flame = Prim::new(Shape::Particle3d, "");
    flame.anim_speed = 2;
    let mut shown = Vec::new();
    for _ in 0..8 {
        flame.tick_on(&mut Master::left(100), 3, &flat);
        shown.push(flame.motion);
    }
    // The next frame comes with the very first tick, as the client counts.
    assert_eq!(shown, [1, 1, 2, 2, 0, 0, 1, 1]);
}

#[test]
fn a_sprite_that_does_not_start_over_ends_after_its_last_frame() {
    let mut splash = Prim::new(Shape::Particle3d, "");
    (splash.anim_speed, splash.repeats) = (1, false);
    let mut lived = 0;
    while splash.tick_on(&mut Master::left(100), 4, &flat) {
        lived += 1;
    }
    // Frames 1, 2 and 3 are shown; the tick that would bring a fifth is its last.
    assert_eq!(lived, 3);
}

#[test]
fn a_sprite_whose_frames_are_not_known_stays_on_its_first() {
    let mut flame = Prim::new(Shape::Particle3d, "");
    flame.anim_speed = 1;
    assert!(flame.tick(100) && flame.tick(100));
    assert_eq!(flame.motion, 0);
}

#[test]
fn a_trail_follows_the_particle_smaller_and_fainter() {
    // As a spark of a blow is: drawn three times.
    let mut spark = Prim::new(Shape::Particle3d, "");
    (spark.segments, spark.speed, spark.size, spark.alpha) = (3, 1.0, 1.2, 240.0);
    spark.put = (2.0, -10.0);
    spark.tick_on(&mut Master::left(100), 1, &flat);
    let seen = spark.seen();
    assert_eq!(seen.len(), 3);
    // Where it is now; the older two are still where the client begins them, at the effect's own place.
    assert_eq!(seen[0], Seen { at: [2.0, 10.0, 1.0], scale: 1.2, alpha: 240.0 });
    assert_eq!((seen[1].at, seen[2].at), ([0.0; 3], [0.0; 3]));
    spark.tick_on(&mut Master::left(100), 1, &flat);
    spark.tick_on(&mut Master::left(100), 1, &flat);
    let seen = spark.seen();
    assert_eq!((seen[0].at, seen[1].at, seen[2].at), ([2.0, 10.0, 3.0], [2.0, 10.0, 2.0], [2.0, 10.0, 1.0]));
    // The second of three is a sixth smaller and a third fainter, the third a third smaller and two thirds fainter.
    assert!(close(seen[1].scale, 1.0) && close(seen[1].alpha, 160.0));
    assert!(close(seen[2].scale, 0.8) && close(seen[2].alpha, 80.0));
}

#[test]
fn a_flickering_particle_runs_up_and_down_until_it_fades_out() {
    // As a spark of Blessing is: a tenth of its most a tick, up and back down. Ten such steps fall a hair short
    // of the most in the client's arithmetic too, so each way takes eleven ticks.
    let mut spark = Prim::new(Shape::Particle3d, "");
    (spark.duration, spark.flickers, spark.alpha, spark.fade_out) = (80, true, 0.0, 64);
    spark.alpha_speed = spark.max_alpha / 10.0;
    let mut alphas = Vec::new();
    for _ in 0..80 {
        spark.tick(100);
        alphas.push(spark.alpha);
    }
    assert!(close(alphas[10], 254.0) && close(alphas[21], 0.0) && close(alphas[32], 254.0));
    // At its time to fade it stops flickering and is gone by its last tick.
    assert!(!spark.flickers);
    assert!(alphas[64] < alphas[63] && close(alphas[79], 0.0));
}

/// A bolt as `SoulStrike` launches it at a place 40 to the north: turned to it, 1 a tick, drawing back 3.5.
fn soul_bolt(roll: f32) -> Prim {
    let mut bolt = Prim::new(Shape::ParticleSpline3d, "");
    bolt.duration = 40;
    bolt.longitude = -faced(0.0, 40.0);
    bolt.speed = 40.0 / 40.0;
    (bolt.grav_speed, bolt.grav_accel) = (3.5, -(3.5 / 40.0) * 2.0);
    (bolt.lati_speed, bolt.lati_accel) = (-2.0, 2.0 / 40.0 * 2.0);
    bolt.roll = roll;
    bolt
}

#[test]
fn a_bolt_in_a_bow_draws_back_swings_out_and_comes_to_its_place() {
    let mut bolt = soul_bolt(0.0);
    assert!(bolt.shows_sprite());
    bolt.tick(100);
    // Its first tick: 1 less 3.325 along the way, so back from the place it flies to; and 2 out to the side:
    // with no roll, straight up.
    assert!(close(bolt.moved[2], -2.325) && close(bolt.moved[0], 0.0));
    assert!(close(bolt.aside[1], 2.0) && close(bolt.aside[0], 0.0) && close(bolt.aside[2], 0.0));
    bolt.tick(100);
    // The swing slows by a tenth a tick.
    assert!(close(bolt.aside[1], 3.9));
    let mut highest = 0f32;
    for _ in 2..40 {
        bolt.tick(100);
        highest = highest.max(bolt.aside[1]);
    }
    // It swings out to 21 and is nearly back at the end, 3.5 past the place it was sent to.
    assert!(close(highest, 21.0));
    assert!(close(bolt.aside[1], 2.0));
    assert!(close(bolt.moved[2], 43.5) && close(bolt.moved[0], 0.0));
}

#[test]
fn a_bolt_rolled_a_quarter_swings_out_level() {
    // Flying north, the client's forward is south (its speed takes it against it); a quarter roll about that
    // puts the swing level, to the west, and three quarters to the east.
    for (roll, east) in [(90.0, -3.9), (270.0, 3.9)] {
        let mut bolt = soul_bolt(roll);
        bolt.tick(100);
        bolt.tick(100);
        assert!(close(bolt.aside[0], east) && close(bolt.aside[1], 0.0) && close(bolt.aside[2], 0.0), "{roll}: {:?}", bolt.aside);
    }
    // Where it shows is its flight and its swing together.
    let mut bolt = soul_bolt(90.0);
    bolt.put.1 = -10.0;
    bolt.tick(100);
    bolt.tick(100);
    let seen = bolt.seen();
    assert!(close(seen[0].at[0], -3.9) && close(seen[0].at[1], 10.0));
}

#[test]
fn a_spark_in_orbit_stands_its_radius_out_and_rises_slowing() {
    // As a spark of the small healing: put 3 out to the east, with a radius of 3 that way besides.
    let mut spark = Prim::new(Shape::ParticleOrbit3d, "");
    (spark.radius, spark.longitude, spark.put.0) = (3.0, 90.0, 3.0);
    (spark.grav_speed, spark.grav_accel) = (-1.2, 0.2);
    spark.tick(100);
    // The place it was put at rises by its speed, changed first; it stands its radius out from there.
    let seen = spark.seen()[0];
    assert!(close(seen.at[0], 6.0) && close(seen.at[1], 1.0) && close(seen.at[2], 0.0));
    spark.tick(100);
    assert!(close(spark.seen()[0].at[1], 1.8));
    // Turning, it goes round the place it was put at; its radius changes by its speed.
    (spark.long_speed, spark.radius_speed) = (90.0, 1.0);
    spark.tick(100);
    let seen = spark.seen()[0];
    assert!(close(seen.at[0], 3.0) && close(seen.at[2], -4.0));
}

#[test]
fn a_spark_in_orbit_takes_a_new_speed_of_its_radius_at_the_tick_it_was_given() {
    // As a spark of `Sphere`: out by 0.15 a tick, and from the tick after its 95th back in as fast.
    let mut spark = Prim::new(Shape::ParticleOrbit3d, "");
    (spark.duration, spark.radius, spark.radius_speed) = (300, 0.1, 0.15);
    spark.changes = Some(crate::prim::Change { speed: -0.15, accel: 0.0, ..crate::prim::Change::at(95) });
    (0..96).for_each(|_| drop(spark.tick(300)));
    assert!(close(spark.radius, 0.1 + 96.0 * 0.15));
    spark.tick(300);
    assert!(close(spark.radius, 0.1 + 95.0 * 0.15));
}
