//! The effects of blows whose primitives are of the kinds the client builds out of its parts
//! (`crate::prim::Tei`): streaks, fans of beams, waves along the ground, sparks, funnels of light and what
//! comes down on the one struck. Each is its function of `CRagEffect` with the same figures; what an effect
//! does to the body of the one it is on (`CRenderObject::SetColor`, `render.flags`) is said through
//! `Course::touch`:
//!
//! - `EF_TRIPLEATTACK` (329), `EF_TRIPLEATTACK4` (892): the streaks of Triple Attack, yellow, and the blue
//!   ones of Crescent Elbow (`CloudDrift`, `CloudBurst`); `EF_DARKPIERCING` (1030): long dark ones from far
//!   behind the striker through the one struck (`CloudSpread`);
//! - `EF_BASH3D` (366), `EF_BASH3D2` (377), `EF_BASH3D5` (628), `EF_BASH3D6` (885): fans of beams round the
//!   one who strikes, for Sacrifice, Tiger Fist and Rage Burst, Dust, Dragon Combo (`TexturedQuadSpin`);
//! - `EF_BASH3D3` (399), `EF_BASH3D4` (400): the same fans for Head Crush and Joint Beat, which outline the
//!   striker in a pale yellow or in white;
//! - `EF_SOULBREAKER2` (409): Meteor Assault, squares that slide away every way from the striker, who is lit
//!   purple (`PurpleSlash`);
//! - `EF_SOULBREAKER` (361), `EF_XENO_SLASHER` (1082): the squares of Soul Breaker and Xeno Slasher that
//!   slide at the one struck (`SonicWave`);
//! - `EF_HITLINE` (330): the sparks that leap from the one who ends a combo (`WhiteSparks`); `EF_HITLINE2`
//!   (376): red ones from the one struck by Palm Strike, who is outlined in orange for a moment;
//! - `EF_MAGNUM2` (339): Spiral Pierce, funnels of light one after the other (`Magnum2`);
//! - `EF_PRESSURE` (365), `EF_PRESSURE2` (906): the cross of Pressure and the shield of Shield Press that
//!   come down on the one struck, over a square of light that widens (`TexturedRain`, `TexturedLevelRing`).

use super::launched::square;
use super::launched::{of_parts, part, quad_spin};
use super::*;
use crate::body::flag;
use crate::prim::{Tei, angle_of};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 8] = [
    "cloud11.tga", "alpha_center.tga", "purpleslash.tga", "white02.bmp", "ring_yellow.tga", "cross_old.bmp",
    "shield.bmp", "explosive_1_128.bmp"
];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with at
/// its first tick (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        329 | 339 | 892 => (100, ""),
        330 | 366 | 376 | 377 | 885 => (200, ""),
        628 => (200, "effect/더스트샷.wav"),
        399 => (200, "effect/헤드 크러쉬.wav"),
        400 => (200, ""),
        409 => (300, "effect/메테오 어썰트.wav"),
        361 | 365 | 906 | 1082 => (300, ""),
        1030 => (5000, ""),
        _ => return None,
    })
}

/// Whether an effect of this file is one the striker launches and is then told the place of the one struck
/// (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS`, for Triple Attack, Soul Breaker and
/// Xeno Slasher). `EF_DARKPIERCING` is on the one struck and told where the striker stands (`NPC_DARKPIERCING`
/// in `Contents/Jobs/JobInheritance.cpp`, which launches the effect twice).
fn thrown(effect: u32) -> bool {
    matches!(effect, 329 | 361 | 1030 | 1082)
}

/// The sounds an effect of this file starts after its first tick, each with its tick.
fn sounds(effect: u32) -> &'static [(i32, &'static str)] {
    match effect {
        329 => &[(10, "effect/ef_hit2.wav"), (20, "effect/ef_hit4.wav"), (30, "effect/ef_hit2.wav")],
        892 | 1030 => &[(10, "effect/ef_hit2.wav"), (20, "effect/ef_hit4.wav")],
        339 => &[(20, "permeter_attack.wav"), (30, "effect/ef_magnumbreak.wav")],
        361 | 1082 => &[(20, "effect/기공포.wav")],
        365 => &[(3, "effect/프레셔.wav")],
        906 => &[(3, "effect/프레셔.wav"), (25, "effect/LG_SHIELDPRESS.wav")],
        366 => &[(22, "effect/세크리파이스.wav")],
        400 => &[(22, "effect/비트 조인트.wav")],
        377 | 885 => &[(5, "effect/mon_맹룡과강.wav")],
        _ => &[],
    }
}

/// What an effect of this file does at a tick to the body of the one it is on: Soul Breaker and Xeno Slasher
/// have the striker's light added, in purple and in green, for the first eleven ticks; Sacrifice gives it a
/// pale outline and a pale red from the 20th to the 40th tick; `EF_BASH3D2` has its light added from the
/// fifth to the 35th.
fn touch(effect: u32, course: &mut Course) {
    match (effect, course.tick) {
        (361, 0..=10) => course.touch.flagged(flag::ADDS, 255, 0, 255),
        (1082, 0..=10) => course.touch.flagged(flag::ADDS, 0, 255, 0),
        (366, 20..=40) => course.touch.flagged(flag::OUTLINE, 255, 200, 200),
        (377, 5..=35) => course.touch.flags |= flag::ADDS,
        // `EF_HITLINE2`: outlined and orange from the tenth tick to the 20th.
        (376, 10..=20) => course.touch.flagged(flag::OUTLINE, 255, 120, 50),
        // `EF_BASH3D3`, `EF_BASH3D4`: outlined, and a pale yellow or white, from the 20th tick to the 50th.
        (399 | 400, 20..=50) => course.touch.flagged(flag::OUTLINE, 255, 255, if effect == 399 { 200 } else { 255 }),
        // `EF_SOULBREAKER2`: lit and purple for the first eleven ticks.
        (409, 0..=10) => course.touch.flagged(flag::ADDS, 255, 0, 255),
        _ => {}
    }
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    course.starts(sounds(effect));
    touch(effect, course);
    match effect {
        329 if tick < 2 => prims.push(cloud_streaks(0.0, course)),
        892 if tick < 2 => prims.push(cloud_streaks(4.0, course)),
        1030 if tick < 2 => prims.push(cloud_spread(course)),
        330 if tick < 4 => prims.push(white_sparks(0, course)),
        376 if tick < 4 => prims.push(white_sparks(2, course)),
        339 => prims.extend(funnel(course)),
        // The cross asks for a shake of the screen as it stops, at its 33rd tick (`PrimBubbleSwirl`).
        365 | 906 if tick == 32 => course.quake = Some(0),
        _ => {}
    }
    if tick != 0 {
        return;
    }
    match effect {
        366 => prims.extend((0..5).map(|phase| quad_spin(phase, 0, course))),
        377 => prims.extend((0..8).map(|phase| quad_spin(phase, 2, course))),
        628 => prims.extend((0..6).map(|phase| quad_spin(phase, 5, course))),
        885 => prims.extend((0..8).map(|phase| quad_spin(phase, 6, course))),
        399 => prims.extend((0..6).map(|phase| quad_spin(phase, 4, course))),
        400 => prims.extend((0..6).map(|phase| quad_spin(phase, 5, course))),
        409 => prims.extend((0..8).map(|eighth| purple_slash(eighth * 45, course))),
        361 => prims.push(sonic_wave(0, course)),
        1082 => prims.push(sonic_wave(2, course)),
        365 => prims.extend([falling("cross_old.bmp", 0, course), square("explosive_1_128.bmp", 0, 28, course.lasts)]),
        906 => prims.extend([falling("shield.bmp", 0, course), square("explosive_1_128.bmp", 0, 28, course.lasts)]),
        _ => {}
    }
}

/// The whole degrees of the way from the one the effect is on to the place it aims at (`Get_Angle`).
fn way_to_aim(course: &Course) -> i32 {
    angle_of(course.shared.aim[0] - course.walked[0], course.shared.aim[2] - course.walked[2])
}

/// `CRagEffect::CloudDrift` (the kind 0) and `CRagEffect::CloudBurst` (the kind 4), each called at the first
/// two ticks: four streaks of `cloud11.tga` that point from the one who strikes at the place of the one
/// struck. Each begins up to 3 to each side of the striker and up to 15 behind, waits up to 19 ticks, is 8
/// to 12 over the ground, a quarter thick and 20 to 40 long; yellow for the kind 0 and blue for the kind 4.
fn cloud_streaks(kind: f32, course: &Course) -> Prim {
    let way = way_to_aim(course);
    let mut streaks = of_parts(Tei::PentaRing, "cloud11.tga", false, course);
    streaks.size = 0.0;
    streaks.parts.extend((0..4).map(|_| {
        let mut streak = part();
        (streak.process, streak.turned) = (-(lot(20) as i32), way);
        let (east, north, back) = (lot(100) * 0.06 - 3.0, lot(100) * 0.06 - 3.0, lot(100) * 0.15);
        streak.heights[1] = back;
        let near = [east - cosine(way as f32) * back, 0.0, north - sine(way as f32) * back];
        (streak.at[0], streak.at[1]) = (near, near);
        streak.tall = 0.25;
        streak.heights[0] = lot(100) * 0.04 + 8.0;
        streak.out = lot(100) * 0.2 + 20.0;
        streak.heights[4] = kind;
        streak
    }));
    streaks
}

/// `CRagEffect::CloudSpread`, at its first two ticks: four dark streaks of `cloud11.tga` (the kind 5 of the
/// primitive) that point from the one the effect is on at the place it is told. Each begins up to 15 to each
/// side of that one and 100 behind, waits up to 19 ticks, is 8 to 12 over the ground, a quarter thick and 100
/// to 150 long.
fn cloud_spread(course: &Course) -> Prim {
    let way = way_to_aim(course);
    let mut streaks = of_parts(Tei::PentaRing, "cloud11.tga", false, course);
    streaks.size = 0.0;
    streaks.parts.extend((0..4).map(|_| {
        let mut streak = part();
        (streak.process, streak.turned) = (-(lot(20) as i32), way);
        let (east, north) = (lot(100) * 0.3 - 15.0, lot(100) * 0.3 - 15.0);
        streak.heights[1] = 100.0;
        let near = [east - cosine(way as f32) * 100.0, 0.0, north - sine(way as f32) * 100.0];
        (streak.at[0], streak.at[1]) = (near, near);
        streak.tall = 0.25;
        streak.heights[0] = lot(100) * 0.04 + 8.0;
        streak.out = lot(100) * 0.5 + 100.0;
        streak.heights[4] = 5.0;
        streak
    }));
    streaks
}

/// `CRagEffect::SonicWave` with `purpleslash.tga`, for its kinds 0 and 2: four squares, 20 from the middle to
/// a corner, 6 to 10 over the ground where the effect began, that begin one after the other after 25 to 28
/// ticks and slide towards the place of the one struck, each up to 15 degrees to a side by lot. Those of the
/// kind 2 are green.
///
/// (The primitive, of the kind 86, takes each square's way as the square begins; here it is given as the
/// squares are launched: `prim/tei/squares.rs`.)
fn sonic_wave(kind: i8, course: &Course) -> Prim {
    let way = way_to_aim(course);
    let mut squares = of_parts(Tei::FloorSquare, "purpleslash.tga", false, course);
    squares.size = 0.0;
    squares.parts.extend((0..4).map(|index| {
        let mut square = part();
        (square.process, square.out) = (index - 28, 20.0);
        square.turned = (way + lot(31) as i32 - 15).rem_euclid(360);
        square.at[0][1] = lot(5) - 10.0;
        square.heights[1] = 0.8;
        square.flags[1] = kind;
        square
    }));
    squares
}

/// `CRagEffect::WhiteSparks`, called at the first four ticks: four sparks of `white02.bmp` that leap every
/// way from where the one the effect is on stood, 6 to 13 far at their first leap's half, a tenth thick, and
/// begin within 25 ticks. `number` gives their colour (0: orange; 2: red).
fn white_sparks(number: i32, course: &Course) -> Prim {
    let mut sparks = of_parts(Tei::HitLine, "white02.bmp", false, course);
    sparks.size = 0.0;
    sparks.parts.extend((0..4).map(|_| {
        let mut spark = part();
        (spark.process, spark.tall) = (-(lot(26) as i32), 0.1);
        (spark.alpha, spark.alpha_top) = (lot(51) as i32 + 125, number);
        (spark.rise, spark.out) = (lot(360) as i32, lot(8) + 6.0);
        spark
    }));
    sparks
}

/// `CRagEffect::Magnum2` with `ring_yellow.tga`: at every third tick from the 27th to the 54th, two walls
/// round the one the effect is on that lean outwards like a funnel, the second fainter and a little more
/// upright; the later they come, the wider, lower and flatter they are, and the further they are turned.
///
/// (The client launches ten at the most, counting them in the effect's `flag`; there are ten such ticks.)
fn funnel(course: &Course) -> Option<Prim> {
    let since = course.tick - 25;
    if !(0..=30).contains(&since) || course.tick % 3 != 0 {
        return None;
    }
    let mut walls = of_parts(Tei::Magnum2, "ring_yellow.tga", true, course);
    walls.size = 5.0;
    walls.parts.extend((0..2).map(|index| {
        let mut wall = part();
        (wall.round, wall.tall, wall.out) = (360, 20.0 - since as f32 * 0.7, since as f32 * 0.2 + 5.0);
        (wall.turned, wall.rise, wall.alpha_top) = (since * 6 + index * 9, (35 - index * 4 - since) * 2, index);
        wall
    }));
    Some(walls)
}

/// `CRagEffect::TexturedRain` of the kind 0: a square of that picture, white and added, 12 from its middle to
/// a corner, that comes down turning from 115 over the one the effect is on, 3 a tick, three fainter ones
/// behind it, and stops 19 over the feet. Of the kinds 1, 2 and 3 (the potions of Slim Potion Pitcher:
/// `fields.rs`) it is 8 to a corner, mixed in, in red, yellow or its own colours (`RenderFieldPanel`, 9 with
/// 1, 3 and 4), and comes down from 80 over the feet for 24 ticks, to 8 over them.
pub(super) fn falling(picture: &'static str, kind: i8, course: &Course) -> Prim {
    let mut drop = of_parts(Tei::Drop, picture, true, course);
    drop.size = 0.0;
    let mut head = part();
    head.out = if kind == 0 { 12.0 } else { 8.0 };
    head.at[2] = [0.0, if kind == 0 { -115.0 } else { -80.0 }, 0.0];
    (head.heights[1], head.round) = (3.0, lot(360) as i32);
    (head.flags[2], head.flags[3], head.flags[4]) = if kind == 0 { (7, 0, 0) } else { (9, [1, 3, 4][kind as usize - 1], kind) };
    drop.parts.push(head);
    drop.parts.extend((0..3).map(|_| part()));
    drop
}

/// `CRagEffect::PurpleSlash`: four squares of `purpleslash.tga`, 6 to 10 over the ground where the effect
/// began, that slide away along `angle` one after the other, beginning after 25 to 33 ticks.
fn purple_slash(angle: i32, course: &Course) -> Prim {
    let mut squares = of_parts(Tei::FloorSquare, "purpleslash.tga", false, course);
    let wait = lot(6) as i32;
    squares.parts.extend((0..4).map(|index| {
        let mut square = part();
        (square.process, square.out, square.turned) = (index - 28 - wait, 20.0, angle);
        square.at[0][1] = lot(5) - 10.0;
        square.heights[1] = 0.8;
        square
    }));
    squares
}
