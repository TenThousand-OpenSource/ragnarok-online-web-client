//! `PP_WIND` (62), `PP_SPHEREWIND` (77) and `PP_SPHEREWIND2` (91): bands of 21 pieces that go round the place
//! of the effect, each drawn in the colour its primitive's `size` stands for (`Rect::piece`).

use super::*;

impl Part {
    /// `PrimWind`, for one part: it goes round by 5 degrees a tick and grows 3 degrees longer a tick.
    /// `alpha_top` says which wind it is. Of the kind 2 it grows to half a turn, comes up by 1 a tick for its
    /// first eleven ticks, goes down by 1 a tick after its fiftieth, and from its thirteenth on stands 0.1
    /// further out a tick. Any other grows to a third of a turn and comes up by 10 a tick for its first eleven
    /// ticks (to 250 at most); of the kind 1 it goes down by 2 a tick after its twentieth, standing 0.1
    /// further out a tick. After 1400 ticks every kind but the third goes down by 3 a tick.
    pub(super) fn blow_as_wind(&mut self) {
        if self.waits() {
            return;
        }
        self.turned += 5;
        self.turned = under_a_round(self.turned);
        self.round += 3;
        if self.alpha_top == 2 {
            self.round = self.round.min(180);
            if self.process > 50 {
                self.alpha = (self.alpha - 1).max(0);
            }
            if self.process > 12 {
                self.out += 0.1;
            }
        } else {
            self.round = self.round.min(120);
            if self.alpha_top == 1 && self.process > 20 {
                self.alpha = (self.alpha - 2).max(0);
                self.out += 0.1;
            }
        }
        if self.process > 1400 {
            if self.alpha_top != 3 {
                self.alpha -= 3;
            }
            self.alpha = self.alpha.max(0);
        }
        if self.process < 12 {
            self.alpha = if self.alpha_top == 2 { self.alpha + 1 } else { (self.alpha + 10).min(250) };
        }
    }

    /// `RenderRiseArc`, for one part: a band along the arc it shows, from `turned` on for `round` degrees in
    /// steps of a 21st of that, `out` from the effect's place and `tall` over it. Each edge of it rises by its
    /// own height (`heights`, the 21st for every edge past the 21st), leaning outwards as `rise` says (90 is
    /// upright). A piece shows its 21st of the picture; the pieces past the twentieth show the twentieth's.
    pub(super) fn rising_arc(&self, picture: &'static str, size: f32) -> Vec<Rect> {
        let step = self.round / 21;
        if step <= 0 || self.alpha <= 0 {
            return Vec::new();
        }
        let edge = |(order, offset): (i32, i32)| {
            let angle = if self.round == 360 && order == 20 { self.turned } else { (self.turned + offset) % 360 } as f32;
            let (cos, sin) = (cosine(angle), sine(angle));
            let foot = [self.out * cos, -self.tall, self.out * sin];
            let height = self.heights[order as usize];
            let lean = height * cosine(self.rise as f32);
            (foot, [lean * cos + foot[0], -(self.tall + height * sine(self.rise as f32)), lean * sin + foot[2]])
        };
        Rect::band(picture, self.alpha, size, 20, edges(self.round, step, false, 20).map(edge))
    }

    /// `PP_SPHEREWIND` in `CEffectPrim::OnProcess`, for one part: it goes round by its own speed
    /// (`heights[1]`, whole degrees a tick).
    pub(super) fn go_round(&mut self) {
        self.process += 1;
        self.turned = (self.turned as f32 + self.heights[1]) as i32;
        self.turned = under_a_round(self.turned);
    }

    /// `PrimSphereWind2`, for one part: it comes up by 2 a tick for 25 ticks and goes down by 2 a tick after
    /// the seventieth; it stands a hundredth further out every tick; its ring tips by a degree a tick, and by
    /// one more at every other tick, drawn by lot; and it goes round by its own speed.
    pub(super) fn go_round_widening(&mut self) {
        if self.waits() {
            return;
        }
        if self.process <= 25 {
            self.alpha += 2;
        } else if self.process > 70 {
            self.alpha -= 2;
        }
        self.alpha = self.alpha.max(0);
        self.out *= 1.01;
        self.rise += 1;
        if self.lot(2) == 0 {
            self.rise += 1;
        }
        self.rise = under_a_round(self.rise);
        self.turned = (self.turned as f32 + self.heights[1]) as i32;
        self.turned = under_a_round(self.turned);
    }

    /// `RenderSphereWind`, for one part: a band along the arc it shows of a ring `out` wide about a place
    /// `heights[0]` under the effect's (so over it for a height under nothing), from `turned` on for `round`
    /// degrees in twenty steps. The ring is tipped by `rise` about the line north, and the band is `tall` to
    /// each side of it, across the ring's own flat.
    pub(super) fn tipped_band(&self, picture: &'static str, size: f32) -> Vec<Rect> {
        let step = self.round / 20;
        if step <= 0 || self.alpha <= 0 {
            return Vec::new();
        }
        let (tip_cos, tip_sin) = (cosine(self.rise as f32), sine(self.rise as f32));
        let side = [-(self.tall * tip_sin), self.tall * tip_cos, 0.0];
        let edge = |(order, count): (i32, i32)| {
            let angle = self.turned + count;
            let angle = if self.round == 360 && order == 20 {
                self.turned
            } else if angle >= 360 {
                angle - 360
            } else if angle < 0 {
                angle + 360
            } else {
                angle
            } as f32;
            let middle = [cosine(angle) * tip_cos * self.out, cosine(angle) * tip_sin * self.out + self.heights[0], sine(angle) * self.out];
            let (upper, lower): ([f32; 3], [f32; 3]) = (std::array::from_fn(|axis| middle[axis] + side[axis]), std::array::from_fn(|axis| middle[axis] - side[axis]));
            (upper, lower)
        };
        Rect::band(picture, self.alpha, size, 20, edges(self.round, step, true, 20).map(edge))
    }
}
