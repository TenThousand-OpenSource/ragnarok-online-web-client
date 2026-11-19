//! What follows a blow on the one struck: effects only the client's tables of what follows on the one struck
//! have (`skillFollowEffects`), or its actors launch as a blow comes (`CGameActor::ProcessSkillMessage`). Each
//! with the figures of its function of `CRagEffect`:
//!
//! - `EF_BOWLINGBASH` (73): Bowling Bash (`BowlingBash`);
//! - `EF_YUFITELHIT` (94), `EF_YUFITEL2` (452): the ball of lightning on one struck by Jupitel Thunder, and
//!   its dark twin (`YufitelHit`);
//! - `EF_BLITZBEAT` (115): Blitz Beat (`BlitzBeat`);
//! - `EF_GRIMTOOTHATK` (132): the spikes of stone of Grimtooth (`GrimToothAtk`);
//! - `EF_CURSEATTACK` (196): the sign of a curse over the one struck (in `CRagEffect::OnProcess` itself).
//!
//! Each takes its parent's place every tick: none stays where it began, and none is told a place.

use super::launched::{ahead, band, way};
use super::*;
use crate::prim::Change;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES, &BALL, &DARK_BALL], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 8] = [
    "magic_green.tga",
    "ring_blue.tga",
    "ring_yellow.tga",
    "stone.bmp",
    "pokjuk_d.bmp",
    "thunder_pang.bmp",
    "curse.bmp",
    "ac_center2.tga",
];

/// The five pictures the ball of lightning runs through, one a tick, and those of its dark twin.
const BALL: [&str; 5] = ["thunder_plazma_blast_a.bmp", "thunder_plazma_blast_b.bmp", "thunder_ball_d.bmp", "thunder_ball_e.bmp", "thunder_ball_f.bmp"];
const DARK_BALL: [&str; 5] = ["twirl_soft.bmp", "thunder_ball_b.bmp", "twirl_soft.bmp", "thunder_ball_c.bmp", "twirl_soft.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        73 => (250, ""),
        94 | 452 => (250, ""),
        // (`BlitzBeat` starts its two sounds itself, at its sixth and its thirty-first tick.)
        115 => (250, ""),
        // `GrimToothAtk` makes the effect last 1000 ticks, whatever the client's table of lifetimes has for it.
        132 => (1000, ""),
        196 => (150, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        73 => bowling_bash(way(course.facing), tick, prims),
        94 => thunder_hit(false, tick, prims),
        115 => blitz_beat(course, prims),
        132 if tick == 0 => prims.extend(grimtooth(lasts)),
        196 if tick == 0 => prims.push(curse()),
        452 => thunder_hit(true, tick, prims),
        _ => {}
    }
}

/// `CRagEffect::BowlingBash`: at its first and its sixth tick a blue band lying round the one who strikes,
/// both its rings widening as they slow, the second fainter and turned on by 100 degrees; with the first a
/// faint yellow ring on the ground that widens, slower and slower, and from its twentieth tick steadily (its
/// `changePoint`), fading from its 35th of 50. (The client draws the ring on the ground over everything,
/// whatever stands before it; here it is drawn as the other effects are.)
fn bowling_bash(way: f32, tick: i32, prims: &mut Vec<Prim>) {
    if tick != 0 && tick != 5 {
        return;
    }
    let mut band = Prim::new(Shape::Cylinder3d, "ring_blue.tga");
    band.duration = 20 - tick;
    band.longitude = way + (tick * 20) as f32;
    (band.outer, band.outer_speed, band.outer_accel) = (8.0, 0.5, -0.03);
    (band.inner, band.inner_speed, band.inner_accel) = (3.0, 0.5, -0.03);
    band.height = 3.5;
    band.alpha = (240 - tick * 7) as f32;
    band.fade_out = band.duration - band.duration / 2;
    prims.push(band);
    if tick == 0 {
        let mut ring = Prim::new(Shape::Circle3d, "ring_yellow.tga");
        ring.duration = 50;
        ring.latitude = 90.0;
        (ring.radius, ring.radius_speed) = (8.0, 0.7);
        ring.radius_accel = -(ring.radius_speed / 30.0);
        ring.changes = Some(Change { speed: 0.08, accel: 0.0, ..Change::at(20) });
        ring.inner = 5.0;
        (ring.alpha, ring.fade_out) = (45.0, 35);
        prims.push(ring);
    }
}

/// `CRagEffect::GrimToothAtk`: three spikes of stone around the one struck, 12 out, each leaning 15 degrees
/// from upright, pushed up from 40 under the ground at 3.5 a tick for ten ticks; they stand as long as the
/// effect and fade over its last 10 ticks. (The client draws them over everything, whatever stands before
/// them; here they are drawn as the other effects are.)
fn grimtooth(lasts: i32) -> Vec<Prim> {
    [(0.0, -12.0), (12.0, 6.0), (-12.0, 6.0)]
        .into_iter()
        .enumerate()
        .map(|(index, (east, north))| {
            let mut spike = Prim::new(Shape::QuadHorn3d, "stone.bmp");
            spike.duration = lasts;
            (spike.first_pattern, spike.settles_at) = (true, Some(10));
            (spike.put, spike.deep) = ((east, 40.0), north);
            (spike.size, spike.height) = (0.9, 25.0);
            (spike.longitude, spike.latitude) = (((6 - index) * 120) as f32, 75.0);
            (spike.speed, spike.accel) = (3.5, 0.001);
            spike.fade_out = spike.duration - 10;
            spike
        })
        .collect()
}

/// A picture that stands in the world turned to the camera, 5 above the feet, its light added to what is
/// behind it, as `YufitelHit` launches its two kinds. (The client adds it whole, whatever its alpha is — a
/// `renderFlag` of 2 — and over everything; here it is added by its alpha, as the other effects are.)
fn glow(picture: &'static str, duration: i32) -> Prim {
    let mut glow = Prim::new(Shape::Texture3d, picture);
    glow.duration = duration;
    (glow.faces_camera, glow.adds) = (true, true);
    glow.put.1 = -5.0;
    glow
}

/// `CRagEffect::YufitelHit`, plain or with its flag (`dark`: `EF_YUFITEL2`): every twentieth tick a flash that
/// swells from nothing by 2.5 a tick for ten ticks, fading; and at the tenth tick a ball 7.5 to each side that
/// runs through five pictures, one a tick, to the end of the effect (it is told 300 ticks, more than the
/// effect lives, and so fades with it from the first).
fn thunder_hit(dark: bool, tick: i32, prims: &mut Vec<Prim>) {
    if tick % 20 == 0 {
        let mut flash = glow(if dark { "pokjuk_d.bmp" } else { "thunder_pang.bmp" }, 10);
        (flash.width_speed, flash.height_speed) = (2.5, 2.5);
        flash.changes = Some(Change { width_speed: 0.0, height_speed: 0.0, width_accel: 0.0, height_accel: 0.0, ..Change::at(10) });
        flash.fade_out = 1;
        prims.push(flash);
    }
    if tick == 10 {
        let pictures: &'static [&'static str] = if dark { &DARK_BALL } else { &BALL };
        let mut ball = glow(pictures[0], 300);
        (ball.width, ball.height) = (7.5, 7.5);
        (ball.pictures, ball.anim_speed) = (pictures, 1);
        prims.push(ball);
    }
}

/// `EF_CURSEATTACK` in `CRagEffect::OnProcess`: the sign of a curse, `curse.bmp` 4 to each side, standing 30
/// above the feet of the one struck and turning about the upright by 4.5 degrees a tick (its `longSpeed`). (It
/// is told 9999 ticks, more than the effect lives, and so fades with it from the first.)
fn curse() -> Prim {
    let mut sign = Prim::new(Shape::Texture3d, "curse.bmp");
    sign.duration = 9999;
    sign.long_speed = 4.5;
    sign.put.1 = -30.0;
    (sign.width, sign.height) = (4.0, 4.0);
    sign
}

/// `CRagEffect::BlitzBeat`. At its first tick ten thin streaks of light, each begun 15 ahead of the one
/// struck (the way that one faces) from somewhere 3.5 around a place 7 above the feet, flying back faster and
/// faster, and from their tenth tick slowly (their `changePoint`); they come up in two ticks and fade over
/// their last two of 20. Its sounds at its sixth and its thirty-first tick. Then, after its 36th tick, every
/// third tick a small green ring for each hit it was told of, begun 2 along the way the one struck faces and 1
/// down from a place up to 2 to the east and somewhere 2 to 12 above the feet.
///
/// (The client counts the rings launched in one count for every effect of this kind at once,
/// `g_blitzBeatCount`; here each effect has its own, in its `step`. It draws the streaks over everything.)
fn blitz_beat(course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, facing) = (course.tick, course.facing.trunc());
    match tick {
        5 => course.sound("effect/hunter_blitzbeat_1st.wav"),
        30 => course.sound("effect/hunter_blitzbeat.wav"),
        0 => prims.extend((0..10).map(|_| blitz_streak(facing))),
        _ => {}
    }
    if tick <= 35 || tick % 3 != 0 || course.step[0] as i32 >= course.count {
        return;
    }
    course.step[0] += 1.0;
    let mut ring = band("magic_green.tga", 12, facing);
    ring.moved = ahead(facing, 1.0);
    // The client divides its lots as whole numbers.
    ring.put = (((lot(10) + 1.0) / 5.0).floor(), ((lot(20) + 1.0) / 2.0).floor() - 12.0);
    ring.speed = 0.4;
    ring.accel = -(ring.speed / ring.duration as f32) * 0.5;
    (ring.outer, ring.inner, ring.height) = (1.5, 0.5, 3.5);
    ring.fade_out = ring.duration - ring.duration / 2;
    prims.push(ring);
}

/// One of the ten streaks of `BlitzBeat`, turned the way the one struck faces.
fn blitz_streak(facing: f32) -> Prim {
    let mut streak = Prim::new(Shape::CrossTexture3d, "ac_center2.tga");
    streak.duration = 20;
    streak.longitude = facing;
    streak.moved = streak.forward(15.0);
    let round = lot(360);
    (streak.put, streak.deep) = ((sine(round) * 3.5, -7.0), cosine(round) * -3.5);
    (streak.speed, streak.accel) = (-1.2, -0.1);
    streak.changes = Some(Change { speed: -0.3, accel: 0.001, ..Change::at(10) });
    (streak.width, streak.height) = ((lot(40) + 20.0) / 10.0, 0.2);
    (streak.width_speed, streak.width_accel) = (0.1, -0.01);
    (streak.alpha, streak.max_alpha) = (0.0, 250.0);
    streak.alpha_speed = streak.max_alpha * 0.5;
    streak.fade_out = streak.duration - 2;
    streak
}
