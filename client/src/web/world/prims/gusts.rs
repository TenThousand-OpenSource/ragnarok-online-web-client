//! Squares of wind that spin and fly over the ground from the one who strikes (the kinds 119, 122, 125 and
//! 126 of the client's primitives), each effect its function of `CRagEffect` with the same figures:
//!
//! - `EF_STIN` (547): Estin, one square that flies to the one struck (`TexturedSlashFan`);
//! - `EF_STIN2` (553): Esma, two squares that fly round to the one struck, one by each side
//!   (`TexturedSlashLevel` with its kinds 0 and 1), and `EF_STIN_BLACK` (1206), the same in dark grey (its
//!   kinds 4 and 5);
//! - `EF_STIN4` (621): Huujin, green squares that swerve to the one struck (`TexturedSlashWave`);
//! - `EF_STIN5` (624): Kamaitachi, five green wheels of wind that roll to the one struck
//!   (`TexturedSlashCross`).
//!
//! The client has the striker launch each and tells it the place of the one struck, or for Esma and Huujin
//! who that one is (`CSession::PlaySkillEffect`: `AM_SETPOS` with the account number), so that their squares
//! follow that one; here the effect tells its squares every tick where that one is (`Course::other`).

use super::*;
use crate::prim::{Part, Tei, angle_of};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The picture the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 1] = [WIND];
const WIND: &str = "270바람.tga";

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// at its first tick (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        547 | 553 | 624 | 1206 => (200, ""),
        621 => (200, "effect/풍인.wav"),
        _ => return None,
    })
}

/// Every effect of this file is launched by the striker and told the one struck.
fn thrown(effect: u32) -> bool {
    known(effect).is_some()
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        547 if tick == 10 => course.sound("effect/t_바람마법.wav"),
        553 | 1206 if matches!(tick, 5 | 20 | 35 | 50 | 65) => course.sound("effect/t_날라차기.wav"),
        624 if tick == 1 => course.sound("effect/t_바람방출.wav"),
        _ => {}
    }
    match effect {
        547 if tick == 0 => prims.push(slash_fan(course)),
        553 if tick == 0 => prims.extend([slash_level(0, course), slash_level(1, course)]),
        1206 if tick == 0 => prims.extend([slash_level(4, course), slash_level(5, course)]),
        // (The client launches as many as it was told, the level of the skill; here as many as the blow.)
        621 if tick == 0 => prims.extend((0..course.count.max(1)).map(|index| slash_wave((index % 2) as i8, course))),
        624 if tick < 5 => prims.push(slash_cross(course)),
        _ => {}
    }
    // The squares that follow the one struck are told where that one is now, from where they were launched
    // (they were launched at the effect's first tick, where the striker stood then).
    if matches!(effect, 553 | 621 | 1206) {
        let target = course.other.map(|other| std::array::from_fn(|axis| other[axis] + course.walked[axis]));
        for first in prims.iter_mut().filter_map(|prim| prim.parts.first_mut()) {
            (first.at[1], first.flags[5]) = (target.map_or([0.0; 3], |at: [f32; 3]| [at[0], -at[1], at[2]]), i8::from(target.is_some()));
        }
    }
}

/// A primitive of four squares of wind that stays in the world where it is launched.
fn squares(kind: Tei, course: &Course, part: impl Fn(i32) -> Part) -> Prim {
    let mut prim = launched::of_parts(kind, WIND, false, course);
    prim.parts.extend((0..4).map(part));
    prim
}

/// The way from the effect's place to the place it aims at, the client's degrees (`Get_Angle`, which the
/// client's primitive takes at its first tick).
fn way(course: &Course) -> i32 {
    angle_of(course.shared.aim[0], course.shared.aim[2])
}

/// `CRagEffect::TexturedSlashFan`: one square of the kind 119, 20 from its middle to a corner, 6 to 10 over
/// the ground by lot, blue; it sets off after three ticks at 0.8 a tick, towards the place aimed at.
fn slash_fan(course: &Course) -> Prim {
    let turned = way(course);
    squares(Tei::Gust, course, |index| {
        let mut part = Part { process: index - 3, out: 20.0, rise: index, turned, ..Part::default() };
        (part.at[0][1], part.heights[1]) = (lot(5) - 10.0, 0.8);
        part
    })
}

/// `CRagEffect::TexturedSlashLevel`, for its kinds 0, 1, 4 and 5: one square of the kind 122, 25 to a
/// corner, 10 over the ground, that flies at 2 a tick; an even kind bows to one side and an odd one to the
/// other. The kinds 0 and 1 are blue, their light added; 4 and 5 dark grey, mixed in.
fn slash_level(kind: i8, course: &Course) -> Prim {
    squares(Tei::GustChase, course, |index| {
        let mut part = Part { out: 25.0, rise: index, ..Part::default() };
        (part.at[0][1], part.heights[1]) = (-10.0, 2.0);
        (part.flags[2], part.flags[3]) = (kind % 2, if kind >= 4 { 3 } else { 0 });
        part
    })
}

/// `CRagEffect::TexturedSlashWave`: one square of the kind 125, 10 to 15 to a corner by lot, 10 over the
/// ground, green, that flies at 1 to 3 a tick by lot and bows to the side of its `kind`.
fn slash_wave(kind: i8, course: &Course) -> Prim {
    squares(Tei::GustSwerve, course, |index| {
        let mut part = Part { out: lot(11) * 0.5 + 10.0, rise: index, ..launched::part() };
        (part.at[0][1], part.heights[1]) = (-10.0, lot(21) * 0.1 + 1.0);
        (part.flags[2], part.flags[3]) = (kind, 1);
        part
    })
}

/// `CRagEffect::TexturedSlashCross`, at each of its first five ticks: one wheel of the kind 126, 20 to a
/// corner, 6 to 10 over the ground by lot, green; the first from the effect's place, the next four from 5 to
/// the east, west, north and south of it. It sets off after three ticks at 1.5 a tick, towards the place
/// aimed at. (The client turns it some way by lot first, and the primitive then takes the way to the place.)
fn slash_cross(course: &Course) -> Prim {
    let from = [[0.0, 0.0], [5.0, 0.0], [-5.0, 0.0], [0.0, 5.0], [0.0, -5.0]][course.tick.clamp(0, 4) as usize];
    let turned = way(course);
    squares(Tei::GustRing, course, |index| {
        let mut part = Part { process: index - 3, out: 20.0, rise: index, turned, ..Part::default() };
        part.at[0] = [from[0], lot(5) - 10.0, from[1]];
        (part.heights[1], part.flags[3]) = (1.5, 1);
        part
    })
}
