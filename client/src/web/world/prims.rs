//! The effects the client draws by code, out of its primitives, under way in the world.
//!
//! In the client such an effect (`CRagEffect`) is a function run every tick of its life, which launches
//! primitives (`CEffectPrim`) and sets their figures; each tick a primitive moves by its speeds and is drawn by
//! its kind. The primitives themselves are in `crate::prim`; the effects made of them are in the files of
//! `prims/`, each named for what its effects are and listed once, in `files!` below: a file answers for its
//! effects through its `FILE`, and nothing else names a file. An effect under way on someone is a `Burst`
//! (`prims/burst.rs`), what it remembers its `Course` (`prims/course.rs`), and what it hands to the frame its
//! `Piece`s (`prims/drawn.rs`); what several effects launch alike is `prims/launched.rs`.
//!
//! An effect launched for a blow is told more than whom it is on: a place, mostly that of the one struck (the
//! client's `AM_SETPOS`) and of how many hits the blow is (`AM_SETCOUNT`); `Blow` (`world/launch.rs`) is that, and `Course`
//! what the effect keeps of it and remembers from tick to tick.
//!
//! The client measures the flat kinds in pixels of its screen. Here a pixel of them is a pixel of a sprite, as
//! for the effects played from files: they keep their size beside the characters when the camera moves. A
//! sprite of an effect the client draws as large as the sprites of those standing at that depth
//! (`RenderSegmentSprites2`, `CRenderer::ScaleDistance`), and so it is here; it draws the layers of the
//! sprite's frame one by one, each in the primitive's colour alone, and so does the frame here
//! (`crate::sprite::layer_quads`, `gpu/layers.rs`).

use super::*;
use crate::prim::{Master, Prim, Shape, Slot, cosine, faced, heading, pointing, sine};

/// The files of effects, each a module with a `FILE`: the one list of them. A new file is a name here.
macro_rules! files {
    ($($file:ident),*) => {
        $(mod $file;)*
        const FILES: &[&File] = &[$(&$file::FILE),*];
    };
}
files!(
    auras,
    blasts,
    blows,
    bodies,
    bolts,
    bursts,
    falls,
    grounds,
    gusts,
    heals,
    held,
    hits,
    links,
    marks,
    max_level,
    missiles,
    motes,
    pillars,
    places,
    played,
    prints,
    puffs,
    rays,
    shields,
    songs,
    souls,
    sounds,
    spirits,
    sprays,
    sprites,
    struck,
    trails,
    walls,
    waves,
    weather,
    winds
);
mod launched;
mod course;
mod burst;
mod drawn;

pub(super) use burst::{Burst, begin, burst};
pub(super) use course::{Asked, Course};
pub(super) use drawn::Piece;

/// What a file of effects answers for the effects it has.
struct File {
    /// How many ticks an effect lives (the client's `EffectLifetime`, or what its function makes of it) and
    /// the sound it starts with (none: empty); nothing for an effect that is not the file's. These functions
    /// are named `known` in every file: `tools/client-tables/skill_effects.py` reads the numbers they name to
    /// tell which effects are drawn here.
    known: fn(u32) -> Option<(i32, &'static str)>,
    /// Whether an effect is one that is told a place: one the client has the striker launch and then tells
    /// the place of the one struck (`CSession::PlaySkillEffect`: `source->LaunchEffect` and `AM_SETPOS`).
    thrown: fn(u32) -> bool,
    /// Whether an effect stays where it began: one whose function does not take its parent's place every
    /// tick. (Asked only of the effects for which it matters here.)
    stays: fn(u32) -> bool,
    /// One tick of an effect, as its course says: it launches what it launches at that tick.
    run: fn(u32, &mut Course, &mut Vec<Prim>),
    /// The pictures its effects are drawn with, among the client's pictures of effects, and the sprites they
    /// show, among the client's sprites: lists of the file's.
    pictures: &'static [&'static [&'static str]],
    sprites: &'static [&'static [&'static str]],
}

/// What a file answers whose effects are none of them thrown, or none of them stay.
fn none(_effect: u32) -> bool {
    false
}

/// Whether an effect that is told a place is one the client launches on the one struck and tells the place of
/// the striker (`target->LaunchEffect` and `AM_SETPOS` with `source->render.position`, in its handlers of
/// skills): `EF_BLOODDRAIN`, `EF_HITLINE5`, `EF_DARKPIERCING`, `EF_BLOODDRAIN2`. (Asked only of a blow's
/// effects: one of a skill that strikes nobody is told the other of the two whichever it is on.)
fn aims_back(effect: u32) -> bool {
    matches!(effect, 216 | 430 | 1030 | 1052)
}

/// The file an effect is of, how many ticks it lives and the sound it starts with; nothing for an effect not
/// ported yet.
fn file_of(effect: u32) -> Option<(&'static File, i32, &'static str)> {
    FILES.iter().find_map(|file| (file.known)(effect).map(|(lasts, sound)| (*file, lasts, sound)))
}

/// How many ticks an effect lives (the client's `EffectLifetime`); none for one not ported.
pub(super) fn lasts(effect: u32) -> i32 {
    file_of(effect).map_or(0, |(_, lasts, _)| lasts)
}

/// The pictures the effects are drawn with and the sprites they show: what the world reads ahead of their use
/// (`world/loading.rs`).
pub(super) fn pictures() -> impl Iterator<Item = &'static str> {
    FILES.iter().flat_map(|file| file.pictures.iter().flat_map(|list| list.iter().copied()))
}

pub(super) fn sprites() -> impl Iterator<Item = &'static str> {
    FILES.iter().flat_map(|file| file.sprites.iter().flat_map(|list| list.iter().copied()))
}

/// A whole number from 0 up to, not reaching, `under` (the client's `rand() % under`), as the figures of a
/// primitive take it.
fn lot(under: u32) -> f32 {
    super::lot(under) as f32
}
