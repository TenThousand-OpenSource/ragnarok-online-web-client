//! What falls on the one an effect is on from high above, each effect its function of `CRagEffect` with the
//! same figures:
//!
//! - `EF_RAIN_PARTICLE2` (944): the stones of the elemental's Stone Rain (`TexturedFloatingDust`; the kind
//!   157).

use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 1] = ["stone_part.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty; Stone Rain's comes at its second tick, in `run`); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        944 => (100, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if effect != 944 {
        return;
    }
    if course.tick == 1 {
        course.sound("effect/매직 크래쉬.wav");
    }
    if course.tick % 10 == 0 {
        prims.push(floating_dust("stone_part.bmp", course));
    }
}

/// `CRagEffect::TexturedFloatingDust`, every tenth tick: four squares of the kind 157 that stay in the world
/// where they are launched, each 7 to 10 from its middle to a corner, up to 32 from the place any way round
/// it and 60 to 80 over it, by lot; it falls 2.5 to 3 a tick, white, pale red or red by lot.
fn floating_dust(picture: &'static str, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::GlowSquare, picture, false, course);
    prim.parts.extend((0..4).map(|_| {
        let mut part = Part { tall: lot(51) * 0.01 + 2.5, out: lot(31) * 0.1 + 7.0, round: lot(360) as i32, ..Part::default() };
        part.heights[0] = lot(33);
        part.at[0][1] = lot(21) - 80.0;
        part.flags[0] = lot(3) as i8;
        part
    }));
    prim
}
