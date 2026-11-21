//! Pictures that lie level over the ground under the one an effect is on, each with the figures of its function
//! of `CRagEffect`:
//!
//! - `EF_LOCKON` (60): the mark that turns under the one looked at by Sense (`LockOnMarker`);
//! - `EF_TOPRANK` (159): the same mark, small, under one a script marks out, until it is ended (in
//!   `CRagEffect::OnProcess` itself);
//! - `EF_DETECT2` (725), `EF_DETECT3` (778), `EF_DETECT4` (780): a ring that widens over the ground
//!   (`TexturedSkillAura`);
//! - `EF_STASIS` (799), `EF_GROUNDSHAKE` (894), `EF_GROUNDSHAKE2` (905): squares of light over the ground
//!   that widen (`TexturedLevelRing`);
//! - `EF_FOOT3` (447), `EF_FOOT4` (448): the footprints of one who runs, the left and the right
//!   (`TexturedRise` with its kind 1; the kind 67 of the primitives). No skill shows them: a character whose
//!   body an effect has running leaves them where it is (`crate::body`, `world/ailments.rs`), each told how
//!   it is turned.
//!
//! The three rings stay where they began (`TexturedSkillAura` never takes its parent's place); the others
//! follow the one they are on. Beside its squares `EF_STASIS` flashes the one it is on blue from its sixth
//! tick to its 40th (`SetColor` on `state.parent`, `crate::body`).

use super::launched::{skill_aura, square};
use super::*;
use crate::prim::{Change, Cycle};

pub(super) const FILE: File = File { known, thrown: none, stays, run, pictures: &[&PICTURES, &THUNDER], sprites: &[] };

/// The pictures the effects here are drawn with, among the client's pictures of effects.
const PICTURES: [&str; 5] = ["explosive_1_128.bmp", "fashasha.tga", "lockon128.tga", "foot_l2.tga", "foot_r2.tga"];

/// The six pictures of the squares of `EF_STASIS`, the lowest first.
const THUNDER: [&str; 6] = [
    "thunder_ball_0001.bmp", "thunder_ball_0002.bmp", "thunder_ball_0003.bmp", "thunder_ball_0004.bmp",
    "thunder_ball_0005.bmp", "thunder_ball_0006.bmp"
];

/// How many ticks an effect of this file lives (the client's `EffectLifetime`) and the sound it starts with
/// (none: empty); nothing for any other effect.
fn known(effect: u32) -> Option<(i32, &'static str)> {
    Some(match effect {
        60 => (200, ""),
        // `EF_TOPRANK` puts its count back to nothing at its 9998th tick: it never ends by itself.
        159 => (99_999_999, ""),
        447 | 448 => (340, ""),
        725 => (250, "effect/priest_benedictio.wav"),
        778 => (250, ""),
        780 => (250, "effect/cru_grand cross.wav"),
        // `EF_STASIS` starts its sound at its seventh tick, in `stasis`.
        799 => (300, ""),
        894 | 905 => (200, ""),
        _ => return None,
    })
}

/// Whether an effect stays where it began: `TexturedSkillAura` never takes its parent's place.
fn stays(effect: u32) -> bool {
    matches!(effect, 725 | 778 | 780)
}

/// One tick of an effect of this file: it launches what it launches at that tick into `prims`.
fn run(effect: u32, course: &mut Course, prims: &mut Vec<Prim>) {
    let (tick, lasts) = (course.tick, course.lasts);
    match effect {
        60 if tick == 0 => prims.extend(lock_on(lasts)),
        159 if tick % 9998 == 0 => prims.push(top_rank(course.count)),
        447 if tick == 0 => prims.push(footprint("foot_l2.tga", course)),
        448 if tick == 0 => prims.push(footprint("foot_r2.tga", course)),
        725 | 778 | 780 if tick == 0 => prims.push(skill_aura(match effect {
            725 => 2,
            778 => 3,
            _ => 4,
        })),
        799 => stasis(course, prims),
        894 if tick == 0 => prims.push(square("explosive_1_128.bmp", 4, 0, lasts)),
        905 if tick == 0 => prims.push(square("explosive_1_128.bmp", 10, 0, lasts)),
        _ => {}
    }
}

/// `CRagEffect::LockOnMarker`: the mark `LockOn128.tga` twice, an eighth of a turn apart, lying 2 above the
/// ground under the one looked at and turning about the upright by 4.5 degrees a tick (its `longSpeed`); each
/// 30 to a side at first, closing in by 1 a tick for twenty ticks. Its colour goes round (`pattern & 0x800`):
/// from a pale red its green and blue fall by 15 a tick, and at their most they wait twenty ticks.
fn lock_on(lasts: i32) -> [Prim; 2] {
    [0.0, 45.0].map(|turned| {
        let mut mark = Prim::new(Shape::Texture3d, "lockon128.tga");
        mark.duration = lasts;
        (mark.longitude, mark.long_speed, mark.roll) = (turned, 4.5, 90.0);
        mark.put.1 = -2.0;
        (mark.width, mark.height, mark.width_speed, mark.height_speed) = (30.0, 30.0, -1.0, -1.0);
        mark.changes = Some(Change { width_speed: 0.0, height_speed: 0.0, width_accel: 0.0, height_accel: 0.0, ..Change::at(20) });
        mark.colour = [250.0, 150.0, 150.0];
        mark.cycle = Some(Cycle { speeds: [0.0, -15.0, -15.0], delay: 20, waited: 0 });
        mark
    })
}

/// `EF_TOPRANK` in `CRagEffect::OnProcess`: the mark `LockOn128.tga` once, 4 to a side, lying 2 above the
/// ground and turning about the upright by 4.5 degrees a tick, for 9999 ticks; the effect launches it anew
/// every 9998. Its colour is by the count the effect was told (`AM_SETCOUNT`): green for 11, else red with
/// 25 less of green and blue for each one of the count (none told: all but white).
fn top_rank(count: i32) -> Prim {
    let mut mark = Prim::new(Shape::Texture3d, "lockon128.tga");
    mark.duration = 9999;
    (mark.long_speed, mark.roll, mark.put.1) = (4.5, 90.0, -2.0);
    (mark.width, mark.height) = (4.0, 4.0);
    // (The client writes the figure into a byte.)
    let pale = f32::from(((10 - count) * 25) as u8);
    mark.colour = if count == 11 { [0.0, 250.0, 0.0] } else { [250.0, pale, pale] };
    mark
}

/// `EF_STASIS` in `CRagEffect::OnProcess`: six blue squares one after the other, the first ten ticks late and
/// each four ticks after the one before, each with its own picture of the ball of lightning; and at the
/// seventh tick the sound.
fn stasis(course: &mut Course, prims: &mut Vec<Prim>) {
    if course.tick == 6 {
        course.sound("effect/wl_stasis.wav");
    }
    if (6..=40).contains(&course.tick) {
        course.touch.flashes(course.tick, [0, 0, 255]);
    }
    if course.tick == 0 {
        prims.extend(THUNDER.into_iter().zip(0..).map(|(picture, index)| square(picture, 3, 10 + index * 4, course.lasts)));
    }
}

/// `CRagEffect::TexturedRise` with its kind 1: a footprint, a level square of that picture a touch over the
/// ground (the kind 67), 6 across, its first corner turned to the degrees the effect was told (`deltaPos`'s
/// first figure), that begins at 150 and fades; drawn twice, its light added (a `size` of 100).
fn footprint(picture: &'static str, course: &Course) -> Prim {
    let mut print = launched::of_parts(crate::prim::Tei::Footprint, picture, true, course);
    print.size = 100.0;
    let mut part = crate::prim::Part { out: 6.0, alpha: 150, turned: (course.shared.aim[0] as i32).rem_euclid(360), ..Default::default() };
    part.heights[0] = -0.2;
    print.parts.push(part);
    print
}
