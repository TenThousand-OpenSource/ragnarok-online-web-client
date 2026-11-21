//! The kind 81: thin spikes of light that stand out every way from a point over the place of the effect, each
//! growing longer and shorter over and over (`CEffectPrim::OnProcess`, `case 81`; drawn by `RenderBeam`), as
//! `CRagEffect::TexturedSparkle` launches them.
//!
//! Of a part: `out` is half the length it swings about, `round` the angle whose sine it swings by and `tall`
//! how far that goes on a tick; `turned` and `rise` the way it points, round the upright and up or down;
//! `heights[3]` how high over the place its foot is, and `heights[0]` to `[2]` its colour.

use super::*;

impl Part {
    /// The kind 81, for one part: it grows by a fiftieth a tick until it is 10, its swing goes on, and after
    /// 500 ticks it goes down by 1 a tick.
    pub(super) fn pulse_as_spike(&mut self) {
        self.process += 1;
        if self.out < 10.0 {
            self.out *= 1.02;
        }
        if self.process > 500 && self.alpha > 0 {
            self.alpha -= 1;
        }
        self.round = under_a_round((self.round as f32 + self.tall) as i32);
    }

    /// `RenderBeam`, for one part that shows: two triangles that cross, each with a foot 1 wide (one east to
    /// west, one north to south) at the height of its part and both ends of its top at one point, the tip:
    /// as far from the foot as the part reaches now, the way it points. Its light is added, in its colour.
    pub(super) fn spike(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let lift = self.heights[3];
        let length = self.out * sine(self.round as f32) + self.out;
        let reach = length * cosine(self.rise as f32);
        let tip = [reach * cosine(self.turned as f32), length * sine(self.rise as f32) - lift, reach * sine(self.turned as f32)];
        let colour = [self.heights[0] as u8, self.heights[1] as u8, self.heights[2] as u8];
        [[0.5, 0.0], [0.0, 0.5]]
            .into_iter()
            .map(|[east, north]| Rect::new(picture, self.alpha, [[east, -lift, north], tip, tip, [-east, -lift, -north]], (0.0, 1.0), colour, true))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    #[test]
    fn a_spike_grows_swings_and_is_two_triangles_to_one_tip() {
        let mut prim = Prim::new(Shape::Tei(Tei::Spike), "white02.bmp");
        prim.duration = 1000;
        // As `TexturedSparkle` launches one; this one points east and level, its swing at a quarter.
        let mut part = Part { alpha: 40, out: 1.0, rise: 0, turned: 0, round: 88, tall: 2.0, ..Part::default() };
        part.heights[..4].copy_from_slice(&[105.0, 105.0, 255.0, 10.0]);
        prim.parts.push(part);
        prim.tick(1000);
        assert!((prim.parts[0].out - 1.02).abs() < 0.0001 && prim.parts[0].round == 90 && prim.parts[0].alpha == 40);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 2);
        assert_eq!((rects[0].colour, rects[0].alpha, rects[0].adds), ([105, 105, 255], 40.0, true));
        // The foot of the first lies east to west, 10 over the place; both top corners are the tip, twice the
        // part's reach to the east.
        let near = |a: [f32; 3], b: [f32; 3]| (0..3).all(|axis| (a[axis] - b[axis]).abs() < 0.001);
        assert!(near(rects[0].corners[0].0, [0.5, 10.0, 0.0]) && near(rects[0].corners[3].0, [-0.5, 10.0, 0.0]));
        assert!(near(rects[0].corners[1].0, [2.04, 10.0, 0.0]) && near(rects[0].corners[2].0, [2.04, 10.0, 0.0]));
        assert!(near(rects[1].corners[0].0, [0.0, 10.0, 0.5]));
        // Once 10 long it grows no more; after 500 ticks it goes down by 1 a tick.
        for _ in 0..510 {
            prim.tick(1000);
        }
        assert!(prim.parts[0].out >= 10.0 && prim.parts[0].out < 10.2 && prim.parts[0].alpha == 40 - 11);
    }
}
