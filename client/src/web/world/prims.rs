//! The effects the client draws by code, out of its primitives, under way in the world.
//!
//! In the client such an effect (`CRagEffect`) is a function run every tick of its life, which launches
//! primitives (`CEffectPrim`) and sets their figures; each tick a primitive moves by its speeds and is drawn by
//! its kind. The primitives themselves are in `crate::prim`; `prims/effects.rs` has the effects made of them;
//! this file runs an effect on someone and hands its triangles to the frame.
//!
//! The client measures the flat kinds in pixels of its screen. Here a pixel of them is a pixel of a sprite, as
//! for the effects played from files: they keep their size beside the characters when the camera moves.

use super::*;
use crate::prim::{Prim, Shape, cosine, sine};

mod effects;

pub(super) use effects::PICTURES;

/// A whole number from 0 up to, not reaching, `under` (the client's `rand() % under`).
fn lot(under: u32) -> f32 {
    (js_sys::Math::random() * f64::from(under)).floor() as f32
}

/// An effect of the client's under way on someone.
pub(super) struct Burst {
    /// On whom: no one named is the player's own character.
    pub(super) on: Option<u32>,
    /// When it begins, and how many of its ticks have been lived through.
    pub(super) begins: f64,
    ticks: i32,
    /// Which effect it is, by the client's number, and how many ticks it lives (the client's `EffectLifetime`).
    effect: u32,
    lasts: i32,
    prims: Vec<Prim>,
    /// The way the one it is on faced as it began, in the client's degrees (0 south, 90 east).
    facing: f32,
    faced: bool,
}

/// The effect of that number of the client's, to begin at `begins` on someone, and the sound it starts with (none:
/// empty); nothing for an effect not ported yet.
pub(super) fn burst(effect: u32, on: Option<u32>, begins: f64) -> Option<(Burst, &'static str)> {
    let (lasts, sound) = effects::known(effect)?;
    Some((Burst { on, begins, ticks: 0, effect, lasts, prims: Vec::new(), facing: 0.0, faced: false }, sound))
}

impl Burst {
    /// Lives up to `time`, a tick of the client's at a time. Gives back whether anything of it is left.
    pub(super) fn live(&mut self, time: f64) -> bool {
        let due = ((time - self.begins) / CAST_FRAME_MS) as i32;
        while self.ticks < due && self.ticks < self.lasts {
            // The effect's own tick first: it may launch primitives, which live from this tick on.
            effects::run(self.effect, self.ticks, self.lasts, self.facing, &mut self.prims);
            let left = self.lasts - self.ticks;
            self.prims.retain_mut(|prim| prim.tick(left));
            self.ticks += 1;
        }
        self.ticks < self.lasts
    }

    /// Tells it which way the one it is on faces (a step of one on the ground), before its first tick: the
    /// client's effect looks at its parent's angle as it starts.
    pub(super) fn face(&mut self, facing: (f32, f32)) {
        if !self.faced {
            (self.facing, self.faced) = (facing.0.atan2(-facing.1).to_degrees(), true);
        }
    }

    /// Its triangles now, two at a time: around `place`, the feet of the one it is on; the flat kinds across
    /// the screen along `right` and `up`. `draw` is given the picture, each of the six corners as its place in the world and where
    /// of the picture it shows, and the colour.
    pub(super) fn draw(&self, place: Vector, right: Vector, up: Vector, mut draw: impl FnMut(&'static str, [(Vector, [f32; 2]); 6], [f32; 4])) {
        for prim in self.prims.iter().filter(|prim| prim.alpha >= 1.0) {
            let colour = [1.0, 1.0, 1.0, prim.alpha.trunc() / 255.0];
            for pair in prim.band(place).chunks_exact(6).filter(|_| prim.kind == Shape::Cylinder3d) {
                draw(prim.picture, std::array::from_fn(|index| pair[index]), colour);
            }
            for pair in prim.corners().chunks_exact(6) {
                let corner = |index: usize| (along(along(place, right, pair[index][0] * SPRITE_SCALE), up, -pair[index][1] * SPRITE_SCALE), [pair[index][2], pair[index][3]]);
                draw(prim.picture, std::array::from_fn(corner), colour);
            }
        }
    }
}
