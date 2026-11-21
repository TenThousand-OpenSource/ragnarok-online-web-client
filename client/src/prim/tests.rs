//! The tests of the client's effect primitives: how each kind moves a tick and how it is shaped.

use super::*;

use crate::prim::testing::close;

#[test]
fn a_primitive_lives_its_ticks_and_fades_to_nothing() {
    let mut prim = Prim::new(Shape::Circle2d, "");
    (prim.duration, prim.fade_out) = (10, 5);
    let mut lived = 0;
    while prim.tick(100) {
        lived += 1;
    }
    // It is told to go on for its ten ticks, and the eleventh is its last.
    assert_eq!(lived, 10);
    // From its fifth tick it fades evenly, and is gone by its last.
    assert!(close(prim.alpha, 0.0));
}

#[test]
fn a_primitive_that_would_outlive_its_effect_fades_with_it() {
    let mut prim = Prim::new(Shape::Circle2d, "");
    prim.duration = 100;
    // Four ticks of the effect left: it loses a quarter of what it has left each tick.
    prim.tick(4);
    assert!(close(prim.alpha, 254.0 * 0.75));
}

#[test]
fn a_flat_picture_stops_changing_size_when_it_settles() {
    let mut prim = Prim::new(Shape::Texture2d, "");
    (prim.width, prim.width_speed, prim.settles_at) = (10.0, -1.0, Some(3));
    for _ in 0..3 {
        prim.tick(100);
    }
    assert!(close(prim.width, 7.0));
    for _ in 0..5 {
        prim.tick(100);
    }
    assert!(close(prim.width, 7.0));
}

#[test]
fn a_ring_on_the_screen_is_ten_pieces_of_two_triangles() {
    let mut ring = Prim::new(Shape::Circle2d, "");
    (ring.radius, ring.size) = (10.0, 2.0);
    let corners = ring.corners();
    assert_eq!(corners.len(), 10 * 6);
    // Its first corner is on the inner edge, to the right of the middle.
    assert!(close(corners[0][0], 8.0) && close(corners[0][1], 0.0));
    // Every corner is between the inner edge and the outer.
    assert!(corners.iter().all(|corner| (7.99..=10.01).contains(&corner[0].hypot(corner[1]))));
}

#[test]
fn a_band_lying_flat_has_its_far_ring_above_its_near_one() {
    let mut band = Prim::new(Shape::Cylinder3d, "");
    (band.inner, band.outer, band.height) = (5.0, 6.0, 3.0);
    band.put.1 = -10.0;
    let corners = band.band([100.0, 20.0, 200.0]);
    assert_eq!(corners.len(), 10 * 6);
    // The near ring is 10 over the place, the far ring its height above that, each around the place.
    let (near, far) = (corners[0].0, corners[1].0);
    assert!(close(near[1], 30.0) && close(far[1], 33.0));
    assert!(close((near[0] - 100.0).hypot(near[2] - 200.0), 5.0));
    assert!(close((far[0] - 100.0).hypot(far[2] - 200.0), 6.0));
}

#[test]
fn a_band_stood_up_goes_the_way_it_is_turned() {
    // As the ring of a blow is: tipped up, and turned to face south (the one struck looks south: 180 - 0).
    let mut band = Prim::new(Shape::Cylinder3d, "");
    (band.latitude, band.longitude, band.speed) = (-90.0, 180.0, 1.0);
    band.tick(100);
    // South is towards smaller north; it has gone one that way and not up.
    assert!(close(band.moved[0], 0.0) && close(band.moved[1], 0.0) && close(band.moved[2], -1.0));
}

#[test]
fn a_band_grows_no_higher_than_it_may() {
    // As the column of `Enhance`: it grows by 2 a tick, to 50 at the most (any other to 100).
    let mut band = Prim::new(Shape::Cylinder3d, "");
    (band.height_speed, band.max_height) = (2.0, 50.0);
    (0..30).for_each(|_| drop(band.tick(100)));
    assert!(close(band.height, 50.0));
    let mut band = Prim::new(Shape::Cylinder3d, "");
    band.height_speed = 2.0;
    (0..60).for_each(|_| drop(band.tick(100)));
    assert!(close(band.height, 100.0));
}

#[test]
fn a_ring_in_the_world_widens_and_is_filled_until_it_is_wider_than_its_inner_size() {
    // As the rings of Magnum Break are: lying flat, widening as they slow.
    let mut ring = Prim::new(Shape::Circle3d, "");
    (ring.latitude, ring.radius_speed, ring.radius_accel, ring.inner) = (90.0, 2.0, -0.5, 3.0);
    ring.tick(100);
    // Its speed changes before it widens; narrower than its inner size, it is filled to its middle.
    assert!(close(ring.radius, 1.5) && close(ring.size, 1.5));
    ring.tick(100);
    ring.tick(100);
    assert!(close(ring.radius, 3.0) && close(ring.size, 3.0));
    (ring.radius_speed, ring.radius_accel) = (7.0, 0.0);
    ring.tick(100);
    assert!(close(ring.radius, 10.0) && close(ring.size, 3.0));
    let corners = ring.solid([100.0, 20.0, 200.0]);
    assert_eq!(corners.len(), 10 * 6);
    // It lies on the ground about its place, from 7 out to 10; its first corner is on the inner edge, east.
    assert!(corners.iter().all(|(at, _)| close(at[1], 20.0) && (6.99..=10.01).contains(&(at[0] - 100.0).hypot(at[2] - 200.0))));
    assert!(close(corners[0].0[0], 107.0) && close(corners[0].0[2], 200.0) && corners[0].1 == [0.0, 1.0]);
    assert!(close(corners[2].0[0], 110.0) && corners[2].1 == [0.0, 0.0]);
}

#[test]
fn a_ring_with_the_first_pattern_is_a_disc() {
    let mut disc = Prim::new(Shape::Circle3d, "");
    (disc.first_pattern, disc.latitude, disc.radius, disc.inner) = (true, 90.0, 10.0, 0.0);
    disc.tick(100);
    assert!(close(disc.size, 10.0));
    // Its inner edge is its middle.
    let corners = disc.solid([0.0; 3]);
    assert!(close(corners[0].0[0], 0.0) && close(corners[0].0[2], 0.0));
}

#[test]
fn a_ball_swells_turns_and_is_closed_all_round() {
    let mut ball = Prim::new(Shape::Sphere3d, "");
    (ball.radius_speed, ball.radius_accel, ball.long_speed) = (2.0, -1.0, 3.0);
    ball.tick(100);
    ball.tick(100);
    assert!(close(ball.radius, 1.0) && close(ball.longitude, 6.0));
    ball.radius = 5.0;
    let corners = ball.solid([10.0, 20.0, 30.0]);
    // Five rows of ten pieces, each two triangles, every corner its radius from the middle.
    assert_eq!(corners.len(), 5 * 10 * 6);
    assert!(corners.iter().all(|(at, _)| close(((at[0] - 10.0).powi(2) + (at[1] - 20.0).powi(2) + (at[2] - 30.0).powi(2)).sqrt(), 5.0)));
    // The first row starts at the bottom with the top of the picture, the last ends at the top.
    assert!(close(corners[0].0[1], 15.0) && corners[0].1 == [0.0, 0.0]);
    let last = corners[corners.len() - 3];
    assert!(close(last.0[1], 25.0) && close(last.1[0], 1.0) && close(last.1[1], 1.0));
}

#[test]
fn crossed_pictures_stood_up_rise_and_stand_on_end() {
    // As the streaks of Increase Agility are: tipped a quarter turn, which makes their forward the way up.
    let mut streak = Prim::new(Shape::CrossTexture3d, "");
    (streak.latitude, streak.speed, streak.accel, streak.width, streak.height) = (90.0, 1.0, 0.5, 4.0, 0.5);
    (streak.put.0, streak.deep) = (3.0, 2.0);
    streak.tick(100);
    // Its speed changes before it goes.
    assert!(close(streak.moved[0], 0.0) && close(streak.moved[1], 1.5) && close(streak.moved[2], 0.0));
    let corners = streak.solid([100.0, 20.0, 200.0]);
    assert_eq!(corners.len(), 2 * 6);
    let (first, second) = (&corners[..6], &corners[6..]);
    // Both are its width up and down from where it is; one is its height to the east and west, the other to
    // the north and south.
    assert!(corners.iter().all(|(at, _)| close((at[1] - 21.5).abs(), 4.0)));
    assert!(first.iter().all(|(at, _)| close((at[0] - 103.0).abs(), 0.5) && close(at[2], 202.0)));
    assert!(second.iter().all(|(at, _)| close(at[0], 103.0) && close((at[2] - 202.0).abs(), 0.5)));
    // Tipped the other way, as those of Decrease Agility, it sinks.
    let mut sinking = Prim::new(Shape::CrossTexture3d, "");
    (sinking.latitude, sinking.accel) = (-90.0, 0.25);
    sinking.tick(100);
    sinking.tick(100);
    assert!(close(sinking.moved[1], -0.75));
}

#[test]
fn crossed_pictures_stop_growing_past_the_most_they_may_be() {
    let mut picture = Prim::new(Shape::CrossTexture3d, "");
    (picture.height, picture.height_speed) = (99.0, 2.0);
    picture.tick(100);
    picture.tick(100);
    assert!(close(picture.height, 101.0));
}
