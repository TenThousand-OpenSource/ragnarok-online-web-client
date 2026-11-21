//! `PP_3DQUADHORN` (9): a spike of four sides, a picture on each, that is pushed along its own forward: the
//! ice of Frost Diver and its like; and `PP_3DQUADHORN2` (117), the same spike going up and down over and
//! over (`Spike::Pumps`).
//!
//! Not ported, as no effect ported uses it: the spike drawn in a colour that flickers, its light added
//! (`slots[0].flag1[0] == 3`, which `FrostDiver(3)` asks for). (Its change of speed at a tick,
//! `changePoint`, is in `prim/change.rs`.)

use super::solid::add;
use super::*;

/// What a spike does besides being pushed along its forward.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spike {
    /// Nothing more.
    Plain,
    /// Its second pattern (`pattern & 2`): 30 ticks before its end it takes this speed and this acceleration
    /// (`changeSpeed3d.y`, `changeAccel3d.y`), and it never fades (the client leaves `ProcessAlpha` out).
    Withdraws(f32, f32),
    /// `PP_3DQUADHORN2`: until its 550th tick it takes this speed and this acceleration (`changeSpeed`,
    /// `changeAccel`) at every sixth tick from its third, and 1.18 and 0.01 at every sixth from its sixth; at
    /// its 550th it takes the first two for good. Neither its first pattern nor its `changePoint` is looked
    /// at.
    Pumps(f32, f32),
}

impl Prim {
    /// `AdvanceEffectPrimitive`: it goes along its own forward by its speed, which changes by its acceleration;
    /// with the first pattern it stops for good at the tick it was told (`chCnt`), and with the second it
    /// takes its last speed 30 ticks before its end. A spike that pumps is the kind 117's arm of
    /// `CEffectPrim::OnProcess`.
    pub(super) fn advance(&mut self) {
        if let Spike::Pumps(speed, accel) = self.spike {
            match (self.count, self.count % 6) {
                (550, _) | (..550, 2) => (self.speed, self.accel) = (speed, accel),
                (..550, 5) => (self.speed, self.accel) = (1.18, 0.01),
                _ => {}
            }
            self.speed += self.accel;
            self.go_forward();
            return;
        }
        self.speed += self.accel;
        self.go_forward();
        if self.first_pattern && self.settles_at == Some(self.count) {
            (self.accel, self.speed) = (0.0, 0.0);
        }
        if let (Spike::Withdraws(speed, accel), true) = (self.spike, self.count == self.duration - 30) {
            (self.speed, self.accel) = (speed, accel);
        }
    }

    /// Whether it fades as the other kinds do (`ProcessAlpha`): all but a spike of the second pattern.
    pub(super) fn spike_fades(&self) -> bool {
        !matches!(self.spike, Spike::Withdraws(..))
    }

    /// `RenderPyramid`: four triangles from the corners of a square of twice its size, lying across its own
    /// forward, to a point its height along that forward, turned as the primitive is. The picture goes round
    /// it a fifth to a side, its top at the point.
    pub(super) fn horn(&self, place: [f32; 3]) -> Vec<([f32; 3], [f32; 2])> {
        let middle = self.middle(place);
        let size = self.size;
        let corners = [[-size, -size], [size, -size], [-size, -size], [-size, size], [-size, size], [size, size], [size, size], [size, -size]];
        let point = |own: [f32; 3], u: f32, v: f32| (add(middle, self.turned(own)), [u, v]);
        let mut triangles = Vec::new();
        for (side, pair) in corners.chunks_exact(2).enumerate() {
            let u = side as f32 * 0.2;
            triangles.extend([
                point([pair[0][0], pair[0][1], 0.0], u, 1.0),
                point([0.0, 0.0, self.height], u, 0.0),
                point([pair[1][0], pair[1][1], 0.0], u + 0.2, 1.0)
            ]);
        }
        triangles
    }
}
