//! The kind `PP_HITLINE` (70) of the client's primitives: up to four sparks that leap from a place in a bow,
//! come down and leap on, each leap seven tenths of the one before, each spark a thin streak of three sides
//! with a tail of twenty pieces (`CEffectPrim::OnProcess`, `case PP_HITLINE`; drawn by `RenderHitLine`): the
//! sparks of a combo's last blow.

use super::*;

impl Part {
    /// `PP_HITLINE` in `CEffectPrim::OnProcess`, for one part: once its count is over nothing its head goes 8
    /// degrees further along its bow a tick (`heights[0]`), and past 180 begins the next bow (`flags[0]`
    /// counts them); where it was at each of the twenty ticks before is kept behind it. After 35 ticks it
    /// goes down by 5 a tick.
    pub(super) fn bounce_as_spark(&mut self) {
        if self.waits() {
            return;
        }
        if self.process > 35 {
            self.alpha -= 5;
        }
        self.alpha = self.alpha.max(0);
        self.heights[0] += 8.0;
        if self.heights[0] > 180.0 {
            self.heights[0] -= 180.0;
            self.flags[0] += 1;
        }
        for piece in (1..=20).rev() {
            self.heights[piece] = self.heights[piece - 1];
            self.flags[piece] = self.flags[piece - 1];
        }
    }

    /// Where the spark was that many pieces ago, from its place (`at[0]`), in the client's directions: the
    /// bows before this one lie one after the other along the way `rise`, the first twice `out` long and each
    /// next seven tenths of the one before; along its own bow it is a 90th of its degrees times the bow's
    /// half, and over the ground the sine of the degrees times one and a half of that half.
    fn bounced(&self, piece: usize) -> [f32; 3] {
        let (mut along, mut half) = (0.0, 1.0);
        for _ in 0..self.flags[piece] {
            along += self.out * 2.0 * half;
            half *= 0.7;
        }
        let degrees = self.heights[piece];
        if degrees > 0.0 {
            along += self.out * half * (degrees / 90.0);
        }
        [along * cosine(self.rise as f32), -self.out * sine(degrees.trunc()) * 1.5 * half, along * sine(self.rise as f32)]
    }

    /// `RenderHitLine`, for one part that has begun: twenty pieces, each a tube of three sides between two
    /// places of the spark a tick apart, `tall` thick, its light added, each piece 5 fainter than the one
    /// before. `alpha_top` gives the colour, which changes along the tail: red for the number 2, blue for 8,
    /// any other orange.
    pub(super) fn bouncing(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 || self.process <= 0 {
            return Vec::new();
        }
        let mut rects = Vec::new();
        for piece in 0..20 {
            let (head, tail) = (self.bounced(piece), self.bounced(piece + 1));
            let across = ((angle_of(head[0] - tail[0], head[2] - tail[2]) + 90) % 360) as f32;
            let alpha = self.alpha - piece as i32 * 5;
            if alpha <= 0 {
                continue;
            }
            let shade = |from: i32| (from - (20 - piece as i32) * 7).rem_euclid(256) as u8;
            let colour = match self.alpha_top {
                2 => [255, shade(200), shade(200)],
                8 => [shade(150), shade(150), 255],
                _ => [255, 200, shade(150)],
            };
            let edge = |third: i32| {
                let wide = self.tall * cosine((third * 120) as f32);
                let off = [wide * cosine(across), self.tall * sine((third * 120) as f32), wide * sine(across)];
                let ends: ([f32; 3], [f32; 3]) = (std::array::from_fn(|axis| head[axis] + off[axis] + self.at[0][axis]), std::array::from_fn(|axis| tail[axis] + off[axis] + self.at[0][axis]));
                ends
            };
            for third in 1..=3 {
                let ((head_before, tail_before), (head_now, tail_now)) = (edge(third - 1), edge(third));
                rects.push(Rect::new(picture, alpha, [head_before, head_now, tail_now, tail_before], (0.0, 1.0), colour, true));
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    /// A spark as `WhiteSparks` launches one, to leap east, 10 far at its first leap's half.
    fn spark(wait: i32) -> Prim {
        let mut sparks = Prim::new(Shape::Tei(Tei::HitLine), "white02.bmp");
        sparks.duration = 200;
        sparks.parts.push(Part { process: -wait, tall: 0.1, alpha: 150, rise: 0, out: 10.0, ..Part::default() });
        sparks
    }

    #[test]
    fn a_spark_waits_leaps_and_leaps_on_lower() {
        let mut sparks = spark(3);
        for _ in 0..3 {
            sparks.tick(200);
        }
        assert!(sparks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
        // Twelve ticks on its head is 96 degrees along its first bow: past the top, 10.67 out.
        for _ in 0..12 {
            sparks.tick(200);
        }
        let part = &sparks.parts[0];
        assert_eq!((part.heights[0], part.flags[0], part.heights[1], part.heights[12]), (96.0, 0, 96.0, 8.0));
        let head = part.bounced(0);
        assert!(close(head[0], 10.0 * 96.0 / 90.0) && close(head[1], -10.0 * 1.5 * 96.0_f32.to_radians().sin()) && close(head[2], 0.0));
        // At its 23rd tick it has come down (184 degrees) and is 4 degrees along its second bow, which
        // begins 20 out and is seven tenths as large.
        for _ in 0..11 {
            sparks.tick(200);
        }
        let part = &sparks.parts[0];
        assert_eq!((part.heights[0], part.flags[0], part.flags[1], part.flags[2]), (4.0, 1, 1, 0));
        let head = part.bounced(0);
        assert!(close(head[0], 20.0 + 7.0 * 4.0 / 90.0) && close(head[1], -10.0 * 1.5 * 0.7 * 4.0_f32.to_radians().sin()));
    }

    #[test]
    fn a_spark_is_a_tube_of_three_sides_with_a_fainter_tail_and_fades_after_35_ticks() {
        let mut sparks = spark(0);
        for _ in 0..5 {
            sparks.tick(200);
        }
        let rects = sparks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // Twenty pieces of three sides, orange, added; the head's piece at the spark's own alpha.
        assert_eq!(rects.len(), 60);
        assert_eq!((rects[0].alpha, rects[0].adds, rects[0].colour), (150.0, true, [255, 200, 10]));
        assert_eq!((rects[3].alpha, rects[3].colour), (145.0, [255, 200, 17]));
        // Flying east, its sides stand across that way: the first side's first edge is 0.1 to the north.
        let (at, _) = rects[0].corners[0];
        assert!(close(at[2].abs(), 0.1) && close(at[0], 10.0 * 40.0 / 90.0));
        for _ in 0..30 {
            sparks.tick(200);
        }
        assert_eq!(sparks.parts[0].alpha, 150);
        sparks.tick(200);
        assert_eq!(sparks.parts[0].alpha, 145);
        for _ in 0..40 {
            sparks.tick(200);
        }
        assert!(sparks.parts[0].alpha == 0 && sparks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
    }
}
