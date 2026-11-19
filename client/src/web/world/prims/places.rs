//! The effects a map places about itself (its `.rsw` lists them, `CWorld::LoadMapEffects`; the world launches
//! each at its place again and again, `world/scenery/placed.rs`), which the client draws by code in
//! `CRagEffect::OnProcess`:
//!
//! - `EF_SMOKE` (44): the smoke of a chimney, one to four puffs that rise turning and fade;
//! - `EF_FIREFLY` (45): a firefly, a light whose way wanders (`FireSparks`);
//! - `EF_TORCH` (47): the flame of a torch, as large and as fast as the map's figures say;
//! - `EF_BANJJAKII` (165): a twinkle of the Christmas lights, one of five by lot;
//! - `EF_FORESTLIGHT` and its second, third and fourth (307, 322, 323, 324): shafts of light that slant down
//!   through the trees (`CRagEffect::ForestLight` with its flags 0, 1, 2 and 10).
//!
//! Each of the first four is a sprite 10 over the place (the smoke 9). None has a sound.

use super::launched::{PARTICLE1, of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&[SHAFT]], sprites: &[&SPRITES] };

/// The picture the shafts of light are drawn with.
const SHAFT: &str = "cloud11.tga";

const SMOKE: &str = "이팩트/굴뚝연기";
const TORCH: &str = "이팩트/torch_01";
const TWINKLE: &str = "이팩트/크리스마스";

/// The sprites the effects here show, among the client's sprites.
const SPRITES: [&str; 4] = [SMOKE, PARTICLE1, TORCH, TWINKLE];

/// How many ticks each effect lives (the client's `EffectLifetime`; a firefly's is set by its function, and a
/// twinkle's, 1000 there, is cut here to the 125 its one sprite lives, after which it shows nothing).
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        44 => (50, ""),
        45 => (140, ""),
        47 => (250, ""),
        165 => (125, ""),
        307 | 322 | 323 | 324 => (36000, ""),
        _ => return None,
    })
}

/// One tick of an effect: everything at its first.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick != 0 {
        return;
    }
    match effect {
        44 => prims.extend((0..=lot(4) as u32).map(|_| puff(course.lasts))),
        45 => prims.push(firefly()),
        47 => prims.push(flame(course)),
        165 => prims.push(twinkle(course)),
        307 => prims.push(shafts(0, course)),
        322 => prims.push(shafts(1, course)),
        323 => prims.push(shafts(2, course)),
        324 => prims.push(shafts(10, course)),
        _ => {}
    }
}

/// `EF_SMOKE`: a puff tipped 75 to 104 degrees and turned any way round, that rises along that way by 0.3 to
/// 0.5 a tick, turning 0.75 a tick, half as large again as its sprite, and fades over its last third.
fn puff(lasts: i32) -> Prim {
    let mut puff = Prim::new(Shape::Particle3d, SMOKE);
    puff.duration = lasts;
    (puff.latitude, puff.longitude) = (lot(30) + 75.0, lot(360));
    puff.put.1 = -9.0;
    (puff.roll_speed, puff.speed, puff.size) = (0.75, (lot(3) + 3.0) / 10.0, 1.5);
    puff.fade_out = lasts - lasts / 3;
    puff
}

/// `CRagEffect::FireSparks`: a light 2 to 14 from a point 10 over the place, any way round and up or down,
/// 0.3 to 0.99 of its sprite's size, that flies on by 0.15 to 0.49 a tick and changes its way every few ticks
/// (`Shape::Roamer`); its alpha flickers up to 80.
fn firefly() -> Prim {
    let mut fly = Prim::new(Shape::Roamer, PARTICLE1);
    (fly.duration, fly.yaw_first, fly.flickers) = (140, true, true);
    fly.speed = (lot(35) + 15.0) / 100.0;
    (fly.longitude, fly.latitude) = (lot(360), lot(360));
    let out = fly.forward(lot(13) + 2.0);
    (fly.put, fly.deep) = ((out[0], -out[1] - 10.0), out[2]);
    let mut wander = part();
    wander.process = lot(25) as i32 + 5;
    fly.parts.push(wander);
    fly.size = (lot(70) + 30.0) / 100.0;
    (fly.alpha, fly.max_alpha) = (0.0, 80.0);
    fly.alpha_speed = fly.max_alpha / 1.5;
    fly
}

/// `EF_TORCH`: a flame in its file's own colours for as long as the effect lives, as many times its sprite's
/// size as the map's first figure says (1 for none) and a frame every so many ticks as its second (1 at least).
fn flame(course: &Course) -> Prim {
    let mut flame = Prim::new(Shape::Particle3d, TORCH);
    (flame.duration, flame.own_colours) = (course.lasts, true);
    flame.put.1 = -10.0;
    flame.size = if course.figures[0] <= 0.0 { 1.0 } else { course.figures[0] };
    flame.anim_speed = (course.figures[1].max(1.0)) as i32;
    flame
}

/// `EF_BANJJAKII`: one of the five actions of the Christmas lights' sprite by a lot of ten, in its file's own
/// colours, a frame every so many ticks as the map's second figure says; three lots of the ten show it so
/// small that nothing is seen (a hundredth of its size, as the client has it).
fn twinkle(course: &Course) -> Prim {
    let mut twinkle = Prim::new(Shape::Particle3d, TWINKLE);
    (twinkle.duration, twinkle.own_colours) = (125, true);
    twinkle.put.1 = -10.0;
    twinkle.anim_speed = (course.figures[1].max(1.0)) as i32;
    let kind = lot(10) as usize;
    twinkle.size = if kind % 3 == 1 { 0.01 } else { 1.0 };
    twinkle.action = kind / 2;
    twinkle
}

/// `CRagEffect::ForestLight` with one of its flags 0, 1, 2 and 10: four shafts of five sides (`PP_FOREST`) from
/// the place up to one 300 over it and 70 to the west and to the south (60 over and 15 aside for the flag
/// 10), each begun 25 degrees further round, for as long as the effect lives. From the middle to a side they
/// are 2, 3 and 4 (the flag 1: 4, 6 and 8; the flag 2: 1, 1.5 and 2), and the fourth, which begins at a count
/// of 180, as the first. Their alpha is 40 (the flag 2: 30; the flag 10: 25) and stays so.
fn shafts(flag: i8, course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Forest, SHAFT, false, course);
    prim.parts.extend((0..4).map(|index| {
        let number = if index == 3 { 0.0 } else { index as f32 };
        let wide = match flag {
            1 => number * 2.0 + 4.0,
            2 => number * 0.5 + 1.0,
            _ => number + 2.0,
        };
        let alpha = match flag {
            2 => 30,
            10 => 25,
            _ => 40,
        };
        let mut shaft = crate::prim::Part { process: if index == 3 { 180 } else { 0 }, alpha, tall: wide, turned: index * 25, ..part() };
        (shaft.heights[0], shaft.at[3], shaft.flags[0]) = (wide, if flag == 10 { [-15.0, -60.0, -15.0] } else { [-70.0, -300.0, -70.0] }, flag);
        shaft
    }));
    prim
}
