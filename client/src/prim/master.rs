//! What a primitive shares with the effect it belongs to (the client's `master`, a `CRagEffect`), and the
//! patterns of the kinds that read or write it:
//!
//! - how a crossed picture moves (`MoveEffectPrimitive`): one may tell its effect where it is
//!   (`pattern & 0x10`), stop on the ground and tell its effect so (`pattern & 4`), and run through several
//!   pictures;
//! - a sprite that ends on reaching the place its effect aims at (`pattern & 0x100`, `Prim3DParticle`).

use super::*;

/// The fields of the effect its primitives use: how many ticks it still has (its `duration` less its
/// `stateCnt`), the place it aims at, from its own (east, up, north: its `deltaPos`, the client's y turned
/// over), and whether a primitive of its has landed (its `flag`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Master {
    pub left: i32,
    pub aim: [f32; 3],
    pub landed: bool,
}

impl Master {
    /// An effect with so many ticks left that aims nowhere but at its own place.
    pub fn left(left: i32) -> Self {
        Self { left, aim: [0.0; 3], landed: false }
    }
}

impl Prim {
    /// `MoveEffectPrimitive`, what of it the crossed pictures here go through: it goes along its own forward by
    /// its speed, and its width and height change by theirs; a height past 100 (the client's `maxHeightSize`)
    /// stops growing. Its turn is the client's `transform`, which the effect sets and nothing changes; it is
    /// kept here as a latitude and a longitude. Gives back whether it lives on.
    ///
    /// At the tick of its change it takes the speeds it was given (`changePoint`: `Prim::take_given`). Not
    /// ported, as no effect ported uses them: speeds given along the three directions (`pattern & 0x100`,
    /// `0x400`), the turns that follow the camera (`0x40`, `0x200`), the picture that slides (`2`), the height
    /// that swings (`0x20`) and the colours that change.
    pub(super) fn cross_flight(&mut self, master: &mut Master, ground: &dyn Fn(f32, f32) -> f32) -> bool {
        self.take_given();
        self.width_speed += self.width_accel;
        self.speed += self.accel;
        self.height_speed += self.height_accel;
        self.go_forward();
        self.width += self.width_speed;
        self.height += self.height_speed;
        if self.height > 100.0 {
            (self.height_accel, self.height_speed) = (0.0, 0.0);
        }
        self.lead_and_land(master, ground);
        self.next_picture()
    }

    /// `MoveEffectPrimitive`, its `pattern & 0x10` and `pattern & 4`: one that leads writes where it is into
    /// its effect's aim; one that lands, once it is under the ground, lands no more, stands still, has 25
    /// ticks left to live and sets its effect's flag.
    pub(super) fn lead_and_land(&mut self, master: &mut Master, ground: &dyn Fn(f32, f32) -> f32) {
        let at = self.middle([0.0; 3]);
        if self.leads {
            master.aim = at;
        }
        if self.lands && at[1] < ground(at[0], at[2]) {
            self.lands = false;
            self.count = self.duration - 25;
            self.speed = 0.0;
            master.landed = true;
        }
    }

    /// `MoveEffectPrimitive`, its last step: every `anim_speed` ticks the next of its pictures comes. Gives
    /// back whether it lives on: one whose pictures do not start over ends after the last.
    pub(super) fn next_picture(&mut self) -> bool {
        if self.count % self.anim_speed != 0 {
            return true;
        }
        let total = self.pictures.len().max(1);
        self.motion += 1;
        if self.repeats {
            self.motion %= total;
            return true;
        }
        self.motion != total
    }

    /// The picture it shows now.
    pub fn shown(&self) -> &'static str {
        self.pictures.get(self.motion).copied().unwrap_or(self.picture)
    }

    /// `Prim3DParticle`, its `pattern & 0x100`: whether one that seeks is within 3 of the place its effect aims
    /// at, measured level (`CalcDist` of two figures), where it ends.
    pub(super) fn reached(&self, master: &Master) -> bool {
        let at = self.middle([0.0; 3]);
        self.seeks && f32::hypot(master.aim[0] - at[0], master.aim[2] - at[2]) <= 3.0
    }
}
