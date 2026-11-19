//! The kind `PP_MAGNUM2` (73) of the client's primitives: up to four walls of light round the one the effect is
//! on, each a full ring that leans outwards like a funnel, flashes up and is gone (`CEffectPrim::OnProcess`,
//! `case PP_MAGNUM2`; drawn by `RenderMagnum2`). Spiral Pierce launches one after the other, each wider and
//! lower than the one before.

use super::*;

impl Part {
    /// `PP_MAGNUM2` in `CEffectPrim::OnProcess`, for one part: over its first eight ticks it comes up by 10 a
    /// tick, to 80 at the most (one whose `alpha_top` is not nothing by 5, to 40); from the ninth it goes down
    /// by 30 a tick and is over once nothing of it shows.
    pub(super) fn flash_as_cone(&mut self) {
        self.process += 1;
        self.rise = self.rise.max(0);
        self.turned = under_a_round(self.turned);
        if self.process < 9 {
            self.alpha = if self.alpha_top == 0 { (self.alpha + 10).min(80) } else { (self.alpha + 5).min(40) };
        } else {
            self.alpha -= 30;
            self.over_once_gone();
        }
    }

    /// `RenderMagnum2`, for one part: a wall in pieces, an edge every twentieth of how far round it goes
    /// (`round`) from `turned` on; the last edge of a full round is the first again. An edge's foot is 3 over
    /// the ground, `out` from the effect's place; its top is `tall` from the foot, leaning outwards as `rise`
    /// says (90 is upright). A piece shows its 21st of the picture (past the twentieth, the twentieth's) in
    /// the colour of the primitive's `size`.
    pub(super) fn cone(&self, picture: &'static str, size: f32) -> Vec<Rect> {
        let step = self.round / 20;
        if step <= 0 || self.alpha <= 0 {
            return Vec::new();
        }
        let (lean, lift) = (cosine(self.rise as f32) * self.tall, sine(self.rise as f32) * self.tall);
        let edge = |(order, offset): (i32, i32)| {
            let turned = self.turned + offset;
            let angle = if self.round == 360 && order == 20 { self.turned } else if turned >= 360 { turned - 360 } else { turned } as f32;
            let foot = [cosine(angle) * self.out, -3.0, sine(angle) * self.out];
            (foot, [cosine(angle) * lean + foot[0], -(lift + 3.0), sine(angle) * lean + foot[2]])
        };
        Rect::band(picture, self.alpha, size, 20, edges(self.round, step, true, 20).map(edge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::close;

    /// The two walls of the first primitive `Magnum2` launches, at its 25th tick.
    fn walls() -> Prim {
        let mut cones = Prim::new(Shape::Tei(Tei::Magnum2), "ring_yellow.tga");
        (cones.duration, cones.size) = (100, 5.0);
        cones.parts.extend((0..2).map(|index| Part { round: 360, tall: 20.0, turned: index * 9, out: 5.0, rise: (35 - index * 4) * 2, alpha_top: index, ..Part::default() }));
        cones
    }

    #[test]
    fn a_wall_flashes_up_the_second_half_as_strong_and_both_are_gone() {
        let mut cones = walls();
        for _ in 0..8 {
            cones.tick(100);
        }
        assert_eq!((cones.parts[0].alpha, cones.parts[1].alpha), (80, 40));
        cones.tick(100);
        cones.tick(100);
        assert_eq!((cones.parts[0].alpha, cones.parts[0].live, cones.parts[1].live), (20, true, false));
        cones.tick(100);
        assert!(!cones.parts[0].live && cones.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
    }

    #[test]
    fn a_wall_is_a_full_ring_of_twenty_pieces_leaning_outwards() {
        let mut cones = walls();
        cones.tick(100);
        let rects = cones.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 40);
        // The first piece of the first wall begins due east, 5 out and 3 over the ground, and leans out by
        // the cosine of 70 degrees of its 20.
        let (foot, top) = (rects[0].corners[0].0, rects[0].corners[3].0);
        assert!(close(foot[0], 5.0) && close(foot[1], 3.0) && close(foot[2], 0.0));
        assert!(close(top[0], 5.0 + 20.0 * 70.0_f32.to_radians().cos()) && close(top[1], 3.0 + 20.0 * 70.0_f32.to_radians().sin()));
        // The last piece ends where the first began; each shows its 21st of the picture.
        assert!(close(rects[19].corners[1].0[0], 5.0) && close(rects[19].corners[1].0[2], 0.0));
        assert!(close(rects[1].corners[0].1[0], 1.0 / 21.0));
        assert_eq!(rects[0].alpha, 10.0);
    }
}
