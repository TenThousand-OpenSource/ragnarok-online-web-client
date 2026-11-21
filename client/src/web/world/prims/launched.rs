//! Primitives as several files of effects launch them, each written once: what a function of the client's
//! `CRagEffect` that several effects call launches, or a beginning that several of its functions share.
//!
//! - sprites: one thrown out from over the one the effect is on (`thrown`), one that flies in a bow (`bowed`),
//!   one thrown level to a place (`thrown_level`), and the names of the sprites several files show;
//! - the band of a hit, the way it goes and where it starts (`band`, `way`, `ahead`);
//! - a primitive whose picture is added whole to what is behind it (`glowing`);
//! - a primitive of a kind made of the client's parts and a part of one (`of_parts`, `part`,
//!   `crate::prim::Tei`), and what several effects make of them: the beams of `TexturedQuadSpin`
//!   (`quad_spin`), the rings of a portal (`ripples`), the sparks of a soul's light (`soul_light`);
//! - pictures over the ground: the mark of `TexturedSkillAura` (`skill_aura`) and the square of
//!   `TexturedLevelRing` (`square`).
//!
//! A file that uses one of these names its pictures in its own `PICTURES`.

use super::*;
use crate::prim::{Change, Part, Tei, Walls};

/// The sprites several files show, among the client's sprites (`이팩트\name.spr`; the shadow is `Shadow.spr`).
/// `sprites.rs` has them read.
pub(super) const PARTICLE1: &str = "이팩트/particle1";
pub(super) const PARTICLE2: &str = "이팩트/particle2";
pub(super) const PARTICLE3: &str = "이팩트/particle3";
pub(super) const PARTICLE4: &str = "이팩트/particle4";
pub(super) const FIREBALL: &str = "이팩트/fireball";
pub(super) const SPEAR: &str = "이팩트/창";
pub(super) const SIGHT: &str = "이팩트/sight";
pub(super) const SAKURA: &str = "이팩트/sakura01";
pub(super) const SHADOW: &str = "shadow";

/// A sprite turned by a longitude and then tipped by a latitude, put so far along its own forward from a place
/// 10 above the one the effect is on: how `Hit1` and `Steal` begin theirs.
pub(super) fn thrown(kind: Shape, sprite: &'static str, longitude: f32, latitude: f32, far: f32) -> Prim {
    let mut spark = Prim::new(kind, sprite);
    (spark.yaw_first, spark.longitude, spark.latitude) = (true, longitude, latitude);
    let reach = spark.forward(far);
    (spark.put, spark.deep) = ((reach[0], -reach[1] - 10.0), reach[2]);
    spark
}

/// A sprite that flies in a bow as `SoulStrike`, `WaterBall2`, `BloodDrain` and `SonicSlashFan` launch theirs
/// (`PP_3DPARTICLESPLINE`): from 10 above the one the effect is on, for so many ticks, it draws back at `back`
/// a tick, slowing to a stop at half its time and then returning, and swings out to a side at 2 a tick and
/// back in the same way. `roll` turns the side it swings to about the line of flight. Which way it flies and
/// how fast is the caller's to give.
pub(super) fn bowed(sprite: &'static str, duration: i32, back: f32, roll: f32) -> Prim {
    let mut bolt = Prim::new(Shape::ParticleSpline3d, sprite);
    bolt.duration = duration;
    bolt.put.1 = -10.0;
    bolt.grav_speed = back;
    bolt.grav_accel = -(bolt.grav_speed / bolt.duration as f32) * 2.0;
    bolt.lati_speed = -2.0;
    bolt.lati_accel = -(bolt.lati_speed / bolt.duration as f32) * 2.0;
    bolt.roll = roll;
    bolt
}

/// A sprite thrown as `FireBall` and `SpearBmr` throw theirs: from 15 above the striker it flies level to where
/// the one struck stood, there in the ticks it lives, and ends on coming within 3 of that place
/// (`pattern & 0x100`); it is turned on the screen the way it flies (`pattern & 0x20`), and shows a new frame
/// every tick.
pub(super) fn thrown_level(sprite: &'static str, duration: i32, aim: [f32; 3]) -> Prim {
    let mut thrown = Prim::new(Shape::Particle3d, sprite);
    thrown.duration = duration;
    (thrown.seeks, thrown.put.1) = (true, -15.0);
    thrown.longitude = -faced(aim[0], aim[2]);
    thrown.points = Some(pointing(aim[0], aim[2]));
    thrown.speed = -(f32::hypot(aim[0], aim[2]) / thrown.duration as f32);
    thrown.anim_speed = 1;
    thrown
}

/// A primitive the client draws by adding its picture's colours to what is behind it, whatever its alpha is
/// (a `renderFlag` with 2: both blends are "one"). Its alpha is put at its most and left there: nothing the
/// client does to the alpha of such a primitive shows.
pub(super) fn glowing(mut prim: Prim) -> Prim {
    (prim.adds, prim.alpha, prim.fade_out) = (true, prim.max_alpha, -1);
    prim
}

/// The way a band of a hit goes, the client's `(int)(180 - angle)` of the way the one struck faces, a turn
/// added to one under nothing.
pub(super) fn way(facing: f32) -> f32 {
    let way = (180.0 - facing).trunc();
    if way < 0.0 { way + 360.0 } else { way }
}

/// A band as the hits launch it: 10 above the feet of the one struck, upright and across a way, going that way
/// at 0.7 a tick as it slows by half.
pub(super) fn band(picture: &'static str, duration: i32, way: f32) -> Prim {
    let mut band = Prim::new(Shape::Cylinder3d, picture);
    band.duration = duration;
    (band.latitude, band.longitude) = (-90.0, way);
    band.put.1 = -10.0;
    band.speed = 0.7;
    band.accel = -(band.speed / duration as f32) * 0.5;
    band
}

/// How far a band tipped as the hits tip theirs (a latitude of -90) is from where it was put when the client
/// starts it off along its own space (`deltaPos += transform * (0, -2, lift)`): its own upright lies level and
/// points the way it is turned, so it starts 2 along that way; what is along its own forward is down.
pub(super) fn ahead(way: f32, lift: f32) -> [f32; 3] {
    [2.0 * sine(way), -lift, 2.0 * cosine(way)]
}

/// A primitive of a kind made of the client's parts, that lives as long as its effect and is drawn with one
/// picture. One that `follows` goes with the one its effect is on (the client's `FollowMaster`); any other
/// stays in the world where it was launched.
pub(super) fn of_parts(kind: Tei, picture: &'static str, follows: bool, course: &Course) -> Prim {
    let mut prim = Prim::new(Shape::Tei(kind), picture);
    prim.duration = course.lasts;
    prim.left = (!follows).then_some(course.walked);
    prim
}

/// A part with a number of its own for the lots it draws as it goes.
pub(super) fn part() -> Part {
    Part { seed: lot(1 << 30) as u32, ..Part::default() }
}

/// `CRagEffect::TexturedQuadSpin` with `alpha_center.tga`: four beams a quarter of a turn apart, 12 over the
/// feet, each in a plane tipped by lot. `phase` turns the four on, by 7 degrees for the kind 3 and by 22 for
/// the others; the kinds 2, 3 and 6 begin at once, the others after 24 ticks.
pub(super) fn quad_spin(phase: i32, kind: i32, course: &Course) -> Prim {
    let mut beams = of_parts(Tei::FanBeam, "alpha_center.tga", true, course);
    let at_once = matches!(kind, 2 | 3 | 6);
    beams.parts.extend((0..4).map(|index| {
        let mut beam = part();
        (beam.heights[0], beam.heights[4]) = (-12.0, kind as f32);
        (beam.process, beam.out) = (if at_once { 0 } else { -24 }, 2.0);
        beam.rise = phase * if kind == 3 { 7 } else { 22 } + index * 90;
        beam.turned = lot(360) as i32;
        beam
    }));
    beams
}

/// The rings of `CRagEffect::BlueRingWave` and `VioletRingCloud` (the kind 63): three rings that widen from
/// the middle one after the other, the second 10 ticks and the third 20 after the first, each 6.001 tall and
/// all but flat (rising at 2, 3 and 4 degrees), and a fourth part that says which rings they are (`kind`: 1
/// for those of a portal about to open, which are over after 120 ticks; none for those of an open one).
/// `holds` is the number the client gives each ring besides (`alphaT`), which the kind does not read.
pub(super) fn ripples(picture: &'static str, size: f32, holds: i32, kind: i32, course: &Course) -> Prim {
    let mut rings = of_parts(Tei::Ripple, picture, true, course);
    rings.size = size;
    rings.parts.extend((0..3).map(|index| Part {
        process: -10 * index,
        round: 360,
        tall: 6.001,
        turned: 25 * index,
        rise: index + 2,
        alpha_top: holds,
        flags: [1; 21],
        ..Part::default()
    }));
    rings.parts.push(Part { live: false, alpha_top: kind, ..Part::default() });
    rings
}

/// `CRagEffect::TexturedSkillAura` with its type of 2 (`EF_DETECT2`), 3 (`EF_DETECT3`), 4 (`EF_DETECT4`), 5
/// (`EF_NC_ANAL`) or 6 (`EF_BLACKBODY`): a picture lying flat 5 above the ground where the effect began, that
/// widens from nothing faster and faster for 17 ticks (10 for the types 3 and 6) and slowly after, fades from
/// its 40th tick (its 30th) and is gone at its 57th. Type 4 is tinted red, type 5 a pale green, type 6 all but
/// black. (The function also gives it a speed along the three directions, which a primitive without
/// `pattern & 0x100` never goes by.)
pub(super) fn skill_aura(kind: i32) -> Prim {
    let mut aura = Prim::new(Shape::Texture3d, "fashasha.tga");
    (aura.duration, aura.roll, aura.put.1) = (57, 90.0, -5.0);
    (aura.width_speed, aura.height_speed) = (1.5, 1.5);
    (aura.width_accel, aura.height_accel) = (aura.width_speed / 14.0, aura.height_speed / 14.0);
    let (turns, fades) = if matches!(kind, 3 | 6) { (10, 30) } else { (17, 40) };
    aura.fade_out = fades;
    aura.changes = Some(Change { width_speed: 0.1, width_accel: 0.1 / 30.0, height_speed: 0.1, height_accel: 0.1 / 30.0, ..Change::at(turns) });
    aura.colour = match kind {
        4 => [255.0, 140.0, 140.0],
        5 => [210.0, 250.0, 210.0],
        6 => [10.0; 3],
        _ => aura.colour,
    };
    aura
}

/// `CRagEffect::TexturedLevelRing`, by its level (0 to 4 or 10 here): a square of the kind 88 round the one it
/// is on, its light added, that begins `waits` ticks late. Level 0: 15 to its corners, 5 over the ground,
/// white; levels 1 and 2 the same in red and in yellow (the sizes 101 and 102; the red one the client draws
/// twice, and whoever launches it launches two). Level 3: 13 to its corners, 5 to 10 over the
/// ground, blue. Levels 4 and 10: 15 to its corners and 5 over the ground, a deep blue that comes up three
/// times as fast, and a dark red. (`RenderSizedSquare` by the primitive's `size`, 111, 113 and 114; the last
/// two it draws over everything, whatever stands before them, and here they are drawn as the other effects
/// are.)
pub(super) fn square(picture: &'static str, level: i32, waits: i32, lasts: i32) -> Prim {
    let mut square = Prim::new(Shape::Walls(Walls::Square { quick: level == 4 }), picture);
    (square.duration, square.adds) = (lasts, true);
    let (colour, out, up) = match level {
        0 => ([255.0; 3], 15.0, 5.0),
        1 => ([255.0, 0.0, 0.0], 15.0, 5.0),
        2 => ([255.0, 255.0, 0.0], 15.0, 5.0),
        3 => ([55.0, 55.0, 255.0], 13.0, 5.0 + lot(6)),
        4 => ([5.0, 5.0, 255.0], 15.0, 5.0),
        _ => ([155.0, 5.0, 5.0], 15.0, 5.0),
    };
    square.colour = colour;
    let mut part = Slot::wall(0, out, 0, 0.0, level);
    (part.process, part.at) = (-waits, [0.0, up, 0.0]);
    square.slots = vec![part];
    square
}

/// `CRagEffect::TexturedStorm2` with its kind 0, as `EF_SOULLIGHT` calls it: two parts of three sparks each
/// (`PP_SOULPARTICLE`), all at the place they are told (the client's directions, from the one linked), 1 to 2
/// from the middle to a corner by lot, blue, that part and fade over the 100 ticks the effect lives.
pub(super) fn soul_light(at: [f32; 3], course: &Course) -> Prim {
    let mut prim = of_parts(Tei::SoulParticle, "whitelight.tga", true, course);
    prim.duration = 100;
    prim.parts.extend((0..2).map(|_| {
        let mut part = Part { tall: 0.07, out: lot(11) * 0.1 + 1.0, ..part() };
        (part.at[0], part.at[2], part.at[3]) = (at, at, at);
        part.heights[..12].iter_mut().for_each(|turn| *turn = lot(360));
        part
    }));
    prim
}
