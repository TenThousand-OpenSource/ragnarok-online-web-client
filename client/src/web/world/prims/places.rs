//! The effects a map places about itself (its `.rsw` lists them, `CWorld::LoadMapEffects`; the world launches
//! each at its place again and again, `world/scenery/placed.rs`), which the client draws by code in
//! `CRagEffect::OnProcess`:
//!
//! - `EF_SMOKE` (44): the smoke of a chimney, one to four puffs that rise turning and fade;
//!   and `EF_DRAGONSMOKE` (373), one such puff, which no map places: a script shows it on someone;
//! - `EF_FIREFLY` (45): a firefly, a light whose way wanders (`FireSparks`);
//! - `EF_TORCH` (47): the flame of a torch, as large and as fast as the map's figures say;
//! - `EF_BANJJAKII` (165): a twinkle of the Christmas lights, one of five by lot;
//! - `EF_FORESTLIGHT` and its second, third and fourth (307, 322, 323, 324): shafts of light that slant down
//!   through the trees (`CRagEffect::ForestLight` with its flags 0, 1, 2 and 10);
//! - `EF_MAPPILLAR` and its second, third and fourth (231, 247, 259, 260): a pillar of light that stands up
//!   and sinks again (`CRagEffect::TexturedCrossStaged`), which scripts show on someone too;
//! - `EF_BIG_PORTAL` and its second (561, 562): the walls and the winds of a large portal
//!   (`CRagEffect::TexturedMagicRing` with a type alone);
//! - `EF_GHOST`, `EF_BAT` and `EF_BAT2` (358, 359, 360): a ghost or a bat that goes round the place, with
//!   fainter ones behind it, as the maps of Niflheim place them and scripts show them (`CRagEffect::Ghost`
//!   with its types 0, 1 and 2).
//!
//! Each of the first four is a sprite 10 over the place (the smoke 9). None has a sound. The waterfalls a map
//! places are in `waterfalls.rs`.

use super::launched::{PARTICLE1, of_parts, part};
use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[&SPRITES] };

/// The pictures the effects here are drawn with.
const PICTURES: [&str; 5] = [SHAFT, "ring_blue.tga", "magic_green.tga", "ring_red.tga", "magic_violet.tga"];

/// The picture the shafts of light are drawn with.
const SHAFT: &str = "cloud11.tga";

const SMOKE: &str = "이팩트/굴뚝연기";
const TORCH: &str = "이팩트/torch_01";
const TWINKLE: &str = "이팩트/크리스마스";
const GHOST: &str = "이팩트/유령";
const BAT: &str = "이팩트/박쥐";

/// The sprites the effects here show, among the client's sprites.
const SPRITES: [&str; 6] = [SMOKE, PARTICLE1, TORCH, TWINKLE, GHOST, BAT];

/// How many ticks each effect lives (the client's `EffectLifetime`; a firefly's is set by its function, and a
/// twinkle's, 1000 there, is cut here to the 125 its one sprite lives, after which it shows nothing).
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        44 | 373 => (50, ""),
        45 => (140, ""),
        47 => (250, ""),
        165 => (125, ""),
        307 | 322 | 323 | 324 => (36000, ""),
        231 | 247 | 259 | 260 => (999, ""),
        561 => (2000, ""),
        562 => (99_999_999, ""),
        358..=360 => (4000, ""),
        _ => return None,
    })
}

/// One tick of an effect: everything at its first, but the winds of a large portal, at its third, and a ghost
/// or a bat, one at each of its first ten.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick <= 9 && matches!(effect, 358..=360) {
        prims.push(ghost((effect - 358) as i8, course));
    }
    if course.tick == 2 && matches!(effect, 561 | 562) {
        prims.push(great_winds(course));
    }
    if course.tick != 0 {
        return;
    }
    match effect {
        231 => prims.push(pillar("ring_blue.tga", 0, course)),
        247 => prims.push(pillar("ring_blue.tga", 1, course)),
        259 => prims.push(pillar("magic_green.tga", 2, course)),
        260 => prims.push(pillar("ring_red.tga", 3, course)),
        561 => prims.push(great_walls(1400, course)),
        562 => prims.push(great_walls(-1, course)),
        44 => prims.extend((0..=lot(4) as u32).map(|_| puff(course.lasts))),
        // `EF_DRAGONSMOKE` (in `CRagEffect::OnProcess2`): one such puff, on whoever a script shows it on.
        373 => prims.push(puff(course.lasts)),
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

/// `CRagEffect::Ghost` with its type 0 (a ghost), 1 (a bat) or 2 (a bat that flies a figure of eight), at one
/// of the effect's first ten ticks: a sprite half as large again as its file has it (`PP_GHOST`), a frame
/// every second tick, that goes round the place where the effect is now, 16 out, a degree a tick (the type
/// 2: two), bobbing 4 up and down. Its alpha comes up from nothing to 30 for the one of the first tick and 3
/// less for each after it, so that fainter ones follow the first; the first bat of the type 2 comes up to
/// 250. It lives as long as the effect and stays in the world where it was launched.
///
/// (The client also has it drawn in its own turn among what is see-through, `renderFlag |= 0x200`: a queue
/// of `RenderQueues::Submit` sorted by depth. Here it is drawn with the other sprites of effects. Its
/// `SetAction(0, 16, 1)` is `CRenderObject::SetAction`, which takes the first action at its first frame and
/// the speed its file gives that action, and does not look at the 16.)
fn ghost(kind: i8, course: &Course) -> Prim {
    let mut ghost = Prim::new(Shape::Ghost, if kind == 0 { GHOST } else { BAT });
    (ghost.duration, ghost.size, ghost.alpha, ghost.anim_speed) = (course.lasts, 1.5, 0.0, 2);
    ghost.left = Some(course.walked);
    let mut round = part();
    (round.out, round.tall) = (16.0, 4.0);
    round.heights[1] = if kind == 2 && course.tick == 0 { 250.0 } else { ((10 - course.tick) * 3) as f32 };
    (round.flags[0], round.flags[2]) = (if kind == 2 { 2 } else { 1 }, kind);
    ghost.parts.push(round);
    ghost
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

/// `CRagEffect::TexturedCrossStaged` with one of its levels 0 to 3: four walls of light all the way round (the
/// kind 41), a quarter of a turn apart and each begun 30 ticks further on, 2 to 3.5 from the place (the
/// level 1: 11 to 12.5) and all but upright, that stand up to 120 after their 200th tick and sink from
/// their 800th. Their alpha is 50 (the level 1: 70) and stays so. Blue for the levels 0 and 1, green for the
/// level 2 and red for the level 3 (`RenderTeiRect` for a primitive of size 4, 7 and 6), their light added.
fn pillar(picture: &'static str, level: i32, course: &Course) -> Prim {
    let mut prim = of_parts(Tei::MapPillar, picture, true, course);
    prim.size = match level {
        2 => 7.0,
        3 => 6.0,
        _ => 4.0,
    };
    prim.parts.extend((0..4).map(|index| Part {
        process: index * 30,
        round: 360,
        tall: 120.0,
        turned: index * 90,
        alpha: if level == 1 { 70 } else { 50 },
        out: index as f32 * 0.5 + if level == 1 { 11.0 } else { 2.0 },
        rise: 89,
        ..Part::default()
    }));
    prim
}

/// `CRagEffect::TexturedMagicRing` with a type of 0 or 1, its first tick: three upright walls of
/// `magic_violet.tga` round the place (the kind 60, its columns of the kind 3), 7, 5.5 and 4 out and 80
/// tall, each begun at any turn; they turn, and go down from the tick they hold to (1400 for the type 0;
/// never for the type 1). Blue (`RenderTeiRect` for a primitive of size 4), their light added.
fn great_walls(holds: i32, course: &Course) -> Prim {
    let mut walls = of_parts(Tei::Columns, "magic_violet.tga", true, course);
    walls.size = 4.0;
    walls.parts.extend([7.0, 5.5, 4.0].map(
        |out| Part { round: 360, tall: 80.0, turned: lot(360) as i32, out, rise: 90, alpha_top: holds, flags: [3; 21], ..Part::default() }
    ));
    walls
}

/// `CRagEffect::TexturedMagicRing` with a type of 0 or 1, its third tick: four arcs of `cloud11.tga` that go
/// round the place (`PP_WIND`, numbered 3: they do not go down), a quarter of a turn apart, the nearer the
/// higher (13, 11, 9 and 8 out; 2.5, 5, 7.5 and 10 over the ground), each 1.5 tall and upright; blue as the
/// walls are.
fn great_winds(course: &Course) -> Prim {
    let mut winds = of_parts(Tei::Wind, SHAFT, true, course);
    winds.size = 4.0;
    winds.parts.extend([13.0, 11.0, 9.0, 8.0].into_iter().enumerate().map(|(index, out)| Part {
        round: 30,
        tall: (index + 1) as f32 * 2.5,
        turned: index as i32 * 90,
        out,
        rise: 90,
        alpha_top: 3,
        heights: [1.5; 21],
        ..Part::default()
    }));
    winds
}
