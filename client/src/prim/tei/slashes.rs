//! `PP_SLASH1` (75): thin blades of light that shoot out along the ground from the place of the effect, each
//! growing longer and taller and fading (`CEffectPrim::OnProcess`, `case PP_SLASH1`; drawn by `RenderSlash1`).

use super::*;

impl Part {
    /// `PP_SLASH1`, for one part: every tick its near end is 1 further from the middle (`heights[0]`); once
    /// that is past 10 it goes down by 2 a tick and is over when nothing of it shows. One whose `flags[0]` is
    /// 1 grows 1 longer a tick (`out`) and 0.2 taller until it is 2 tall, coming up by 5 a tick over its
    /// first 5 ticks; any other grows 2 longer a tick and 0.4 taller until it is 4 tall, coming up over its
    /// first 7.
    pub(super) fn shoot_as_slash(&mut self) {
        self.process += 1;
        self.heights[0] += 1.0;
        if self.heights[0] > 10.0 {
            self.alpha -= 2;
            self.over_once_gone();
        }
        let (longer, tallest, taller, comes_up) = if self.flags[0] == 1 { (1.0, 2.0, 0.2, 5) } else { (2.0, 4.0, 0.4, 7) };
        self.out += longer;
        if self.tall < tallest {
            self.tall += taller;
        }
        if self.process <= comes_up {
            self.alpha += 5;
        }
    }

    /// `RenderSlash1`, for one part: three upright blades a degree apart about the way it points (`turned`),
    /// each from `heights[0]` out to `out` further. The middle one is a fifth of the part's height (`tall`)
    /// at its near end and the whole of it at its far end, at the part's alpha; the two beside it a tenth
    /// and four tenths, at a third of the alpha. The picture runs from the near end to the far one, its
    /// bottom on the ground; white, its light added.
    pub(super) fn slash(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        (0..3)
            .map(|blade| {
                let angle = (self.turned - 1 + blade).rem_euclid(360) as f32;
                let (cos, sin) = (cosine(angle), sine(angle));
                let (near, far) = (self.heights[0], self.heights[0] + self.out);
                let (near_tall, far_tall, alpha) = if blade == 1 { (self.tall * 0.2, self.tall, self.alpha) } else { (self.tall * 0.1, self.tall * 0.4, self.alpha / 3) };
                let corners = [[cos * near, 0.0, sin * near], [cos * far, 0.0, sin * far], [cos * far, -far_tall, sin * far], [cos * near, -near_tall, sin * near]];
                Rect::new(picture, alpha, corners, (0.0, 1.0), [255; 3], true)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four blades as `JobLvUp50_2` (`flag` of 0, 1.5 tall) and `RunStop` (of 1, flat) launch them.
    fn blades(flag: i8, tall: f32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Slash1), "magic_blue.tga");
        (prim.duration, prim.size) = (200, 4.0);
        let mut heights = [0.0; 21];
        heights[0] = 1.0;
        let mut flags = [0; 21];
        flags[0] = flag;
        prim.parts = (0..4).map(|index| Part { tall, turned: index * 90, out: 2.0, heights, flags, ..Part::default() }).collect();
        prim
    }

    #[test]
    fn a_blade_comes_up_grows_and_is_over_once_it_has_faded() {
        let mut prim = blades(0, 1.5);
        for _ in 0..7 {
            prim.tick(200);
        }
        let blade = &prim.parts[0];
        assert_eq!((blade.alpha, blade.out, blade.heights[0]), (35, 16.0, 8.0));
        // 0.4 taller a tick until it is 4 tall.
        assert!(blade.tall >= 4.0 && blade.tall < 4.4);
        for _ in 0..5 {
            prim.tick(200);
        }
        // Past 10 from the middle (from its tenth tick) it goes down by 2 a tick.
        assert_eq!(prim.parts[0].alpha, 35 - 3 * 2);
        for _ in 0..20 {
            prim.tick(200);
        }
        assert!(prim.parts.iter().all(|blade| !blade.live));
    }

    #[test]
    fn the_blades_of_a_run_that_stops_are_slower_and_lower() {
        let mut prim = blades(1, 0.0);
        for _ in 0..12 {
            prim.tick(200);
        }
        let blade = &prim.parts[1];
        assert_eq!((blade.alpha, blade.out), (25 - 3 * 2, 14.0));
        assert!(blade.tall >= 2.0 && blade.tall < 2.2);
    }

    #[test]
    fn a_blade_is_drawn_three_times_a_degree_apart_the_middle_one_tall_and_bright() {
        let mut prim = blades(0, 1.5);
        for _ in 0..6 {
            prim.tick(200);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 12);
        // The first part points east: its middle blade from 7 out to 21, standing on the ground.
        let middle = &rects[1];
        assert_eq!((middle.alpha, middle.colour, middle.adds), (30.0, [255; 3], true));
        assert_eq!((middle.corners[0].0, middle.corners[1].0), ([7.0, 0.0, 0.0], [21.0, 0.0, 0.0]));
        let tall = prim.parts[0].tall;
        assert!((middle.corners[2].0[1] - tall).abs() < 0.001 && (middle.corners[3].0[1] - tall * 0.2).abs() < 0.001);
        // Those beside it are lower and at a third of the alpha, a degree to each side.
        assert_eq!(rects[0].alpha, 10.0);
        assert!((rects[0].corners[2].0[1] - tall * 0.4).abs() < 0.001 && rects[0].corners[1].0[2] < 0.0 && rects[2].corners[1].0[2] > 0.0);
    }
}
