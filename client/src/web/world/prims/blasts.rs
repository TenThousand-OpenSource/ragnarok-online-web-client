//! What bursts out over the ground round the one an effect is on, each with the figures of its function of
//! `CRagEffect`:
//!
//! - `EF_MAGNUMBREAK` (17): Magnum Break (`MagnumBreak`);
//! - `EF_SIGHTRASHER` (62): Sightrasher (`Sightrasher`);
//! - `EF_EARTHSPIKE` (79): Earth Spike (`EarthSpike`); `EF_HYOUSENSOU` (619): the same spikes of ice;
//! - `EF_WATERBALL3` (942): the splash of a ball of water (`WaterBallBurst`).
//!
//! `EarthSpike` shakes the screen as it begins (`MM_QUAKE` with 1) and `Sightrasher` ends a Sight running on
//! the one who cast it (`AM_DELETE_EFFECT`). What bursts out across the screen is in `bursts.rs`.

use super::launched::{SHADOW, SIGHT};
use super::*;
use crate::prim::Change;

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 5] = ["ring_yellow.tga", "대폭발.tga", "ring_blue.tga", "ice.tga", "stone.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        17 => (30, "effect/ef_magnumbreak.wav"),
        62 => (250, "effect/wizard_sightrasher.wav"),
        79 => (250, "effect/wizard_earthspike.wav"),
        // `EarthSpike` with its type of 2 starts one of three sounds by lot (`run`).
        619 => (250, ""),
        942 => (100, "effect/wizard_waterball_chulung.wav"),
        _ => return None,
    })
}

/// Whether an effect stays where it began: one whose function does not take its parent's place every tick.
fn stays(effect: u32) -> bool {
    matches!(effect, 62 | 79 | 619)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        17 if tick == 0 => prims.extend(blast(("ring_yellow.tga", "대폭발.tga"), [lasts, lasts, 30], |lives| lives - 15)),
        62 => {
            if tick == 0 {
                course.ends.push(22);
            }
            sightrasher(tick, prims);
        }
        79 if tick == 0 => {
            course.quake = Some(1);
            earth_spike("stone.bmp", prims);
        }
        619 if tick == 0 => {
            course.sound(&format!("effect/ef_icearrow{}.wav", lot(3) as u32 + 1));
            course.quake = Some(1);
            earth_spike("ice.tga", prims);
        }
        942 if tick == 0 => prims.extend(blast(("ring_blue.tga", "ice.tga"), [30, 30, 20], |_| 10)),
        _ => {}
    }
}

/// What `CRagEffect::MagnumBreak` and `CRagEffect::WaterBallBurst` launch at their first tick, at the feet of
/// the one the effect is on: two rings one on the other, lying on the ground and widening as they slow, 12
/// thick once they are wider than that, and between them a ball that swells as it slows, turning 3 degrees a
/// tick. `lives` is how many ticks the first ring, the ball and the second ring live; each comes up over 15
/// ticks and begins to fade at the tick `fades` makes of the ticks it lives.
///
/// `MagnumBreak`: yellow rings and a ball of fire, that fade over their last 15 ticks; the second ring is told
/// 30 ticks, which is what the effect lives. `WaterBallBurst`: blue rings that live 30 and 20 ticks and a ball
/// of ice that lives 30, all fading from their tenth tick. (The client draws the rings over everything,
/// whatever stands before them; here they are drawn as the other effects are.)
fn blast((rings, ball): (&'static str, &'static str), lives: [i32; 3], fades: fn(i32) -> i32) -> [Prim; 3] {
    let ring = |duration: i32| {
        let mut ring = Prim::new(Shape::Circle3d, rings);
        ring.duration = duration;
        ring.latitude = 90.0;
        ring.radius_speed = 1.75;
        ring.radius_accel = -(ring.radius_speed / ring.duration as f32) * 0.5;
        ring.inner = 12.0;
        ring.alpha = 0.0;
        ring.alpha_speed = ring.max_alpha / 15.0;
        ring.fade_out = fades(duration);
        ring
    };
    let mut blast = Prim::new(Shape::Sphere3d, ball);
    blast.duration = lives[1];
    blast.radius_speed = 1.15;
    blast.radius_accel = -(blast.radius_speed / blast.duration as f32) * 0.5;
    blast.long_speed = 3.0;
    blast.alpha = 0.0;
    blast.alpha_speed = blast.max_alpha / 15.0;
    blast.fade_out = fades(blast.duration);
    [ring(lives[0]), blast, ring(lives[2])]
}

/// `CRagEffect::Sightrasher`. At ticks 0, 5, 10 and 15 eight flames, one every 45 degrees round the caster, and
/// under each its shadow: they fly outwards at 1 a tick until the effect's 35th tick, then at half that (and
/// from then on all are drawn aside by the same small pull, as the client's figures have it), and fade from the
/// effect's 65th tick to its 85th. Each later round begins a little further out and lower, smaller and fainter.
/// At the tenth tick a yellow ring widens on the ground, 10 thick outside its radius.
fn sightrasher(tick: i32, prims: &mut Vec<Prim>) {
    if tick % 5 == 0 && tick < 20 {
        let reach = (tick / 5) as f32 + 5.0;
        for shadow in [false, true] {
            for index in 0..8 {
                let (sin, cos) = (sine(index as f32 * -45.0), cosine(index as f32 * -45.0));
                let mut flame = Prim::new(Shape::Particle3d, if shadow { SHADOW } else { SIGHT });
                flame.duration = 85 - tick;
                flame.by_axes = true;
                (flame.put, flame.deep) = ((reach * sin, if shadow { 0.0 } else { (tick / 3 - 20) as f32 }), -reach * cos);
                flame.speed3 = [sin, 0.0, -cos];
                flame.anim_speed = if shadow { 3 } else { lot(2) as i32 + 3 };
                flame.size = if shadow { 1.0 - tick as f32 / 30.0 } else { 2.5 - tick as f32 / 10.0 };
                flame.alpha = ((75 - tick) * 2) as f32;
                flame.changes = Some(Change { speed3: [sin * 0.5, 0.0, cos * -0.5], accel3: [-0.004, 0.0, 0.004], ..Change::at(35 - tick) });
                flame.fade_out = 65 - tick;
                prims.push(flame);
            }
        }
    }
    if tick == 10 {
        let mut ring = Prim::new(Shape::Circle3d, "ring_yellow.tga");
        ring.duration = 30;
        ring.latitude = 90.0;
        ring.radius_speed = 2.5;
        ring.radius_accel = -(ring.radius_speed / ring.duration as f32) / 1.5;
        ring.inner = -10.0;
        ring.alpha = 150.0;
        ring.fade_out = ring.duration - 10;
        prims.push(ring);
    }
}

/// `CRagEffect::EarthSpike`: seven spikes of stone (of ice with its type of 2) at once under the one struck. The first, 3 to 3.5 wide and
/// 18 long, within 10 degrees of upright, is pushed up from 10 under the ground at 1 a tick, quickening, draws
/// back at 1.2 from its twelfth tick and stands still from its fifteenth; it does not fade (the client's second
/// pattern of the kind) and is gone at its 240th tick. The six others, 4 to 5 wide and 20 long, lean 10 degrees
/// out, each turned somewhere in its own sixth of the round, from 20 under the ground and 3.5 to a side; they
/// are pushed up at 1.5 for three ticks and fade over their last 10.
fn earth_spike(picture: &'static str, prims: &mut Vec<Prim>) {
    let spike = |settles: i32| {
        let mut spike = Prim::new(Shape::QuadHorn3d, picture);
        (spike.duration, spike.first_pattern, spike.settles_at) = (240, true, Some(settles));
        spike.alpha = 250.0;
        spike
    };
    let mut first = spike(15);
    first.put.1 = 10.0;
    (first.size, first.height) = ((lot(50) + 300.0) / 100.0, 18.0);
    (first.longitude, first.latitude) = (lot(360), lot(20) + 80.0);
    (first.speed, first.accel) = (1.0, 0.01);
    first.changes = Some(Change { speed: -1.2, accel: 0.0, ..Change::at(12) });
    prims.push(first);
    for step in (60u32..420).step_by(60) {
        let offset = lot(step);
        let round = lot(360);
        let mut side = spike(2);
        (side.put, side.deep) = ((sine(round) * 3.5, 20.0), cosine(round) * -3.5);
        (side.size, side.height) = ((lot(100) + 400.0) / 100.0, 20.0);
        (side.longitude, side.latitude) = ((offset + step as f32 - 60.0).min(360.0), 100.0);
        side.speed = 1.5;
        side.fade_out = side.duration - 10;
        prims.push(side);
    }
}
