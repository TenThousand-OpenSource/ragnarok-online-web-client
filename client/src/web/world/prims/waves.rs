//! Walls of light that fly from the one who strikes towards the one struck, one after the other (the kind 146
//! of the client's primitives), each effect its function of `CRagEffect` with the same figures:
//!
//! - `EF_HEAL_N` (832): Sonic Wave's bubbles, and `EF_HEAL_N2` (943), the green ones of the elemental's Wind
//!   Slash (`BubbleFanSweep`); both flash the body of the one who strikes;
//! - `EF_NEW_SONICWAVE` (1678): the waves of Sonic Wave, three times over (`TexturedFanSweep`).
//!
//! The striker launches each and it is told the place of the one struck (`CSession::PlaySkillEffect`), which
//! also starts the sound of the skill; that sound is given here with the bubbles.

use super::*;
use crate::prim::{Part, Tei, angle_of};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&BUBBLES, &WAVES], sprites: &[] };

/// The thirteen pictures a wall of bubbles runs through, one a tick (`W_bubble02.tga` to `W_bubble26.tga`).
const BUBBLES: [&str; 13] = [
    "w_bubble02.tga",
    "w_bubble04.tga",
    "w_bubble06.tga",
    "w_bubble08.tga",
    "w_bubble10.tga",
    "w_bubble12.tga",
    "w_bubble14.tga",
    "w_bubble16.tga",
    "w_bubble18.tga",
    "w_bubble20.tga",
    "w_bubble22.tga",
    "w_bubble24.tga",
    "w_bubble26.tga",
];

/// The two pictures of the waves of Sonic Wave, the wide one first. (With the option to show fewer effects on
/// the client is no different here.)
const WAVES: [&str; 2] = ["new_sonicwave/new_sonicwave/son_wave_01.tga", "new_sonicwave/new_sonicwave/son_wave_00.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        832 | 943 => (2000, "effect/기공포.wav"),
        1678 => (1678, ""),
        _ => return None,
    })
}

/// Every effect of this file is launched by the striker and told the place of the one struck.
fn thrown(effect: u32) -> bool {
    known(effect).is_some()
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        832 | 943 => {
            // From the fifth to the 53rd tick the body is blue (for Wind Slash green) with a pale copy
            // behind and its light added at every even tick, and white at every odd one.
            if (4..=52).contains(&tick) {
                if tick % 2 == 0 {
                    course.touch.flags |= crate::body::flag::OUTLINE;
                }
                course.touch.flashes(tick, if effect == 832 { [5, 5, 255] } else { [55, 255, 55] });
            }
            if tick == 0 {
                let mut bubbles = fan_sweep(BUBBLES[0], 30.0, 3.0, if effect == 943 { 7.0 } else { 0.0 }, course);
                bubbles.pictures = &BUBBLES;
                prims.push(bubbles);
            }
        }
        1678 if matches!(tick, 0 | 20 | 40) => prims.extend([fan_sweep(WAVES[0], 20.0, 9.0, 0.0, course), fan_sweep(WAVES[1], 30.0, 6.0, 0.0, course)]),
        _ => {}
    }
}

/// What `CRagEffect::BubbleFanSweep` and `TexturedFanSweep` launch: four walls of the kind 146 that stay in
/// the world where they are launched, each 8 ticks after the one before, so `tall` and `out` from the middle
/// (each a tenth nearer than the one before), open towards the back of the way to the place aimed at and all
/// but an eighth of the way round, leaning at 70, 65, 60 and 55 degrees. `size` is what they are told to be
/// drawn as (`vecTPre.x`): nothing for white, mixed in; 7 for green, its light added.
fn fan_sweep(picture: &'static str, tall: f32, out: f32, size: f32, course: &Course) -> Prim {
    let mut walls = launched::of_parts(Tei::DriftRing, picture, false, course);
    let turned = (angle_of(course.shared.aim[0], course.shared.aim[2]) + 195) % 360;
    walls.parts.extend((0..4).map(|index| {
        let mut part = Part { process: index * -8, tall, turned, round: 315, out: out - index as f32 * 0.1, rise: (14 - index) * 5, ..Part::default() };
        part.at[3][0] = size;
        part
    }));
    walls
}
