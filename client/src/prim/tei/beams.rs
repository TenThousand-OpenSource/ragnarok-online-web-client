//! The kinds 89 and 69 of the client's primitives: beams that open like a fan, and a streak in five sides.

use super::*;

impl Part {
    /// The kind 89 in `CEffectPrim::OnProcess`, for one part. `heights[4]` says which beam it is: of the kinds
    /// 2 and 6 it comes up by 10 a tick for ten ticks and goes down by 3 a tick after the eleventh; of the
    /// kind 3 by 6 and by 1; both reach 3 further a tick. Any other comes up by 20 a tick for ten ticks, goes
    /// down by 15 a tick after the twelfth, and reaches 1.15 times as far every tick.
    pub(super) fn open_as_beam(&mut self) {
        if self.waits() {
            return;
        }
        let kind = self.heights[4];
        let (rises, falls, from) = if kind == 2.0 || kind == 6.0 {
            (10, 3, 11)
        } else if kind == 3.0 {
            (6, 1, 11)
        } else {
            (20, 15, 12)
        };
        if self.process <= 10 {
            self.alpha += rises;
        } else if self.process > from {
            self.alpha = (self.alpha - falls).max(0);
        }
        self.out = if kind == 2.0 || kind == 3.0 || kind == 6.0 { self.out + 3.0 } else { self.out * 1.15 };
    }

    /// `RenderFanBeam`, for one part: two triangles one on the other, each from the middle (`heights[0]` under
    /// the effect's place, so over it for a height under nothing) out to `out`, between a few degrees before
    /// and after `rise` in a plane that `turned` tips: a narrow one (1 degree to each side for the kind 2, else
    /// 2) and a wide one (2, else 5). Each kind has its two colours; the wide one of the kinds 3, 4 and 5 is
    /// mixed in, all else is added.
    ///
    /// (The effects give a `rise` past a full turn to some parts; the client looks such an angle up in its
    /// table of cosines past the 360th entry, and what it finds there was not made out. Here it is the cosine
    /// of that angle.)
    pub(super) fn fan(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (kind, lift) = (self.heights[4], self.heights[0]);
        let (tip_cos, tip_sin) = (cosine(self.turned as f32), sine(self.turned as f32));
        let triangle = |spread: i32, colour: [u8; 3], adds: bool| {
            let edge = |angle: i32| {
                let along = self.out * cosine(angle as f32);
                [tip_cos * along, tip_sin * along + lift, self.out * sine(angle as f32)]
            };
            let (before, after) = (self.rise - spread, self.rise + spread);
            let corners = [
                edge(if before < 0 { before + 360 } else { before }),
                edge(if after >= 360 { after - 360 } else { after }),
                [0.0, lift, 0.0],
                [0.0, lift, 0.0]
            ];
            Rect::new(picture, self.alpha, corners, (0.0, 1.0), colour, adds)
        };
        let narrow = if kind == 2.0 || kind == 4.0 {
            [0, 0, 250]
        } else if kind == 3.0 {
            [250; 3]
        } else if kind == 5.0 {
            [50; 3]
        } else if kind == 6.0 {
            [0, 0, 255]
        } else {
            [0, 250, 250]
        };
        let (wide, mixed) = if kind == 2.0 {
            ([250, 250, 0], false)
        } else if kind == 3.0 || kind == 5.0 {
            ([250; 3], true)
        } else if kind == 4.0 {
            ([250, 250, 0], true)
        } else if kind == 6.0 {
            ([50, 50, 255], false)
        } else {
            ([250, 0, 0], false)
        };
        vec![triangle(if kind == 2.0 { 1 } else { 2 }, narrow, true), triangle(if kind == 2.0 { 2 } else { 5 }, wide, !mixed)]
    }

    /// The kind 69 in `CEffectPrim::OnProcess`, for one part: its far end (`at[1]`) is `out` from its near end
    /// (`at[0]`) along the way `turned`. `heights[4]` says how it comes and goes: 5 comes up by 8 a tick for
    /// twenty ticks and goes down by 2 after; 2 comes up by 2 a tick for five ticks and goes down by 1; any
    /// other by 8 and by 4.
    pub(super) fn reach_as_streak(&mut self) {
        if self.waits() {
            return;
        }
        let kind = self.heights[4];
        if kind == 5.0 {
            self.alpha = if self.process > 20 { (self.alpha - 2).max(0) } else { self.alpha + 8 };
        } else if self.process < 6 {
            self.alpha += if kind == 2.0 { 2 } else { 8 };
        } else {
            self.alpha = (self.alpha - if kind == 2.0 { 1 } else { 4 }).max(0);
        }
        self.at[1][0] = self.out * cosine(self.turned as f32) + self.at[0][0];
        self.at[1][2] = self.out * sine(self.turned as f32) + self.at[0][2];
    }

    /// `RenderPentaRing`, for one part: a tube of five sides from its near end to its far end, `heights[0]`
    /// over both, twice: once `tall` thick and mixed in, and once thicker (twice for the kind 3, else by a
    /// quarter) and added. Its five corners stand across the way it goes. The kind (`heights[4]`) gives the
    /// colour.
    pub(super) fn five_sides(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let kind = self.heights[4];
        let across = ((self.turned + 90) % 360) as f32;
        let mut rects = Vec::new();
        for outer in [false, true] {
            let thick = match (outer, kind == 3.0) {
                (false, _) => self.tall,
                (true, true) => self.tall + self.tall,
                (true, false) => self.tall + 0.25,
            };
            let colour = if kind == 2.0 {
                [255; 3]
            } else if kind == 3.0 {
                [255, 89, 182]
            } else if kind == 4.0 {
                [55, 55, 255]
            } else if kind == 5.0 {
                [55; 3]
            } else if outer {
                [255, 255, 200]
            } else {
                [255, 255, 55]
            };
            let ends = |fifth: i32| {
                let angle = (fifth * 72) as f32;
                let wide = thick * cosine(angle);
                let off = [wide * cosine(across), thick * sine(angle) - self.heights[0], wide * sine(across)];
                (std::array::from_fn(|axis| off[axis] + self.at[0][axis]), std::array::from_fn(|axis| off[axis] + self.at[1][axis]))
            };
            for fifth in 1..=5 {
                let ((near_before, far_before), (near, far)): (([f32; 3], [f32; 3]), ([f32; 3], [f32; 3])) = (ends(fifth - 1), ends(fifth));
                rects.push(Rect::new(picture, self.alpha, [near_before, near, far, far_before], (0.0, 1.0), colour, outer));
            }
        }
        rects
    }
}
