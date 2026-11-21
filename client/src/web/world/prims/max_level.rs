//! The aura of the highest level, three effects that live until they are ended (`world/aura.rs` keeps them on
//! those `crate::aura` names). The client has a class for each, not a function of `CRagEffect`
//! (`Contents/Effects/Skills`; `EffectLists::RegisterAllocator` gives each its number, and `Spawn` makes that
//! class whoever asks for the number); its particles are primitives of `crate::prim`'s here
//! (`prim/tei/max_level.rs`), launched at the first tick with the figures the class's `Open` gives them:
//!
//! - `EF_LEVEL99` (200), `CLevel99Effect`: two squares of `pikapika2.bmp` lying over the ground, which
//!   breathe, and three bands of `ring_blue.tga` that go round, blue (`Open`, `AddRings`): what `/aura` adds;
//! - `EF_LEVEL99_ORB1` (976), `CLevel99Orb1Effect`: sixteen lights of `whitelight.tga` that rise, blue;
//! - `EF_LEVEL99_ORB2` (977), `CLevel99Orb2Effect`: sixteen smaller lights of `freezing_circle.bmp`, white:
//!   what a baby job shows in place of 976 (the client's `Level99AuraTable`).
//!
//! Each lives while its actor is there (`IsAlive` is `ActorExists`; `EffectLifetime` has 99999999 for them).
//!
//! The particles of 200 and of 977 are made with the `flags` 13 (`CreateParticle`, `Part::UNDER_BODIES`),
//! those of 976 with 5 (`Part::OVER_BODIES`): the renderer draws the first ahead of every shadow and body, so
//! that whoever stands there covers them, and the second over everyone (`crate::prim`'s `tei/max_level.rs`
//! says from where). A part keeps them in its `flags[1]`; the frame asks `Prim::under_bodies`
//! (`frame/effects.rs`).
//!
//! Not here: `CLevel99Effect::SetParticleTexture` and `SetColor`, by which a state of a later time
//! (`EFST_EQUIPED_FLOOR`, `EffectLists::SpawnLevel99`) gives the same effect another picture
//! (`번개3.bmp`) and colour; the other numbers the client calls `EF_LEVEL99…` (201, 202, 362, 397, 398, 881),
//! which are functions of `CRagEffect` that nothing of the aura launches; and the auras of the later jobs
//! (978 and 979, 1022 and 1023, 1226 and 1227, 2275 and 2276).

use super::launched::{of_parts, part};
use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 4] = ["pikapika2.bmp", "ring_blue.tga", "whitelight.tga", "freezing_circle.bmp"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`: until it is ended) and the
/// sound it starts with (none has one); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    matches!(effect, 200 | 976 | 977).then_some((99_999_999, ""))
}

/// One tick of an effect of this file: each launches what it shows at its first.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick != 0 {
        return;
    }
    match effect {
        200 => prims.extend([glows(course), swirls(course)]),
        976 => prims.push(orbs("whitelight.tga", 1.8, [80.0, 80.0, 255.0], (false, Part::OVER_BODIES), course)),
        977 => prims.push(orbs("freezing_circle.bmp", 0.55, [255.0; 3], (true, Part::UNDER_BODIES), course)),
        _ => {}
    }
}

/// `CLevel99Effect::Open`: two squares of light, white (`Part::level99_glow`).
fn glows(course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Level99Glow, "pikapika2.bmp", true, course);
    prim.parts.extend((0..2).map(Part::level99_glow));
    prim
}

/// `CLevel99Effect::AddRings`: three bands in the blue (100, 100, 255) (`Part::level99_swirl`).
fn swirls(course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Level99Swirl, "ring_blue.tga", true, course);
    prim.colour = [100.0, 100.0, 255.0];
    prim.parts.extend((0..3).map(Part::level99_swirl));
    prim
}

/// `CLevel99Orb1Effect::Open` and `CLevel99Orb2Effect::Open`: sixteen lights that rise 0.15 a tick, each
/// begun up to 99 under the feet with its turns by lot, showing nothing until it comes up. Those of the
/// second effect are turned over (`over`); `flags` is what the particles are made with (`CreateParticle`).
fn orbs(picture: &'static str, size: f32, colour: [f32; 3], (over, flags): (bool, i8), course: &Course) -> Prim {
    let mut prim = of_parts(Tei::Level99Orb, picture, true, course);
    prim.colour = colour;
    prim.parts.extend((0..16).map(|_| {
        let mut orb = part();
        (orb.tall, orb.out, orb.flags[0], orb.flags[1]) = (0.15, size, i8::from(over), flags);
        orb.begin_as_orb();
        orb
    }));
    prim
}
