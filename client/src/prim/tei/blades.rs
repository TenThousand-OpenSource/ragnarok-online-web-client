//! The kind 159 of the client's primitives: thin needles of light that fly in from every side towards the
//! one the effect is on and whirl round that one (`PrimSpiralCollapse`, drawn by `RenderBladeQuads`): the
//! lights of Prestige and of Inspiration.

use super::*;

impl Part {
    /// `PrimSpiralCollapse`, for one part: it waits while its count is under one, and comes up by 5 a tick
    /// over its first 19 ticks. It flies `heights[1]` a tick against the way it points: `rise` up from level
    /// along the client's z, `turned` round in the plane of its x and y. Nearer than 12 to the middle it
    /// whirls for ten ticks, pointing 9 degrees further a tick, 2 fainter and a hundredth shorter each; from
    /// then on it points 3 degrees back a tick and turns a degree, and after its 300th tick goes down by 1 a
    /// tick, a hundredth shorter each.
    pub(super) fn collapse_in_a_spiral(&mut self) {
        if self.waits() {
            return;
        }
        if self.process < 20 {
            self.alpha = (self.alpha + 5).min(255);
        }
        let far = self.at[0].iter().map(|along| along * along).sum::<f32>().sqrt();
        if self.flags[0] == 0 && far < 12.0 {
            (self.flags[0], self.flags[1]) = (1, 0);
        }
        if self.flags[0] == 1 {
            self.rise = (self.rise + 9).rem_euclid(360);
            self.turned = self.turned.rem_euclid(360);
            self.flags[1] += 1;
            if self.flags[1] > 9 {
                self.flags[0] = 2;
            }
            self.alpha = (self.alpha - 2).max(0);
            self.out *= 0.99;
        }
        if self.flags[0] == 2 {
            self.rise = (self.rise - 3).rem_euclid(360);
            self.turned = (self.turned + 1).rem_euclid(360);
            if self.process > 300 {
                self.alpha = (self.alpha - 1).max(0);
                self.out *= 0.99;
            }
        }
        let (level, speed) = (cosine(self.rise as f32), self.heights[1]);
        self.at[0][0] -= speed * level * cosine(self.turned as f32);
        self.at[0][1] -= speed * level * sine(self.turned as f32);
        self.at[0][2] -= speed * sine(self.rise as f32);
    }

    /// `RenderBladeQuads`, for one part: two needles through its place (`at[0]`, lowered by `heights[0]`)
    /// along the way it points, each `out` to both ends: a thin one, 2 degrees to each side of that way, and
    /// a thicker one of 8 over it; their light added. `flags[2]` gives the two colours: cyan and red (0),
    /// white and yellow (1), pale red and pink (2), pale blue and blue (3).
    pub(super) fn blades(&self, picture: &'static str) -> Vec<Rect> {
        let (thin, thick) = match self.flags[2] {
            0 => ([0, 250, 250], [250, 0, 0]),
            1 => ([250; 3], [250, 250, 0]),
            2 => ([250, 100, 100], [255, 166, 210]),
            3 => ([200, 200, 255], [5, 5, 255]),
            _ => return Vec::new(),
        };
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (cos, sin) = (cosine(self.turned as f32), sine(self.turned as f32));
        let point = |degrees: i32| {
            let angle = (self.rise + degrees).rem_euclid(360) as f32;
            let level = self.out * cosine(angle);
            [level * cos + self.at[0][0], level * sin + self.heights[0] + self.at[0][1], self.out * sine(angle) + self.at[0][2]]
        };
        [(2, thin), (8, thick)].into_iter().map(|(half, colour)| Rect::new(picture, self.alpha, [point(-half), point(half), point(180 - half), point(180 + half)], (0.0, 1.0), colour, true)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A needle as `TexturedScatterSphere` launches one: 70 from the middle the way it points.
    fn needle(rise: i32, turned: i32, flag: i8) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::SpiralBlade), "alpha_center.tga");
        prim.duration = 500;
        let mut part = Part { out: 4.0, rise, turned, ..Part::default() };
        (part.heights[0], part.heights[1], part.flags[2]) = (-5.0, 0.5, flag);
        let level = cosine(rise as f32) * 70.0;
        part.at[0] = [level * cosine(turned as f32), level * sine(turned as f32) - 5.0, sine(rise as f32) * 70.0];
        prim.parts.push(part);
        prim
    }

    #[test]
    fn it_flies_in_half_a_step_a_tick_and_whirls_once_it_is_near() {
        let mut prim = needle(0, 0, 1);
        for _ in 0..19 {
            prim.tick(500);
        }
        assert_eq!(prim.parts[0].alpha, 95);
        assert!((prim.parts[0].at[0][0] - 60.5).abs() < 1e-3);
        // 70 less half a step a tick is nearer than 12 (with the 5 it is lowered by) at its 120th tick.
        for _ in 19..120 {
            prim.tick(500);
        }
        assert_eq!((prim.parts[0].flags[0], prim.parts[0].rise), (1, 9));
        for _ in 0..9 {
            prim.tick(500);
        }
        // The tenth tick of its whirl it also takes the first step of what follows.
        assert_eq!((prim.parts[0].flags[0], prim.parts[0].rise, prim.parts[0].turned, prim.parts[0].alpha), (2, 87, 1, 75));
        assert!((prim.parts[0].out - 4.0 * 0.99_f32.powi(10)).abs() < 1e-4);
    }

    #[test]
    fn it_is_drawn_as_two_needles_along_its_way_in_the_colours_of_its_kind() {
        let mut prim = needle(0, 0, 3);
        prim.tick(500);
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 2);
        assert_eq!((rects[0].colour, rects[1].colour, rects[0].adds, rects[0].alpha), ([200, 200, 255], [5, 5, 255], true, 5.0));
        // It points east: its ends are 4 to the east and to the west of its place, which is 10 over the
        // ground (lowered by -5 twice: as it was put and as it is drawn).
        let (first, third) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert!((first[0] - (69.5 + 4.0 * cosine(2.0))).abs() < 1e-3 && (third[0] - (69.5 - 4.0 * cosine(2.0))).abs() < 1e-3 && (first[1] - 10.0).abs() < 1e-3);
        // The thin one is 2 degrees to each side of the way along the client's z, the thick one 8.
        assert!((first[2] + 4.0 * sine(2.0)).abs() < 1e-3 && (rects[1].corners[1].0[2] - 4.0 * sine(8.0)).abs() < 1e-3);
    }
}
