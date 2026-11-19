//! The effects of things struck or torn out of someone, that fly every way from a place: coins, gear, streaks
//! and sparks. Their primitives are of the kinds the client builds out of its parts (`crate::prim::Tei`); each
//! effect is its function of `CRagEffect` with the same figures:
//!
//! - `EF_RG_COIN` (274): the coins of Steal Coin (`TexturedSpray`);
//! - `EF_RG_COIN2` (495), `EF_RG_COIN3` (627), `EF_RG_COIN4` (770), `EF_RG_COIN5` (820), `EF_RG_COIN6` (869),
//!   `EF_RG_COIN7` (907), `EF_RG_COIN_BLUE` (2436), `EF_RG_COIN_BLUE_2` (2437): the shields and swords, the
//!   weapons, the rings and the notes that a Strip and its like tear off (`TexturedSprayKind`). Some of them
//!   show pictures of items, which are not among the client's pictures of effects: such a picture is named
//!   here from the folder of the interface's pictures (`유저인터페이스\item\`), and `world/loading.rs` reads a
//!   name that begins so from there;
//! - `EF_TEIHIT2` (264), `EF_BACKSTAP` (275): the sparks struck out of the one hit by Investigate and by Back
//!   Stab (`TEIHIT2`);
//! - `EF_TEIHIT1X` (266), `EF_TEIHIT1T` (889), `EF_TEIHIT3` (276): the streaks that burst from the one struck
//!   by Asura Strike and by Tiger Cannon, and from the one who Raids (`TexturedScatter`,
//!   `TexturedScatterLow`).
//! - `EF_HITLINE4` (429), `EF_HITLINE6` (431): the sparks that leap from one kicked (`White02Burst`,
//!   `WhiteSparkStreak`); `EF_HITLINE5` (430): those that leap from one kicked back, away from the one who
//!   kicked (`WhiteSparkRing`);
//! - `EF_CARTTER` (518): the sparks of Cart Termination (`TexturedTumble`).
//!
//! The client has an option to show fewer effects (`TT_MIN_EFFECT_ON_OFF`); what is here is what it shows with
//! the option off.
//!
//! `EF_RG_COIN7` also has the weapon of the one it is on drawn with its light added at two ticks of every four
//! (`render.flags` of 2, `crate::body`).

use super::launched::{of_parts, part};
use super::*;
use crate::prim::{Tei, angle_of};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES, &RINGS, &WEAPONS, &GUNS], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 11] = ["coin_a.bmp", "shield.bmp", "sword.bmp", "melody_a.bmp", "melody_b.bmp", "thunder_center.bmp", "lens1.tga", "lens2.tga", "alpha_center.tga", "white02.bmp", "whitelight.tga"];

/// The pictures of items that `EF_RG_COIN5`, `EF_RG_COIN4` and `EF_RG_COIN3` show, named from the folder of
/// the interface's pictures.
const RINGS: [&str; 8] = [
    "유저인터페이스\\item\\갈색반지.bmp",
    "유저인터페이스\\item\\금반지.bmp",
    "유저인터페이스\\item\\금방울.bmp",
    "유저인터페이스\\item\\꽃반지.bmp",
    "유저인터페이스\\item\\낡은펜던트.bmp",
    "유저인터페이스\\item\\눈구슬반지.bmp",
    "유저인터페이스\\item\\상자.bmp",
    "유저인터페이스\\item\\수제초콜렛.bmp",
];
const WEAPONS: [&str; 8] = [
    "유저인터페이스\\item\\루드라의활.bmp",
    "유저인터페이스\\item\\아트로스의흉기.bmp",
    "유저인터페이스\\item\\모닝스타.bmp",
    "유저인터페이스\\item\\바스타드소드.bmp",
    "유저인터페이스\\item\\글레이브.bmp",
    "유저인터페이스\\item\\블레이드휩.bmp",
    "유저인터페이스\\item\\드래곤슬레이어.bmp",
    "유저인터페이스\\item\\배틀액스.bmp",
];
const GUNS: [&str; 4] = ["유저인터페이스\\item\\골드룩스.bmp", "유저인터페이스\\item\\더개리슨.bmp", "유저인터페이스\\item\\더블샷건.bmp", "유저인터페이스\\item\\더스크.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with at
/// its first tick (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        274 => (300, "effect/rog_steal coin.wav"),
        // The sound of Asura Strike is started by what launches the effect for the skill, with it
        // (`MO_EXTREMITYFIST` in `Contents/Jobs/JobInheritance.cpp`), not by the effect's own function.
        266 => (300, "effect/mon_아수라 패황권.wav"),
        264 | 275 | 276 | 495 | 627 | 770 | 820 | 869 | 889 | 907 => (300, ""),
        2436 | 2437 => (3000, ""),
        429 | 430 | 431 | 518 => (200, ""),
        _ => return None,
    })
}

/// Whether an effect of this file is one the striker launches and is then told the place of the one struck
/// (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS`, for Investigate, Back Stab and
/// Pinpoint Attack). The others that are told a place are launched on the one struck and told that one's own.
/// `EF_HITLINE5` is on the one struck and told where the striker stands (`TK_COUNTER` in
/// `Contents/Jobs/JobInheritance.cpp`).
fn thrown(effect: u32) -> bool {
    matches!(effect, 264 | 275 | 430 | 907)
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    match effect {
        275 if tick == 20 => course.sound("effect/rog_back stap.wav"),
        495 | 770 | 820 | 869 | 2436 | 2437 if matches!(tick, 1 | 11 | 21 | 31) => course.sound("effect/복호격.wav"),
        627 if tick == 21 => course.sound("effect/디스암.wav"),
        889 if tick == 5 => course.sound("effect/mon_맹룡과강.wav"),
        907 if tick == 26 => course.sound("effect/LG_PINPOINTATTACK.wav"),
        518 if tick == 30 => course.sound("effect/wizard_fire_pillar_b.wav"),
        _ => {}
    }
    // `EF_RG_COIN7`: the light of the weapon added, at two ticks of every four.
    if effect == 907 && tick % 4 <= 1 {
        course.touch.flags |= crate::body::flag::WEAPON_ADDS;
    }
    // The sparks of the kicks and of Cart Termination are launched at ticks of their own.
    match effect {
        429 if (6..10).contains(&tick) => prims.push(leaping_sparks(false, course)),
        430 if tick < 3 => prims.push(sparks_away(course)),
        431 if tick < 5 => prims.push(leaping_sparks(true, course)),
        518 if tick == 30 => prims.extend((0..4).map(|_| tumble(course))),
        _ => {}
    }
    if tick != 0 {
        return;
    }
    let gear = |number: i8| (0..4).flat_map(move |_| [("shield.bmp", 5.0, number), ("sword.bmp", 9.0, number)]);
    match effect {
        274 => prims.extend((0..30).map(|_| spray("coin_a.bmp", 1.5, 3.5, 1, course))),
        495 => prims.extend(gear(2).map(|(picture, large, number)| spray(picture, 1.0, large, number, course))),
        627 => prims.extend(GUNS.map(|picture| spray(picture, 1.0, 5.0, 2, course))),
        770 => prims.extend(WEAPONS.map(|picture| spray(picture, 1.0, 5.0, 2, course))),
        // (The box, `상자.bmp`, is a little smaller than the others.)
        820 => prims.extend(RINGS.iter().enumerate().map(|(index, picture)| spray(*picture, 1.0, if index == 6 { 4.0 } else { 5.0 }, 6, course))),
        869 => prims.extend((0..2).flat_map(|_| ["melody_a.bmp", "melody_b.bmp"]).map(|picture| spray(picture, 1.0, 6.0, 6, course))),
        907 => prims.extend(gear(7).map(|(picture, large, number)| spray(picture, 1.0, large, number, course))),
        2436 => prims.extend(gear(8).map(|(picture, large, number)| spray(picture, 1.0, large, number, course))),
        2437 => prims.extend(gear(9).map(|(picture, large, number)| spray(picture, 1.0, large, number, course))),
        264 => prims.extend((0..5).map(|_| struck_sparks(course))),
        275 => prims.extend((0..20).map(|_| struck_sparks(course))),
        266 => {
            // (The shake of the screen, too, is asked for by what launches the effect for Asura Strike.)
            course.quake = Some(0);
            prims.extend((0..24).map(|_| scatter("lens1.tga", 1, false, course)));
        }
        889 => prims.extend((0..20).map(|_| scatter("alpha_center.tga", 2, false, course))),
        276 => prims.extend((0..20).map(|_| scatter("lens2.tga", 2, true, course))),
        _ => {}
    }
}

/// The place the effect aims at from where the one it is on stands now, in the client's directions (x east,
/// y down, z north): what a part is given whose place the client has as a place of the world.
fn aimed(course: &Course) -> [f32; 3] {
    [course.shared.aim[0] - course.walked[0], course.walked[1] - course.shared.aim[1], course.shared.aim[2] - course.walked[2]]
}

/// `CRagEffect::TexturedSpray` and `CRagEffect::TexturedSprayKind`: four squares of that picture, each
/// `large` from its middle to a corner, that begin 12 from the place the effect aims at, every way by lot,
/// wait 18 to 47 ticks and fly out `speed` a tick, turning. `number` is the part's (`flag1[0]`): 1 for the
/// coins of `TexturedSpray`; `TexturedSprayKind` gives 6 for its kind 1, 7, 8 and 9 for those kinds and 2 for
/// any other. It gives the colour and whether the square stands spinning like a coin or faces the screen
/// (`prim/tei/orbit.rs`).
fn spray(picture: &'static str, speed: f32, large: f32, number: i8, course: &Course) -> Prim {
    let mut squares = of_parts(Tei::Intimidate, picture, false, course);
    squares.parts.extend((0..4).map(|_| {
        let mut square = part();
        square.at[0] = aimed(course);
        (square.process, square.out) = (-18 - lot(30) as i32, 12.0);
        square.heights[0] = speed;
        (square.rise, square.round, square.turned) = (lot(360) as i32, lot(360) as i32, lot(360) as i32);
        (square.heights[2], square.heights[1]) = (lot(360), large);
        square.flags[0] = number;
        square
    }));
    squares
}

/// `CRagEffect::TEIHIT2` with `thunder_center.bmp`: four small squares, blue and added, that wait 31 to 40
/// ticks 8 over the place the effect aims at and fly on from there, within 30 degrees of the way from the one
/// the effect is on to that place, 0.75 to 1.24 a tick, rising or sinking up to 1 a tick.
fn struck_sparks(course: &Course) -> Prim {
    let mut sparks = of_parts(Tei::TeiHit2, "thunder_center.bmp", false, course);
    sparks.size = 0.0;
    let [east, down, north] = aimed(course);
    sparks.parts.extend((0..4).map(|_| {
        let mut spark = part();
        (spark.process, spark.out) = (lot(10) as i32 - 40, lot(16) * 0.05 + 0.1);
        spark.at[0] = [east, down - 8.0, north];
        spark.turned = angle_of(east, north);
        (spark.heights[1], spark.heights[2]) = (lot(50) * 0.01 + 0.75, lot(200) * 0.01 - 1.0);
        spark.flags[2] = 2;
        spark
    }));
    sparks
}

/// `CRagEffect::TexturedScatter` and `CRagEffect::TexturedScatterLow` (`low`): four long plates of that
/// picture through the feet of the one the effect is on as it began, each along a line of its own drawn by
/// lot, that begin 10 behind the feet, wait (10 to 34 ticks; the low ones 20 to 29) and shoot out 3 a tick
/// (2), 0.7 wide to each side and 24 times as long (1 wide and 16 times). `number` gives the colour (0 yellow,
/// 1 white, 2 blue).
///
/// (Both functions lift the effect by 9 after launching the primitive, which stays where it was launched;
/// the lift moves nothing that is seen.)
fn scatter(picture: &'static str, number: i8, low: bool, course: &Course) -> Prim {
    let mut plates = of_parts(Tei::ExpandPulse, picture, false, course);
    plates.parts.extend((0..4).map(|_| {
        let mut plate = part();
        plate.process = if low { -20 - lot(10) as i32 } else { -10 - lot(25) as i32 };
        plate.out = -10.0;
        (plate.heights[0], plate.heights[1]) = if low { (2.0, 1.0) } else { (3.0, 0.7) };
        (plate.rise, plate.round) = (lot(360) as i32, lot(360) as i32);
        (plate.flags[0], plate.flags[1]) = (number, if low { 16 } else { 24 });
        plate
    }));
    plates
}

/// `CRagEffect::White02Burst` and `CRagEffect::WhiteSparkStreak` (`streak`), each of its kind 3, with
/// `white02.bmp`: four sparks that leap from the feet of the one struck every way, 6 to 11 far and about as
/// high, grey in a blue glow. The burst launches four at each of its seventh to tenth ticks, which begin
/// within 30 ticks; the streak four at each of its first five ticks, which begin after 30 to 70 and are a
/// little thicker and may leap lower.
fn leaping_sparks(streak: bool, course: &Course) -> Prim {
    let mut sparks = of_parts(Tei::HitLine3, "white02.bmp", true, course);
    sparks.parts.extend((0..4).map(|_| {
        let mut spark = part();
        spark.process = if streak { -30 - lot(41) as i32 } else { -(lot(31) as i32) };
        spark.tall = if streak { 0.07 } else { 0.05 };
        (spark.alpha, spark.alpha_top) = (lot(121) as i32 + 125, 3);
        (spark.rise, spark.out) = (lot(360) as i32, lot(6) + 6.0);
        // How high it leaps for how far (`vecTNow.x`), and of how many pieces it is (`vecTNow.y`).
        spark.at[1] = [lot(7) * 0.1 + if streak { 0.6 } else { 0.8 }, 4.0, 0.0];
        spark
    }));
    sparks
}

/// `CRagEffect::WhiteSparkRing` with its kind of 3: four sparks at each of its first three ticks, as
/// `White02Burst` launches them, but each leaps within 60 degrees of the way from the place the effect is told
/// (the striker's) to the one it is on, 6 to 12 far; they begin after 25 to 45 ticks and are of three pieces.
fn sparks_away(course: &Course) -> Prim {
    let mut sparks = of_parts(Tei::HitLine3, "white02.bmp", true, course);
    let away = angle_of(-course.shared.aim[0], -course.shared.aim[2]);
    sparks.parts.extend((0..4).map(|_| {
        let mut spark = part();
        spark.process = -25 - lot(21) as i32;
        spark.tall = 0.06;
        (spark.alpha, spark.alpha_top) = (lot(121) as i32 + 125, 3);
        (spark.rise, spark.out) = ((away + lot(121) as i32 - 60).rem_euclid(360), lot(7) + 6.0);
        spark.at[1] = [lot(7) * 0.1 + 0.8, 3.0, 0.0];
        spark
    }));
    sparks
}

/// `CRagEffect::TexturedTumble` with `whitelight.tga`: four parts of three sparks each, thrown from the feet of
/// the one struck every way upwards, each part with its speed of 1 a tick, its size and its shade of red.
fn tumble(course: &Course) -> Prim {
    let mut sparks = of_parts(Tei::Spread, "whitelight.tga", true, course);
    sparks.parts.extend((0..4).map(|_| {
        let mut thrown = part();
        (thrown.tall, thrown.out) = (1.0, lot(6) * 0.1 + 2.5);
        for (turn, tip, alpha) in [(0, 2, 12), (4, 6, 13), (8, 10, 14)] {
            (thrown.heights[turn], thrown.heights[tip], thrown.heights[alpha]) = (lot(360), lot(180), 230.0);
        }
        thrown.heights[15] = lot(3) + 1.0;
        thrown
    }));
    sparks
}
