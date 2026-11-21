//! Thin shafts of light that shoot up out of the ground (the kind 137 of the client's primitives), each
//! effect its function of `CRagEffect` with the same figures:
//!
//! - `EF_TETRA` (804): Tetra Vortex, the blast over the one struck four times over (`TetraVortexPillar`; the
//!   kind 140), shafts of every colour round that one (`TexturedSpread`), and the screen shaken three times;
//! - `EF_BLUELINE` (899): the blue shafts of Hesperus Lit round the one struck (`TexturedSpreadFlat`).
//!
//! And one that comes down: `EF_FORESTLIGHT5` (719), the light of Renovatio, four shafts one inside the other
//! that slant down to the one it is cast on from high in the south west (`ForestLight` with its flag 12;
//! `PP_FOREST`); the sound is what the client starts with it (its handler of `AB_RENOVATIO`).
//! `EF_ITEM_LIGHT` (581) is the same light, wider and fainter, let down by a thing that is used (its flag 11).

use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES, &BLAST], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 3] = ["ring_red.tga", "ring_blue.tga", "cloud11.tga"];

/// The eight pictures the blast of Tetra Vortex runs through.
const BLAST: [&str; 8] = ["tv-1.bmp", "tv-2.bmp", "tv-3.bmp", "tv-4.bmp", "tv-5.bmp", "tv-6.bmp", "tv-7.bmp", "tv-8.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty; Tetra Vortex starts two at once, in `run`); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        581 => (180, ""),
        719 => (999, "effect/ab_renovatio.wav"),
        804 => (600, ""),
        899 => (999, ""),
        _ => return None,
    })
}

/// Tetra Vortex is launched by the one who casts and told the place of the one struck
/// (`CSession::PlaySkillEffect`); Hesperus Lit's shafts are launched on the one struck.
fn thrown(effect: u32) -> bool {
    effect == 804
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        581 if tick == 0 => prims.push(forest_light(11, course)),
        719 if tick == 0 => prims.push(forest_light(12, course)),
        804 => {
            match tick {
                0 => course.sounds.extend(["effect/wizard_meteo.wav".to_owned(), "effect/hunter_blastmine.wav".to_owned()]),
                10 | 20 | 50 | 80 => course.sound("effect/wl_tetravortex.wav"),
                _ => {}
            }
            if matches!(tick, 0 | 40 | 80) {
                course.quake = Some(0);
            }
            if tick <= 80 && tick % 25 == 0 {
                prims.push(blast(course));
            }
            if tick == 0 {
                prims.extend((0..7).map(|_| {
                    let size = lot(5) + 4.0;
                    shafts("ring_red.tga", size, course.shared.aim, course, |_| (150, 120.0, lot(100) as i32, lot(41)))
                }));
            }
        }
        899 if tick == 0 => prims.extend((0..6).map(|_| {
            let size = [4.0, 40.0, 41.0][lot(3) as usize];
            shafts("ring_blue.tga", size, [0.0; 3], course, |index| (130, (index * 30 + 30) as f32, lot(50) as i32 + 50, lot(17)))
        })),
        _ => {}
    }
}

/// `CRagEffect::TetraVortexPillar` with its kind 4, at its ticks 0, 25, 50 and 75: a picture that faces the
/// screen (the kind 140) 35 over the place aimed at, 30 from its middle to a corner and taller than wide, at
/// an alpha of 200; it runs through the eight pictures of the blast, three ticks each, and fades. For
/// `EF_TETRA` itself the function names none of its four colours, so it is white and mixed in.
fn blast(course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::FrameFade, BLAST[0], false, course);
    prim.pictures = &BLAST;
    let aim = course.shared.aim;
    let mut part = Part { out: 30.0, round: 120, turned: lot(360) as i32, alpha: 200, ..Part::default() };
    (part.at[0], part.heights[10], part.flags[4]) = ([aim[0], -aim[1] - 35.0, aim[2]], 1.0, 4);
    prim.parts.push(part);
    prim
}

/// What `CRagEffect::TexturedSpread` and `TexturedSpreadFlat` launch: four shafts of the kind 137 about the
/// place `about` from the effect's (east, up, north), each any way round it by lot and 0.15 to 0.3 thick,
/// drawn in the colour `size` stands for. `figures` gives for each of the four its alpha, how tall it gets,
/// the count it begins at (it shoots up at 100) and how far from the place it stands. (The lots are drawn in
/// the client's order.)
fn shafts(picture: &'static str, size: f32, about: [f32; 3], course: &Course, figures: impl Fn(i32) -> (i32, f32, i32, f32)) -> Prim {
    let mut prim = launched::of_parts(Tei::ArcWall, picture, false, course);
    prim.size = size;
    prim.parts.extend((0..4).map(|index| {
        let (alpha, tall, process, radius) = figures(index);
        let angle = lot(360);
        let mut part = Part { process, round: 360, turned: index * 90, alpha, tall, rise: 90, out: lot(11) * 0.015 + 0.15, ..Part::default() };
        (part.at[0], part.at[3]) = ([radius * cosine(angle), 0.0, radius * sine(angle)], [about[0], -about[1], about[2]]);
        part
    }));
    prim
}

/// `CRagEffect::ForestLight` with its flag 12: four shafts of five sides (`PP_FOREST`) from the feet of the
/// one the effect is on up to a place 300 over them and 70 to the west and to the south, 1, 1.5, 2 and 1 from
/// the middle to a side, each begun 25 degrees further round. They begin at an alpha of 30, come up for 40
/// ticks and go after the 140th; the fourth begins at a count of 180 and so only goes. They stay in the world
/// where they are launched.
///
/// With its flag 11 (`EF_ITEM_LIGHT`, the light a Box of Sunlight lets down): the same, wider (4, 6, 8 and
/// 4 to a side), from nothing.
fn forest_light(flag: i8, course: &Course) -> Prim {
    let (wider, widest, alpha) = if flag == 11 { (2.0, 4.0, 0) } else { (0.5, 1.0, 30) };
    let mut prim = launched::of_parts(Tei::Forest, "cloud11.tga", false, course);
    prim.parts.extend((0..4).map(|index| {
        let wide = if index == 3 { widest } else { index as f32 * wider + widest };
        let mut part = Part { process: if index == 3 { 180 } else { 0 }, alpha, tall: wide, turned: index * 25, ..Part::default() };
        (part.heights[0], part.at[3], part.flags[0]) = (wide, [-70.0, -300.0, -70.0], flag);
        part
    }));
    prim
}
