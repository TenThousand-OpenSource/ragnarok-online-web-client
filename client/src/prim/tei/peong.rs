//! `PP_PEONG` (94) and `PP_PEONGUP` (96) of the client's primitives, the puff of one who lands from a leap:
//!
//! - 94: scraps that are thrown out level from round a place, each a square facing the screen that lifts a
//!   little and comes down again as it goes (`PrimPeong`, drawn by `RenderPeong`);
//! - 96: small lights that rise from round a place, wandering as they go (`PrimPeongUp`, drawn by
//!   `RenderPeongUp`).
//!
//! Both keep a square's place in the part's `vecBPre` (`at[2]`).

use super::*;

impl Part {
    /// `PrimPeong`, for one part: it waits while its count is under one. Over its first 90 ticks it goes
    /// `tall` a tick the way `rise` says, and stands over the level it was put at (`at[0]`, its `vecBNow`)
    /// by 50 times `tall`, as a sine of two degrees a tick goes. It comes up by 2 a tick for 25 ticks and
    /// after its 70th goes down by 1.
    pub(super) fn puff_out(&mut self) {
        if self.waits() {
            return;
        }
        if self.process <= 90 {
            self.at[2][0] += cosine(self.rise as f32) * self.tall;
            self.at[2][2] += sine(self.rise as f32) * self.tall;
            self.at[2][1] = self.at[0][1] - sine((self.process * 2 % 360) as f32) * self.tall * 50.0;
        }
        if self.process <= 25 {
            self.alpha += 2;
        }
        if self.process > 70 {
            self.alpha -= 1;
        }
        self.alpha = self.alpha.max(0);
    }

    /// `RenderPeong`, for one part: a square that faces the screen about its place, `out` from its middle to
    /// a corner, white and mixed in.
    pub(super) fn scrap(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        vec![Rect::new(picture, self.alpha, Self::facing(self.at[2], self.out, 45, right, up), (0.0, 1.0), [255; 3], false)]
    }

    /// `PrimPeongUp`, for one part: it waits while its count is under one. It comes up by 5 a tick and from
    /// its 30th goes down by 2; its two turns wander (`WanderAngle`), and while it is over the level of the
    /// effect's place it drifts by their sines, 0.03 a tick; it rises by `tall` a tick.
    pub(super) fn puff_up(&mut self) {
        if self.waits() {
            return;
        }
        self.alpha = (self.alpha + if self.process >= 30 { -2 } else { 5 }).max(0);
        self.wander(0);
        self.wander(2);
        if self.at[2][1] < 0.0 {
            self.at[2][0] += sine(self.heights[0]) * 0.03;
            self.at[2][2] += sine(self.heights[2]) * 0.03;
        }
        self.at[2][1] -= self.tall;
    }

    /// `RenderPeongUp`, for one part: a square that faces the screen about its place, `out` from its middle
    /// to a corner, its light added: blue for a part whose `flags[0]` is 2, dark grey for 3, white for any
    /// other. One whose first corner is under the level of the effect's place is left out.
    pub(super) fn rising_light(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let corners = Self::facing(self.at[2], self.out, 45, right, up);
        let colour = match self.flags[0] {
            2 => [105, 105, 225],
            3 => [55; 3],
            _ => [255; 3],
        };
        if corners[0][1] <= 0.0 { vec![Rect::new(picture, self.alpha, corners, (0.0, 1.0), colour, true)] } else { Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    #[test]
    fn a_scrap_waits_goes_out_lifting_and_fades() {
        let mut prim = Prim::new(Shape::Tei(Tei::Peong), "peong1.tga");
        prim.duration = 300;
        // As `TexturedScatterArc` launches one: 4 out to the east, 4 over the ground.
        let mut part = Part { out: 3.0, process: -2, rise: 0, tall: 0.03, ..Part::default() };
        (part.at[2], part.at[0][1]) = ([4.0, 0.0, 0.0], -4.0);
        prim.parts.push(part);
        prim.tick(300);
        prim.tick(300);
        assert_eq!((prim.parts[0].alpha, prim.parts[0].at[2]), (0, [4.0, 0.0, 0.0]));
        for _ in 0..45 {
            prim.tick(300);
        }
        // 45 ticks out: 1.35 further east, and at the top of its lift, 1.5 over where it was put.
        let part = &prim.parts[0];
        assert!((part.at[2][0] - 5.35).abs() < 1e-3 && (part.at[2][1] + 5.5).abs() < 1e-3);
        assert_eq!(part.alpha, 50);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (1, [255; 3], false, 50.0));
        for _ in 45..90 {
            prim.tick(300);
        }
        // It is back on its level, stands still from then on, and is going.
        assert!((prim.parts[0].at[2][1] + 4.0).abs() < 1e-3 && prim.parts[0].alpha == 30);
        let stood = prim.parts[0].at[2];
        for _ in 0..40 {
            prim.tick(300);
        }
        assert_eq!((prim.parts[0].at[2], prim.parts[0].alpha), (stood, 0));
    }

    #[test]
    fn a_light_rises_wandering_and_is_gone_after_a_hundred_ticks() {
        let mut prim = Prim::new(Shape::Tei(Tei::PeongUp), "whitelight.tga");
        prim.duration = 300;
        let mut part = Part { out: 3.5, tall: 0.2, seed: 7, ..Part::default() };
        part.heights[..4].copy_from_slice(&[90.0, 200.0, 270.0, 100.0]);
        (part.at[2], part.flags[0]) = ([2.5, -3.0, 0.0], 1);
        prim.parts.push(part);
        prim.tick(300);
        let part = prim.parts[0].clone();
        // The first turn goes up towards its mark and the second down; it drifts by their sines and rises.
        assert_eq!((part.heights[0], part.heights[2], part.alpha), (93.0, 268.0, 5));
        assert!((part.at[2][0] - 2.5 - sine(93.0) * 0.03).abs() < 1e-5 && (part.at[2][2] - sine(268.0) * 0.03).abs() < 1e-5 && (part.at[2][1] + 3.2).abs() < 1e-5);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (1, [255; 3], true, 5.0));
        for _ in 1..29 {
            prim.tick(300);
        }
        assert_eq!(prim.parts[0].alpha, 145);
        for _ in 29..102 {
            prim.tick(300);
        }
        assert_eq!(prim.parts[0].alpha, 0);
    }

    #[test]
    fn a_light_under_the_level_of_the_place_is_left_out() {
        let mut part = Part { out: 3.0, alpha: 100, ..Part::default() };
        part.at[2] = [0.0, 5.0, 0.0];
        assert!(part.rising_light("whitelight.tga", RIGHT, UP).is_empty());
        part.flags[0] = 2;
        part.at[2] = [0.0, -5.0, 0.0];
        assert_eq!(part.rising_light("whitelight.tga", RIGHT, UP)[0].colour, [105, 105, 225]);
    }
}
