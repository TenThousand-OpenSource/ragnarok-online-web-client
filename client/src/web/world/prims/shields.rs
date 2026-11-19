//! The effects of walls, shields and what is thrown like one, whose primitives are of the kinds the client
//! builds out of its parts (`crate::prim::Tei`); each is its function of `CRagEffect` with the same figures:
//!
//! - `EF_GRANDCROSS` (226), `EF_GRANDCROSS2` (450): Grand Cross and Grand Darkness, four quarters of a wall
//!   round a cross of two bars (`GrandCrossRings`, `GRANDCROSS2`);
//! - `EF_SHIELDBOOMERANG` (249), `EF_SHIELDBOOMERANG2` (494): a shield, or an axe, thrown at someone that
//!   comes back (`TexturedShieldThrow`);
//! - `EF_SHIELDBOOMERANG3` (520): the five shields of Shield Chain, each with a tail of itself
//!   (`ShieldBoomerang3`);
//! - `EF_TANJI` (265): the spheres of Finger Offensive (`TANJI`);
//! - `EF_GUARD` (336), `EF_GUARD3` (675), `EF_MAXPAIN` (1032), `EF_GUARD5` (1600): half a ball of panels
//!   before the one who guards (`KyrieGuard`);
//! - `EF_GUARD2` (496): Preserve, the same panels with another picture, that go round (`BlackMaximize`), and
//!   the body of the one it is on flashing; `EF_GUARD4` (782): the same with the picture of Neutral Barrier,
//!   the body flashing yellow.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::{Tei, angle_of};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 10] = [
    "ring_white.tga", "ring_black.tga", "ring_yellow.tga", "shield_boomerang.bmp", "토마.bmp", "blue_ivy.bmp",
    "guardK.tga", "guardK2.tga", "a01.bmp", "성체강복.tga"
];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with at
/// its first tick (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        226 | 450 => (999, "effect/cru_grand cross.wav"),
        249 | 494 | 520 => (499, "effect/cru_shield boomerang.wav"),
        265 => (300, "effect/mon_탄지신통.wav"),
        336 | 675 | 1600 => (200, "effect/kyrie_guard.wav"),
        496 | 782 => (200, "effect/black_maximize_power_sword_bic.wav"),
        1032 => (999, "effect/kyrie_guard.wav"),
        _ => return None,
    })
}

/// Whether an effect of this file is one the striker launches and is then told the place of the one struck
/// (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS`, for Shield Boomerang, the thrown axe
/// and Finger Offensive).
fn thrown(effect: u32) -> bool {
    matches!(effect, 249 | 265 | 494)
}

/// One tick of an effect of this file; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    // `EF_GUARD2`: from its tenth to its thirtieth tick the body of the one it is on flashes.
    if effect == 496 && (10..=30).contains(&course.tick) {
        course.touch.flashes(course.tick, [250, 250, 250]);
    }
    // `EF_GUARD4`: over the same ticks it flashes yellow.
    if effect == 782 && (10..=30).contains(&course.tick) {
        course.touch.flashes(course.tick, [250, 250, 0]);
    }
    if effect == 520 && matches!(course.tick, 0 | 3 | 6 | 9 | 12) {
        prims.push(chained_shield(course));
    }
    if course.tick != 0 {
        return;
    }
    match effect {
        782 => prims.push(panels(&["성체강복.tga", "a01.bmp"], 10.0, 1, course)),
        226 => prims.extend([cross_walls(false, course), cross_bars(false, course)]),
        450 => prims.extend([cross_walls(true, course), cross_bars(true, course)]),
        249 => prims.push(shield_throw("shield_boomerang.bmp", course)),
        494 => prims.push(shield_throw("토마.bmp", course)),
        265 => prims.push(spheres(course)),
        336 => prims.push(cage(0, course)),
        675 => prims.push(cage(2, course)),
        1032 => prims.push(cage(3, course)),
        1600 => prims.push(cage(5, course)),
        496 => prims.push(turning_cage(course)),
        _ => {}
    }
}

/// `CRagEffect::GrandCrossRings` with no angle: four quarters of a wall of `ring_white.tga`, pale red (`dark`,
/// for Grand Darkness: `ring_black.tga`, dark grey and mixed in), 120 tall, each 19.9 round a middle of its own
/// 23 to the east or west and 23 to the north or south of where the effect began, their hollow sides outwards.
fn cross_walls(dark: bool, course: &Course) -> Prim {
    let mut walls = of_parts(Tei::Gi3, if dark { "ring_black.tga" } else { "ring_white.tga" }, false, course);
    walls.size = if dark { 12.0 } else { 1.0 };
    walls.parts.extend([(180, 23, 23), (90, 23, -23), (0, -23, -23), (270, -23, 23)].map(|(turned, east, north)| {
        let mut quarter = part();
        (quarter.round, quarter.tall, quarter.out, quarter.rise, quarter.turned) = (90, 120.0, 19.9, 90, turned);
        (quarter.flags[0], quarter.flags[1], quarter.flags[4]) = (east, north, i8::from(dark));
        quarter
    }));
    walls
}

/// `CRagEffect::GRANDCROSS2`: two long boxes of `ring_yellow.tga` (`dark`: `ring_black.tga`) across each other
/// where the effect began, each 24 to each side along its length and 3.2 across, that rise to 60.
fn cross_bars(dark: bool, course: &Course) -> Prim {
    let mut bars = of_parts(Tei::GrandCross2, if dark { "ring_black.tga" } else { "ring_yellow.tga" }, false, course);
    bars.size = 1.0;
    bars.parts.extend([0, 90].map(|turned| {
        let mut bar = part();
        (bar.tall, bar.turned) = (60.0, turned);
        (bar.heights[0], bar.heights[1]) = (24.0, 3.2);
        bar
    }));
    bars
}

/// How far the place the effect aims at is from where the one it is on stands now, level, the whole degrees
/// of the way there (`Get_Angle`), and how far down it is (the client's y).
fn way_to_aim(course: &Course) -> (f32, i32, f32) {
    let (east, north) = (course.shared.aim[0] - course.walked[0], course.shared.aim[2] - course.walked[2]);
    (east.hypot(north), angle_of(east, north), course.walked[1] - course.shared.aim[1])
}

/// The first part of a primitive of the kind 42 as `TexturedShieldThrow` and `TANJI` launch it: 8 over the
/// feet of the thrower, to fly 2 a tick as far as the place aimed at, and down as much a tick as brings it to
/// that place's height there. (For a place no way off the client divides by nothing; here it flies level.)
fn thrown_head(course: &Course) -> crate::prim::Part {
    let (far, way, down) = way_to_aim(course);
    let mut head = part();
    (head.tall, head.turned) = (far, way);
    head.at[0] = [0.0, -8.0, 0.0];
    head.heights[1] = 2.0;
    head.heights[2] = if far > 0.0 { 2.0 / far * down } else { 0.0 };
    head
}

/// `CRagEffect::TexturedShieldThrow`: a square of that picture, 7 from its middle to a corner, thrown from the
/// one the effect is on at the place of the one struck, that turns round there and comes back, three fainter
/// ones behind it.
fn shield_throw(picture: &'static str, course: &Course) -> Prim {
    let mut shield = of_parts(Tei::ShieldBoomerang, picture, false, course);
    shield.size = 0.0;
    let mut head = thrown_head(course);
    (head.out, head.alpha) = (7.0, 255);
    shield.parts.push(head);
    shield.parts.extend((0..3).map(|_| part()));
    shield
}

/// `CRagEffect::TANJI`: a sphere of `blue_ivy.bmp`, pale blue, that waits 15 ticks, flies at the place of the
/// one struck and on past it, three fainter ones behind it; the more it was told of (`count`), the larger.
fn spheres(course: &Course) -> Prim {
    let mut sphere = of_parts(Tei::ShieldBoomerang, "blue_ivy.bmp", false, course);
    sphere.size = 0.0;
    let mut head = thrown_head(course);
    (head.process, head.out, head.alpha) = (-15, course.count as f32 * 0.4 + 2.0, 5);
    head.flags[2] = 1;
    sphere.parts.push(head);
    sphere.parts.extend((0..3).map(|_| part()));
    sphere
}

/// The primitive of the kind 61 as `KyrieGuard` and `BlackMaximize` launch it: half a ball of panels that goes
/// with the one the effect is on, before that one as it faced when the effect began (the client's
/// `render.angle` and 270).
fn panels(pictures: &'static [&'static str], tall: f32, number: i8, course: &Course) -> Prim {
    let mut cage = of_parts(Tei::Guard, pictures[0], true, course);
    (cage.pictures, cage.size) = (pictures, 11.0);
    let mut half = part();
    (half.tall, half.out, half.turned) = (tall, 1.0, (course.facing as i32 + 270).rem_euclid(360));
    half.flags[0] = number;
    cage.parts.push(half);
    cage
}

/// `CRagEffect::KyrieGuard` with `guardK.tga`: the panels 7.8 from their middle, `guardK2.tga` on the
/// outermost; the kind gives the colour (0 pale green, 2 and 3 orange, 5 blue).
fn cage(kind: i8, course: &Course) -> Prim {
    panels(&["guardK.tga", "guardK2.tga"], 7.8, kind, course)
}

/// `CRagEffect::BlackMaximize` with `a01.bmp`: the panels 10 from their middle, all of `a01.bmp`, that go
/// round 10 degrees a tick.
fn turning_cage(course: &Course) -> Prim {
    panels(&["a01.bmp", "a01.bmp"], 10.0, 1, course)
}

/// `CRagEffect::ShieldBoomerang3` with `shield_boomerang.bmp`, one of its five: a shield that flies 8 over the
/// ground from 37.5 away, a side drawn by lot, through the place of the one struck and on, 2.5 a tick, with
/// three fainter shields behind it.
fn chained_shield(course: &Course) -> Prim {
    let mut shields = of_parts(Tei::ShieldBoomerang3, "shield_boomerang.bmp", false, course);
    let mut head = part();
    (head.out, head.alpha, head.turned) = (5.0, 25, lot(360) as i32);
    head.heights[1] = 2.5;
    let back = head.heights[1] * 15.0;
    head.at[0] = [-back * cosine(head.turned as f32), -8.0, -back * sine(head.turned as f32)];
    shields.parts.push(head);
    shields.parts.extend((0..3).map(|_| part()));
    shields
}
