//! Rings and walls of light round one who comes or goes by a teleport or a portal, each with the figures of
//! its arm of `CRagEffect::OnProcess` or of the function that arm calls. A script of an NPC shows most of
//! these by their number (`specialeffect`):
//!
//! - `EF_ENTRY` (6): one who arrives, a low blue band that widens and a tall one that shoots up and sinks
//!   (`Entry`);
//! - `EF_TELEPORTATION` (34): one who teleports away, a blue band that shoots up faster and faster and closes
//!   at its top (in `OnProcess` itself);
//! - `EF_PORTAL` (36): a portal, two tall blue bands that turn, and every 14th tick a ring that widens over
//!   the ground (`Portal`);
//! - `EF_TELEPORTATION2` (304): four violet walls that turn, rise and sink back once (`TeleportRings` with its
//!   level 0; the kind 57 of the primitives);
//! - `EF_EXIT2` (314): four tall purple walls that come up and go, their tops swinging (`TeleportSpiral`; the
//!   kind 60).
//!
//! Only `TeleportSpiral` takes its parent's place every tick; the others stay where they began.

use super::*;
use crate::prim::Walls;

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 3] = ["ring_blue.tga", "magic_violet.tga", "ring_purple.tga"];

/// The blue `RenderTeiRect` draws a wall in whose primitive's `size` is 4, its light added.
const BLUE: [f32; 3] = [100.0, 100.0, 255.0];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        6 | 36 => (100, ""),
        34 => (100, "effect/ef_teleportation.wav"),
        304 | 314 => (200, "effect/ef_teleportation.wav"),
        _ => return None,
    })
}

/// Whether an effect stays where it began: all but `EF_EXIT2`, whose function takes its parent's place every
/// tick.
fn stays(effect: u32) -> bool {
    matches!(effect, 6 | 34 | 36 | 304)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        6 if tick == 0 => prims.extend(entry()),
        34 if tick == 0 => prims.push(teleportation()),
        36 => portal(tick, lasts, prims),
        304 if tick == 0 => prims.push(teleport_rings(lasts)),
        314 if tick == 0 => prims.push(teleport_spiral(lasts)),
        _ => {}
    }
}

/// A band of `ring_blue.tga` round the place of the effect (`PP_3DCYLINDER`, 13), for so many ticks.
fn band(duration: i32) -> Prim {
    let mut band = Prim::new(Shape::Cylinder3d, "ring_blue.tga");
    band.duration = duration;
    band
}

/// `CRagEffect::Entry`: two bands for 55 ticks. One 6.5 tall that sinks by 0.1 a tick, its foot 5 from the
/// middle and its top 6, widening; it turns 10 degrees a tick, comes up by 10 a tick to 245 and fades over
/// its last fifth. And one in pieces of 45 degrees, 4.5 from the middle, that turns the other way and shoots
/// up by 2.5 a tick, slowing so that it sinks back by its end; it begins at 160 and fades over its last third.
fn entry() -> [Prim; 2] {
    let mut low = band(55);
    (low.accel, low.long_speed) = (-0.00007, -10.0);
    (low.outer, low.outer_speed, low.inner) = (6.0, 0.08, 5.0);
    (low.height, low.height_speed) = (6.5, -0.1);
    (low.alpha, low.max_alpha, low.alpha_speed) = (5.0, 245.0, 10.0);
    low.fade_out = low.duration - low.duration / 5;
    let mut tall = band(55);
    (tall.arc, tall.long_speed) = (45.0, 10.0);
    (tall.outer, tall.inner) = (4.5, 4.5);
    tall.height_speed = 2.5;
    tall.height_accel = -(tall.height_speed / tall.duration as f32) * 2.0;
    tall.alpha = 160.0;
    tall.fade_out = tall.duration - tall.duration / 3;
    [low, tall]
}

/// `EF_TELEPORTATION` in `CRagEffect::OnProcess`: a band 4 from the middle, for 60 ticks, that shoots up by 5
/// a tick and a quarter more each tick, turning 2 degrees a tick; it fades from its first tick, and its top
/// closes as it does (`pattern & 1`).
fn teleportation() -> Prim {
    let mut band = band(60);
    band.first_pattern = true;
    (band.outer, band.inner) = (4.0, 4.0);
    (band.height_speed, band.height_accel) = (5.0, 0.25);
    band.long_speed = 2.0;
    band.fade_out = 0;
    band
}

/// `CRagEffect::Portal`: at its first tick two bands for as long as the effect lives, 3.5 and 4 from the
/// middle and 40 and 50 tall, turning 4 and 5 degrees a tick, that come up to 128 over six ticks and fade over
/// the last six. And every 14th tick a ring lying 1 over the ground for 30 ticks, 8 thick once it is wider
/// than that, that widens by 1 a tick, slowing by half, and fades over its second half.
fn portal(tick: i32, lasts: i32, prims: &mut Vec<Prim>) {
    if tick == 0 {
        prims.extend([(3.5, 4.0, 40.0), (4.0, 5.0, 50.0)].map(|(out, turns, tall)| {
            let mut band = band(lasts);
            (band.outer, band.inner, band.long_speed, band.height) = (out, out, turns, tall);
            (band.alpha, band.max_alpha) = (0.0, 128.0);
            band.alpha_speed = band.max_alpha / 6.0;
            band.fade_out = band.duration - 6;
            band
        }));
    }
    if tick % 14 == 0 {
        let mut ring = Prim::new(Shape::Circle3d, "ring_blue.tga");
        ring.duration = 30;
        (ring.put.1, ring.inner, ring.latitude) = (-1.0, 8.0, 90.0);
        ring.radius_speed = 1.0;
        ring.radius_accel = -(ring.radius_speed / ring.duration as f32) * 0.5;
        ring.fade_out = ring.duration - ring.duration / 2;
        prims.push(ring);
    }
}

/// `CRagEffect::TeleportRings` with `Magic_Violet.tga` and its level 0: four walls of the kind 57 all the way
/// round, 1.5, 3, 4 and 5 from the middle, 100, 70, 40 and 15 tall, the wider the fainter (100, 80, 60 and
/// 40) and the more they lean out, begun a quarter round from each other. Blue (a `size` of 4), their light
/// added.
fn teleport_rings(lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Walls(Walls::Rise), "magic_violet.tga");
    (walls.duration, walls.colour, walls.adds) = (lasts, BLUE, true);
    walls.slots = [(180, 1.5, 90, 100.0, 100), (270, 3.0, 89, 70.0, 80), (0, 4.0, 88, 40.0, 60), (90, 5.0, 87, 15.0, 40)]
        .into_iter()
        .map(|(turned, out, rise, tall, alpha)| Slot { alpha, ..Slot::wall(turned, out, rise, tall, 0) })
        .collect();
    walls
}

/// `CRagEffect::TeleportSpiral` with `ring_purple.tga`: four walls of the kind 60 all the way round, 1.8, 2,
/// 2.2 and 2.4 from the middle and 180, 70, 45 and 20 tall, the wider the more they lean out, begun a quarter
/// round from each other; they come up over 15 ticks and go from their 60th tick (the first from its 58th).
/// Blue (a `size` of 4), their light added.
fn teleport_spiral(lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Casting3d, "ring_purple.tga");
    (walls.duration, walls.colour, walls.adds) = (lasts, BLUE, true);
    walls.slots = [(270, 1.8, 90, 180.0, 58), (180, 2.0, 88, 70.0, 60), (90, 2.2, 86, 45.0, 60), (0, 2.4, 84, 20.0, 60)]
        .into_iter()
        .map(|(turned, out, rise, tall, holds)| Slot::wall(turned, out, rise, tall, holds))
        .collect();
    walls
}
