//! Effects that burst out from the one they are on across the screen, each with the figures
//! of its function of `CRagEffect`:
//!
//! - `EF_ENDURE` (11): Endure (`Endure`);
//! - `EF_BASH` (16): the flash a few skills are launched with (`Bash`);
//! - `EF_TURNUNDEAD` (82): Turn Undead (`TurnUndead`);
//! - `EF_HASTEUP` (98): Adrenaline Rush (`HasteUp`);
//! - `EF_SONICBLOW` (121): the ring Sonic Blow is launched with (in the client's `CRagEffect::OnProcess` itself);
//! - `EF_OVERTHRUST` (128): Overthrust (`StoneCurseRing`);
//! - `EF_SUMMONSLAVE` (215): the wedges of light of a monster calling its slaves (`AlphaCenterRise`);
//! - `EF_CHEMICALPROTECTION` (300): the rays of light of Chemical Protection and Full Protection
//!   (`ShockwaveRing`, twice).
//!
//! Not done here, as it is not drawn by primitives: the smoke `AlphaCenterRise` has its parent make at its
//! 35th tick (`AM_MAKE_SMOKE_FFECT`). What bursts out over the ground is in `blasts.rs`.

use super::launched::PARTICLE1;
use super::*;
use crate::prim::{Cast, Change};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 5] = ["endure.tga", "alpha_down.tga", "ring_liner.tga", "alpha_center.tga", "shockwave_c.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`, or what its function makes of
/// it) and the sound it starts with (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        11 => (80, "effect/ef_endure.wav"),
        16 => (40, "effect/ef_bash.wav"),
        // `TurnUndead` starts its sound at its 37th tick.
        82 => (250, ""),
        // `HasteUp` gives itself 300 ticks.
        98 => (300, "effect/black_adrenalinerush_a.wav"),
        121 => (40, "effect/ef_stonecurse.wav"),
        // `StoneCurseRing` gives itself 9999 ticks and shows nothing after the 40 its ring lives; it is ended
        // here with the ring. It starts a second sound with this one.
        128 => (40, "effect/black_overthrust.wav"),
        215 => (500, ""),
        300 => (300, "apocalips_attack.wav"),
        _ => return None,
    })
}

/// Whether an effect is one the client has the striker launch and then tells the place of the one struck
/// (`CSession::PlaySkillEffect`, `Contents/Jobs/JobInheritance.cpp`).
fn thrown(effect: u32) -> bool {
    matches!(effect, 82 | 121)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        11 => endure(tick, lasts, prims),
        16 if tick == 0 => prims.extend(bash()),
        82 => turn_undead(course, prims),
        98 => haste_up(course, prims),
        121 if tick == 0 => prims.push(widening_ring(lasts)),
        128 if tick == 0 => {
            course.sound("effect/ef_stonecurse.wav");
            prims.push(widening_ring(40));
        }
        215 if tick == 0 => prims.extend((0..20).map(|_| rising_wedge())),
        300 if tick == 0 => prims.extend([30, 60].into_iter().flat_map(|turned| [false, true].map(|wide| rays(turned, wide, course)))),
        _ => {}
    }
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
/// fades, and twenty thin wedges turning about its middle, growing as they slow (those `AlphaCenterRise`
/// launches too: `rising_wedge`).
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
    prims.extend((0..20).map(|_| rising_wedge()));
    prims
}

/// `CRagEffect::TurnUndead`. At its first tick, 40 above the caster: a disc of light that closes in from 370
/// to nothing in 20 ticks, and five wedges 350 to 380 long that shorten fast as they turn. At the 37th tick
/// the sound of Bash. At the 50th a ring lying 7 above the ground that widens faster and faster for 15 ticks
/// and slowly after, and fades over its last 10.
fn turn_undead(course: &mut Course, prims: &mut Vec<Prim>) {
    match course.tick {
        0 => {
            let mut disc = Prim::new(Shape::Circle2d, "alpha_down.tga");
            (disc.duration, disc.first_pattern, disc.put.1) = (20, true, -40.0);
            disc.radius = 370.0;
            disc.radius_speed = -(disc.radius / 20.0);
            (disc.alpha, disc.max_alpha) = (0.0, 130.0);
            disc.alpha_speed = disc.max_alpha / 6.0;
            disc.fade_out = disc.duration - 10;
            prims.push(disc);
            for _ in 0..5 {
                let mut wedge = turning_wedge(20);
                wedge.height = lot(30) + 350.0;
                wedge.height_speed = -(lot(30) + 150.0) / 10.0;
                (wedge.alpha, wedge.max_alpha) = (0.0, 150.0);
                wedge.alpha_speed = wedge.max_alpha / 10.0;
                wedge.fade_out = wedge.duration - wedge.duration / 3;
                prims.push(wedge);
            }
        }
        37 => course.sound("effect/ef_bash.wav"),
        50 => {
            let mut ring = Prim::new(Shape::Texture3d, "ring_liner.tga");
            (ring.duration, ring.roll, ring.put.1) = (30, 90.0, -7.0);
            (ring.width_speed, ring.height_speed) = (1.5, 1.5);
            (ring.width_accel, ring.height_accel) = (ring.width_speed / 10.0, ring.height_speed / 10.0);
            ring.alpha = 150.0;
            ring.fade_out = ring.duration - 10;
            ring.changes = Some(Change { width_speed: 0.5, width_accel: -0.5 / 15.0, height_speed: 0.5, height_accel: -0.5 / 15.0, ..Change::at(15) });
            prims.push(ring);
        }
        _ => {}
    }
}

/// A wedge 40 above the one the effect is on, pointing any way, 0.5 to 3 degrees to each side, that turns by 1
/// to 7 degrees a tick, slowing: what `TurnUndead`, `HasteUp`, `Bash` and `AlphaCenterRise` launch theirs as.
fn turning_wedge(duration: i32) -> Prim {
    let mut wedge = Prim::new(Shape::Flash2d, "alpha_center.tga");
    (wedge.duration, wedge.put.1) = (duration, -40.0);
    wedge.longitude = lot(360);
    wedge.long_speed = (lot(60) + 10.0) / 10.0;
    wedge.long_accel = -(wedge.long_speed / wedge.duration as f32) / 1.5;
    wedge.arc = (lot(25) + 5.0) / 10.0;
    wedge
}

/// A wedge of `CRagEffect::AlphaCenterRise`, which launches twenty at its first tick, 40 above the one it is
/// on: 20 to 59 long, it lengthens by 2 to 5 a tick as it turns, comes up over 10 ticks and fades over the last
/// third of its 40. (They are the wedges of `Bash`, without its disc.)
fn rising_wedge() -> Prim {
    let mut wedge = turning_wedge(40);
    wedge.height = lot(40) + 20.0;
    wedge.height_speed = (lot(30) + 20.0) / 10.0;
    (wedge.alpha, wedge.max_alpha) = (0.0, 200.0);
    wedge.alpha_speed = wedge.max_alpha / 10.0;
    wedge.fade_out = wedge.duration - wedge.duration / 3;
    wedge
}

/// `CRagEffect::HasteUp`. At its first tick twenty wedges 40 above the one it is on, that lengthen by about 4
/// to 5 a tick as they turn, and from their 40th tick stay as long and turn at 1.5 to 2.4 degrees a tick, to
/// their 80th; and four sparks 7 from that one, a quarter apart, that go round faster and faster as they rise,
/// and fade over the last tenth of their 100 ticks. At the 30th tick the second sound.
///
/// (The sparks are in the world round the place where the effect began; the wedges follow the one it is on.
/// Here all of it follows that one.)
fn haste_up(course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 30 {
        course.sound("effect/black_adrenalinerush_b.wav");
    }
    if course.tick != 0 {
        return;
    }
    for _ in 0..20 {
        let mut wedge = turning_wedge(80);
        wedge.height_speed = (lot(20) + 50.0) / 13.0;
        wedge.changes = Some(Change { speed: (lot(10) + 15.0) / 10.0, accel: 0.0, height_speed: 0.0, ..Change::at(40) });
        prims.push(wedge);
    }
    for index in 0..4 {
        let mut spark = Prim::new(Shape::ParticleOrbit3d, PARTICLE1);
        spark.duration = 100;
        spark.radius = 7.0;
        (spark.grav_speed, spark.grav_accel) = (-0.1, -0.002);
        (spark.longitude, spark.long_speed, spark.long_accel) = (index as f32 * 90.0, 0.3, 0.2);
        spark.size = 1.2;
        spark.fade_out = spark.duration - spark.duration / 10;
        prims.push(spark);
    }
}

/// The ring of `EF_SONICBLOW` and of `CRagEffect::StoneCurseRing`: on the screen, 20 above the one it is on,
/// in pieces of 10 degrees, it widens by 7 a tick, slowing, 10 thick once it is wider than that, and fades from
/// its first tick to its last.
fn widening_ring(duration: i32) -> Prim {
    let mut ring = Prim::new(Shape::Circle2d, "alpha_center.tga");
    (ring.duration, ring.put.1) = (duration, -20.0);
    (ring.arc, ring.inner) = (10.0, 10.0);
    ring.radius_speed = 7.0;
    ring.radius_accel = -(ring.radius_speed / (ring.duration + 40) as f32) * 2.0;
    ring.fade_out = 0;
    ring
}

/// `CRagEffect::ShockwaveRing`, one of the two drawings of its primitive of the kind 54 (`Cast::Rays`): four
/// rays of light a quarter apart, the first turned by what the function is told (30 and 60 for the two calls
/// of `EF_CHEMICALPROTECTION`), each 100 long, in the flat of the screen. They come up over ten ticks, hold for
/// ten and are gone by the 32nd; the wide drawing is a pale blue and the narrow one white, both added to what
/// is behind them. They stay in the world where they were launched.
///
/// (The function lowers its effect's place by 9 after launching; the primitive has taken the place before
/// that, and the next tick puts the effect back at its parent's, so nothing comes of it.)
fn rays(turned: i32, wide: bool, course: &Course) -> Prim {
    let mut rays = Prim::new(Shape::Cast(Cast::Rays { wide }), "shockwave_c.bmp");
    (rays.duration, rays.adds, rays.left) = (course.lasts, true, Some(course.walked));
    if wide {
        rays.colour = [130.0, 130.0, 255.0];
    }
    rays.slots = (0..4).map(|index| Slot::ray(index * 90 + turned)).collect();
    rays
}
