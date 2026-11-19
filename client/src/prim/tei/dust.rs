//! The kind 157 of the client's primitives: squares facing the screen that fall from high over a place and
//! go out on reaching its level (`PrimOrbitFade`, drawn by `RenderGlowSquare`): the stones of Stone Rain.

use super::*;

impl Part {
    /// `PrimOrbitFade`, for one part: over its first nine ticks it comes up by 25 a tick, to 255 at most.
    /// While it is over the level of the effect's place it falls by `tall` a tick; from then on it goes down
    /// by 15 a tick and is over once nothing of it shows. It stands `heights[0]` from the middle, the way
    /// `round` says.
    pub(super) fn fall_as_dust(&mut self) {
        self.process += 1;
        if self.process < 10 {
            self.alpha += 25;
        }
        self.alpha = self.alpha.min(255);
        if self.at[0][1] < 0.0 {
            self.at[0][1] += self.tall;
        } else {
            self.alpha -= 15;
        }
        (self.at[0][0], self.at[0][2]) = (self.heights[0] * cosine(self.round as f32), self.heights[0] * sine(self.round as f32));
        self.over_once_gone();
        self.round = self.round.rem_euclid(360);
    }

    /// `RenderGlowSquare`, for one part: a square that faces the screen about its place, `out` from its
    /// middle to a corner, white, pale red or red by its `flags[0]` (0, 1, 2), and once more a tenth larger
    /// in a deep red; their light added. (The client draws them over everything; here they are drawn as the
    /// other effects are.)
    pub(super) fn glow_square(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let colour = match self.flags[0] {
            0 => Some([255; 3]),
            1 => Some([255, 160, 160]),
            2 => Some([255, 90, 90]),
            _ => None,
        };
        let square = |reach: f32, colour: [u8; 3]| Rect::new(picture, self.alpha.max(0), Self::facing(self.at[0], reach, 45, right, up), (0.0, 1.0), colour, true);
        colour.map(|colour| square(self.out, colour)).into_iter().chain([square(self.out * 1.1, [255, 1, 1])]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stone_falls_to_the_level_of_the_place_and_goes_out_there() {
        let mut prim = Prim::new(Shape::Tei(Tei::GlowSquare), "stone_part.bmp");
        prim.duration = 100;
        let mut part = Part { tall: 2.5, out: 8.0, round: 90, ..Part::default() };
        (part.heights[0], part.at[0][1], part.flags[0]) = (20.0, -60.0, 1);
        prim.parts.push(part);
        for _ in 0..10 {
            prim.tick(100);
        }
        let part = prim.parts[0].clone();
        assert_eq!((part.alpha, part.at[0][1]), (225, -35.0));
        assert!(part.at[0][0].abs() < 1e-4 && (part.at[0][2] - 20.0).abs() < 1e-4);
        // Two squares about its place, the second a tenth larger and deep red, both added.
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((rects.len(), rects[0].colour, rects[1].colour, rects[0].adds, rects[0].alpha), (2, [255, 160, 160], [255, 1, 1], true, 225.0));
        let (first, third) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert!(((first[1] + third[1]) / 2.0 - 35.0).abs() < 1e-3 && ((first[0] - third[0]).abs() - 16.0 * cosine(45.0)).abs() < 1e-3);
        assert!(((rects[1].corners[0].0[0] - rects[1].corners[2].0[0]).abs() - 17.6 * cosine(45.0)).abs() < 1e-3);
        // 24 ticks to the level of the place, then down by 15 a tick.
        for _ in 10..25 {
            prim.tick(100);
        }
        assert_eq!((prim.parts[0].at[0][1], prim.parts[0].alpha), (0.0, 210));
        for _ in 0..15 {
            prim.tick(100);
        }
        assert!(!prim.parts[0].live);
    }
}
