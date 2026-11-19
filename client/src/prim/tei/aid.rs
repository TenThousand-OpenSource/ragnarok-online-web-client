//! `PP_FIRSTAID` (58) and the kind 67 of the client's primitives, two pictures that only come and go:
//!
//! - `PP_FIRSTAID`: the glitter of First Aid, a square facing the screen that swells and shrinks a little,
//!   comes up over 20 ticks and from its hundredth goes (`PrimFirstAid`, drawn by `RenderFieldPanel`:
//!   `tei/field.rs`);
//! - the kind 67: a footprint, a square lying level over the ground that loses 1 of its alpha every other
//!   tick (`CEffectPrim::OnProcess`, `case 67`, drawn by `RenderSizedSquare`: `Part::sized_square` in
//!   `tei/halo.rs`).

use super::*;

impl Part {
    /// `PrimFirstAid`, for the first part: its swell (`heights[8]`) goes on 3 degrees a tick; over its first
    /// 20 ticks it comes up by 4 a tick, and after its hundredth goes down by 1.
    pub(super) fn glitter_as_aid(&mut self) {
        self.process += 1;
        self.heights[8] += 3.0;
        if self.heights[8] >= 360.0 {
            self.heights[8] -= 360.0;
        }
        if self.process <= 20 {
            self.alpha += 4;
        }
        if self.process > 100 {
            self.alpha = (self.alpha - 1).max(0);
        }
    }

    /// The kind 67, for the first part: at every odd tick it loses 1 of its alpha.
    pub(super) fn fade_as_print(&mut self) {
        self.process += 1;
        if self.process > 0 && self.process % 2 > 0 {
            self.alpha = (self.alpha - 1).max(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    /// The glitter as `CRagEffect::FirstAid` launches it: 10 to a corner, 12 over the feet.
    fn glitter() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::FirstAid), "pikapika2.bmp");
        prim.duration = 299;
        let mut part = Part { out: 10.0, ..Part::default() };
        (part.at[0], part.flags[2]) = ([0.0, -12.0, 0.0], 1);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn the_glitter_of_first_aid_comes_up_stays_and_goes() {
        let mut prim = glitter();
        assert!(prim.rects(RIGHT, UP).is_empty());
        for _ in 0..20 {
            prim.tick(299);
        }
        assert_eq!(prim.parts[0].alpha, 80);
        for _ in 20..100 {
            prim.tick(299);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[0].heights[8]), (80, 300.0));
        // One square about a place 12 over the feet, bluish, its light added.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (1, [150, 150, 250], true, 80.0));
        let middle: f32 = rects[0].corners.iter().take(4).map(|(at, _)| at[1]).sum::<f32>() / 4.0;
        assert!((middle - 12.0).abs() < 1e-3);
        for _ in 100..180 {
            prim.tick(299);
        }
        assert_eq!(prim.parts[0].alpha, 0);
        assert!(prim.rects(RIGHT, UP).is_empty());
    }

    #[test]
    fn a_footprint_fades_every_other_tick() {
        let mut prim = Prim::new(Shape::Tei(Tei::Footprint), "foot_l2.tga");
        (prim.duration, prim.size) = (340, 100.0);
        let mut part = Part { out: 6.0, alpha: 150, turned: 135, ..Part::default() };
        part.heights[0] = -0.2;
        prim.parts.push(part);
        // A level square a touch over the ground, its first corner where it was turned to, drawn twice.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].alpha, rects[0].adds), (2, 150.0, true));
        let first = rects[0].corners[0].0;
        assert!((first[0] - 5.4 * cosine(135.0)).abs() < 1e-3 && (first[1] - 0.2).abs() < 1e-6 && (first[2] - 5.4 * sine(135.0)).abs() < 1e-3);
        for _ in 0..10 {
            prim.tick(340);
        }
        assert_eq!(prim.parts[0].alpha, 145);
        for _ in 10..300 {
            prim.tick(340);
        }
        assert_eq!(prim.parts[0].alpha, 0);
    }
}
