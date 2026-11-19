//! What a state shows on the one in it for as long as it holds (`crate::state_effects` begins and ends
//! these): the signs of one in a fury, and the sign of one who is stopped. Each is its function of
//! `CRagEffect` with the same figures:
//!
//! - `EF_GUMGANG` (203): Fury, one set of four signs that flash up round the head, blue (`SuperSlashFan`
//!   with a count of 1; the kind 35);
//! - `EF_STEELBODY` (254): Steel Body, four sets of them (`SuperSlashFan` with 4);
//! - `EF_GUMGANGNPC` (455): a monster's Power Up, two sets in red (`SuperSlashFan` with 2 and its value 1);
//! - `EF_DOUBLEGUMGANG` (418): the Warmths, two sets of smaller signs, red and each drawn three times over
//!   (`SuperSlashSpread`; `PP_DOUBLEGUMGANG`);
//! - `EF_NPC_STOP` (453): the sign over one held by Stop, a sprite that plays over and over at that one's
//!   feet (`NpcStop` with its type 0; the kind 51).
//!
//! The signs stand as high as the name of the one they are on (`CGameActor::GetNameOffset`), which the world
//! tells the effect every frame (`Course::lift`).
//!
//! Beside `EF_GUMGANG` Fury launches `EF_MAKEBLUR`, which flickers the body pale red (`prims/bodies.rs`,
//! `crate::body::copies::blur`).

use super::launched::{of_parts, part};
use super::*;
use crate::prim::Tei;

pub(super) const FILE: File = File { known, thrown: none, stays: none, run, pictures: &[&SIGNS], sprites: &[&SPRITES] };

/// The five pictures a sign is one of, among the client's pictures of effects.
const SIGNS: [&str; 5] = ["super1.bmp", "super2.bmp", "super3.bmp", "super4.bmp", "super5.bmp"];

/// The sprite of Stop's sign, among the client's sprites.
const STOP: &str = "이팩트/스톱";
const SPRITES: [&str; 1] = [STOP];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`: until its state ends, for all
/// but the monster's) and the sound it starts with (none of them has one in its function); nothing for any
/// other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        203 | 453 => (99_999, ""),
        254 | 418 => (99_999_999, ""),
        455 => (1500, ""),
        _ => return None,
    })
}

/// One tick of an effect of this file: each launches what it shows at its first.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick != 0 {
        return;
    }
    prims.push(match effect {
        203 => signs(Tei::OrbitIcons, 1, 0, course),
        254 => signs(Tei::OrbitIcons, 4, 0, course),
        455 => signs(Tei::OrbitIcons, 2, 1, course),
        418 => signs(Tei::DoubleGumGang, 2, 2, course),
        453 => stop(course),
        _ => return,
    });
}

/// `CRagEffect::SuperSlashFan` (the kind 35) and `SuperSlashSpread` (`PP_DOUBLEGUMGANG`): `sets` parts of
/// four signs each, that go with the one the effect is on. Every sign begins at a count of 10, at an angle
/// and with one of the five pictures by lot, up to 9 over the level of that one's name, the four of a set
/// at alphas of 70, 130, 190 and 250, so that they go out and begin anew one after the other. Those of
/// `SuperSlashFan` are 3.5 from the middle to a corner; those of `SuperSlashSpread` 2, at alphas of 40, 80,
/// 120 and 160. `value` is the colour the primitive's drawing picks (`flags[4]`).
fn signs(kind: Tei, sets: usize, value: i8, course: &Course) -> Prim {
    let mut prim = of_parts(kind, SIGNS[0], true, course);
    prim.pictures = &SIGNS;
    let (reach, first, step) = if kind == Tei::DoubleGumGang { (2.0, 40.0, 40.0) } else { (3.5, 70.0, 60.0) };
    prim.parts.extend((0..sets).map(|_| {
        let mut set = part();
        (set.out, set.flags[4]) = (reach, value);
        // The four signs' places, in the order of their figures: `vecBPre`, `vecBNow`, `vecTPre`, `vecTNow`.
        for (number, place) in [2, 0, 3, 1].into_iter().enumerate() {
            set.at[place][1] = -lot(10);
            set.heights[number * 4..number * 4 + 4].copy_from_slice(&[10.0, first + step * number as f32, lot(5), lot(360)]);
        }
        set
    }));
    prim
}

/// `CRagEffect::NpcStop` with its type 0: the sprite of the sign at the feet of the one the effect is on, at
/// an alpha of 200, a frame every fourth tick, over and over, mixed in by its alpha. The kind 51 of the
/// client's primitives, as this effect launches it (its part told to follow the effect), is a sprite that
/// stays at its effect's place: what `PP_3DPARTICLE` is with no speed, which is what is launched here.
fn stop(course: &Course) -> Prim {
    let mut prim = Prim::new(Shape::Particle3d, STOP);
    (prim.duration, prim.alpha, prim.anim_speed) = (course.lasts, 200.0, 4);
    prim
}
