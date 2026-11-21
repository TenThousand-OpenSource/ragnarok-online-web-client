//! `EF_PEONG` (411): the puff of one who lands from a leap (Lope; the client also launches it for one who
//! jumps to a place, `Contents/Actors/RenderObject.cpp`). At its 36th tick, all at once:
//!
//! - thirty small white lights that rise from round the place, wandering (`WhiteLightTrail` with its kind 0;
//!   `PP_PEONGUP`);
//! - sixty-four scraps of cloud thrown out level every way, four pictures a quarter of the round, that lift a
//!   little and come down as they go (`TexturedScatterArc`; `PP_PEONG`).
//!
//! `EF_STORMKICK4` (462): Kaupe, a hundred and twenty small blue lights that rise from round the one it is on
//! (`WhiteLightTrail` with its kind 1, thirty times at the first tick).
//!
//! Neither function takes the place of the one the effect is on, so all of it stays where the effect began.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effect is drawn with, among the client's pictures of effects.
const PICTURES: [&str; 4] = ["whitelight.tga", "peong1.tga", "peong2.tga", "peong3.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`); neither starts a sound with its
/// first tick. Nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        411 => (300, ""),
        // (Its sound comes at its 16th tick, in `run`.)
        462 => (200, ""),
        _ => return None,
    })
}

/// Both effects stay where they began.
fn stays(effect: u32) -> bool {
    matches!(effect, 411 | 462)
}

/// One tick of the effect: everything at its 36th.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if effect == 462 {
        course.starts(&[(15, "effect/t_회피.wav")]);
        if course.tick == 0 {
            prims.extend((0..30).map(|_| blue_lights(course)));
        }
    }
    if effect != 411 || course.tick != 35 {
        return;
    }
    prims.extend((0..30).map(|_| light(course)));
    for quarter in 0..4 {
        prims.extend([1, 2, 3, 2].map(|picture| scraps(PICTURES[picture], quarter * 90, course)));
    }
}

/// `CRagEffect::WhiteLightTrail` with its kind 0: one light (`PP_PEONGUP`) 2.5 out from the place any way
/// round and up to 7 over it, 3 to 4 from its middle to a corner, that waits up to 25 ticks and rises 0.1 to
/// 0.25 a tick.
fn light(course: &Course) -> Prim {
    let mut prim = of_parts(Tei::PeongUp, PICTURES[0], true, course);
    let mut light = part();
    (light.tall, light.out, light.process) = (lot(16) * 0.01 + 0.1, lot(11) * 0.1 + 3.0, -(lot(26) as i32));
    let (up, way) = (lot(8), lot(360));
    light.at[2] = [cosine(way) * 2.5, -up, sine(way) * 2.5];
    light.heights[..4].copy_from_slice(&[lot(360), lot(360), lot(360), lot(360)]);
    light.flags[0] = 1;
    prim.parts.push(light);
    prim
}

/// `CRagEffect::WhiteLightTrail` with its kind 1: four lights (`PP_PEONGUP`), blue, each 3.5 out from the
/// place any way round and up to 7 over it, 2 to 2.5 from its middle to a corner, that waits up to 25 ticks
/// and rises 0.08 to 0.13 a tick.
fn blue_lights(course: &Course) -> Prim {
    let mut prim = of_parts(Tei::PeongUp, PICTURES[0], true, course);
    prim.parts.extend((0..4).map(|_| {
        let mut light = part();
        (light.tall, light.out, light.process) = (lot(11) * 0.005 + 0.08, lot(11) * 0.05 + 2.0, -(lot(26) as i32));
        let (up, way) = (lot(8), lot(360));
        light.at[2] = [cosine(way) * 3.5, -up, sine(way) * 3.5];
        light.heights[..4].copy_from_slice(&[lot(360), lot(360), lot(360), lot(360)]);
        light.flags[0] = 2;
        light
    }));
    prim
}

/// `CRagEffect::TexturedScatterArc`: four scraps (`PP_PEONG`) in the quarter of the round that begins at
/// `from`, each 4 out from the place the way it will go, 2.5 to 4.5 from its middle to a corner, that waits
/// up to 10 ticks and goes 0.03 a tick, lifting from 4 over the ground.
fn scraps(picture: &'static str, from: i32, course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Peong, picture, true, course);
    prim.parts.extend((0..4).map(|_| {
        let mut scrap = part();
        let out = lot(11) * 0.2 + 2.5;
        let way = (lot(90) as i32 + from) % 360;
        (scrap.out, scrap.process, scrap.rise, scrap.tall) = (out, -(lot(11) as i32), way, 0.03);
        scrap.at[2] = [cosine(way as f32) * 4.0, 0.0, sine(way as f32) * 4.0];
        scrap.at[0][1] = -4.0;
        scrap
    }));
    prim
}
