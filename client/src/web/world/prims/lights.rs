//! Lights round the one an effect is on: a column, a ball, sparks that go round or are thrown up. Each with
//! the figures of its function of `CRagEffect`:
//!
//! - `EF_ENHANCE` (9): a column of light that grows round the one it is on, and streaks that rise beside it
//!   (`Enhance`);
//! - `EF_BARRIER` (63): a ball of fire round the one it is on (in `CRagEffect::OnProcess` itself);
//! - `EF_SPHERE` (72): sparks that wind out from over the one it is on and back in, sinking (`Sphere`);
//! - `EF_REMOVETRAP` (100): the sparks thrown up from a trap taken away (`RemoveTrap`);
//! - `EF_ATTACKENERGY2` (438): the dome of blue light before one off whom Kaite throws a spell back
//!   (`AttackEnergy2`; `PP_ATTACKENERGY2`).
//!
//! The client has an option to show fewer effects (`TT_MIN_EFFECT_ON_OFF`); what is here is what it shows with
//! the option off.
//!
//! `Sphere` takes its parent's place every tick, and the primitives of `Enhance` go with their effect, whose
//! place the client's actor keeps at its own; the arm of `EF_BARRIER` and `RemoveTrap` never take it: those
//! two stay where they began.
//!
//! Not as the client has it: the dome of `EF_ATTACKENERGY2` points the way the one it is on faced as the
//! effect began, where the client turns it with that one at every tick (`prim/tei/domes.rs`). And a spark of
//! `Sphere` that would outlive its effect begins to fade as the others do here, to nothing by the effect's end
//! (`Prim::fade` sets the speed anew every tick); the client sets that speed once, by the time the effect then
//! thinks it has left, which is about twice what it has, so that its sparks are still at half their light as
//! the effect ends.

use super::launched::{PARTICLE1, of_parts, part};
use super::*;
use crate::prim::{Change, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 4] = ["ring_yellow.tga", "alpha_down.tga", "대폭발.tga", "cloud11.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`, or what its function makes of
/// it) and the sound it starts with (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        9 => (80, ""),
        63 => (250, ""),
        // `Sphere` gives itself 500 ticks less the tick it is at, at every tenth tick: its 251st is its last.
        72 => (251, ""),
        100 => (70, "effect/hunter_removetrap.wav"),
        438 => (100, "effect/t_마법반사.wav"),
        _ => return None,
    })
}

/// Whether an effect stays where it began: the ball and the sparks of a trap, whose functions do not take
/// their parent's place.
fn stays(effect: u32) -> bool {
    matches!(effect, 63 | 100)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        9 => enhance(tick, course.lasts, prims),
        63 if tick == 0 => prims.push(barrier()),
        72 => {
            // What its sparks take for the time the effect has left: the time it gave itself last.
            course.shared.left = 500 - tick / 10 * 10 - tick;
            if tick % 10 == 0 {
                prims.push(sphere_spark());
            }
        }
        100 if matches!(tick, 0 | 7) => prims.extend((0..12).map(|_| trap_spark())),
        438 if matches!(tick, 0 | 4) => prims.push(dome(course)),
        _ => {}
    }
}

/// `CRagEffect::Enhance`. At its first tick a yellow ring lying on the ground and a column of light round
/// the one it is on, 6 out, that grows by 2 a tick, slowing, to 50 high; both at half their light, fading over
/// the last third of the effect. Every fifth tick a streak of light 2 to 6 from that one, any way round,
/// 7 long and thin, that rises at 2 a tick for 50 ticks.
///
/// (The ring is given a width, `size`, and neither a radius nor a speed of widening: `Prim3DCircle` makes its
/// width anew every tick, no more than its radius, so nothing of it shows. It is launched as the client
/// launches it.)
fn enhance(tick: i32, lasts: i32, prims: &mut Vec<Prim>) {
    if tick == 0 {
        let mut ring = Prim::new(Shape::Circle3d, "ring_yellow.tga");
        (ring.duration, ring.arc, ring.size, ring.latitude) = (lasts, 22.5, 7.0, 90.0);
        let mut column = Prim::new(Shape::Cylinder3d, "alpha_down.tga");
        column.duration = lasts;
        (column.outer, column.inner) = (6.0, 6.0);
        column.height_speed = 2.0;
        column.height_accel = -(column.height_speed / column.duration as f32) / 1.5;
        column.max_height = 50.0;
        for mut prim in [ring, column] {
            prim.alpha = 128.0;
            prim.fade_out = prim.duration - prim.duration / 3;
            prims.push(prim);
        }
    }
    if tick % 5 != 0 {
        return;
    }
    let mut streak = Prim::new(Shape::CrossTexture3d, "alpha_down.tga");
    streak.duration = 50;
    let (round, away) = (lot(360), lot(5) + 2.0);
    (streak.put.0, streak.deep) = (sine(round) * away, cosine(round) * away);
    (streak.latitude, streak.speed) = (90.0, 2.0);
    (streak.width, streak.height) = (7.0, 0.25);
    prims.push(streak);
}

/// `EF_BARRIER` in `CRagEffect::OnProcess`: a ball of `대폭발.tga`, 13.5 from its middle 10 above the feet,
/// faint (100 of 255), for the 250 ticks of the effect. (It is turned 2 degrees round and never turns on.)
fn barrier() -> Prim {
    let mut ball = Prim::new(Shape::Sphere3d, "대폭발.tga");
    (ball.duration, ball.radius, ball.put.1) = (250, 13.5, -10.0);
    (ball.longitude, ball.alpha) = (2.0, 100.0);
    ball
}

/// `CRagEffect::Sphere`, at every tenth tick: a spark of `particle1.spr` 30 above the one it is on, that goes
/// round it at 8 degrees a tick, slowing, out by 0.15 a tick for 95 ticks and then back in as fast (and out
/// again on the other side), sinking 0.15 a tick; a little larger than the sprite, for 300 ticks or as long
/// as the effect lives.
fn sphere_spark() -> Prim {
    let mut spark = Prim::new(Shape::ParticleOrbit3d, PARTICLE1);
    (spark.duration, spark.size) = (300, 1.2);
    (spark.radius, spark.radius_speed, spark.grav_speed) = (0.1, 0.15, 0.15);
    (spark.longitude, spark.long_speed, spark.long_accel) = (2.0, 8.0, -0.01);
    spark.put = (sine(spark.longitude) * spark.radius, -30.0);
    spark.deep = cosine(spark.longitude) * spark.radius;
    spark.changes = Some(Change { speed: -0.15, accel: 0.0, ..Change::at(95) });
    spark
}

/// A spark of `CRagEffect::RemoveTrap`, which throws twelve at its first and at its eighth tick: of
/// `particle1.spr` at four fifths of its size, it starts up from the place at 2 a tick, slowing by 0.08 to
/// 0.09 a tick so that it comes down again, and goes out from the place any way at 0.2 a tick; it fades over
/// the last 15 of its 50 ticks.
fn trap_spark() -> Prim {
    let mut spark = Prim::new(Shape::ParticleOrbit3d, PARTICLE1);
    (spark.duration, spark.size, spark.fade_out) = (50, 0.8, 35);
    spark.radius_speed = 0.2;
    spark.grav_speed = -2.0;
    spark.grav_accel = lot(10) / 1000.0 + 0.08;
    spark.longitude = lot(360);
    spark
}

/// `CRagEffect::AttackEnergy2` with `cloud11.tga`, at its first and at its fifth tick: four bands of blue
/// light on a dome 8 wide before the one it is on, 8 over the ground, each begun 22 degrees further from the
/// dome's tip than the one before and an eighth of a round further on (`prim/tei/domes.rs`).
fn dome(course: &Course) -> Prim {
    let mut dome = of_parts(Tei::AttackEnergy2, "cloud11.tga", true, course);
    dome.size = 4.0;
    // (`rotStart`: the whole degrees of the way the one it is on faces, half a turn on.)
    let turned = (course.facing + 180.0).trunc().rem_euclid(360.0) as i32;
    dome.parts.extend((0..4).map(|index| {
        let mut band = part();
        (band.tall, band.out, band.round, band.turned) = (8.0, 8.0, 360, turned);
        (band.heights[0], band.heights[1], band.rise) = (10.0, (index * 22) as f32, index * 45);
        band
    }));
    dome
}
