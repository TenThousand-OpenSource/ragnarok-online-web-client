//! The client's effects that are made of primitives, as far as they are ported: each a function of
//! `CRagEffect` in the client, run every tick of the effect's life, here with the same figures.
//!
//! - `EF_HIT1` (0): the ring of a blow that lands (`Hit1`; its four small sparks, sprites that fly off, are not
//!   ported yet);
//! - `EF_HIT2` (1): the flare where a skill lands (`Hit2`);
//! - `EF_ENDURE` (11): Endure (`Endure`);
//! - `EF_BASH` (16): the flash a few skills are launched with (`Bash`).

use super::*;

/// The pictures the effects here are drawn with, among the client's pictures of effects.
pub(in crate::web::world) const PICTURES: [&str; 6] = ["ring_blue.tga", "lens1.tga", "lens2.tga", "alpha_down.tga", "alpha_center.tga", "endure.tga"];

/// How many ticks an effect lives (the client's `EffectLifetime`) and the sound it starts with (none: empty);
/// nothing for an effect not ported yet.
pub(super) fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        0 => (50, ""),
        1 => (50, "effect/ef_hit2.wav"),
        11 => (80, "effect/ef_endure.wav"),
        16 => (40, "effect/ef_bash.wav"),
        _ => return None,
    })
}

/// One tick of an effect: `tick` of the `lasts` it lives, on someone who faced `facing` (the client's degrees) as
/// it began. It launches what it launches at that tick into `prims`.
pub(super) fn run(effect: u32, tick: i32, lasts: i32, facing: f32, prims: &mut Vec<Prim>) {
    match effect {
        0 if tick == 0 => prims.push(hit1(facing)),
        1 if tick == 0 => prims.extend(hit2()),
        11 => endure(tick, lasts, prims),
        16 if tick == 0 => prims.extend(bash()),
        _ => {}
    }
}

/// `CRagEffect::Hit1`: a blue ring 10 above the feet of the one struck, upright and across the way that one
/// faces, that goes that way as it slows and fades over its second half.
fn hit1(facing: f32) -> Prim {
    let mut ring = Prim::new(Shape::Cylinder3d, "ring_blue.tga");
    ring.duration = 10;
    (ring.latitude, ring.longitude) = (-90.0, 180.0 - facing);
    ring.put.1 = -10.0;
    ring.speed = 0.7;
    ring.accel = -(ring.speed / ring.duration as f32) * 0.5;
    (ring.outer, ring.inner, ring.height) = (10.0, 5.0, 3.5);
    ring.fade_out = ring.duration - ring.duration / 2;
    ring
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

/// `CRagEffect::Endure`: 20 above the one who endures, a wide mark that comes up as it closes in to half its
/// size and fades, and for the first fifty ticks a streak a tick, each from a ring around it running inwards.
fn endure(tick: i32, lasts: i32, prims: &mut Vec<Prim>) {
    if tick == 0 {
        let mut mark = Prim::new(Shape::Texture2d, "endure.tga");
        mark.duration = lasts;
        mark.put.1 = -20.0;
        (mark.width, mark.height) = (150.0, 150.0);
        (mark.width_speed, mark.height_speed) = (-5.0, -5.0);
        let settles = mark.duration - 35;
        mark.settles_at = Some(settles);
        (mark.width_accel, mark.height_accel) = (-mark.width_speed / settles as f32, -mark.height_speed / settles as f32);
        mark.alpha = 0.0;
        mark.alpha_speed = mark.max_alpha / 20.0;
        mark.fade_out = mark.duration - mark.duration / 3;
        prims.push(mark);
    }
    if tick >= 50 {
        return;
    }
    let mut streak = Prim::new(Shape::Texture2d, "alpha_down.tga");
    streak.duration = 40;
    let heading = lot(360);
    (streak.roll, streak.longitude) = (heading, heading);
    streak.speed = -4.0;
    streak.accel = -(streak.speed / streak.duration as f32);
    streak.width = (lot(100) + 60.0) / 100.0;
    streak.height = lot(20) + 30.0;
    streak.alpha = 0.0;
    streak.alpha_speed = streak.max_alpha / 20.0;
    streak.fade_out = streak.duration - streak.duration / 3;
    let radius = lot(40) + 100.0;
    streak.put = (sine(heading) * radius, -cosine(heading) * radius - 20.0);
    prims.push(streak);
}

/// `CRagEffect::Bash`: 40 above the one it is on, a disc of light a hundred across its radius that comes up and
/// fades, and twenty thin wedges turning about its middle, growing as they slow.
fn bash() -> Vec<Prim> {
    let mut disc = Prim::new(Shape::Circle2d, "alpha_down.tga");
    disc.duration = 40;
    disc.first_pattern = true;
    disc.put.1 = -40.0;
    disc.radius = 100.0;
    disc.alpha = 0.0;
    disc.max_alpha = 170.0;
    disc.alpha_speed = disc.max_alpha / 6.0;
    disc.fade_out = disc.duration - 10;
    let mut prims = vec![disc];
    prims.extend((0..20).map(|_| {
        let mut wedge = Prim::new(Shape::Flash2d, "alpha_center.tga");
        wedge.duration = 40;
        wedge.put.1 = -40.0;
        wedge.longitude = lot(360);
        wedge.long_speed = (lot(60) + 10.0) / 10.0;
        wedge.long_accel = -(wedge.long_speed / wedge.duration as f32) / 1.5;
        wedge.arc = (lot(25) + 5.0) / 10.0;
        wedge.height = lot(40) + 20.0;
        wedge.height_speed = (lot(30) + 20.0) / 10.0;
        wedge.alpha = 0.0;
        wedge.max_alpha = 200.0;
        wedge.alpha_speed = wedge.max_alpha / 10.0;
        wedge.fade_out = wedge.duration - wedge.duration / 3;
        wedge
    }));
    prims
}
