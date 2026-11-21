//! What a script of an NPC sets a scene with (`specialeffect`), each in its arm of `CRagEffect::OnProcess`
//! itself (`OnProcess3` for the last):
//!
//! - `EF_CONE` (71): a spark that winds down from 30 over the place in a spiral that widens, a cone; one at
//!   the first tick and one at the 130th;
//! - `EF_FIREIVY` (118): the leaf of Fire Ivy, a picture turned to the camera that rises slowly;
//! - `EF_LEVEL99_4` (362): sixteen blue lights that rise wandering about the one it is on, over and over,
//!   until it is ended (`FreezingCircle` with its kind 1; the kind 29 of the primitives);
//! - `EF_SCREEN_QUAKE` (563): the screen shaken every fifth tick of the first hundred (`MM_QUAKE`); it draws
//!   nothing.
//!
//! The lights follow the one they are on (`FreezingCircle` takes its parent's place every tick); the arms of
//! the others never take it, and they stay where they began.

use super::launched::{PARTICLE1, of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[&SPRITES] };

/// The pictures the effects here are drawn with, among the client's pictures of effects, and the sprites.
const PICTURES: [&str; 2] = ["fire_ivy.bmp", "whitelight.tga"];
const SPRITES: [&str; 1] = [PARTICLE1];

/// How many ticks an effect of this file lives and the sound it starts with (none: empty); nothing for any
/// other effect. The times are the client's `EffectLifetime`, but that the arm of `EF_CONE` sets its effect's
/// `duration` to 200 (the table has 250).
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        71 => (200, ""),
        118 => (250, "effect/wizard_fire_ivy.wav"),
        362 => (99_999_999, ""),
        563 => (200, ""),
        _ => return None,
    })
}

/// Whether an effect stays where it began: all but the lights.
fn stays(effect: u32) -> bool {
    matches!(effect, 71 | 118 | 563)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        71 if tick % 130 == 0 => prims.push(cone_spark()),
        118 if tick == 0 => prims.push(ivy(course.lasts)),
        362 if tick == 0 => prims.push(rising_lights(course)),
        563 if tick < 100 && tick % 5 == 0 => course.quake = Some(0),
        _ => {}
    }
}

/// `EF_CONE`: a spark of `particle1.spr` (`PP_3DPARTICLEORBIT`), a fifth larger than its sprite, that goes
/// round a point 30 over the place, 15 degrees a tick and slowing a little, ever further out (0.2 a tick)
/// as the point sinks faster and faster. It is told 300 ticks, more than its effect lives, and so fades with
/// it from the first.
fn cone_spark() -> Prim {
    let mut spark = Prim::new(Shape::ParticleOrbit3d, PARTICLE1);
    spark.duration = 300;
    (spark.radius, spark.radius_speed) = (0.1, 0.2);
    (spark.grav_speed, spark.grav_accel) = (0.1, 0.005);
    (spark.longitude, spark.long_speed, spark.long_accel) = (2.0, 15.0, -0.01);
    spark.size = 1.2;
    (spark.put, spark.deep) = ((sine(spark.longitude) * spark.radius, -30.0), cosine(spark.longitude) * spark.radius);
    spark
}

/// `EF_FIREIVY`: `fire_ivy.bmp` turned to the camera (`pattern & 0x40`), 3.5 to each side, from 5 over the
/// place, that rises 25 over the time the effect lives (`pattern & 0x100`, its speed along the upright). Its
/// light is added to what is behind it (a `renderFlag` with 2; the client adds it whole, whatever its alpha).
fn ivy(lasts: i32) -> Prim {
    let mut leaf = Prim::new(Shape::Texture3d, "fire_ivy.bmp");
    leaf.duration = lasts;
    (leaf.faces_camera, leaf.by_axes, leaf.adds) = (true, true, true);
    (leaf.width, leaf.height, leaf.put.1) = (3.5, 3.5, -5.0);
    leaf.speed3[1] = -(25.0 / leaf.duration as f32);
    leaf
}

/// `CRagEffect::FreezingCircle` with its kind 1: a primitive of the kind 29 with `whitelight.tga`, four parts
/// of four lights each, numbered 1 (blue), 2.4 from their middle to a corner, that rise 0.15 a tick; each
/// light begins at the middle, up to 99 under the feet, and each of its turns anywhere, by lot.
fn rising_lights(course: &Course) -> Prim {
    let mut lights = of_parts(Tei::FloatingMotes, "whitelight.tga", true, course);
    lights.parts.extend((0..4).map(|_| {
        let mut four = part();
        (four.tall, four.out, four.flags[0]) = (0.15, 2.4, 1);
        // In the client's order: `vecBPre`, `vecBNow`, `vecTPre`, `vecTNow`, each before its four turns.
        for (light, place) in [2, 0, 3, 1].into_iter().enumerate() {
            four.at[place][1] = lot(100);
            four.heights[light * 4..light * 4 + 4].iter_mut().for_each(|turn| *turn = lot(360));
        }
        four
    }));
    lights
}
