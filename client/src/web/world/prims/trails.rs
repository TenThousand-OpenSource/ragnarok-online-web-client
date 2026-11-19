//! Lights that leave a trail of sparks behind them. In the client the light itself is a primitive that shows
//! nothing: its drawing function launches another effect where it is, at every frame it is drawn
//! (`RenderSMA3`, `RenderLaunchEffects`; `crate::prim::Prim::asks`). Here the primitive is asked once a tick
//! and the world launches what it asks for, on the one the first effect is on (the client launches the sparks
//! of Estun on the player's own character, whoever cast it, and tells them a place of the world; what is seen
//! is the same). Each effect is its function of `CRagEffect` with the same figures:
//!
//! - `EF_STIN3` (555): Estun, a ball that flies level from the one who casts through the one struck and on
//!   (`SMA` with its flag 1; `PP_SMA`). The client has the caster launch it and tells it the place of the
//!   one struck (`SL_STUN` in `Contents/Jobs/JobInheritance.cpp`);
//! - `EF_SMA3` (556): its sparks, four blue motes that rise where the ball was (`TexturedComet` with its
//!   kind 1; `PP_PARTICLE_UP`);
//! - `EF_LINKPARTICLE` (849): Harmonize, a light that goes round the one it is on, 25 over the ground, in a
//!   ring that widens (`TexturedTumbleOrb` with its flag 0; the kind 151). The client launches it on both
//!   the one who sings and the one sung to, and starts the sound once; here each of the two starts it;
//! - `EF_LINKPARTICLE2` (931): Arrullo, two such lights that go round opposite ways and go out sooner
//!   (`TexturedTumbleOrb` with its flags 2 and 1); the sound is what the client starts with it;
//! - `EF_SOULLIGHT2` (850), `EF_SOULLIGHT3` (932): the sparks of those lights, blue or of three reds by lot
//!   (`TexturedStorm2` with its kinds 2 and 3; `PP_SOULPARTICLE`).

use super::launched::of_parts;
use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 3] = ["whitelight.tga", "thunder_ball_0002.bmp", "thunder_ball_0003.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        555 => (200, "effect/t_에너지방출.wav"),
        556 => (50, ""),
        849 => (500, "effect/하모나이즈.wav"),
        850 | 932 => (100, ""),
        931 => (500, "effect/s포이즌버스터.wav"),
        _ => return None,
    })
}

/// Estun is launched by the one who casts and told the place of the one struck.
fn thrown(effect: u32) -> bool {
    effect == 555
}

/// The sparks of Estun stay where they were launched: `TexturedComet` does not take its parent's place.
fn stays(effect: u32) -> bool {
    effect == 556
}

/// One tick of an effect of this file: each launches what it launches at its first.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick != 0 {
        return;
    }
    // Where a spark's effect was told to be, in the client's directions (its y down).
    let told = [course.shared.aim[0], -course.shared.aim[1], course.shared.aim[2]];
    match effect {
        555 => prims.push(ball(course)),
        556 => prims.push(comet(course)),
        849 => prims.push(orb(0, course)),
        931 => prims.extend([orb(2, course), orb(1, course)]),
        850 => prims.push(sparks(told, |_| 4.0, course)),
        932 => prims.push(sparks(told, |_| lot(3) + 1.0, course)),
        _ => {}
    }
}

/// `CRagEffect::SMA` with its flag 1: four places (`PP_SMA`) 6 to 10 over the feet of the one who casts,
/// that stay in the world and set off one after the other, 0.8 a tick towards the place aimed at; the first
/// waits three ticks. Only the first asks for sparks (`RenderSMA3`); the others show nothing at all.
fn ball(course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Sma, "", false, course);
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { process: index - 3, ..Part::default() };
        part.at[0] = [0.0, lot(5) - 10.0, 0.0];
        (part.heights[1], part.heights[2], part.heights[3], part.flags[0]) = (0.8, course.shared.aim[0], course.shared.aim[2], 1);
        part
    }));
    prim
}

/// `CRagEffect::TexturedComet` with its kind 1: four motes (`PP_PARTICLE_UP`) within 4 east and north of the
/// place the effect is told and 6 over it, 1.2 to 1.92 from the middle to a corner, rising 0.1 to 0.29 a
/// tick; blue, their light added (`RenderFieldPanel`, kind 1, which also turns the picture over; that is not
/// done), showing `thunder_ball_0003.bmp` for two ticks on the way. They stay in the world where they are
/// launched.
fn comet(course: &Course) -> Prim {
    let mut motes = Prim::new(Shape::ParticleUp, PICTURES[1]);
    motes.pictures = &["thunder_ball_0002.bmp", "thunder_ball_0003.bmp"];
    (motes.duration, motes.colour, motes.adds, motes.left) = (course.lasts, [150.0, 150.0, 250.0], true, Some(course.walked));
    let aim = course.shared.aim;
    motes.slots = (0..4)
        .map(|_| {
            let out = lot(10) * 0.08 + 1.2;
            let (east, north) = (lot(9) - 4.0, lot(9) - 4.0);
            Slot::mote([aim[0] + east, aim[1] + 6.0, aim[2] + north], out, lot(20) * 0.01 + 0.1, true)
        })
        .collect();
    motes
}

/// `CRagEffect::TexturedTumbleOrb`: one light (the kind 151) that goes with the one the effect is on, its
/// ring 5 wide to begin with. `mode` is its `flags[0]`: 0 is lit for 270 ticks and asks for `EF_SOULLIGHT2`;
/// 1 and 2 are lit for 90 and ask for `EF_SOULLIGHT3`, and 1 goes round the other way.
fn orb(mode: i8, course: &Course) -> Prim {
    let mut prim = of_parts(Tei::SpiralFade, "whitelight.tga", true, course);
    let mut part = Part { out: 2.5, ..Part::default() };
    (part.at[1][0], part.flags[0]) = (5.0, mode);
    prim.parts.push(part);
    prim
}

/// `CRagEffect::TexturedStorm2` with its kind 2 or 3: the sparks of a soul's light (`souls::soul_light`) at
/// the place the effect is told, of the colour `kind` gives each of its two parts: 4 is blue, each spark its
/// own; 1 to 3, drawn by lot, a pale red to a deeper one.
fn sparks(at: [f32; 3], kind: impl Fn(usize) -> f32, course: &Course) -> Prim {
    let mut prim = launched::soul_light(at, course);
    prim.parts.iter_mut().enumerate().for_each(|(index, part)| part.heights[15] = kind(index));
    prim
}
