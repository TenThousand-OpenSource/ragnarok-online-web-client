//! A ring of eight sprites that spreads over the ground from round the one struck, each the same frames of
//! fire playing once (`CRagEffect::SpriteRing`; the kind 167 of the client's primitives, a sprite drawn four
//! times, `crate::prim::Shape::Fourfold`):
//!
//! - `EF_SPR_LIGHTPRINT` (901): an elemental's Fire Wave, in the sprite's own colours with a red rim
//!   (`SpriteRing` with its kind 9);
//! - `EF_SPR_LIGHTPRINT2` (923): Poison Buster, the same in blue (its kind 10); the sound is what the client
//!   starts with it (`CSession::PlaySkillEffect`).
//!
//! - `EF_SPR_LIGHTPRINT3` (941): an elemental's Water Screw, the same function with its kind 20 and another
//!   sprite (`m_ef05`). The client's drawing function has no colour for the number that kind gives the
//!   parts (`RenderSegmentSprites`, `flag1[0] == 4`: it writes none before it draws), so what the client
//!   shows cannot be told from its code. Here it is drawn in the sprite's own colours, as the first of the
//!   three is: that colour is this project's assumption, not the client's.

use super::*;
use crate::prim::Part;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[], sprites: &[&SPRITES] };

/// The sprite of the ring, among the client's sprites.
const FIRE: &str = "이팩트/m_ef03";
/// The sprite of Water Screw's ring.
const WATER: &str = "이팩트/m_ef05";
const SPRITES: [&str; 2] = [FIRE, WATER];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        901 => (500, ""),
        923 => (500, "effect/s포이즌버스터.wav"),
        941 => (500, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file: at each of its first two ticks, four of the eight sprites.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (look, sprite) = match effect {
        901 => (0, FIRE),
        923 => (1, FIRE),
        941 => (4, WATER),
        _ => return,
    };
    if course.tick <= 1 {
        prims.push(ring(look, sprite, course));
    }
}

/// `CRagEffect::SpriteRing`: one primitive of four parts, an eighth of a round from each other (those of the
/// second tick the other half of the round), each 5 out from the one the effect is on and 5 over the ground,
/// sliding outwards 0.3 a tick. The sprite's frames play once, one every fourth tick, and the primitive ends
/// with the last; its light is added, and each layer is in a colour made of its own by `look`
/// (`crate::prim::Rim`). It goes with the one the effect is on.
fn ring(look: i8, sprite: &'static str, course: &Course) -> Prim {
    let mut prim = Prim::new(Shape::Fourfold, sprite);
    (prim.duration, prim.adds, prim.own_colours, prim.anim_speed, prim.repeats) = (course.lasts, true, true, 4, false);
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { turned: (course.tick * 4 + index) * 45, ..Part::default() };
        let turned = part.turned as f32;
        (part.heights[0], part.flags[0]) = (0.3, look);
        part.at[0] = [cosine(turned) * 5.0, -5.0, sine(turned) * 5.0];
        part
    }));
    prim
}
