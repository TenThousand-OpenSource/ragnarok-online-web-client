//! `PP_3DQUADHORN` (9): a spike of four sides, a picture on each, that is pushed along its own forward: the
//! ice of Frost Diver and its like.
//!
//! Not ported, as no effect ported uses them: the change of its speed at a tick (`changePoint`), its second
//! pattern (`pattern & 2`: a new speed 30 ticks before its end, and no fading), and the spike drawn in a
//! colour that flickers, its light added (`slots[0].flag1[0] == 3`, which `FrostDiver(3)` asks for).

use super::solid::add;
use super::*;

impl Prim {
    /// `AdvanceEffectPrimitive`: it goes along its own forward by its speed, which changes by its acceleration;
    /// with the first pattern it stops for good at the tick it was told (`chCnt`).
    pub(super) fn advance(&mut self) {
        self.speed += self.accel;
        self.go_forward();
        if self.first_pattern && self.settles_at == Some(self.count) {
            (self.accel, self.speed) = (0.0, 0.0);
        }
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
