//! Pictures over the one an effect is on that face the screen (the kinds 82 and 141 of the client's
//! primitives), each effect its function of `CRagEffect` with the same figures:
//!
//! - `EF_LIGHTSPHERE` (348): a ball of light 10 over the one it is on that comes up slowly (`TexturedAura`
//!   with its kind 0), and 320 thin spikes of blue light that stand out from it every way, each growing
//!   longer and shorter (`TexturedSparkle`, eighty times; the kind 81);
//! - `EF_HITTEXTURE` (554): the flash of a hit (`TexturedAura` with its kind 2);
//! - `EF_NPC_SLOWCAST` (676): the clock over one whose casting is slowed (`TexturedAura` with its kind 4);
//! - `EF_NC_ANAL` (786): Analyze, the target over the one looked into and the ring on the ground under it
//!   (`TexturedAura` with its kind 21, `TexturedSkillAura` with its type 5); the body flashes pale;
//! - `EF_STRETCH` (811): the four strokes of Body Painting, bars of light drawn across the one painted, one
//!   under the other (`TexturedDescendingRings`; the kind 141);
//! - `EF_BLACKBODY` (812): Deadly Infect, a dark ring on the ground (`TexturedSkillAura` with its type 6),
//!   and the body flashes black and white. From its 61st tick a character who is poisoned (the bit 1 of its
//!   health state, which the client keeps as `effectState`) is painted the pink of poison again, and that
//!   colour locked: the flashing wrote over it (`Touch::if_poisoned`, `world/ailments.rs`).

use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 7] = ["thunder_center.bmp", "white02.bmp", "freeze_a.bmp", "blast_mine##clock.bmp", "target3.bmp", "fashasha.tga", "freezing_stick.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        348 => (1000, ""),
        554 => (200, ""),
        // (Its sound comes at its tenth tick, in `run`.)
        676 => (300, ""),
        786 => (100, "effect/analyze.wav"),
        811 => (200, ""),
        812 => (100, ""),
        _ => return None,
    })
}

/// Whether an effect stays where it began: `TexturedSkillAura` never takes its parent's place, and Deadly
/// Infect is that function alone.
fn stays(effect: u32) -> bool {
    effect == 812
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        348 if tick == 0 => {
            prims.push(aura("thunder_center.bmp", 0, course));
            prims.extend((0..80).map(|_| sparkle(course)));
        }
        554 if tick == 0 => prims.push(aura("freeze_a.bmp", 2, course)),
        676 => {
            if tick == 0 {
                prims.push(aura("blast_mine##clock.bmp", 4, course));
            }
            if tick == 10 {
                course.sound("effect/ef_decagility.wav");
            }
        }
        786 => {
            // From the tenth to the 90th tick the body is pale and its light added at every even tick,
            // white at every odd one.
            if (10..=90).contains(&tick) {
                course.touch.flashes(tick, [250, 250, 250]);
            }
            if tick == 0 {
                prims.extend([launched::skill_aura(5), aura("target3.bmp", 21, course)]);
            }
        }
        811 if tick == 0 => prims.push(strokes(course)),
        812 => {
            // From the sixth to the 60th tick the body is white at every odd tick and black at every even.
            if (6..=60).contains(&tick) {
                let shade = if tick % 2 == 1 { 255 } else { 0 };
                course.touch.colour(shade, shade, shade);
            } else if tick >= 61 {
                course.touch.if_poisoned = Some([255, 128, 255]);
            }
            if tick == 0 {
                prims.push(launched::skill_aura(6));
            }
        }
        _ => {}
    }
}

/// `CRagEffect::TexturedAura`, for its kinds 0, 2, 4 and 21: one picture that faces the screen over the one the
/// effect is on (the kind 82), a square whose first corner is turned `round` degrees:
///
/// - 0: 5 to a corner, 10 over the feet, its light added (the row of colours from pale blue on); it comes
///   up by 1 a tick;
/// - 2: 10 from its middle to a corner, 10 over the feet, its light added (the client's row of colours from
///   pale blue on); it lives as the kind 2 does, a flash that widens;
/// - 4: 6 to a corner, 30 over the feet, white and mixed in; it lives as the kind 1 does, coming up and going
///   after 200 ticks;
/// - 21: 24 to a corner, 10 over the feet, turned upright, its light added (the row from white on); it shows
///   at once.
fn aura(picture: &'static str, kind: i8, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::Aura, picture, true, course);
    let (out, lift, look, round) = match kind {
        0 => (5.0, 10.0, 2, 135),
        2 => (10.0, 10.0, 2, 135),
        4 => (6.0, 30.0, 0, 135),
        _ => (24.0, 10.0, 4, 90),
    };
    let mut part = Part { out, round, turned: lot(360) as i32, ..Part::default() };
    (part.heights[15], part.flags[2], part.flags[4]) = (lift, look, if kind == 4 { 1 } else { kind });
    prim.parts.push(part);
    prim
}

/// `CRagEffect::TexturedDescendingRings`: four bars of `freezing_stick.bmp` (the kind 141) across the one the
/// effect is on, 20 wide, pointing up and to the left of the screen, 30, 20 and 10 over the feet and at
/// them, each begun 10 ticks after the one above it.
fn strokes(course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::TaperedQuad, "freezing_stick.bmp", true, course);
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { process: index * -10, out: 20.0, round: 215, alpha: 255, ..Part::default() };
        (part.at[0][1], part.flags[2]) = ((index * 10 - 30) as f32, 2);
        part
    }));
    prim
}

/// `CRagEffect::TexturedSparkle` with `white02.bmp`: four spikes of the kind 81 from a point 10 over the one
/// the effect is on, each pointing any way round and up or down, at an alpha of 40, 1 long at first, its
/// swing begun anywhere and going on 1 or 2 degrees a tick; a pale blue or a deeper one, by lot.
fn sparkle(course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::Spike, "white02.bmp", true, course);
    prim.parts.extend((0..4).map(|_| {
        let mut part = Part { alpha: 40, out: 1.0, rise: lot(360) as i32, turned: lot(360) as i32, round: lot(360) as i32, ..Part::default() };
        part.tall = lot(2) + 1.0;
        part.heights[..4].copy_from_slice(&if lot(2) == 0.0 { [105.0, 105.0, 255.0, 10.0] } else { [55.0, 55.0, 225.0, 10.0] });
        part
    }));
    prim
}
