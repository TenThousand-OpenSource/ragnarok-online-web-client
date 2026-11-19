//! Winds and bands that go round the one an effect is on, of the kinds the client builds out of its parts
//! (`crate::prim::Tei`: `PP_WIND`, `PP_SPHEREWIND`, `PP_SPHEREWIND2`, the kind 103); each effect is its
//! function of `CRagEffect` with the same figures:
//!
//! - `EF_PORTAL4` (389): Wind Walk, arcs of cloud round one who is lit a pale green (`Cloud11Burst`);
//! - `EF_SPHEREWIND2` (394): the bands of leaves that go round one under the moonlight, until that state ends
//!   (`FeatherRing`; `crate::state_effects`);
//! - `EF_BABY` (408): bands of leaves that widen (`FeatherBurst`);
//! - `EF_STORMKICK` (435), `EF_STORMKICK2` (460), `EF_STORMKICK3` (461): a whirl of squares one under the
//!   other and winds (`TexturedMark`, `Cloud11Burst`); the first turns the one who kicks.
//!
//! What an effect does to the body of the one it is on (`CRenderObject::SetColor`, `render.flags`,
//! `render.angle`) is said through `Course::touch`. None is told a place and none stays where it began.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 3] = ["cloud11.tga", "fire_ivy.bmp", "storm2.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with at
/// its first tick (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        389 => (200, "effect/윈드워크.wav"),
        394 => (99999, ""),
        408 => (200, "effect/ef_blessing.wav"),
        435 | 460 | 461 => (100, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        // `EF_PORTAL4`: lit and a pale green, from the fifth tick to the 25th.
        389 if (5..=25).contains(&tick) => course.touch.flagged(crate::body::flag::ADDS, 220, 250, 220),
        435 => {
            course.starts(&[(5, "effect/t_회오리차기.wav")]);
            // `EF_STORMKICK`: turned by 30 degrees a tick for the first 48 ticks.
            if tick < 48 {
                course.touch.turn = 30.0;
            }
        }
        460 => course.starts(&[(5, "ice_titan_attack.wav")]),
        _ => {}
    }
    if tick != 0 {
        return;
    }
    match effect {
        389 => prims.push(cloud_burst(0, course)),
        394 => prims.extend((0..5).map(|fifth| feather_ring(fifth * 9, course))),
        408 => prims.extend((0..5).map(|fifth| feather_burst(fifth * 9, course))),
        435 => prims.extend([mark(0, course), cloud_burst(2, course), cloud_burst(3, course)]),
        460 => prims.extend([mark(6, course), cloud_burst(2, course), cloud_burst(3, course)]),
        461 => prims.extend([mark(3, course), cloud_burst(2, course), cloud_burst(3, course)]),
        _ => {}
    }
}

/// `CRagEffect::Cloud11Burst`: four arcs of `cloud11.tga` a quarter of a turn apart that go round the place
/// where the effect began, each 1 tall and leaning by lot. Of the kinds 0 and 1 they stand low and near (5 to
/// 11 over the ground and 4.5 to 7.5 out; the kind 1 lower, nearer and yellow) and are gone soon; of the kinds
/// 2 and 3 they stand one over the other, the wider the higher, and stay faint for long.
fn cloud_burst(kind: i32, course: &Course) -> Prim {
    let mut winds = of_parts(Tei::Wind, "cloud11.tga", false, course);
    winds.size = if kind == 1 { 8.0 } else { 5.0 };
    winds.parts.extend((0..4).map(|index| {
        let step = index as f32;
        let mut wind = part();
        wind.round = 30;
        wind.tall = match kind {
            1 => step * 2.0 + 3.0,
            2 => 48.0 - step * 5.0,
            3 => 28.0 - step * 5.0,
            _ => step * 2.0 + 5.0,
        };
        wind.turned = index * 90;
        wind.alpha_top = if kind >= 2 { 2 } else { 1 };
        wind.out = match kind {
            1 => lot(101) * 0.01 + 2.5,
            2 => 14.0 - step * 2.0,
            3 => 6.0 - step,
            _ => lot(101) * 0.03 + 4.5,
        };
        wind.rise = lot(21) as i32 + 80;
        wind.heights = [1.0; 21];
        wind
    }));
    winds
}

/// `CRagEffect::FeatherRing` with `fire_ivy.bmp` and its large size: four bands of leaves, pink, about a place
/// 14 over the feet and 14 out, their rings tipped an eighth of a turn apart from `angle` on; each shows half
/// to three quarters of a turn and is there at once, at an alpha of 100.
fn feather_ring(angle: i32, course: &Course) -> Prim {
    let mut bands = of_parts(Tei::SphereWind, "fire_ivy.bmp", true, course);
    bands.size = 10.0;
    bands.parts.extend((0..4).map(|index| {
        let mut band = part();
        (band.tall, band.out, band.alpha) = (lot(11) * 0.2 + 4.0, 14.0, 100);
        (band.heights[0], band.heights[1]) = (-14.0, lot(4) + 3.0);
        (band.turned, band.rise, band.round) = (lot(360) as i32, index * 45 + angle, lot(91) as i32 + 180);
        band
    }));
    bands
}

/// `CRagEffect::FeatherBurst` with `fire_ivy.bmp`: four bands of leaves, pink, about a place 10 over the
/// feet, their rings tipped an eighth of a turn apart from `angle` on; each shows half to three quarters of a
/// turn, begins within 20 ticks, and widens as it goes round.
fn feather_burst(angle: i32, course: &Course) -> Prim {
    let mut bands = of_parts(Tei::SphereWind2, "fire_ivy.bmp", true, course);
    bands.size = 10.0;
    bands.parts.extend((0..4).map(|index| {
        let mut band = part();
        band.process = -(lot(20) as i32);
        (band.tall, band.out) = (lot(11) * 0.2 + 2.0, 5.0);
        (band.heights[0], band.heights[1]) = (-10.0, lot(4) + 3.0);
        (band.turned, band.rise, band.round) = (lot(360) as i32, index * 45 + angle, lot(91) as i32 + 180);
        band
    }));
    bands
}

/// `CRagEffect::TexturedMark` with `storm2.tga`, for the kinds other than 10: ten squares one under the other
/// from 50 over the ground down, 40 wide at the top and narrowing, that turn. `kind` gives the colour; the
/// kind 2 does not fade.
fn mark(kind: i8, course: &Course) -> Prim {
    let mut squares = of_parts(Tei::StackedRings, "storm2.tga", false, course);
    let mut stack = part();
    (stack.tall, stack.out, stack.turned) = (40.0, 2.0, lot(360) as i32);
    (stack.heights[0], stack.heights[1]) = (50.0, 5.0);
    (stack.flags[0], stack.flags[4], stack.flags[5]) = (10, kind, if kind == 2 { 10 } else { 0 });
    squares.parts.push(stack);
    squares
}
