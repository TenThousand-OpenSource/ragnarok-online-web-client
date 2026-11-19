//! `EF_SOULLINK` (503): the link of a soul, as its arm of `CRagEffect::OnProcess` and the functions it calls
//! have it:
//!
//! - a light that flies in a bow over the one linked, from the left of the screen to the right, faster and
//!   faster (`TexturedOrbit`; `PP_SOULLINK`);
//! - the sparks it leaves behind. The client's light launches another effect on the one linked at every
//!   frame it is drawn, for its first 180 ticks (`RenderSoulLink`: `EF_SOULLIGHT`, which is `TexturedStorm2`
//!   and lives 100 ticks; `PP_SOULPARTICLE`); here the sparks are launched by this effect, once a tick;
//! - the letters S, O, U, L and then L, I, N, K, flashing up one after the other over the one linked
//!   (`SoulLink`; `PP_ASURACASTING`).
//!
//! The client launches it on the one linked and tells it who that is (`CGameMode`, the handler of the soul
//! linker's skills), together with `EF_ASURABODY`, which is another effect.

use super::launched::{soul_light};
use super::*;
use crate::prim::{Part, Tei};

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&PICTURES, &SOUL, &LINK], sprites: &[] };

/// The pictures the effect is drawn with, among the client's pictures of effects: the light, and the letters
/// of the two words in the order the client names them, the last letter first.
const PICTURES: [&str; 1] = ["whitelight.tga"];
const SOUL: [&str; 4] = ["soul_l.tga", "soul_u.tga", "soul_o.tga", "soul_s.tga"];
const LINK: [&str; 4] = ["soul_k.tga", "soul_n.tga", "soul_i.tga", "soul_l.tga"];

/// How many ticks the effect lives (the client's `EffectLifetime`) and the sound it starts with; nothing for
/// any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        503 => (500, "effect/t_캐스팅.wav"),
        _ => return None,
    })
}

/// One tick of the effect; nothing for any other effect.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if effect != 503 {
        return;
    }
    match course.tick {
        0 => prims.push(orbit(course)),
        // The letters of the first word are 4, 10, 16 and 22 to the left of the middle and begin 60 ticks
        // late, those of the second as far to the right and 120 late.
        1 => prims.push(letters(&SOUL, -4.0, -60, course)),
        21 => prims.push(letters(&LINK, 22.0, -120, course)),
        145 => course.sound("effect/t_영혼.wav"),
        _ => {}
    }
    // Where the light is across the screen and over the ground, while it leaves sparks.
    let light = prims.iter().find(|prim| prim.kind == Shape::Tei(Tei::SoulLink)).and_then(|prim| prim.parts.first()).filter(|light| light.process > 0 && light.process < 180).map(|light| light.at[2]);
    if let Some([across, down, _]) = light {
        prims.push(soul_light([across * cosine(course.yaw), down, -across * sine(course.yaw)], course));
    }
}

/// `CRagEffect::TexturedOrbit`: the light, a square of `whitelight.tga` 2.5 from its middle to a corner
/// (`PP_SOULLINK`), that begins 30 to the left of the one linked and 40 over the ground.
fn orbit(course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::SoulLink, "whitelight.tga", true, course);
    let mut part = Part { out: 2.5, ..Part::default() };
    part.heights[..4].copy_from_slice(&[lot(360), lot(360), lot(360), lot(360)]);
    part.at[2] = [-30.0, -40.0, 0.0];
    prim.parts.push(part);
    prim
}

/// `CRagEffect::SoulLink`, one of its two words: four letters (`PP_ASURACASTING`, its mode 2), each 5 from
/// its middle to a corner, the first `from` to the right of the middle and each next 6 further left, each
/// beginning 20 ticks after the one before, `waits` being the count the first begins at.
fn letters(word: &'static [&'static str; 4], from: f32, waits: i32, course: &Course) -> Prim {
    let mut prim = launched::of_parts(Tei::AsuraCasting, word[0], true, course);
    prim.pictures = word;
    prim.parts.extend((0..4).map(|index| {
        let mut part = Part { process: index * 20 + waits, out: 5.0, ..Part::default() };
        (part.heights[0], part.flags[0], part.flags[4]) = (from - (index * 6) as f32, index as i8, 2);
        part
    }));
    prim
}
