//! The waterfalls a map places about itself (`world/scenery/placed.rs` launches each at its place, once: they
//! live without end), each effect its function of `CRagEffect` with the same figures:
//!
//! - `EF_WATERFALL` and its seven kin (349 to 356): sheets of water that run down (`Waterfall`; the kind 79)
//!   with a spray at their foot (`FrostScatter`, forty times, or twenty for a small fall; the kind 80). The
//!   kin are the fall turned a quarter (`_90`), the small one, and the two again with the other pictures
//!   (`_T2`);
//! - `EF_BLUEFALL` and its three kin (557 to 560): sheets of blue light that run up, slowly or fast, and
//!   the same turned a quarter (`WaterfallSheets`).
//!
//! None has a sound.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&WATER, &WATER2, &SPRAY], sprites: &[] };

/// The three pictures that run down a fall, and those of its second look.
const WATER: [&str; 3] = ["waterfall11.tga", "waterfall12.tga", "waterfall13.tga"];
const WATER2: [&str; 3] = ["waterfall31.tga", "waterfall32.tga", "waterfall33.tga"];
/// The pictures of a spray: nine in ten of its primitives show the first, by lot.
const SPRAY: [&str; 2] = ["freeze_a_small.bmp", "waterp1.tga"];

/// How many ticks each effect lives (the client's `EffectLifetime`).
fn known(effect: u32) -> Option<(i32, &'static str)> {
    matches!(effect, 349..=356 | 557..=560).then_some((99_999_999, ""))
}

/// One tick of an effect: everything at its first.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick != 0 {
        return;
    }
    match effect {
        349..=356 => {
            let kind = effect - 349;
            let (turned, small, pictures) = (kind % 2 == 1, kind / 2 % 2 == 1, if kind < 4 { &WATER } else { &WATER2 });
            prims.push(fall(pictures, turned, small, course));
            prims.extend((0..if small { 20 } else { 40 }).map(|_| spray(turned, small, course)));
        }
        557..=560 => prims.push(blue_fall(effect % 2 == 0, effect >= 559, course)),
        _ => {}
    }
}

/// The sheets `Waterfall` and `WaterfallSheets` both launch (the kind 79): four, each a piece of 40 for
/// every one of its figures, the first 80 ticks to a piece and each next 13 fewer, at the alpha, the width
/// and the depth `sheet` gives for each number.
fn sheets(pictures: &'static [&'static str; 3], turned: bool, course: &Course, sheet: impl Fn(i32) -> Part) -> Prim {
    let mut prim = of_parts(Tei::Sheet, pictures[0], true, course);
    prim.pictures = pictures;
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { tall: (80 - index * 13) as f32, ..sheet(index) };
        part.flags[0] = i8::from(turned);
        part.heights[..20].iter_mut().enumerate().for_each(|(number, height)| *height = (number * 40) as f32);
        part
    }));
    prim
}

/// `CRagEffect::Waterfall`: four sheets of water in the pictures' own colours, at an alpha of 80, 36 to 39
/// wide (a small fall: 18 to 21), one a unit behind the other from 1 before the place, that run down. The
/// map's first figure, if over 10, makes the fall lower (`RenderScrollColumn`).
fn fall(pictures: &'static [&'static str; 3], turned: bool, small: bool, course: &Course) -> Prim {
    let lower = course.figures[0] as i32 as i8;
    sheets(pictures, turned, course, |index| {
        let mut part = Part { alpha: 80, out: (index + if small { 18 } else { 36 }) as f32, ..Part::default() };
        (part.flags[2], part.at[0][2]) = (lower, index as f32 - 1.0);
        part
    })
}

/// `CRagEffect::WaterfallSheets` with its third figure of 0: four sheets of blue light, added, at an alpha of
/// 120, 36 to 39 wide and a quarter of a unit one behind the other, that run up; the `fast` ones take 30
/// ticks to a piece and each next 6 fewer.
fn blue_fall(turned: bool, fast: bool, course: &Course) -> Prim {
    let mut prim = sheets(&WATER, turned, course, |index| {
        let mut part = Part { alpha: 120, out: (index + 36) as f32, ..Part::default() };
        (part.flags[1], part.flags[2], part.at[0][2]) = (1, 10, index as f32 * 0.25 - 1.0);
        part
    });
    if fast {
        prim.parts.iter_mut().enumerate().for_each(|(index, part)| part.tall = (30 - index as i32 * 6) as f32);
    }
    prim
}

/// `CRagEffect::FrostScatter`: a primitive of the spray (the kind 80), four parts of four specks each, that
/// rise 0.05 a tick from under the ground across the width of the fall. Nine in ten, by lot, are of
/// `freeze_a_small.bmp` and 4.5 from the middle to a corner; the tenth of `waterp1.tga` and 7.
fn spray(turned: bool, small: bool, course: &Course) -> Prim {
    let water = lot(10) == 0.0;
    let mut prim = of_parts(Tei::Spray, SPRAY[usize::from(water)], true, course);
    prim.parts.extend((0..4).map(|_| {
        let mut part = Part { tall: 0.05, out: if water { 7.0 } else { 4.5 }, ..part() };
        (part.flags[0], part.flags[11], part.flags[12]) = (2, i8::from(turned), i8::from(small));
        part.scatter_as_spray();
        part
    }));
    prim
}
