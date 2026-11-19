//! The kind 132 of the client's primitives: upright squares of light through the one the effect is on, each
//! turned its own way, that grow, fade and begin anew turned another way (`CEffectPrim::OnProcess`, `case 132`;
//! drawn by `RenderColorPanel`): the lights of the Circle of Nature and of the other circles of power.

use super::*;

impl Part {
    /// The kind 132, for one part: it waits while its count is under one. Every tick it grows by 2 in a
    /// hundred (`tall`, from 10); past 20 it goes down by 3 a tick, and once nothing of it shows it begins
    /// anew at 10, turned some other way by lot (`turned`): at an alpha of 200 for a kind under 2
    /// (`heights[5]`), at 100 for the others, which are over instead once they have counted past 300. The
    /// square is as large as nine tenths of that and a tenth more or less as a sine goes, a degree a tick
    /// (`round`); that size is kept in `heights[2]`.
    pub(super) fn twinkle(&mut self) {
        if self.waits() {
            return;
        }
        self.round = (self.round + 1) % 360;
        self.tall *= 1.02;
        if self.tall > 20.0 {
            self.alpha -= 3;
            if self.alpha < 1 {
                if self.heights[5] >= 2.0 && self.process > 300 {
                    (self.alpha, self.live) = (0, false);
                } else {
                    (self.tall, self.turned) = (10.0, self.lot(360));
                    self.alpha = if self.heights[5] < 2.0 { 200 } else { 100 };
                }
            }
        }
        self.heights[2] = (self.tall * sine(self.round as f32) * 0.1 + self.tall * 0.9).max(0.0);
    }

    /// `RenderColorPanel`, for one part: an upright square as large as `heights[2]`, its middle `heights[4]`
    /// over the effect's place, turned about the upright by `turned`; its light added. Its colour is its
    /// kind's (`heights[5]`: 2 green, 3 blue, 4 yellow, 5 red, 6 purple, 7 cyan, 8 grey, any other blue) in
    /// the shade of its number among the four (`heights[3]`: 105, 5, 55 and 155 in what the colour lacks; of
    /// the grey the last is white).
    pub(super) fn colour_panel(&self, picture: &'static str) -> Vec<Rect> {
        let size = self.heights[2];
        if self.alpha <= 0 || self.process <= 0 || size <= 0.0 {
            return Vec::new();
        }
        let number = (self.heights[3] as usize).min(3);
        let shade = [105, 5, 55, 155][number];
        let colour = match self.heights[5] as i32 {
            2 => [shade, 255, shade],
            4 => [255, 255, shade],
            5 => [255, shade, shade],
            6 => [255, shade, 255],
            7 => [shade, 255, 255],
            8 if number == 3 => [255; 3],
            8 => [shade; 3],
            _ => [shade, shade, 255],
        };
        let half = size * 0.5;
        let top = half - self.heights[4];
        let (cos, sin) = (cosine(self.turned as f32), sine(self.turned as f32));
        let (one, other) = ([half * cos, top, half * sin], [-half * cos, top, -half * sin]);
        let corners = [one, other, [other[0], top - size, other[2]], [one[0], top - size, one[2]]];
        vec![Rect::new(picture, self.alpha, corners, (0.0, 1.0), colour, true)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four squares as `TexturedTwinkleRing` launches them for its level.
    fn squares(level: f32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Twinkle), "freeze_a.bmp");
        prim.duration = 400;
        prim.parts = (0..4)
            .map(|index| {
                let mut part = Part { process: index * -20, round: 0, tall: 10.0, turned: 90, alpha: 100, seed: index as u32 + 1, ..Part::default() };
                (part.heights[3], part.heights[4], part.heights[5]) = (index as f32, 10.0, level);
                part
            })
            .collect();
        prim
    }

    #[test]
    fn a_square_grows_fades_and_begins_anew_until_it_has_counted_past_300() {
        let mut prim = squares(2.0);
        prim.tick(400);
        assert!((prim.parts[0].tall - 10.2).abs() < 1e-4 && prim.parts[1].tall == 10.0);
        // 1.02 to the 36th is the first past 2: it goes down by 3 a tick from its 36th tick.
        for _ in 1..36 {
            prim.tick(400);
        }
        assert_eq!(prim.parts[0].alpha, 97);
        for _ in 0..33 {
            prim.tick(400);
        }
        // Having shown nothing it is back at 10 and at 100, turned by lot.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].tall), (100, 10.0));
        for _ in 0..400 {
            prim.tick(400);
        }
        assert!(prim.parts.iter().all(|part| !part.live));
        // Of a kind under 2 it begins anew at 200 and for ever.
        let mut other = squares(1.0);
        for _ in 0..69 {
            other.tick(400);
        }
        assert_eq!(other.parts[0].alpha, 200);
    }

    #[test]
    fn it_stands_upright_about_ten_over_the_ground_in_the_green_of_its_number() {
        let mut prim = squares(2.0);
        for _ in 0..21 {
            prim.tick(400);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 2);
        assert_eq!((rects[0].colour, rects[1].colour, rects[0].adds, rects[0].alpha), ([105, 255, 105], [5, 255, 5], true, 100.0));
        // Turned a quarter, it reaches north and south of the place, as far up as down from 10 over it.
        let size = prim.parts[0].heights[2];
        let (first, third) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert!(first[0].abs() < 1e-3 && (first[2] - size * 0.5).abs() < 1e-3 && (first[1] - (10.0 - size * 0.5)).abs() < 1e-3);
        assert!((third[2] + size * 0.5).abs() < 1e-3 && (third[1] - (10.0 + size * 0.5)).abs() < 1e-3);
    }
}
