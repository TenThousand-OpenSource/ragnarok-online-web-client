//! The effects the units of skills on the ground launch on themselves (`crate::skill_units`), as far as they are
//! made of primitives ported: each with the figures of its function of `CRagEffect`.
//!
//! - `EF_ICEWALL` (74): the ice of a cell of Ice Wall (`IceWall`);
//! - `EF_FIREPILLARON` (138): the rings of a Fire Pillar that waits (`RingStack` with no flag);
//! - `EF_VENOMDUST2` (171): the motes over a cell of Venom Dust (in `CRagEffect::OnProcess` itself);
//! - `EF_GLASSWALL2` (315): the walls of light of a Safety Wall (`GlassWallRising`; the file that function
//!   plays besides, `SafetyWall.str`, is played as every effect of a file is);
//! - `EF_BOTTOM_VO`, `_DE`, `_VI` (239 to 241): the low wall of light that widens over and over on a cell of
//!   Volcano, Deluge and Violent Gale (`TexturedFlare`), and `EF_BOTTOM_LA` (242): the square that breathes on
//!   a cell of Land Protector (`TexturedHalo`);
//! - `EF_READYPORTAL2` (316): the rings of a Warp Portal that is about to open (`BlueRingWave`), and
//!   `EF_PORTAL2` (317): the walls, rings and winds of one that is open (`VioletRingCloud`).
//!
//! Sanctuary's and Magnus Exorcismus's boxes of light are a primitive kept with the units themselves
//! (`crate::skill_units::Pulse`).

use super::launched::{ripples, PARTICLE3};
use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 9] = [
    "ice.tga", "magic_red.tga", "alpha_down.tga", "ring_red.tga", "ring_blue.tga", "magic_green.tga", "aaa copy.bmp",
    "magic_violet.tga", "cloud11.tga"
];

/// How many ticks an effect here lives (the client's `EffectLifetime`; `IceWall` makes its own 9999) and the
/// sound it starts with.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        74 => (9999, "effect/wizard_icewall.wav"),
        138 => (8000, ""),
        171 => (9999, ""),
        239..=242 => (29999, ""),
        316 => (200, ""),
        317 => (9999, ""),
        315 => (9999, "effect/ef_glasswall.wav"),
        _ => return None,
    })
}

/// One tick of an effect here.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    match effect {
        74 if course.tick == 0 => prims.extend(ice_wall(course.lasts)),
        138 if course.tick == 0 => prims.extend(ring_stack(course.lasts)),
        171 if course.tick % 5 == 0 => prims.push(venom_mote()),
        239..=241 if course.tick == 0 => prims.push(flare(["ring_red.tga", "ring_blue.tga", "magic_green.tga"][effect as usize - 239], course)),
        242 if course.tick == 0 => prims.push(halo(course)),
        315 if course.tick == 0 => prims.push(glass_wall(course.lasts)),
        316 => {
            // `BlueRingWave`: its sound every fourteenth tick, its rings at its first.
            if course.tick % 14 == 0 {
                course.sound("effect/ef_readyportal.wav");
            }
            if course.tick == 0 {
                prims.push(ripples("ring_blue.tga", 5.0, 100, 1, course));
            }
        }
        317 => match course.tick {
            0 => prims.push(portal_walls(course)),
            1 => prims.push(ripples("ring_blue.tga", 4.0, 3111, 0, course)),
            2 => prims.push(portal_winds(course)),
            _ => {}
        },
        _ => {}
    }
}

/// `CRagEffect::IceWall`: three spikes of ice, 2.5 wide and 30 to 40 long, tipped within 3 degrees of upright
/// and turned any way round, in a row through the cell (2 before its middle, on it and 2 behind, along a line
/// drawn by lot). Each comes up from 30 under the ground at 3 a tick, slowing, and stops at its twentieth
/// tick; they stand for as long as the effect does.
fn ice_wall(lasts: i32) -> Vec<Prim> {
    [-2.0f32, 0.0, 2.0]
        .into_iter()
        .map(|offset| {
            let mut spike = Prim::new(Shape::QuadHorn3d, "ice.tga");
            spike.duration = lasts;
            spike.first_pattern = true;
            (spike.put.1, spike.deep) = (30.0, -offset * cosine(lot(360)));
            // (The client adds a lot of 0 to 39 hundredths to the width as a whole number: nothing.)
            spike.size = 2.5;
            spike.height = (lot(100) + 300.0) / 10.0;
            (spike.longitude, spike.latitude) = (lot(360), lot(6) + 87.0);
            spike.settles_at = Some(20);
            spike.alpha = 200.0;
            spike.speed = 3.0;
            spike.accel = -(spike.speed / 40.0) * 2.0;
            spike.fade_out = spike.duration - 10;
            spike
        })
        .collect()
}

/// `CRagEffect::RingStack` with no flag: three bands of `magic_red.tga` round the place, one inside the other
/// and the inner ones taller (4, 5 and 6 out at the top, 2.5, 3.5 and 4.5 at the foot; 27.5, 15.5 and 7.5
/// high), turning 3 degrees a tick against the clock. (The client gives them a `rollSpeed` too, which a band
/// does not use.)
fn ring_stack(lasts: i32) -> Vec<Prim> {
    (1..=3)
        .map(|index| {
            let mut band = Prim::new(Shape::Cylinder3d, "magic_red.tga");
            band.duration = lasts;
            band.long_speed = -3.0;
            (band.outer, band.inner) = (index as f32 + 3.0, index as f32 + 1.5);
            band.height = ((8 - 2 * index) * (5 - index) + 1) as f32 + 2.5;
            band.alpha = 240.0;
            band
        })
        .collect()
}

/// `EF_VENOMDUST2` in `CRagEffect::OnProcess`: every fifth tick a mote of poison that rises from the place,
/// slowing, turning 3 degrees a tick, 0.8 of its size; it lives 10 ticks and fades over the last two. (The
/// client turns it some way round by lot and then makes its turn anew, straight up.)
fn venom_mote() -> Prim {
    let mut mote = Prim::new(Shape::Particle3d, PARTICLE3);
    mote.duration = 10;
    mote.latitude = 90.0;
    mote.speed = (lot(20) + 20.0) / 80.0;
    mote.accel = -(mote.speed / mote.duration as f32) * 0.5;
    mote.size = 0.8;
    mote.roll_speed = 3.0;
    mote.alpha = 70.0;
    mote.alpha_speed = mote.max_alpha / 10.0;
    mote.fade_out = mote.duration - mote.duration / 5;
    mote
}

/// `CRagEffect::GlassWallRising`: three upright walls of light round the place, one inside the other (3.6, 3.3
/// and 3 out; 34, 37 and 40 tall), each begun at any turn, pink (`RenderTeiRect` for a primitive of size 10),
/// their light added. The outer one fades from its 5500th tick, the others from their 3080th.
fn glass_wall(lasts: i32) -> Prim {
    let mut walls = Prim::new(Shape::Casting3d, "alpha_down.tga");
    (walls.duration, walls.colour, walls.adds) = (lasts, [255.0, 89.0, 182.0], true);
    walls.slots = [(3.6, 34.0, 5500), (3.3, 37.0, 3080), (3.0, 40.0, 3080)]
        .into_iter()
        .map(|(out, tall, holds)| Slot::wall(lot(360) as i32, out, 90, tall, holds))
        .collect();
    walls
}

/// A primitive of a kind made of the client's parts kept whole, of that `size`, that lives as long as its effect
/// and stays with it.
fn sized(kind: Tei, picture: &'static str, size: f32, course: &Course) -> Prim {
    let mut prim = launched::of_parts(kind, picture, true, course);
    prim.size = size;
    prim
}

/// `CRagEffect::TexturedFlare` with no size: one low wall of light all the way round (the kind 36), 15 tall,
/// begun 2.6 out at any turn, rising at 80 degrees and showing by lot; mixed in, in its picture's own colours.
fn flare(picture: &'static str, course: &Course) -> Prim {
    let mut wall = sized(Tei::LowRing, picture, 0.0, course);
    wall.parts.push(Part { round: 360, tall: 15.0, turned: lot(360) as i32, alpha: lot(100) as i32, out: 2.6, rise: 80, ..Part::default() });
    wall
}

/// `CRagEffect::TexturedHalo` with its kind 1: a level square of `aaa copy.bmp` 2 over the ground (the kind
/// 39), 7 out, at an alpha of 100, its breath begun anywhere; white, its light added.
fn halo(course: &Course) -> Prim {
    let mut square = sized(Tei::Halo, "aaa copy.bmp", 5.0, course);
    let mut heights = [0.0; 21];
    (heights[0], heights[4]) = (-2.0, 1.0);
    square.parts.push(Part { out: 7.0, alpha: 100, rise: lot(360) as i32, heights, ..Part::default() });
    square
}

/// `CRagEffect::VioletRingCloud`, its first tick: three upright walls of `magic_violet.tga` round the place
/// (the kind 60, its columns of the kind 1), 4, 3 and 2 out and 50 tall, each begun at any turn; they turn,
/// and go down from their 1400th tick. Blue (`RenderTeiRect` for a primitive of size 4), their light added.
fn portal_walls(course: &Course) -> Prim {
    let mut walls = sized(Tei::Columns, "magic_violet.tga", 4.0, course);
    walls.parts.extend([4.0, 3.0, 2.0].into_iter().map(
        |out| Part { round: 360, tall: 50.0, turned: lot(360) as i32, out, rise: 90, alpha_top: 1400, flags: [1; 21], ..Part::default() }
    ));
    walls
}

/// `CRagEffect::VioletRingCloud`, its third tick: four arcs of `cloud11.tga` that go round the place
/// (`PP_WIND`), a quarter of a turn apart, the nearer the higher (9, 7.5, 6 and 4.5 out; 1.5, 3.5, 5.5 and
/// 7.5 over the ground), each 1.5 tall and upright; blue as the walls are.
fn portal_winds(course: &Course) -> Prim {
    let mut winds = sized(Tei::Wind, "cloud11.tga", 4.0, course);
    winds.parts.extend((0..4).map(|index| Part {
        round: 30,
        tall: index as f32 * 2.0 + 1.5,
        turned: index * 90,
        out: 9.0 - index as f32 * 1.5,
        rise: 90,
        heights: [1.5; 21],
        ..Part::default()
    }));
    winds
}
