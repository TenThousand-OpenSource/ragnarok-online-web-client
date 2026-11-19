//! `PP_SOULLINK` (109) and `PP_SOULPARTICLE` (111): the light that flies in a bow over one whose soul is
//! linked, and the sparks it leaves behind (`CEffectPrim::OnProcess`, `case PP_SOULLINK`, drawn by
//! `RenderSoulLink`; `PrimSoulParticle`, drawn by `RenderSoulParticle`, which is `Part::sparks` in
//! `tei/panels.rs`). The angle that wanders (`WanderAngle`), which other kinds use too, is `Part::wander` (`tei/part.rs`).

use super::*;

impl Part {
    /// `PP_SOULLINK`, for one part: it waits while its count is under one. Over its first ten ticks it comes
    /// up by 20 a tick (`heights[13]`), and after its 170th goes down by 5. It goes along a bow faster and
    /// faster: how far along it is (`at[1][0]`, to 359 at most) grows by a thirtieth of its count each tick,
    /// and it is that far, a sixth of a step to a degree, from 30 to the left of the middle across the
    /// screen, and 40 over the ground and up to 20 more as a sine goes (`at[2]`).
    pub(super) fn fly_as_soul(&mut self) {
        if self.waits() {
            return;
        }
        if self.process <= 10 {
            self.heights[13] += 20.0;
        } else if self.process > 170 {
            self.heights[13] = (self.heights[13] - 5.0).max(0.0);
        }
        self.at[1][0] = (self.process as f32 / 10.0 / 3.0 + self.at[1][0]).min(359.0);
        let along = self.at[1][0].trunc();
        self.at[2] = [along / 6.0 - 30.0, -(sine(along) * 20.0) - 40.0, 0.0];
    }

    /// `RenderSoulLink`, for one part: a square that faces the screen, `out` from its middle to a corner, at
    /// the place of its bow (so far across the screen from the effect's place, and so high), in a pale blue,
    /// its light added.
    pub(super) fn soul(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let middle = to_screen([0.0, self.at[2][1], 0.0], self.at[2][0], 0.0, right, [0.0; 3]);
        let corners = Part::facing(middle, self.out, 45, right, up);
        if corners[0][1] > 0.0 {
            return Vec::new();
        }
        vec![Rect::new(picture, self.heights[13] as i32, corners, (0.0, 1.0), [125, 125, 255], true)]
    }

    /// `PrimSoulParticle`, for one part, which is three sparks (`at[2]`, `at[0]`, `at[3]`): its six turns
    /// wander; over its first nine ticks the sparks come up by 20 a tick (`heights[12]` to `[14]`), and
    /// after its 30th go down by 3. The first rises by `tall` a tick and the second sinks by it, each
    /// drifting by the sines of two of the turns while it is over the level of the effect's place; the third
    /// drifts a fifth faster and neither rises nor sinks.
    pub(super) fn drift_as_soul_sparks(&mut self) {
        self.process += 1;
        for pair in 0..6 {
            self.wander(pair * 2);
        }
        if self.process < 10 {
            self.heights[12..15].iter_mut().for_each(|alpha| *alpha += 20.0);
        } else if self.process > 30 {
            self.heights[12..15].iter_mut().for_each(|alpha| *alpha = (*alpha - 3.0).max(0.0));
        }
        let speed = self.tall;
        let turn = |part: &Part, which: usize| sine(part.heights[which].trunc());
        if self.at[2][1] < 0.0 {
            self.at[2][0] += speed * turn(self, 0);
            self.at[2][2] += speed * turn(self, 2);
        }
        self.at[2][1] -= speed;
        if self.at[0][1] < 0.0 {
            self.at[0][0] -= speed * turn(self, 4);
            self.at[0][2] -= speed * turn(self, 6);
        }
        self.at[0][1] += speed;
        if self.at[3][1] < 0.0 {
            self.at[3][0] += speed * 1.2 * turn(self, 8);
            self.at[3][2] -= speed * 1.2 * turn(self, 10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    #[test]
    fn the_light_goes_along_its_bow_faster_and_faster_and_fades_late() {
        let mut prim = Prim::new(Shape::Tei(Tei::SoulLink), "whitelight.tga");
        prim.duration = 500;
        prim.parts.push(Part { out: 2.5, ..Part::default() });
        for _ in 0..30 {
            prim.tick(500);
        }
        // The counts 1 to 30 add up to 465: 15.5 degrees along.
        let part = prim.parts[0].clone();
        assert!((part.at[1][0] - 15.5).abs() < 1e-3 && part.heights[13] == 200.0);
        assert!((part.at[2][0] - (15.0 / 6.0 - 30.0)).abs() < 1e-4 && (part.at[2][1] + sine(15.0) * 20.0 + 40.0).abs() < 1e-4);
        // A pale blue square, added, about its place across the screen and over the ground.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (1, [125, 125, 255], true, 200.0));
        let (first, third) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert!(((first[0] + third[0]) / 2.0 - part.at[2][0]).abs() < 1e-3 && ((first[1] + third[1]) / 2.0 + part.at[2][1]).abs() < 1e-3);
        for _ in 30..171 {
            prim.tick(500);
        }
        assert_eq!((prim.parts[0].at[1][0], prim.parts[0].heights[13]), (359.0, 195.0));
    }

    #[test]
    fn the_sparks_come_up_part_and_go() {
        let mut prim = Prim::new(Shape::Tei(Tei::SoulParticle), "whitelight.tga");
        prim.duration = 100;
        let mut part = Part { tall: 0.07, out: 1.5, seed: 9, ..Part::default() };
        part.heights[..12].copy_from_slice(&[90.0, 200.0, 90.0, 200.0, 90.0, 200.0, 90.0, 200.0, 90.0, 200.0, 90.0, 200.0]);
        (part.at[0], part.at[2], part.at[3]) = ([-20.0, -45.0, 0.0], [-20.0, -45.0, 0.0], [-20.0, -45.0, 0.0]);
        prim.parts.push(part);
        prim.tick(100);
        let part = prim.parts[0].clone();
        // Every turn has gone to 93; the first spark rises and drifts one way, the second sinks the other.
        assert_eq!((part.heights[0], part.heights[10], part.heights[12]), (93.0, 93.0, 20.0));
        assert!((part.at[2][1] + 45.07).abs() < 1e-4 && (part.at[0][1] + 44.93).abs() < 1e-4 && part.at[3][1] == -45.0);
        assert!(part.at[2][0] > -20.0 && part.at[0][0] < -20.0 && part.at[3][2] < 0.0);
        // Three blue squares, their light added.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (3, [80, 80, 255], true));
        for _ in 0..100 {
            prim.tick(100);
        }
        assert_eq!(prim.parts[0].heights[12], 0.0);
    }
}
