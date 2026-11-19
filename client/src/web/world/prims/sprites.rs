//! What of the client's effects is made of sprites (`PP_3DPARTICLE`, 5, and `PP_3DPARTICLEGRAVITY`, 6), each
//! with the figures of its function of `CRagEffect`:
//!
//! - `EF_BLESSING` (42): Blessing, its angel and its sparks, and the light on the ground under them, which is
//!   no sprite (`Blessing`);
//! - `EF_STEAL` (18): Steal (`Steal`);
//! - `EF_DETOXICATION` (21): Detoxify (`Detoxication`);
//! - `EF_SIGHT` (22): the fire going round the one who sees (`Sight`);
//! - `EF_RUWACH` (33): the light going round the one who sees by Ruwach (in the client's
//!   `CRagEffect::OnProcess` itself);
//! - `EF_AQUA` (39): Aqua Benedicta (in the client's `CRagEffect::OnProcess` itself);
//! - `EF_ENCHANTPOISON` (125): Enchant Poison (`PoisonAttack`);
//! - `EF_EDP` (493): Enchant Deadly Poison, bubbles that rise round the one it is on, who flashes purple
//!   (`EnchantPoison2`);
//! - `EF_KOUENKA` (618): the petals of fire that rise round the one struck by Kouenka (`SakuraFireArrow`,
//!   `PP_KOUENKA`), with two crosses of beams (`TexturedQuadSpin`).
//!
//! The client has an option to show fewer effects (`TT_MIN_EFFECT_ON_OFF`); what is here is what it shows with
//! the option off, as it comes.
//!
//! The sprites several files show are read with this file's (`SPRITES`); their names are in `launched.rs`.

use super::launched::{FIREBALL, PARTICLE1, PARTICLE2, PARTICLE3, PARTICLE4, SHADOW, SIGHT, SPEAR, thrown};
use super::*;

/// The sprites the effects here show, among the client's sprites (the client names them `이팩트\name.spr`, the
/// shadow `Shadow.spr`).
const SPRITES: [&str; 13] = [SAKURA, PARTICLE1, PARTICLE2, PARTICLE3, PARTICLE4, PARTICLE6, PARTICLE7, ANGEL, SIGHT, SHADOW, FIREBALL, AQUA, SPEAR];

const PARTICLE6: &str = "이팩트/particle6";
const PARTICLE7: &str = "이팩트/particle7";
const ANGEL: &str = "이팩트/축복";
const AQUA: &str = "이팩트/성수뜨기";
const SAKURA: &str = "이팩트/sakura01";

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[&SPRITES] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 2] = ["alpha_down.tga", "alpha_center.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        18 => (50, "effect/ef_steal.wav"),
        21 => (100, "effect/ef_detoxication.wav"),
        22 => (270, ""),
        // `EF_RUWACH` has no sound in its function.
        33 => (200, ""),
        39 => (100, "effect/ef_aqua.wav"),
        42 => (150, "effect/ef_blessing.wav"),
        // (`PoisonAttack` starts `EF_PoisonAttack.wav` with this one as well; an effect here starts one sound.)
        125 => (250, "effect/assasin_enchantpoison.wav"),
        493 => (120, "effect/assasin_enchantpoison.wav"),
        // (`SakuraFireArrow` starts its sounds from its 13th tick, in `kouenka`.)
        618 => (300, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        18 if tick == 0 => prims.extend(steal(lasts)),
        21 => rising_sparks(false, tick, prims),
        22 => sight(tick, prims),
        33 => ruwach(tick, prims),
        39 if tick == 0 => prims.push(aqua(lasts)),
        42 => {
            if tick == 0 {
                prims.push(blessing_light(lasts));
            }
            blessing(tick, lasts, prims);
        }
        125 => rising_sparks(true, tick, prims),
        493 => {
            // `EnchantPoison2` starts its sound six times, and flashes its parent purple for the first 80 ticks.
            course.starts(&[5, 11, 18, 26, 40].map(|at| (at, "effect/assasin_enchantpoison.wav")));
            if tick < 81 {
                course.touch.flashes(tick, [255, 0, 255]);
            }
            if tick % 3 == 0 {
                prims.push(poison_bubble());
            }
        }
        618 => kouenka(course, prims),
        _ => {}
    }
}

/// `EF_KOUENKA` in `CRagEffect::OnProcess`: two crosses of beams at once (`TexturedQuadSpin` with its kind 2),
/// and `SakuraFireArrow`: at each of the first twenty ticks a petal of `sakura01.spr` (`PP_KOUENKA`) 8 out from
/// the one struck, any way round, in one of the sprite's first three actions and about a third of its size; it
/// rises 0.3 to 0.6 a tick, wandering, and fades from an alpha of 250. At the 13th to the 16th tick one of
/// the three sounds of Fire Arrow, by lot.
fn kouenka(course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 0 {
        prims.extend([launched::quad_spin(0, 2, course), launched::quad_spin(1, 2, course)]);
    }
    if (12..=15).contains(&course.tick) {
        course.sound(&format!("effect/ef_firearrow{}.wav", lot(3) as i32 + 1));
    }
    if course.tick >= 20 {
        return;
    }
    let mut petal = Prim::new(Shape::Kouenka, SAKURA);
    (petal.duration, petal.alpha) = (course.lasts, 250.0);
    let mut wander = launched::part();
    wander.tall = (lot(3) + 2.0) * 0.15;
    petal.size = (lot(6) + 22.0) / 70.0;
    let angle = lot(360);
    petal.moved = [cosine(angle) * 8.0, 0.0, sine(angle) * 8.0];
    wander.heights[..4].copy_from_slice(&[lot(360), lot(360), lot(360), lot(360)]);
    petal.parts.push(wander);
    petal.action = lot(3) as usize;
    prims.push(petal);
}

/// `CRagEffect::Steal`: ten sparks thrown every way from 10 above the one robbed, that go level as they slow
/// and are thrown up to fall back, shrinking to nothing and fading over the last third.
fn steal(lasts: i32) -> Vec<Prim> {
    (0..10)
        .map(|_| {
            let mut spark = thrown(Shape::ParticleGravity3d, PARTICLE7, lot(360), lot(100) - 50.0, lot(4) + 4.0);
            // It goes level: its turn is made anew without the tip (`MakeYRotation`).
            spark.latitude = 0.0;
            spark.duration = lasts;
            spark.speed = (lot(50) + 50.0) / 100.0;
            spark.accel = -(spark.speed / spark.duration as f32) * 0.5;
            spark.grav_speed = -(lot(90) + 60.0) / 100.0;
            spark.grav_accel = -(spark.grav_speed / spark.duration as f32) * 2.0;
            spark.size = (lot(100) + 100.0) / 100.0;
            spark.size_speed = -(spark.size / spark.duration as f32);
            spark.fade_out = spark.duration - spark.duration / 3;
            spark
        })
        .collect()
}

/// A spark that rises turning from the ground, somewhere 2 to 7 around the one the effect is on: it comes up
/// over 10 ticks, slows, and fades over the last fifth of its 40 ticks. `CRagEffect::Detoxication` and
/// `CRagEffect::PoisonAttack` launch one every fifth tick, each with its own sprite.
fn rising(sprite: &'static str) -> Prim {
    let mut spark = Prim::new(Shape::Particle3d, sprite);
    spark.duration = 40;
    let (round, away) = (lot(360), lot(6) + 2.0);
    (spark.put.0, spark.deep) = (sine(round) * away, cosine(round) * away);
    spark.latitude = 90.0;
    spark.speed = (lot(30) + 30.0) / 100.0;
    spark.accel = -(spark.speed / spark.duration as f32) * 0.5;
    spark.size = (lot(50) + 40.0) / 100.0;
    spark.roll_speed = 3.0;
    spark.alpha = 0.0;
    spark.alpha_speed = spark.max_alpha / 10.0;
    spark.fade_out = spark.duration - spark.duration / 5;
    spark
}

/// `CRagEffect::Detoxication` (`poison`: no) and `CRagEffect::PoisonAttack` as `EF_ENCHANTPOISON` runs it
/// (`poison`: yes): a rising spark every fifth tick.
fn rising_sparks(poison: bool, tick: i32, prims: &mut Vec<Prim>) {
    if tick % 5 == 0 {
        prims.push(rising(if poison { PARTICLE3 } else { PARTICLE2 }));
    }
}

/// `CRagEffect::Sight`: every other tick a flame 20 above the ground and its shadow on it, at a place 15 from
/// the one who sees that goes round by 5 degrees a tick. Each stays where it was left for 20 ticks, shrinking
/// and fading, so that a tail of them follows the fire round. (The client gives the flame a way to change its
/// place too, `deltaPosAccel`, which a primitive without `pattern & 0x400` never uses.)
fn sight(tick: i32, prims: &mut Vec<Prim>) {
    if tick % 2 != 0 {
        return;
    }
    let round = tick as f32 * -5.0;
    let at = (sine(round) * 15.0, cosine(round) * -15.0);
    let mut flame = Prim::new(Shape::Particle3d, SIGHT);
    flame.duration = 20;
    (flame.put, flame.deep) = ((at.0, -20.0), at.1);
    flame.anim_speed = 3;
    (flame.size, flame.size_speed) = (2.5, -0.1);
    (flame.alpha, flame.alpha_speed) = (150.0, -3.0);
    let mut shadow = Prim::new(Shape::Particle3d, SHADOW);
    shadow.duration = 20;
    (shadow.put.0, shadow.deep) = at;
    shadow.anim_speed = 2;
    (shadow.size, shadow.size_speed) = (1.5, -0.06);
    (shadow.alpha, shadow.alpha_speed) = (120.0, -1.0);
    prims.extend([flame, shadow]);
}

/// `EF_RUWACH` in `CRagEffect::OnProcess`: every third tick a light 10 above the ground and its shadow on it,
/// at a place 15 from the one who sees that goes round by 5 degrees every 1.3 ticks. Each stays where it was
/// left for 25 ticks, shrinking and fading (and gone the faster over its last 6), so that a tail of them
/// follows the light round.
fn ruwach(tick: i32, prims: &mut Vec<Prim>) {
    if tick % 3 != 0 {
        return;
    }
    let round = (tick as f32 / 1.3 * -5.0).trunc();
    let at = (sine(round) * 15.0, cosine(round) * -15.0);
    let mut light = Prim::new(Shape::Particle3d, PARTICLE2);
    light.duration = 25;
    (light.put, light.deep) = ((at.0, -10.0), at.1);
    (light.size, light.size_speed) = (3.0, -0.1);
    (light.alpha, light.alpha_speed) = (250.0, -3.0);
    light.fade_out = light.duration - 6;
    let mut shadow = Prim::new(Shape::Particle3d, SHADOW);
    shadow.duration = 25;
    (shadow.put.0, shadow.deep) = at;
    shadow.anim_speed = 2;
    (shadow.size, shadow.size_speed) = (1.5, -0.06);
    (shadow.alpha, shadow.alpha_speed) = (150.0, -2.5);
    shadow.fade_out = shadow.duration - 6;
    prims.extend([light, shadow]);
}

/// `CRagEffect::Blessing`, its first primitive: a disc of blue light 10 across its radius on the ground under
/// the one blessed, that comes up over 30 ticks and fades over the last 30. (The client draws it over
/// everything, whatever stands before it; here it is drawn as the other effects are.) The rest of the effect
/// is sprites: `blessing`.
fn blessing_light(lasts: i32) -> Prim {
    let mut light = Prim::new(Shape::Circle3d, "alpha_down.tga");
    light.duration = lasts;
    light.first_pattern = true;
    light.radius = 10.0;
    light.latitude = 90.0;
    light.colour = [32.0, 176.0, 232.0];
    light.alpha = 0.0;
    light.max_alpha = 100.0;
    light.alpha_speed = light.max_alpha / 30.0;
    light.fade_out = light.duration - 30;
    light
}

/// The sprites of `CRagEffect::Blessing`. At ticks 0, 3, 6 and 9 the angel, 25 above the one blessed, each
/// fainter than the one before, its frames two ticks apiece; all four are told the effect's whole time, so the
/// three that begin late fade with it from their first tick, and the first stays as it is to the end. And every
/// fourth tick a spark from 25 above, somewhere 2 to 8 around, that sinks as it slows and flickers, half its
/// own size, for 80 ticks.
fn blessing(tick: i32, lasts: i32, prims: &mut Vec<Prim>) {
    if tick % 3 == 0 && tick < 10 {
        let mut angel = Prim::new(Shape::Particle3d, ANGEL);
        angel.duration = lasts;
        angel.anim_speed = 2;
        angel.alpha = (tick * -6 + 200) as f32;
        angel.put.1 = -25.0;
        prims.push(angel);
    }
    if tick % 4 == 0 {
        let mut spark = Prim::new(Shape::Particle3d, PARTICLE6);
        spark.duration = 80;
        spark.flickers = true;
        let (round, away) = (lot(360), lot(7) + 2.0);
        (spark.put, spark.deep) = ((sine(round) * away, -25.0), cosine(round) * away);
        spark.latitude = -90.0;
        spark.speed = (lot(20) + 20.0) / 100.0;
        spark.accel = -(spark.speed / spark.duration as f32) * 0.5;
        spark.alpha = 0.0;
        spark.size = 0.5;
        spark.fade_out = spark.duration - spark.duration / 5;
        spark.alpha_speed = spark.max_alpha / 10.0;
        prims.push(spark);
    }
}

/// `EF_AQUA` in `CRagEffect::OnProcess`: the water being drawn, 20 above the one who draws it, its frames two
/// ticks apiece, once through: it ends with its last frame. (The layers of that sprite's frames are turned
/// and tinted in its file; the tints are left out, as the client draws every layer in the primitive's colour.)
fn aqua(lasts: i32) -> Prim {
    let mut water = Prim::new(Shape::Particle3d, AQUA);
    water.duration = lasts;
    water.put.1 = -20.0;
    (water.anim_speed, water.repeats) = (2, false);
    water
}

/// `CRagEffect::EnchantPoison2`, its primitive: a bubble that rises turning from the ground, somewhere 2 to 7
/// around the one the effect is on; it comes up over 10 ticks, slows, and fades over the last fifth of its 40
/// ticks.
fn poison_bubble() -> Prim {
    let mut bubble = Prim::new(Shape::Particle3d, PARTICLE3);
    bubble.duration = 40;
    let (round, away) = (lot(360), lot(6) + 2.0);
    (bubble.put.0, bubble.deep) = (sine(round) * away, cosine(round) * away);
    bubble.latitude = 90.0;
    bubble.speed = (lot(50) + 30.0) / 100.0;
    bubble.accel = -(bubble.speed / bubble.duration as f32) * 0.5;
    bubble.size = (lot(20) + 20.0) / 100.0;
    bubble.roll_speed = 3.0;
    bubble.alpha = 0.0;
    bubble.alpha_speed = bubble.max_alpha / 10.0;
    bubble.fade_out = bubble.duration - bubble.duration / 5;
    bubble
}
