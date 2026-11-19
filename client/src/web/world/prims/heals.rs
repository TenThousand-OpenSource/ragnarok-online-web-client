//! Healing and Resurrection, each with the figures of its function of `CRagEffect`:
//!
//! - `EF_HEALSP` (7): the light of a small healing (`HealSP`);
//! - `EF_HEAL` (312), `EF_HEAL3` (320): Heal (`HealBurst`);
//! - `EF_HEAL2` (313), `EF_HEAL4` (325), `EF_ABSORBSPIRITS` (253): the taller healing lights and the blue one
//!   of Absorb Spirits (`HealRingSparkle`);
//! - `EF_REVIVE` (140): Resurrection (in the client's `CRagEffect::OnProcess` itself);
//! - `EF_REPAIRTIME` (785): Repair, spanners that rise round the machine mended (`TexturedRisingOrbs` with its
//!   kind 2);
//! - `EF_FIRSTAID` (309): the glitter of First Aid (`FirstAid`; `PP_FIRSTAID`).
//!
//! The client runs the four `EF_HEAL`s on every pass of its loop, not only on its ticks (`CRagEffect::OnProcess`
//! lets them past its wait for a tick), so how soon they launch their motes and end depends there on the
//! machine, while what they launch moves by ticks; here they run a tick at a time like everything else.
//!
//! The client also has a class of its own for these numbers (`CHealEffect`, made when an effect is asked of its
//! `EffectLists`); Heal, Absorb Spirits and the others ported here are launched with `LaunchEffect`
//! (`CSession::PlaySkillEffect`, `SkillActionSoulAscetic`), which makes a `CRagEffect`: that is what is here.

use super::launched::PARTICLE2;
use super::*;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES, &SPANNERS], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 7] = [
    "pikapika2.bmp",
    "ring_blue.tga",
    "ring_white.tga",
    "pok1.tga",
    "pok3.tga",
    "alpha_down.tga",
    "ring_yellow.tga",
];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        7 => (100, "_heal_effect.wav"),
        // `EF_REVIVE` has no sound in its function.
        140 => (250, ""),
        253 => (100, "effect/흡기.wav"),
        // `TexturedRisingOrbs` has no sound for its kind 2.
        785 => (300, ""),
        309 => (299, "_heal_effect.wav"),
        312 | 313 | 320 | 325 => (100, "_heal_effect.wav"),
        _ => return None,
    })
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    match effect {
        7 => heal_sp(course.tick, prims),
        140 => revive(course.tick, course.lasts, prims),
        253 => heal_rings("ring_blue.tga", 2, course, prims),
        309 if course.tick == 0 => prims.push(first_aid(course)),
        312 => heal_burst(true, course, prims),
        313 => heal_rings("ring_white.tga", 0, course, prims),
        320 => heal_burst(false, course, prims),
        325 => heal_rings("ring_white.tga", 1, course, prims),
        785 if course.tick <= 20 && course.tick % 4 == 0 => prims.push(spanners(course)),
        _ => {}
    }
}

/// The colours `RenderTeiRect` draws a wall in by the primitive's `size`: 9, 5 (none of its own: white) and 4.
/// All three are added to what is behind them.
const GREEN: [f32; 3] = [140.0, 210.0, 140.0];
const WHITE: [f32; 3] = [255.0, 255.0, 255.0];
const BLUE: [f32; 3] = [100.0, 100.0, 255.0];

/// Walls of light round the one healed, for as long as the effect lives.
fn walls(picture: &'static str, colour: [f32; 3], lasts: i32, slots: Vec<Slot>) -> Prim {
    let mut walls = Prim::new(Shape::Casting3d, picture);
    (walls.duration, walls.colour, walls.adds, walls.slots) = (lasts, colour, true, slots);
    walls
}

/// Four motes of light as `HealBurst` and `HealRingSparkle` launch them: each somewhere within 6 east and
/// north of the one healed, at the feet, 2 (or `reach`) to 1 more from its middle to its corners, rising by
/// `climbs` to 0.4 more a tick. They are pale green (`RenderFieldPanel`, kind 11), their light added, and stay
/// in the world where they were launched. One that flashes shows `pok3.tga` for two ticks.
fn motes(course: &Course, reach: f32, climbs: f32, flashes: bool) -> Prim {
    let mut motes = Prim::new(Shape::ParticleUp, "pok1.tga");
    if flashes {
        motes.pictures = &["pok1.tga", "pok3.tga"];
    }
    (motes.duration, motes.colour, motes.adds, motes.left) = (course.lasts, [220.0, 250.0, 220.0], true, Some(course.walked));
    motes.slots = (0..4)
        .map(|_| {
            let out = lot(10) * 0.1 + reach;
            let east = lot(13) - 6.0;
            let north = lot(13) - 6.0;
            Slot::mote([east, 0.0, north], out, lot(40) * 0.01 + climbs, flashes)
        })
        .collect();
    motes
}

/// `CRagEffect::HealBurst`, with its flag of 0 (`first`: `EF_HEAL`) or 1 (`EF_HEAL3`, Heal): two upright walls
/// of light round the one healed, one just outside the other, begun at any turn; and every sixth tick after
/// the first, up to the 50th (the 25th for Heal), four motes.
///
/// `EF_HEAL`: the walls green, 40 tall, 4.6 and 4.8 out, fading from their 60th tick; the motes do not flash.
/// Heal: the walls white, 40 to 59 tall, 4.5 to 5 out and 0.2 more, fading from their 30th tick; the motes
/// rise a little faster and flash.
fn heal_burst(first: bool, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 0 {
        let (colour, holds) = if first { (GREEN, 60) } else { (WHITE, 30) };
        let inner_turn = lot(360) as i32;
        let (tall, out) = if first { (40.0, 4.6) } else { (lot(20) + 40.0, 4.5 + lot(100) * 0.005) };
        let outer_turn = lot(360) as i32;
        let slots = vec![Slot::wall(inner_turn, out, 90, tall, holds), Slot::wall(outer_turn, out + 0.2, 90, tall, holds)];
        prims.push(walls("alpha_down.tga", colour, course.lasts, slots));
        return;
    }
    if course.tick > if first { 50 } else { 25 } || course.tick % 6 != 0 {
        return;
    }
    prims.push(motes(course, 2.0, if first { 0.3 } else { 0.4 }, !first));
}

/// `CRagEffect::HealRingSparkle`, by its type: 0 (`EF_HEAL2`), 1 (`EF_HEAL4`) or 2 (`EF_ABSORBSPIRITS`, with
/// `ring_blue.tga`; the others with `ring_white.tga`). Four walls of light round the one it is on: two upright
/// ones 60 tall, 4.6 and 5 out, begun opposite each other, and two low ones leaning outwards (at 50 and 48
/// degrees), 5 and 5.2 out, begun at any turn, 12 tall for type 0 and 18 for the others. They are green, blue
/// for type 2, and fade from their 60th tick for type 0 and their 40th for the others. Types 0 and 1 launch
/// four motes every sixth tick after the first, up to the 50th.
///
/// Type 1 has more: at its second tick four more walls of `alpha_down.tga`, green, two upright ones 90 tall, 4.2
/// and 4.3 out, a quarter turn apart, and two 10 tall leaning at 46 and 44 degrees, 5.3 and 5.4 out, fading
/// from their 30th tick; and four larger motes at every ninth tick from the third, up to the 50th.
fn heal_rings(picture: &'static str, kind: u8, course: &mut Course, prims: &mut Vec<Prim>) {
    let tick = course.tick;
    if tick == 0 {
        let holds = if kind == 0 { 60 } else { 40 };
        let low = if kind == 0 { 12.0 } else { 18.0 };
        let (third, fourth) = (lot(360) as i32, lot(360) as i32);
        let slots = vec![Slot::wall(0, 4.6, 90, 60.0, holds), Slot::wall(180, 5.0, 90, 60.0, holds), Slot::wall(third, 5.0, 50, low, holds), Slot::wall(fourth, 5.2, 48, low, holds)];
        prims.push(walls(picture, if kind == 2 { BLUE } else { GREEN }, course.lasts, slots));
    } else if tick <= 50 && tick % 6 == 0 && kind < 2 {
        prims.push(motes(course, 2.0, if kind == 0 { 0.4 } else { 0.6 }, true));
    }
    if kind != 1 {
        return;
    }
    if tick == 1 {
        let (third, fourth) = (lot(360) as i32, lot(360) as i32);
        let slots = vec![Slot::wall(0, 4.2, 90, 90.0, 40), Slot::wall(90, 4.3, 90, 90.0, 40), Slot::wall(third, 5.3, 46, 10.0, 30), Slot::wall(fourth, 5.4, 44, 10.0, 30)];
        prims.push(walls("alpha_down.tga", GREEN, course.lasts, slots));
    }
    if tick <= 50 && tick % 9 == 2 {
        prims.push(motes(course, 2.4, 0.4, true));
    }
}

/// `CRagEffect::HealSP`: three upright bands of bluish light round the one healed, one inside the other (2.5,
/// 3.5 and 4.5 out, and 15, 25 and 35 high), each fainter than the next (16, 32 and 64 of 255 at their most),
/// turning; they come up over 30 ticks and fade over the last 20. And every other tick a spark at any side, 3
/// out and 3 more (the client puts it out once as it launches it and once more every tick), that rises from
/// the feet, slowing, at 0.55 of its size, and fades over the last third of its 30 ticks. (The effect makes
/// itself last 100 ticks, whatever the client's table of lifetimes has for it.)
fn heal_sp(tick: i32, prims: &mut Vec<Prim>) {
    if tick == 0 {
        for (out, tall, most) in [(2.5, 15.0, 16.0), (3.5, 25.0, 32.0), (4.5, 35.0, 64.0)] {
            let mut band = Prim::new(Shape::Cylinder3d, "alpha_down.tga");
            band.duration = 100;
            band.long_speed = 5.0;
            (band.outer, band.inner, band.height) = (out, out, tall);
            band.colour = [32.0, 176.0, 232.0];
            (band.alpha, band.max_alpha) = (0.0, most);
            band.alpha_speed = band.max_alpha / 30.0;
            band.fade_out = band.duration - 20;
            prims.push(band);
        }
    }
    if tick % 2 != 0 {
        return;
    }
    let mut spark = Prim::new(Shape::ParticleOrbit3d, PARTICLE2);
    (spark.radius, spark.duration) = (3.0, 30);
    spark.longitude = lot(360);
    spark.grav_speed = -1.2;
    spark.grav_accel = -(spark.grav_speed / spark.duration as f32) / 1.5;
    (spark.put.0, spark.deep) = (sine(spark.longitude) * spark.radius, cosine(spark.longitude) * spark.radius);
    spark.size = 0.55;
    spark.fade_out = spark.duration - spark.duration / 3;
    prims.push(spark);
}

/// `EF_REVIVE` in `CRagEffect::OnProcess`: every 25th tick a ring of yellow round the one raised, a band 7.5
/// high whose foot is 7.5 from the middle, turning faster and faster. The first stands for the whole effect,
/// its top 11 out, and fades over the last tenth. Each later one lives 60 ticks, its top 10.5 out, both its
/// foot and its top closing in, slower and slower, and fades over its second half.
fn revive(tick: i32, lasts: i32, prims: &mut Vec<Prim>) {
    if tick % 25 != 0 {
        return;
    }
    let mut ring = Prim::new(Shape::Cylinder3d, "ring_yellow.tga");
    (ring.long_speed, ring.long_accel) = (1.5, 0.03);
    (ring.inner, ring.height, ring.alpha) = (7.5, 7.5, 240.0);
    if tick == 0 {
        (ring.duration, ring.outer) = (lasts, 11.0);
        ring.fade_out = ring.duration - ring.duration / 10;
    } else {
        (ring.duration, ring.outer) = (60, 10.5);
        (ring.inner_speed, ring.inner_accel, ring.outer_speed, ring.outer_accel) = (-0.12, 0.003, -0.12, 0.003);
        ring.fade_out = ring.duration - ring.duration / 2;
    }
    prims.push(ring);
}

/// `CRagEffect::FirstAid`: one square of `pikapika2.bmp` facing the screen (`PP_FIRSTAID`), 12 over the feet
/// of the one who used it, where that one stood then; 10 from its middle to a corner, turned by lot and
/// swelling from a place of its swell drawn by lot; bluish, its light added (`RenderFieldPanel`'s kind 1).
fn first_aid(course: &Course) -> Prim {
    let mut glitter = launched::of_parts(crate::prim::Tei::FirstAid, "pikapika2.bmp", false, course);
    glitter.size = 0.0;
    let mut part = crate::prim::Part { out: 10.0, round: lot(360) as i32, ..Default::default() };
    (part.at[0], part.heights[8], part.flags[2]) = ([0.0, -12.0, 0.0], lot(360), 1);
    glitter.parts.push(part);
    glitter
}

/// The picture of a spanner, which the client takes from the folder of the pictures of items.
const SPANNER: &str = "유저인터페이스\\item\\스패너.bmp";
const SPANNERS: [&str; 1] = [SPANNER];

/// `CRagEffect::TexturedRisingOrbs` with its kind 2, which the effect runs at every fourth tick up to its
/// 20th: two motes (`PP_PARTICLE_UP`) that show the picture of a spanner, each somewhere within 9 east and
/// north of the one the effect is on and 6 over the feet, 2 to 3 from its middle to its corners, rising 0.2
/// to 0.39 a tick. They are a pale green, their light added (`RenderFieldPanel`, kind 11), and stay in the
/// world where they were launched. (The client has them change picture for two ticks on the way, to the same
/// picture.)
fn spanners(course: &Course) -> Prim {
    let mut motes = Prim::new(Shape::ParticleUp, SPANNER);
    (motes.duration, motes.colour, motes.adds, motes.left) = (course.lasts, [220.0, 250.0, 220.0], true, Some(course.walked));
    motes.slots = (0..2)
        .map(|_| {
            let out = lot(11) * 0.1 + 2.0;
            let (east, north) = (lot(19) - 9.0, lot(19) - 9.0);
            Slot::mote([east, 6.0, north], out, lot(20) * 0.01 + 0.2, false)
        })
        .collect();
    motes
}
