//! The effects of the songs of minstrels and wanderers, each its function of `CRagEffect` with the same
//! figures:
//!
//! - `EF_SPR_PLANT` (853) and `EF_SPR_PLANT2` to `8` (866, 868, 870, 872, 874, 876, 878): monsters that come
//!   up round the one who sings and go again (`MonsterSpriteBurst`): geographers for Death Valley, wargs for
//!   the Song of Despair, violies for Lerad's Dew... each skill its own monster;
//! - `EF_CIRCLEPOWER2` (861): the green lights of Death Valley and of the Circle of Nature
//!   (`TexturedTwinkleRing` with its level 2);
//! - `EF_SECRA2` (862): the sign of a spell over one who sings Randomize Spell (`TexturedSlam` with its
//!   kind 2);
//! - `EF_CHEMICAL_V3` (863): the two white lines of Great Echo from the one who sings towards the one struck
//!   (`TexturedBeam`; the kind 154). The client has the singer launch it and tells it the place of the one
//!   struck, shakes the screen and starts the sound (`CSession::PlaySkillEffect`); those are given here;
//! - `EF_NEW_REVERBERATION` (1569): the note that hovers over one struck by Reverberation (`TexturedDrop`; the
//!   kind 156) and the violet rings that widen under it (`WhiteRingRippleSized`); the sound is what the
//!   client starts with it;
//! - `EF_NEW_REVERBERATION_NOTE` (1594): the notes that fly off from one struck by Reverberation, wandering
//!   (`TexturedSpinShards` with its kind 3; the kind 170). The client launches it beside `EF_NEW_REVERBERATION`,
//!   whose sound is the one it starts for both;
//! - `EF_CHEMICAL_ALLRANGE` (969): the red notes of Deep Sleep Lullaby, the same function with its kind 1;
//!   the sound is what the client starts with it;
//! - `EF_NEW_METALLICSOUND_LINE` (1584): Metallic Sound, the two white lines of Great Echo with another
//!   picture (`TexturedBeam`) and sparks that fly off towards the one struck, their ways wandering
//!   (`TexturedForkBolt`; the kind 155). The singer launches it and is told the place of the one struck, and
//!   the sound is what the client starts with it (`CSession::PlaySkillEffect`).
//!
//! The client has an option to show fewer effects (`TT_MIN_EFFECT_ON_OFF`); what is here is what it shows with
//! the option off.

use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES, &NOTES, &METAL], sprites: &[&MONSTERS] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 5] = ["freeze_a.bmp", "priest_spell.bmp", "line3.tga", "ring_white.tga", NOTE];

/// The note of Reverberation. (With the option to show fewer effects on the client takes the picture of the
/// same name from `mineffect`.)
const NOTE: &str = "new_reverberation/melody_a.bmp";

/// The monsters `MonsterSpriteBurst` shows for its kinds 0 and 2 to 8, among the client's sprites.
const MONSTERS: [&str; 8] = ["몬스터/geographer", "몬스터/워그", "몬스터/violy", "몬스터/succubus", "몬스터/mineral", "몬스터/ktullanux", "몬스터/garm", "몬스터/miyabi_ningyo"];

/// The kind `CRagEffect::OnProcess` hands `MonsterSpriteBurst` for an effect; nothing for any other effect.
fn monsters_of(effect: u32) -> Option<usize> {
    Some(match effect {
        853 => 0,
        866 => 2,
        868 => 3,
        870 => 4,
        872 => 5,
        874 => 6,
        876 => 7,
        878 => 8,
        _ => return None,
    })
}

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty; the monsters have no sound in their function); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        853 | 861 | 866 | 868 | 870 | 872 | 874 | 876 | 878 => (400, ""),
        862 => (300, ""),
        863 => (300, "effect/lg_earthdrive.wav"),
        969 => (200, "effect/안식의자장가.wav"),
        1569 => (150, "effect/reverberation.wav"),
        1584 => (300, "effect/metalic.wav"),
        1594 => (200, ""),
        _ => return None,
    })
}

/// Great Echo and Metallic Sound are launched by the one who sings and told the place of the one struck.
fn thrown(effect: u32) -> bool {
    matches!(effect, 863 | 1584)
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if let Some(kind) = monsters_of(effect) {
        prims.extend(monster(kind, course));
    }
    if effect == 863 && course.tick == 0 {
        course.quake = Some(0);
        prims.push(beam(2, "line3.tga", course));
    }
    if effect == 1569 {
        if course.tick == 0 {
            prims.push(drop_note(NOTE, 2, course));
        }
        // `WhiteRingRippleSized` with its size 28: the rings of a portal about to open, violet, every
        // hundredth tick.
        if course.tick % 100 == 0 {
            prims.push(launched::ripples("ring_white.tga", 28.0, 100, 1, course));
        }
    }
    if effect == 862 && course.tick == 0 {
        prims.push(slam("priest_spell.bmp", 7, course));
    }
    // `TexturedTwinkleRing`: its squares at its first tick, and again at its tenth.
    if effect == 861 && matches!(course.tick, 0 | 10) {
        prims.push(twinkle_ring("freeze_a.bmp", 2.0, course));
    }
    if effect == 1584 && course.tick == 0 {
        prims.push(beam(2, METAL[0], course));
    }
    // `TexturedForkBolt` and `TexturedSpinShards`: at the first tick, the tenth and the twentieth, once for
    // each picture the effect's arm names.
    if matches!(course.tick, 0 | 10 | 20) {
        match effect {
            969 => prims.extend([0, 1, 0, 1].map(|picture| spin_shards(NOTES[picture], 1, course))),
            1584 => prims.extend([1, 2, 3].map(|picture| fork_bolt(METAL[picture], 2, course))),
            1594 => prims.extend([2, 3].map(|picture| spin_shards(NOTES[picture], 3, course))),
            _ => {}
        }
    }
}

/// `LaunchTwinkleRing` (of `CRagEffect::TexturedTwinkleRing`): four upright squares of light through the one
/// the effect is on (the kind 132), each turned its own way by lot, the second 20 ticks after the first and so
/// on; each begins 10 large at an alpha of 100, its middle 10 over the ground, in the colour of the `level`.
fn twinkle_ring(picture: &'static str, level: f32, course: &Course) -> Prim {
    let mut squares = launched::of_parts(Tei::Twinkle, picture, true, course);
    squares.parts.extend((0..4).map(|index| {
        let mut part = Part { process: index * -20, round: lot(360) as i32, tall: 10.0, turned: lot(360) as i32, alpha: 100, ..launched::part() };
        (part.heights[3], part.heights[4], part.heights[5]) = (index as f32, 10.0, level);
        part
    }));
    squares
}

/// `CRagEffect::MonsterSpriteBurst`, for its kinds 0 and 2 to 8: at each of its first ticks (50 for the kind
/// 0, 10 for the others) one monster (the kind 153) that stays where it is put on the ground and lives as
/// long as the effect still does.
///
/// - The kind 0: a geographer at 0.7 of its size, any way round the one who sings and 5 to 49 out, its first
///   action, a frame every 4 ticks.
/// - The others: ten in a ring 40 out, a tenth of a turn apart, each in one of its eight first actions (which
///   are the ways a monster stands) drawn by lot, a frame every 6 to 8 ticks: wargs at 0.6 of their size (2),
///   violies (3), minerals (5), Ktullanuxes (6), Garms (7), each at its own size; succubi (4) in one of the
///   actions 40 to 63, and Miyabi dolls (8) in one of 48 to 55.
fn monster(kind: usize, course: &Course) -> Option<Prim> {
    if course.tick >= if kind >= 2 { 10 } else { 50 } {
        return None;
    }
    let mut prim = Prim::new(Shape::SpriteGlow, MONSTERS[kind.saturating_sub(1)]);
    (prim.duration, prim.alpha, prim.left) = (course.lasts - course.tick, 0.0, Some(course.walked));
    let (angle, radius) = if kind >= 2 { ((course.tick * 36) as f32, 40.0) } else { (lot(360), lot(45) + 5.0) };
    prim.size = match kind {
        0 => 0.7,
        2 => 0.6,
        _ => 1.0,
    };
    prim.moved = [radius * sine(angle), 0.0, radius * cosine(angle)];
    prim.anim_speed = if kind >= 2 { lot(3) as i32 + 6 } else { 4 };
    prim.action = match kind {
        0 => 0,
        // (The client draws which of three poses first, then the way she stands.)
        4 => [56, 48, 40][lot(3) as usize] + lot(8) as usize,
        8 => lot(8) as usize + 48,
        _ => lot(8) as usize,
    };
    Some(prim)
}

/// `CRagEffect::TexturedSlam`, for its kinds 1 and 2 (the modes 6 and 7 of the kind 147): one picture facing
/// the screen 40 over the place where the effect began, wider than tall, that flashes up 21 to a corner and
/// larger, settles, and goes after 90 ticks; white with its light added, and a tenth larger in blue (the
/// mode 7: in red).
fn slam(picture: &'static str, mode: i8, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::RisingFan, picture, false, course);
    let mut part = Part { out: 21.0, ..Part::default() };
    part.flags[4] = mode;
    prim.parts.push(part);
    prim
}

/// `CRagEffect::TexturedBeam`: two wedges of light (the kind 154) that stay where they are launched, from
/// 6 over the effect's place 50 out towards the place aimed at, 25 degrees to each side of the way, 5 wide at
/// their ends. `flag` gives their colours: 2 is white.
fn beam(flag: i8, picture: &'static str, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::BladeFins, picture, false, course);
    let mut part = Part { tall: 5.0, out: 100.0, rise: crate::prim::angle_of(course.shared.aim[0], course.shared.aim[2]), ..Part::default() };
    (part.heights[0], part.heights[2], part.flags[0]) = (50.0, 80.0, flag);
    prim.parts.push(part);
    prim
}

/// `CRagEffect::TexturedDrop`: one picture that faces the screen (the kind 156), 6 from its middle to a
/// corner, at an alpha of 120, hovering about 6 over the one the effect is on from anywhere in its swing by
/// lot. `flag` says how it is laid on: 2 is white, its light added.
fn drop_note(picture: &'static str, flag: i8, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::Hover, picture, true, course);
    let mut part = Part { out: 6.0, alpha: 120, turned: lot(360) as i32, round: 45, ..launched::part() };
    part.flags[1] = flag;
    prim.parts.push(part);
    prim
}

/// The notes of Deep Sleep Lullaby and of Reverberation, and the line and the sparks of Metallic Sound, among
/// the client's pictures of effects. (With the option to show fewer effects on the client takes those of
/// Reverberation and of Metallic Sound from `mineffect`.)
const NOTES: [&str; 4] = ["melody_a.bmp", "melody_b.bmp", "new_reverberation/rev_note_01.tga", "new_reverberation/rev_note_02.tga"];
const METAL: [&str; 4] = [
    "new_metalicsound/new_metalicsound_line/met_damege_line_1.tga",
    "new_metalicsound/new_metalicsound_icon/met_particle_1.tga",
    "new_metalicsound/new_metalicsound_icon/met_particle_2.tga",
    "new_metalicsound/new_metalicsound_icon/met_particle_3.tga",
];

/// A turn brought into the round, from 0 up to 360, as the two functions below leave the turns they give.
fn round(turn: f32) -> f32 {
    turn.rem_euclid(360.0)
}

/// `CRagEffect::TexturedForkBolt`: one part of two sparks (the kind 155), 3 to 5 from the middle to a
/// corner, that begin 5 over the feet of the one the effect is on and fly 0.8 to 1 a tick, each within 45
/// degrees of the way to the place aimed at and making for another turn within that; they stay in the world
/// where they are launched. `flag` says how they are drawn: 2 is white, the light added.
fn fork_bolt(picture: &'static str, flag: i8, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::WanderingFlames, picture, false, course);
    let way = crate::prim::angle_of(course.shared.aim[0], course.shared.aim[2]);
    let mut part = Part { tall: lot(21) * 0.01 + 0.8, out: lot(21) * 0.1 + 3.0, rise: way, ..launched::part() };
    let near = || round(way as f32 - 45.0 + lot(91));
    part.heights[..10].copy_from_slice(&[way as f32, near(), near(), 180.0, 180.0, way as f32, near(), near(), 180.0, 180.0]);
    (part.at[2], part.at[0], part.flags[0]) = ([0.0, -5.0, 0.0], [0.0, -5.0, 0.0], flag);
    prim.parts.push(part);
    prim
}

/// `CRagEffect::TexturedSpinShards`: four parts of two notes each (the kind 170), that begin 5 over the feet
/// of the one the effect is on and fly 0.4 to 0.42 a tick any way round, each making for another turn by lot;
/// 3 to 5 from the middle to a corner, or 1 to 5 for the kind 3. They stay in the world where they are
/// launched. `kind` says how they are drawn: 1 is red over white, 3 white and only one note of each two.
///
/// (The client measures every turn from the way to the place the effect aims at, which these effects are
/// never told: the way from the one they are on to the corner of the map. As every turn is that way and a
/// lot of the whole round more, it makes no difference, and the turns are measured from nothing here.)
fn spin_shards(picture: &'static str, kind: i8, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::WanderingFlames2, picture, false, course);
    prim.parts.extend((0..4).map(|_| {
        let reach = if kind == 3 { lot(41) * 0.1 + 1.0 } else { lot(21) * 0.1 + 3.0 };
        let mut part = Part { tall: lot(21) * 0.001 + 0.4, out: reach, ..launched::part() };
        part.heights[..10].copy_from_slice(&[0.0, lot(360), lot(360), 180.0, 180.0, 0.0, lot(360), lot(360), 180.0, 180.0]);
        (part.at[2], part.at[0], part.flags[0]) = ([0.0, -5.0, 0.0], [0.0, -5.0, 0.0], kind);
        part
    }));
    prim
}
