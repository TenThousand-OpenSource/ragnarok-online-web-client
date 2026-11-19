//! `PP_FLOWERCAST` (108) of the client's primitives: walls of light round the one the effect is on, each most
//! of a round and arched, low at its two ends and tall in its middle like a petal, that breathe as they stand
//! (`PrimFlowerCast`, drawn by `Render3DCasting`, which is `Part::casting`): the Comforts of the Sun, the Moon
//! and the Stars.

use super::*;

impl Part {
    /// `PrimFlowerCast`, for one part: how tall it is (`tall`) goes 2.5 over and under what its effect gave
    /// it (`at[1][1]`, its `vecTNow.y`) as a sine does, four degrees a tick. Each of its 21 edges stands as
    /// tall as the sine of nine degrees an edge says, so that the wall is an arch; over the first 90 ticks
    /// the whole rises as a sine does. It comes up by 5 a tick over its first 19 ticks (to 180 at most), and
    /// from its 111th goes down by 1. (An edge that came to stand over its own height would be held there for
    /// good, its `flags`; by these figures none does.)
    pub(super) fn breathe_as_petal(&mut self) {
        self.process += 1;
        self.turned = self.turned.rem_euclid(360);
        self.rise = self.rise.rem_euclid(360);
        self.tall = sine((self.process % 90 * 4) as f32) * 2.5 + self.at[1][1];
        if self.process >= 111 {
            self.alpha = (self.alpha - 1).max(0);
        } else if self.process < 20 {
            self.alpha = (self.alpha + 5).min(180);
        }
        for edge in 0..21 {
            if self.flags[edge] != 0 {
                continue;
            }
            let most = self.tall * sine((edge * 9) as f32);
            let height = if self.process <= 90 { most * sine(self.process as f32) } else { most };
            self.heights[edge] = height;
            if height > most {
                (self.heights[edge], self.flags[edge]) = (most, 1);
            }
            self.heights[edge] = self.heights[edge].max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wall as `TexturedSlashAir` launches its first: 4.5 out, leaning out by 5 degrees, 27 tall.
    fn petal() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::FlowerCast), "ring_blue.tga");
        (prim.duration, prim.size) = (400, 4.0);
        let mut part = Part { round: 315, out: 4.5, rise: 85, tall: 27.0, ..Part::default() };
        part.at[1] = [85.0, 27.0, 0.0];
        prim.parts.push(part);
        prim
    }

    #[test]
    fn a_petal_rises_as_an_arch_and_breathes() {
        let mut prim = petal();
        for _ in 0..90 {
            prim.tick(400);
        }
        // At its 90th tick it is at its own height again (the sine of 360 degrees) and wholly risen: nothing
        // at its ends, all of it in its middle.
        let part = &prim.parts[0];
        assert!((part.tall - 27.0).abs() < 1e-3 && part.heights[0] == 0.0 && (part.heights[10] - 27.0).abs() < 1e-3 && part.heights[20].abs() < 1e-3);
        assert_eq!(part.alpha, 95);
        for _ in 90..112 {
            prim.tick(400);
        }
        // 22 ticks on it is 2.5 times the sine of 88 degrees taller, and has begun to go.
        assert!((prim.parts[0].tall - 27.0 - 2.5 * sine(88.0)).abs() < 1e-3);
        assert_eq!(prim.parts[0].alpha, 93);
        for _ in 112..210 {
            prim.tick(400);
        }
        assert_eq!(prim.parts[0].alpha, 0);
    }

    #[test]
    fn a_petal_is_most_of_a_round_in_blue_its_light_added() {
        let mut prim = petal();
        for _ in 0..30 {
            prim.tick(400);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // 315 degrees in steps of 15: 21 pieces, the last showing the same of the picture as the one before.
        assert_eq!(rects.len(), 21);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([100, 100, 255], true, 95.0));
        assert_eq!(rects[20].corners[0].1, rects[19].corners[0].1);
        // The first edge's foot is 4.5 out to the east, on the ground.
        assert_eq!(rects[0].corners[0].0, [4.5, 0.0, 0.0]);
    }
}
