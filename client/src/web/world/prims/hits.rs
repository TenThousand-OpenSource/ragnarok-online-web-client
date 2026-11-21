//! What a blow shows where it lands, each with the figures of its function of `CRagEffect`:
//!
//! - `EF_HIT1` (0): the ring of a blow that lands and its four sparks (`Hit1`);
//! - `EF_HIT2` (1): the flare where a skill lands (`Hit2`);
//! - `EF_HIT3` (2), `EF_HIT4` (3): flares where a blow lands (`Hit3`, `Hit4`);
//! - `EF_HIT5` (4), `EF_HIT6` (5): a cross of two flares that turns (`Hit5`, `Hit6`);
//! - `EF_NAPALMBEAT` (32): the bursts round one struck by Napalm Beat (`NapalmFragments`);
//! - `EF_FIRESPLASHHIT` (50): a ring of fire that opens turning (in `CRagEffect::OnProcess` itself);
//! - `EF_COLDHIT` (51): shards of ice and a puff of smoke (`ColdHit`);
//! - `EF_PIERCE` (81): the ring of Pierce and Spear Stab (`RingBurstSparks`);
//! - `EF_SONICBLOWHIT` (122): the red ring of a hit of Sonic Blow (`SonicBlowHit`);
//! - `EF_HITDARK` (180): the dark hit (`HitDark`).
//!
//! Two the client gives no time to live (`EffectLifetime` of 0) and so shows nothing of: `EF_DARKATTACK`
//! (184), which would be the dark hit, and `EF_TELEKHIT` (198), whose arm of `OnProcess` is empty.
//!
//! The client runs the hits of fire, cold and wind on every pass of its loop, not only on its ticks
//! (`CRagEffect::OnProcess` lets them past its wait for a tick), so the tick at which `ColdHit` launches its
//! second puff comes sooner there on a fast machine; here they run a tick at a time like everything else.
//!
//! The hits of fire and of wind (`EF_FIREHIT`, 49; `EF_WINDHIT`, 52) are files picked by lot, and so not here
//! but with the effects that are played from files (`crate::effect::played`).
//!
//! What follows a blow on the one struck for longer is in `struck.rs`, which takes the bands of this file.

use super::launched::{ahead, band, way, PARTICLE1};
use super::*;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES, &BURST], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 8] = [
    "firering.tga",
    "ring_blue.tga",
    "lens1.tga",
    "lens2.tga",
    "ring_yellow.tga",
    "magic_red.tga",
    "ice.tga",
    "smoke.tga",
];

/// The pictures a burst of `NapalmFragments` runs through (`effect\폭발1.TGA` to `폭발8.TGA`).
const BURST: [&str; 8] = ["폭발1.tga", "폭발2.tga", "폭발3.tga", "폭발4.tga", "폭발5.tga", "폭발6.tga", "폭발7.tga", "폭발8.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        0 => (50, ""),
        1 => (50, "effect/ef_hit2.wav"),
        2 => (50, "effect/ef_hit3.wav"),
        3 => (50, "effect/ef_hit4.wav"),
        4 => (50, "effect/ef_hit5.wav"),
        5 => (50, "effect/ef_hit6.wav"),
        32 => (100, "effect/ef_napalmbeat.wav"),
        50 | 51 => (50, ""),
        81 => (250, ""),
        122 => (250, "effect/assasin_sonicblow.wav"),
        180 => (50, ""),
        // No time to live: nothing of these two is seen.
        184 | 198 => (0, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, facing) = (course.tick, course.facing);
    match effect {
        0 if tick == 0 => hit1(facing, prims),
        1 if tick == 0 => prims.extend(hit2()),
        2 if tick == 0 => hit3(way(facing), prims),
        3 if tick == 0 => hit4(way(facing), prims),
        4 if tick == 0 => prims.extend(cross((20.0, 5), (30.0, 10), 2.5)),
        5 if tick == 0 => prims.extend(cross((10.0, 5), (15.0, 5), 1.7)),
        32 if tick < 15 => prims.push(napalm_fragment()),
        50 if tick == 0 => prims.push(fire_ring()),
        51 => cold_hit(tick, prims),
        81 if tick == 0 => prims.push(pierce(way(facing))),
        122 if tick == 0 => prims.push(sonic_blow_hit(way(facing))),
        180 if tick == 0 => hit_dark(facing, prims),
        _ => {}
    }
}

/// `CRagEffect::Hit1`: a blue ring 10 above the feet of the one struck, upright and across the way that one
/// faces, that goes that way as it slows and fades over its second half; and its four sparks.
fn hit1(facing: f32, prims: &mut Vec<Prim>) {
    let mut ring = band("ring_blue.tga", 10, 180.0 - facing);
    (ring.outer, ring.inner, ring.height) = (10.0, 5.0, 3.5);
    ring.fade_out = ring.duration - ring.duration / 2;
    prims.push(ring);
    hit1_sparks(way(facing), prims);
}

/// The sparks of `CRagEffect::Hit1`, from 10 above the one struck, the ring going `way`: two that fly off
/// within 40 degrees of that way, and two thrown the other way that fall. Each is drawn three times, a trail,
/// and fades from its first tick to its last.
fn hit1_sparks(way: f32, prims: &mut Vec<Prim>) {
    for falls in [false, true] {
        for _ in 0..2 {
            let kind = if falls { Shape::ParticleGravity3d } else { Shape::Particle3d };
            let longitude = lot(80) + way + if falls { 140.0 } else { -40.0 };
            let mut spark = launched::thrown(kind, PARTICLE1, longitude, lot(100) - 50.0, lot(4) + 4.0);
            spark.speed = (lot(90) + 60.0) / 100.0;
            spark.duration = lot(24) as i32 + 6;
            spark.accel = -(spark.speed / spark.duration as f32) * 0.5;
            if falls {
                // One that falls goes level: its turn is made anew without the tip (`MakeYRotation`).
                spark.latitude = 0.0;
                spark.grav_speed = -(lot(90) + 30.0) / 100.0;
                spark.grav_accel = -(spark.grav_speed / spark.duration as f32 * 2.0);
            }
            spark.size = (lot(100) + 60.0) / 100.0;
            spark.fade_out = 0;
            spark.segments = 3;
            prims.push(spark);
        }
    }
}

/// `CRagEffect::HitDark`: the blue ring of a plain blow with its four sparks (`Hit1` has the same figures),
/// the sparks nearly black.
fn hit_dark(facing: f32, prims: &mut Vec<Prim>) {
    let before = prims.len();
    hit1(facing, prims);
    for spark in prims[before..].iter_mut().filter(|prim| prim.shows_sprite()) {
        spark.colour = [20.0; 3];
    }
}

/// `CRagEffect::Hit2`: eight flares around a place 20 above the one struck, each a narrowing ray that runs
/// outwards and slows, lengthening as it goes.
fn hit2() -> Vec<Prim> {
    (0..8)
        .map(|index| {
            let degree = index as f32 * 45.0;
            let mut flare = Prim::new(Shape::Texture2d, if index % 2 == 0 { "lens1.tga" } else { "lens2.tga" });
            flare.first_pattern = true;
            flare.duration = lot(20) as i32 + 10;
            let heading = degree - 15.0 + lot(30);
            (flare.roll, flare.longitude) = (heading, heading);
            flare.speed = (lot(45) + 5.0) / 10.0;
            // Half of what would stop it by its last tick (the client's `g_depthAverageHalf`, 0.5).
            flare.accel = -(flare.speed / flare.duration as f32) * 0.5;
            flare.width = lot(15) + 5.0;
            flare.height = lot(20) + 20.0;
            flare.width_speed = -(flare.width / flare.duration as f32);
            (flare.height_speed, flare.height_accel) = (1.5, 0.25);
            let spread = lot(5);
            flare.put = (sine(degree) * spread, -cosine(degree) * spread - 20.0);
            flare.alpha = 0.0;
            flare.alpha_speed = flare.max_alpha * 0.125;
            flare
        })
        .collect()
}

/// `CRagEffect::Hit5` and `Hit6`: two flares of `lens2.tga` crossed at a place 20 above the one struck, the
/// first turned by lot and the second a quarter on, each reaching its height both ways. They turn back at 5
/// degrees a tick, slowing, narrow to nothing over their 17 ticks and lengthen faster and faster (by `grows`
/// and half more a tick), and fade from their ninth tick. `wide` and `long` are each the least a flare is at
/// first and under how much more by lot: `Hit5` has 20 to 24 by 30 to 39, `Hit6` 10 to 14 by 15 to 19.
fn cross(wide: (f32, u32), long: (f32, u32), grows: f32) -> [Prim; 2] {
    let turned = lot(360);
    [turned, turned + 90.0].map(|roll| {
        let mut flare = Prim::new(Shape::Texture2d, "lens2.tga");
        flare.duration = 17;
        flare.put.1 = -20.0;
        (flare.roll, flare.roll_speed) = (roll, -5.0);
        flare.roll_accel = -(flare.roll_speed / flare.duration as f32) / 1.5;
        (flare.width, flare.height) = (lot(wide.1) + wide.0, lot(long.1) + long.0);
        flare.width_speed = -(flare.width / flare.duration as f32);
        (flare.height_speed, flare.height_accel) = (grows, 0.5);
        flare.fade_out = flare.duration - flare.duration / 2;
        flare
    })
}

/// `CRagEffect::NapalmFragments`, at each of its first fifteen ticks: a burst somewhere 10 to 59 from a place
/// 20 above the one struck, any way across the screen, 20 to 59 to each side, that comes up over ten ticks
/// and shows a new picture of its eight every other tick, ending after the last (so with its 15th tick, before
/// the 20th at which it would begin to fade).
///
/// (The decompiled function leaves out a tick when `stateCnt % 5 > stateCnt`, which is never so: it reads as
/// one burst a tick here. If the client's own test was another, every fifth tick, there are three.)
fn napalm_fragment() -> Prim {
    let mut burst = Prim::new(Shape::Texture2d, BURST[0]);
    (burst.pictures, burst.anim_speed, burst.repeats) = (&BURST, 2, false);
    burst.duration = 30;
    let (far, turned) = (lot(50) + 10.0, lot(360));
    burst.put = (sine(turned) * far, -cosine(turned) * far - 20.0);
    let size = lot(40) + 20.0;
    (burst.width, burst.height) = (size, size);
    burst.alpha = 0.0;
    burst.alpha_speed = burst.max_alpha / 10.0;
    burst.fade_out = burst.duration - burst.duration / 3;
    burst
}

/// `EF_FIRESPLASHHIT` in `CRagEffect::OnProcess`: a ring of fire 20 above the one struck, 10 to each side,
/// turned any way, that widens by 5.7 a tick and turns back at 18 degrees a tick, slowing, for 30 ticks, and
/// fades over the second half of them.
fn fire_ring() -> Prim {
    let mut ring = Prim::new(Shape::Texture2d, "firering.tga");
    ring.duration = 30;
    ring.put.1 = -20.0;
    (ring.width, ring.height, ring.width_speed, ring.height_speed) = (10.0, 10.0, 5.7, 5.7);
    (ring.roll, ring.roll_speed) = (lot(360), -18.0);
    ring.roll_accel = -(ring.roll_speed / ring.duration as f32) / 2.5;
    ring.fade_out = ring.duration - ring.duration / 2;
    ring
}

/// Sparks as `Hit3` and `Hit4` throw them: each from 10 above the one struck, within 40 degrees of the way the
/// band goes, as fast as `quick` hundredths and up to `spread` more, slowing; it shrinks to nothing, is drawn
/// three times, a trail, and fades from its first tick to its last.
fn sparks(many: usize, (spread, quick): (u32, f32), way: f32, prims: &mut Vec<Prim>) {
    for _ in 0..many {
        let mut spark = launched::thrown(Shape::Particle3d, PARTICLE1, lot(80) + way - 40.0, lot(100) - 50.0, lot(4) + 4.0);
        spark.speed = (lot(spread) + quick) / 100.0;
        spark.duration = lot(24) as i32 + 6;
        spark.accel = -(spark.speed / spark.duration as f32) * 0.5;
        spark.size = (lot(100) + 60.0) / 100.0;
        spark.size_speed = -(spark.size / spark.duration as f32);
        (spark.fade_out, spark.segments) = (0, 3);
        prims.push(spark);
    }
}

/// A band of `lens2.tga` as `Hit3` and `Hit4` launch it: its near ring `inner` wide and its far one `outer`,
/// with no height at first, lengthening by `grows` and faster by `faster` a tick; it fades over its last third.
fn flare(way: f32, (outer, inner): (f32, f32), (grows, faster): (f32, f32)) -> Prim {
    let mut flare = band("lens2.tga", 15, way);
    (flare.outer, flare.inner) = (outer, inner);
    (flare.height_speed, flare.height_accel) = (grows, faster);
    flare.fade_out = flare.duration - flare.duration / 3;
    flare
}

/// `CRagEffect::Hit3`: two bands of light, a thin one and one that opens out, and eight sparks.
fn hit3(way: f32, prims: &mut Vec<Prim>) {
    prims.extend([flare(way, (1.5, 1.5), (0.5, 0.2)), flare(way, (4.0, 1.5), (0.25, 0.2))]);
    sparks(8, (120, 80.0), way, prims);
}

/// `CRagEffect::Hit4`: one band of light that opens out from next to nothing, and five sparks.
fn hit4(way: f32, prims: &mut Vec<Prim>) {
    prims.push(flare(way, (4.0, 0.5), (0.25, 0.15)));
    sparks(5, (90, 60.0), way, prims);
}

/// `CRagEffect::RingBurstSparks` as `EF_PIERCE` runs it: a yellow ring like that of a plain blow, a little
/// wider, fading over its second half. (The function launches one for each level and sparks with all but the
/// first; the effect's level is 1, which nothing changes.)
fn pierce(way: f32) -> Prim {
    let mut ring = band("ring_yellow.tga", 15, way);
    (ring.outer, ring.inner, ring.height) = (11.0, 6.0, 2.5);
    ring.fade_out = ring.duration - ring.duration / 2;
    ring
}

/// `CRagEffect::SonicBlowHit`: a red ring across the way the one struck faces, begun 2 along that way from a
/// place up to 2 to the east and somewhere 1 to 11 above the feet, that goes on slowly and fades over its
/// second half.
fn sonic_blow_hit(way: f32) -> Prim {
    let mut ring = band("magic_red.tga", 12, way);
    ring.moved = ahead(way, 0.0);
    // The client divides its lots as whole numbers.
    ring.put = (((lot(10) + 1.0) / 4.0).floor(), ((lot(20) + 1.0) / 2.0).floor() - 11.0);
    ring.speed = 0.4;
    ring.accel = -(ring.speed / ring.duration as f32) * 0.5;
    (ring.outer, ring.inner, ring.height) = (7.0, 4.0, 3.5);
    ring.fade_out = ring.duration - ring.duration / 2;
    ring
}

/// `CRagEffect::ColdHit`. At its first tick nine shards of ice, kites that fly outwards from around a place 20
/// above the one struck, one every 40 degrees and each within 15 before and 45 after its own way, slowing;
/// every third is large (100 to 109 long, 10 wide, fast) and the others small (45 to 49 long, 4 wide). They
/// live 15 ticks and fade over the last 10. At its first and its eighth tick a puff of smoke, which rises
/// within 20 degrees of straight up, swelling fast for 10 ticks and slowly after, and fades over its last
/// third.
fn cold_hit(tick: i32, prims: &mut Vec<Prim>) {
    if tick == 0 {
        for index in 0..9 {
            let degree = index as f32 * 40.0;
            let mut shard = Prim::new(Shape::Quad2d, "ice.tga");
            shard.duration = 15;
            let heading = degree - 15.0 + lot(60);
            (shard.roll, shard.longitude) = (heading, heading);
            // The client divides its lot by ten as a whole number before adding it.
            let quickness = (lot(45) / 10.0).floor();
            (shard.height, shard.width, shard.speed) =
                if index % 3 == 0 { (lot(10) + 100.0, 10.0, quickness + 12.8) } else { (lot(5) + 45.0, 4.0, quickness + 4.8) };
            shard.accel = -(shard.speed / shard.duration as f32) / 1.5;
            let spread = lot(5);
            shard.put = (sine(degree) * spread, -cosine(degree) * spread - 20.0);
            shard.alpha = 200.0;
            shard.fade_out = shard.duration - 10;
            prims.push(shard);
        }
    }
    if tick != 0 && tick != 7 {
        return;
    }
    let mut smoke = Prim::new(Shape::Texture2d, "smoke.tga");
    smoke.duration = 25;
    smoke.speed = 0.85;
    smoke.longitude = lot(40) - 20.0;
    (smoke.width, smoke.height) = (1.0, 1.0);
    (smoke.width_speed, smoke.height_speed) = (6.0, 6.0);
    smoke.turns_at = Some((10, 0.5, 0.5));
    smoke.fade_out = smoke.duration - smoke.duration / 3;
    prims.push(smoke);
}
