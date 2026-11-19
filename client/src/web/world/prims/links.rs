//! Effects that link the one they are on to another actor with a tube of light that follows both
//! (`PP_LINELINK`, `crate::prim::Tei::LineLink`). The client tells such an effect the other actor's ID
//! (`AM_SETPOS` with the ID where a height would be), and its primitive looks that actor up every tick; here
//! the effect is told whom it aims at as any thrown effect is, and the world tells it where that one is now
//! (`Course::other`):
//!
//! - `EF_LINELINK` (232): the threads of Devotion, from the crusader to each of those guarded (`TrackCloud`
//!   with its kind of 0, as Soul Unity's), until the server lists the wards anew (`ReceiveDevotionList`:
//!   `app/incoming/fight.rs`);
//! - `EF_LINELINK2` (384): Soul Change, from the one who uses it to the other (`TrackTarget`);
//! - `EF_LINELINK3` (395): the strings of Marionette Control, from the master to the puppet, for as long as
//!   the state holds (`TrackTargetWide`; ended by `crate::state_effects`);
//! - `EF_LINELINK5` (1210): Soul Unity, from the one it is used on to the one who used it (`TrackCloud` with
//!   its kind of 0);
//! - `EF_LINELINK4` (819): Shadow Form, a dark tube from the one who uses it to the one it is used on
//!   (`TrackCloud` with its kind of 2), until the state ends (`crate::state_effects`); the sound is what the
//!   client starts with it (its handler of `SC_SHADOWFORM`).

use super::launched::{of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown, stays: none, run, pictures: &[&PICTURES], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 2] = ["alpha_center.tga", "cloud11.tga"];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        384 => (200, "effect/소울 체인지.wav"),
        395 => (99_999_999, "effect/소울 체인지.wav"),
        819 => (99_999, "effect/shadowform.wav"),
        1210 => (200, ""),
        232 => (99_999, ""),
        _ => return None,
    })
}

/// Every effect of this file is told another actor (the client's handlers of `PF_SOULCHANGE`,
/// `CG_MARIONETTE` and `SP_SOULUNITY`, `Client/Mode/GameModePackets.cpp`).
fn thrown(effect: u32) -> bool {
    known(effect).is_some()
}

/// One tick of an effect of this file: at its first it launches its tube.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick != 0 {
        return;
    }
    // The picture, how thick the tube is, which of the primitive's kinds it is, and its alpha to begin with.
    let (picture, thick, kind, alpha) = match effect {
        384 => ("alpha_center.tga", 0.2, 2.0, 0),
        395 => ("alpha_center.tga", 0.7, 3.0, 0),
        819 => ("cloud11.tga", 0.4, 4.0, 45),
        _ => ("cloud11.tga", 0.4, 0.0, 45),
    };
    let mut tube = of_parts(Tei::LineLink, picture, true, course);
    let mut link = part();
    (link.tall, link.alpha) = (thick, alpha);
    link.heights[4] = kind;
    tube.parts.push(link);
    prims.push(tube);
}
