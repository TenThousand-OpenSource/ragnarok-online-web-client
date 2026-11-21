//! The effects of a thing thrown at someone, whose primitive is the client's kind 53 (`crate::prim::Tei::
//! ThrowItem`); each is `CRagEffect::ThrowItem` with the picture of an item and a kind
//! (`Source/Contents/Effects/Core/RagEffect.cpp:12130`):
//!
//! - `EF_THROWITEM` (298): the bottle of Acid Terror;
//! - `EF_THROWITEM2` (299): the potion of Potion Pitcher, in the colour of the level it was thrown at;
//! - `EF_THROWITEM3` (308): the stone of Throw Stone;
//! - `EF_THROWITEM5` (541): the potion of Berserk Pitcher;
//! - `EF_THROWITEM6` to `EF_THROWITEM10` (600, 613 to 616): a venom knife, a shuriken, a kunai, a huuma
//!   shuriken, a coin.
//!
//! `EF_THROWITEM4` (539, Acid Demonstration's two bottles) the client of this version makes with a class of
//! its own (`CGameMode::LaunchTargetEffect`, `HasAllocator`), and `EF_THROWITEM11` (914) is a later job's:
//! neither is here. The function starts no sound.

use super::launched::{of_parts, part};
use super::*;
use crate::prim::{Tei, angle_of};

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures of the items thrown: the client's own pictures of them in a bag, and its coin.
const PICTURES: [&str; 9] = [
    "유저인터페이스\\item\\빈병.bmp",
    "유저인터페이스\\item\\빈포션병.bmp",
    "유저인터페이스\\item\\돌.bmp",
    "유저인터페이스\\item\\버서크포션.bmp",
    "유저인터페이스\\item\\베넘나이프.bmp",
    "유저인터페이스\\item\\수리검.bmp",
    "유저인터페이스\\item\\쿠나이_독.bmp",
    "유저인터페이스\\item\\풍마_열화.bmp",
    "coin_a.bmp",
];

/// The picture an effect of this file throws and the kind `ThrowItem` is told with it; nothing for any other
/// effect.
fn item(effect: u32) -> Option<(&'static str, i8)> {
    Some(match effect {
        298 => (PICTURES[0], 0),
        299 => (PICTURES[1], 1),
        308 => (PICTURES[2], 2),
        541 => (PICTURES[3], 1),
        600 => (PICTURES[4], 3),
        613 => (PICTURES[5], 3),
        614 => (PICTURES[6], 3),
        615 => (PICTURES[7], 3),
        616 => (PICTURES[8], 3),
        _ => return None,
    })
}

/// How many ticks an effect of this file lives (the client's `EffectLifetime`); none starts a sound.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        298 | 299 | 541 => (300, ""),
        308 | 600 | 613 | 614 | 615 | 616 => (200, ""),
        _ => return None,
    })
}

/// Each is launched by the one who throws and told the place of the one thrown at (`CGameMode::
/// LaunchTargetEffect`: `source->LaunchEffect` and `AM_SETPOS`).
fn thrown(effect: u32) -> bool {
    item(effect).is_some()
}

/// One tick of an effect of this file: at its first the thing is thrown.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let Some((picture, kind)) = item(effect).filter(|_| course.tick == 0) else { return };
    // From where the thrower stands to the place told, level, and how much lower that place is (the
    // client's y grows downwards).
    let (east, north) = (course.shared.aim[0] - course.walked[0], course.shared.aim[2] - course.walked[2]);
    let lower = course.shared.aim[1] - course.walked[1];
    let mut flying = of_parts(Tei::ThrowItem, picture, false, course);
    flying.size = 0.0;
    flying.parts.push(part().thrown_item(kind, course.count.clamp(0, 127) as i8, east.hypot(north), angle_of(east, north), lower));
    // The three behind it show nothing until they take the first one's look.
    flying.parts.extend((0..3).map(|_| {
        let mut behind = part();
        (behind.live, behind.alpha) = (true, 0);
        behind
    }));
    prims.push(flying);
}
