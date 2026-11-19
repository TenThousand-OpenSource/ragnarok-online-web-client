//! Lines of light that go out from the one an effect is on, or fly in to that one, each effect its function of
//! `CRagEffect` with the same figures:
//!
//! - `EF_TRUESIGHT` (386): True Sight, twelve beams round the one who sees, seven degrees apart
//!   (`TexturedQuadSpin`);
//! - `EF_STOPEFFECT` (444): the blades that shoot out along the ground under one who stops running
//!   (`RunStop`), and `EF_KAIZEL` (590): the taller ones of Kaizel (`JobLvUp50_2`); both `PP_SLASH1`;
//! - `EF_ELECTRIC3` (781): the arcs of lightning that reach out every way from a Magnetic Field
//!   (`ElectricRingEarly`; `PP_ELECTRIC`);
//! - `EF_INFRAREDSCAN` (794): the two beams of Infrared Scan that sweep round, one each way (`ElectricFan`;
//!   the kind 135);
//! - `EF_PRIMECHARGE2` (908) and `EF_PRIMECHARGE4` (910): the needles of light that fly in from every side
//!   and whirl round one who takes Prestige or Inspiration (`TexturedScatterSphere`; the kind 159).

use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 3] = ["magic_blue.tga", "elec1.tga", "alpha_center.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        386 => (250, "effect/hunter_detecting.wav"),
        444 => (100, "effect/t_효과음1.wav"),
        590 => (200, ""),
        // (`EF_ELECTRIC3` starts its sound at its sixth tick, in `run`.)
        781 => (300, ""),
        794 => (300, "effect/infraredscan.wav"),
        // (`EF_PRIMECHARGE4` starts its sound at its eleventh tick, in `run`.)
        908 | 910 => (500, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    match (effect, course.tick) {
        (386, 0) => prims.extend((0..12).map(|phase| launched::quad_spin(phase, 3, course))),
        (444, 0) => prims.extend([blades(0, true, course), blades(45, true, course)]),
        (590, 15) => prims.extend([blades(0, false, course), blades(45, false, course)]),
        (781, 0..=3) => prims.extend([electric_ring(0, course), electric_ring(180, course)]),
        (781, 5) => course.sound("effect/magneticfield.wav"),
        (794, 0) => prims.extend([scan(0, course), scan(1, course)]),
        (908, 0) => prims.extend((0..40).map(|_| scatter_sphere(1, course))),
        (910, 0) => prims.extend((0..40).map(|_| scatter_sphere(3, course))),
        (910, 10) => course.sound("effect/lg_inspiration.wav"),
        _ => {}
    }
}

/// `CRagEffect::RunStop` (`low`) and `CRagEffect::JobLvUp50_2`: four blades of `magic_blue.tga` a quarter of a
/// turn apart, the first turned by `angle` (`PP_SLASH1`), each beginning 1 out from the middle and 2 long.
/// `RunStop`'s begin flat and grow slower and lower; the others begin 1.5 tall.
fn blades(angle: i32, low: bool, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::Slash1, "magic_blue.tga", true, course);
    prim.size = 4.0;
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { tall: if low { 0.0 } else { 1.5 }, turned: index * 90 + angle, out: 2.0, ..Part::default() };
        (part.heights[0], part.flags[0]) = (1.0, i8::from(low));
        part
    }));
    prim
}

/// `CRagEffect::ElectricFan`: a beam of `elec1.tga` (the kind 135) that reaches 20 from the one who scans, 10
/// over the ground, and begins a quarter of a turn to one side of the way that one faces; a line 0.1 thick at
/// an alpha of 250 with three sheets of light behind it (at 110, 80 and 50). With a `fade` of 1 it sweeps
/// with the clock, else against it.
fn scan(fade: i32, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::JitterSweep, "elec1.tga", true, course);
    let rise = (course.facing as i32 - 90).rem_euclid(360);
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { tall: 0.1, alpha: if index == 0 { 250 } else { 140 - index * 30 }, rise, out: 20.0, alpha_top: fade, ..launched::part() };
        part.at[1] = [lot(11) * 0.01, 20.0, 0.0];
        part
    }));
    prim
}

/// `CRagEffect::TexturedScatterSphere`: four needles of `alpha_center.tga` (the kind 159), each 4 to an end,
/// that begin up to 100 ticks late by lot, 70 from a place 5 over the one the effect is on, any way round
/// and up or down from it by lot, and fly in at half a step a tick. `flag` gives their colours: 1 white and
/// yellow, 3 pale blue and blue.
fn scatter_sphere(flag: i8, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::SpiralBlade, "alpha_center.tga", true, course);
    prim.parts.extend((0..4).map(|_| {
        let mut part = Part { process: -(lot(101) as i32), out: 4.0, rise: lot(360) as i32, turned: lot(360) as i32, ..Part::default() };
        (part.heights[0], part.heights[1], part.flags[2]) = (-5.0, 0.5, flag);
        let level = cosine(part.rise as f32) * 70.0;
        part.at[0] = [level * cosine(part.turned as f32), level * sine(part.turned as f32) - 5.0, sine(part.rise as f32) * 70.0];
        part
    }));
    prim
}

/// `CRagEffect::ElectricRingEarly`, at each of its first four ticks: four arcs of `elec1.tga`
/// (`PP_ELECTRIC`) that stay in the world where they are launched, from 5 over the effect's place, an eighth
/// of a turn apart from `angle` on; each a line 0.1 thick that reaches out further and further, lifted in a
/// low bow.
fn electric_ring(angle: i32, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::Electric, "elec1.tga", false, course);
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { tall: 0.1, rise: index * 45 + angle, round: 3, ..launched::part() };
        (part.at[0], part.at[1]) = ([0.0, -5.0, 0.0], [lot(11) * 0.01, 20.0, 0.0]);
        part
    }));
    prim
}
