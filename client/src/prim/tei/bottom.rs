//! `PP_BOTTOM` (47) of the client's primitives: a picture facing the screen that hovers up and down over the
//! one the effect is on, with up to three larger and fainter ones behind it that take its look every sixth
//! tick (`PrimBottom`, drawn by `RenderFieldPanel`: `tei/field.rs`): the cross over a cell of Gospel.
//!
//! The client keeps the places of the ones behind as places of the world, so behind one that walks they stay
//! back; here every part is put from the effect's own place and goes with it. (No effect ported has both the
//! ones behind and an actor that walks.)

use super::*;

impl Prim {
    /// `PrimBottom`: the first part hovers as the note of the kind 156 does (`Part::hover`), and one whose
    /// `flags[2]` is 7 turns 10 degrees a tick besides. At every sixth tick of the first part each living
    /// part behind takes the look of the one before it, the last first: half a unit larger and 50 fainter.
    pub(super) fn hover_with_a_tail(&mut self) {
        let [head, tail @ ..] = &mut self.parts[..] else { return };
        if !head.live {
            return;
        }
        if (head.process + 1) % 6 == 0 {
            for index in (0..tail.len()).rev() {
                let before = if index == 0 { head.clone() } else { tail[index - 1].clone() };
                let part = &mut tail[index];
                if part.live {
                    part.follow(&before, 50);
                    part.out += 0.5;
                    (part.flags[2], part.heights[7]) = (2, 0.0);
                }
            }
        }
        head.hover();
        if head.flags[2] == 7 {
            head.round = under_a_round(head.round + 10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP, close};

    /// The cross as `Bottom_Music` launches it with its kind 9, begun at no turn of its hovering.
    fn cross() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Bottom), "cross_old.bmp");
        prim.duration = 19999;
        let mut part = Part { out: 1.5, alpha: 200, round: 45, ..Part::default() };
        (part.flags[2], part.flags[3]) = (9, 6);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn the_cross_of_gospel_hovers_about_six_over_the_ground() {
        let mut prim = cross();
        for _ in 0..90 {
            prim.tick(19999);
        }
        // A degree a tick: after 90 it is at the low end of its swing, 3 over the ground (the client's y is
        // down), and its swell has gone round once and a quarter.
        assert!(close(prim.parts[0].at[0][1], -3.0) && prim.parts[0].heights[8] == 90.0);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 1);
        // White, its light added (`RenderFieldPanel`, 9 and 6), a twentieth larger at the top of its swell.
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([255; 3], true, 200.0));
        let reach = 1.5 * 1.05 * cosine(45.0);
        assert!(close(rects[0].corners[0].0[0], reach) && close(rects[0].corners[0].0[1], 3.0 - reach));
        // It does not turn.
        assert_eq!(prim.parts[0].round, 45);
    }

    #[test]
    fn the_ones_behind_take_the_look_of_the_one_before_every_sixth_tick() {
        let mut prim = cross();
        prim.parts[0].flags[2] = 7;
        prim.parts.extend((0..3).map(|_| Part::default()));
        for _ in 0..5 {
            prim.tick(19999);
        }
        assert_eq!((prim.parts[0].round, prim.parts[1].alpha), (95, 0));
        prim.tick(19999);
        // At the sixth tick the second takes the first's look as it was after five: larger, fainter.
        assert_eq!((prim.parts[1].alpha, prim.parts[1].out, prim.parts[1].round, prim.parts[1].flags[2]), (150, 2.0, 95, 2));
        assert!(close(prim.parts[1].at[0][1], sine(5.0) * 3.0 - 6.0));
        // The third took the second's as it was then: nothing yet.
        assert_eq!(prim.parts[2].alpha, 0);
        for _ in 0..6 {
            prim.tick(19999);
        }
        assert_eq!((prim.parts[2].alpha, prim.parts[2].out, prim.parts[3].alpha), (100, 2.5, 0));
        // All are drawn as the first part's number says: white and added.
        assert_eq!(prim.rects(RIGHT, UP).len(), 3);
    }
}
