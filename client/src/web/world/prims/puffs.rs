//! `EF_PEONG` (411): the puff of one who lands from a leap (Lope; the client also launches it for one who
//! jumps to a place, `Contents/Actors/RenderObject.cpp`). At its 36th tick, all at once:
//!
//! - thirty small white lights that rise from round the place, wandering (`WhiteLightTrail` with its kind 0;
//!   `PP_PEONGUP`);
//! - sixty-four scraps of cloud thrown out level every way, four pictures a quarter of the round, that lift a
//!   little and come down as they go (`TexturedScatterArc`; `PP_PEONG`).
//!
//! Neither function takes the place of the one the effect is on, so all of it stays where the effect began.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effect is drawn with, among the client's pictures of effects.
const PICTURES: [&str; 4] = ["whitelight.tga", "peong1.tga", "peong2.tga", "peong3.tga"];

/// How many ticks the effect lives (the client's `EffectLifetime`); it has no sound in its function. Nothing
/// for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        411 => (300, ""),
        _ => return None,
    })
}

/// The effect stays where it began.
fn stays(effect: u32) -> bool {
    effect == 411
}

/// One tick of the effect: everything at its 36th.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
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
