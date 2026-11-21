//! The primitives that show a sprite: `PP_3DPARTICLE` (5), `PP_3DPARTICLEGRAVITY` (6), `PP_3DPARTICLEORBIT`
//! (7) and `PP_3DPARTICLESPLINE` (8). Each is a frame of the
//! first action of a sprite at a place in the world, drawn facing the screen (`RenderSegmentSprites2`), once
//! for where it is and once more for each place it was the ticks before, smaller and fainter (`ShiftSegment`).
//!
//! Of `Prim3DParticle` the plain flight is here; its end on reaching the place the effect aims at
//! (`pattern & 0x100`) is in `master.rs`, and its turn against the camera's (`0x20`) is `Prim::rolled`
//! (`facing.rs`). How the layers of its sprite's frame are drawn is `crate::sprite::layer_quads`. Not ported, as none
//! of the effects ported uses them: its wandering (`pattern & 2`, `4`,
//! `8`), its speeds given along the three directions (`0x400`), the change of its speeds at a tick
//! (`changePoint`), the wait before it moves (`fadeInCnt`), and a size that changes faster and faster
//! (`sizeAccel`). Of `AnimMotion`, staying on the last frame (`pattern & 0x800`) is not ported either.

use super::*;

/// One drawing of a sprite primitive: where its sprite's origin is, from the effect's place (east, up, north),
/// how many times the sprite's own size it is, and its alpha of 255.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seen {
    pub at: [f32; 3],
    pub scale: f32,
    pub alpha: f32,
}

impl Prim {
    /// Whether it is of a kind that shows a sprite.
    pub fn shows_sprite(&self) -> bool {
        matches!(
            self.kind,
            Shape::Particle3d | Shape::ParticleGravity3d | Shape::ParticleOrbit3d | Shape::ParticleSpline3d
            | Shape::SpriteGlow | Shape::Kouenka | Shape::Fourfold | Shape::Roamer | Shape::Petal { .. }
            | Shape::Ghost
        )
    }

    /// Where a point so far along its own forward is, turned as it is, in the world's directions (east, up,
    /// north): what the effects put it at before it flies (the client's `RotatedForward` of its `transform`).
    pub fn forward(&self, far: f32) -> [f32; 3] {
        self.turned([0.0, 0.0, far])
    }

    /// `Prim3DParticle`: it goes along its own forward by its speed, which changes by its acceleration, and
    /// its size and its roll change by their speeds.
    pub(super) fn fly(&mut self) {
        self.speed += self.accel;
        let step = self.forward(self.speed);
        self.moved = [self.moved[0] + step[0], self.moved[1] + step[1], self.moved[2] + step[2]];
        self.size += self.size_speed;
        self.roll += self.roll_speed;
    }

    /// `Prim3DParticleGravity`: it goes level along its own forward by its speed, which never turns back, and
    /// falls by a speed of its own; falling under the ground it is thrown back up at two thirds of that speed.
    /// (The client plays the sound of a coin there for a primitive told to, `playWaveType`; none here is.)
    pub(super) fn fall(&mut self, ground: &dyn Fn(f32, f32) -> f32) {
        self.grav_speed += self.grav_accel;
        self.speed = (self.speed + self.accel).max(0.0);
        let step = self.forward(self.speed);
        self.moved = [self.moved[0] + step[0], self.moved[1] - self.grav_speed, self.moved[2] + step[2]];
        self.size += self.size_speed;
        let at = self.middle([0.0; 3]);
        if self.grav_speed > 0.0 && at[1] < ground(at[0], at[2]) {
            self.grav_speed /= -1.5;
        }
    }

    /// `Prim3DParticleOrbit`: it stands its radius out from where it was put, the way its longitude says about
    /// the upright, and goes round as that turns; where it was put goes down by a speed that changes
    /// (`gravSpeed`: up when under nothing); its radius, its size and its roll change by their speeds; at a
    /// tick it may be given (`chPoint`, here `changes`) its radius takes a new speed. (Not ported, as no
    /// effect ported uses them: the change of its falling at a tick, `changePoint`, and the turn of its own
    /// that the client adds for kinds drawn by it, `latiSpeed`.)
    pub(super) fn orbit(&mut self) {
        self.long_speed += self.long_accel;
        self.grav_speed += self.grav_accel;
        self.radius_speed += self.radius_accel;
        self.longitude += self.long_speed;
        self.put.1 += self.grav_speed;
        self.radius += self.radius_speed;
        self.roll += self.roll_speed;
        // (`chRadiSpeed`: at the tick it was given, its radius takes a new speed and a new change of it.)
        if let Some(change) = self.changes.filter(|change| change.at == self.count) {
            (self.radius_speed, self.radius_accel) = (change.speed, change.accel);
        }
        self.moved = [sine(self.longitude) * self.radius, 0.0, cosine(self.longitude) * self.radius];
        self.size += self.size_speed;
    }

    /// `Prim3DParticleSpline`: it flies level against the forward of its turn about the upright (its
    /// longitude) by its speed less a speed of its own that changes (`gravSpeed`: larger than its speed at
    /// first, so that it draws back before it flies), and stands out from that line by a length that swings out
    /// and back (`latitude`, by `latiSpeed`), up from the line and then turned about it by its roll (the
    /// client's `MakeTransform` about the forward as axis, and `Lift`).
    pub(super) fn bow(&mut self) {
        let swing = self.lati_speed;
        self.lati_speed += self.lati_accel;
        self.latitude += swing;
        self.speed += self.accel;
        self.grav_speed += self.grav_accel;
        let (east, north) = (sine(self.longitude), cosine(self.longitude));
        let far = self.speed - self.grav_speed;
        self.size += self.size_speed;
        self.moved = [self.moved[0] - east * far, self.moved[1], self.moved[2] - north * far];
        // The upright turned about the axis (east, 0, north) by the roll is, in the client's directions,
        // (-sin * north, cos, sin * east), its y pointing down.
        let (sin, cos) = (sine(self.roll), cosine(self.roll));
        self.aside = [-sin * north * self.latitude, -cos * self.latitude, sin * east * self.latitude];
    }

    /// `ShiftSegment` and `AnimMotion`: where it is now goes to the head of the places it has been, the oldest
    /// is forgotten; and every so many ticks the next frame of the sprite comes, of the `frames` its action has
    /// (0: not known yet, and the frame stays). Gives back whether it lives on: one whose frames do not start
    /// over ends after the last.
    ///
    /// Before its first tick the client has all its places at the effect's own (`InitSegment`, called before the
    /// primitive has been anywhere), so the older drawings of a trail begin there.
    pub(super) fn trail_and_frame(&mut self, frames: usize) -> bool {
        self.trail.resize(self.segments.max(1), [0.0; 3]);
        self.trail.rotate_right(1);
        self.trail[0] = self.middle([0.0; 3]);
        if frames == 0 || self.count % self.anim_speed != 0 {
            return true;
        }
        self.motion += 1;
        if self.repeats {
            self.motion %= frames;
            return true;
        }
        self.motion != frames
    }

    /// Every drawing of its sprite now, the newest first: where it is, at its size and alpha, and each place it
    /// was before, down to half its size and to nothing the older it is (`ShiftSegment`: the drawing `index` of
    /// `count` has the size less `size / count * index * 0.5` and the alpha less `alpha / count * index`).
    pub fn seen(&self) -> Vec<Seen> {
        let count = self.trail.len() as f32;
        let older = |index: usize| index as f32;
        self.trail
            .iter()
            .enumerate()
            .map(|(index, at)| Seen {
                at: *at,
                scale: self.size - self.size / count * older(index) * 0.5,
                alpha: self.alpha - self.alpha / count * older(index)
            })
            .collect()
    }
}
