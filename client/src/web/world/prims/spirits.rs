//! The spirit spheres about a monk and the coins about a gunslinger: `EF_CHOOKGI` (228), `EF_CHOOKGI2`
//! (504, a baby monk's) and `EF_CHOOKGI3` (629), each `CRagEffect::ElementalOrbit` and nothing else (its
//! kinds 0, 1 and 3, from the arms of `CRagEffect::OnProcess` and `OnProcess3`).
//!
//! The effect is told how many spheres it is as it is launched (`AM_SETPOS`, the first figure:
//! `Burst::tell`), launches them all at its first tick, and lives until it is ended
//! (`world/spirits.rs`): the client's `EffectLifetime` of each is 999999. It starts no sound.
//!
//! (The client's `CEnergyOrbEffect` is another thing, of later content: `crate::spirits` says what.)

use super::*;
use crate::prim::Tei;
use crate::spirits::{EFFECTS, Sphere, spheres};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The picture the spheres are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 1] = ["thunder_center.bmp"];

/// How many ticks the effect lives (the client's `EffectLifetime`) and the sound it starts with; nothing for
/// any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        228 | 504 | 629 => (999_999, ""),
        _ => return None,
    })
}

/// One tick of the effect; nothing for any other effect. `ElementalOrbit` does nothing but at the first
/// tick (the effect itself takes its parent's place every tick, as every effect that does not stay does
/// here).
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 0 && EFFECTS.contains(&effect) {
        prims.extend(spheres(effect, course.shared.aim[0]).into_iter().map(|sphere| orb(sphere, course)));
    }
}

/// One sphere as `ElementalOrbit` launches it (the kind 32, one part): where and how `crate::spirits` says,
/// its three turns, their marks and its swell at places by lot, showing nothing yet.
fn orb(sphere: Sphere, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::DriftingOrbs, PICTURES[0], true, course);
    let mut part = launched::part();
    part.heights[..6].iter_mut().for_each(|turn| *turn = lot(360));
    (part.heights[6], part.heights[7], part.heights[8]) = (sphere.round, sphere.high, lot(360));
    (part.heights[10], part.out, part.flags[0]) = (if sphere.near { 1.0 } else { 0.0 }, sphere.out, sphere.flag);
    prim.parts.push(part);
    prim
}
