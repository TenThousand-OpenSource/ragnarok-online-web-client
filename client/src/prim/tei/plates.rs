//! The kind 44 of the client's primitives: up to four long plates of light that shoot out from a place every
//! way, each along a line of its own (`PrimExpandPulse`, drawn by `RenderRotatedPlate`): the streaks that
//! burst from the one struck by Asura Strike and Raid.

use super::*;

impl Part {
    /// `PrimExpandPulse`, for one part: once its count is over nothing it stands `heights[0]` further out
    /// every tick. From its eleventh to its twentieth tick it comes up by 15 a tick; at every other tick it
    /// goes down by 5, to nothing at the least.
    ///
    /// (With the option to show fewer effects on, `TT_MIN_EFFECT_ON_OFF`, the client brings a part whose
    /// `flags[1]` is 24 up by 8 a tick; what is here is the option off.)
    pub(super) fn shoot_as_plate(&mut self) {
        self.process += 1;
        if self.process > 0 {
            self.out += self.heights[0];
        }
        if (11..=20).contains(&self.process) {
            self.alpha += 15;
        } else {
            self.alpha = (self.alpha - 5).max(0);
        }
    }

    /// `RenderRotatedPlate`, for one part: a plate along a line through the effect's place that `round` tips
    /// from level (down the client's y for an angle over nothing) and `rise` turns round the upright. Its
    /// middle is `out` along the line; it is `flags[1]` times `heights[1]` long to each side of its middle
    /// and `heights[1]` wide to each side of the line. `flags[0]` gives the colour: yellow (0), white (1) and
    /// blue (2) with their light added, any other a pale blue mixed in.
    ///
    /// (The number 2 the client draws over everything; here it is drawn where it stands.)
    pub(super) fn plate(&self, picture: &'static str) -> Option<Rect> {
        if self.alpha <= 0 {
            return None;
        }
        let (tip_cos, tip_sin) = (cosine(self.round as f32), sine(self.round as f32));
        let (turn_cos, turn_sin) = (cosine(self.rise as f32), sine(self.rise as f32));
        let wide = self.heights[1];
        let long = f32::from(self.flags[1]) * wide;
        let corner = |along: f32, aside: f32| {
            let level = tip_cos * along - tip_sin * aside;
            [level * turn_cos, tip_sin * along + tip_cos * aside, level * turn_sin]
        };
        let (far, near) = (self.out + long, self.out - long);
        let (colour, adds) = match self.flags[0] {
            0 => ([225, 225, 80], true),
            1 => ([255; 3], true),
            2 => ([65, 184, 240], true),
            _ => ([200, 200, 255], false),
        };
        Some(Rect::new(picture, self.alpha, [corner(far, -wide), corner(far, wide), corner(near, wide), corner(near, -wide)], (0.0, 1.0), colour, adds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::close;

    /// A streak as `TexturedScatter` launches one with `lens1.tga` for Asura Strike.
    fn streak(wait: i32) -> Prim {
        let mut streaks = Prim::new(Shape::Tei(Tei::ExpandPulse), "lens1.tga");
        streaks.duration = 300;
        let mut part = Part { process: -wait, out: -10.0, rise: 0, round: 0, ..Part::default() };
        (part.heights[0], part.heights[1], part.flags[0], part.flags[1]) = (3.0, 0.7, 1, 24);
        streaks.parts.push(part);
        streaks
    }

    #[test]
    fn a_streak_waits_shoots_out_and_shows_for_a_while() {
        let mut streaks = streak(10);
        for _ in 0..10 {
            streaks.tick(300);
        }
        assert_eq!((streaks.parts[0].out, streaks.parts[0].alpha), (-10.0, 0));
        for _ in 0..10 {
            streaks.tick(300);
        }
        // Ten ticks out, still dark.
        assert_eq!((streaks.parts[0].out, streaks.parts[0].alpha), (20.0, 0));
        assert!(streaks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
        for _ in 0..10 {
            streaks.tick(300);
        }
        assert_eq!((streaks.parts[0].out, streaks.parts[0].alpha), (50.0, 150));
        for _ in 0..30 {
            streaks.tick(300);
        }
        assert_eq!(streaks.parts[0].alpha, 0);
    }

    #[test]
    fn a_plate_lies_along_its_line_long_and_narrow() {
        let mut streaks = streak(0);
        (streaks.parts[0].out, streaks.parts[0].alpha) = (50.0, 150);
        let rects = streaks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (1, [255; 3], true));
        // Level and unturned it lies along the east: from 33.2 to 66.8 out, 0.7 over and under the line.
        let corners = rects[0].corners;
        assert!(close(corners[0].0[0], 66.8) && close(corners[0].0[1], 0.7) && close(corners[0].0[2], 0.0));
        assert!(close(corners[1].0[0], 66.8) && close(corners[1].0[1], -0.7));
        assert!(close(corners[2].0[0], 33.2) && close(corners[3].0[0], 33.2));
        // Tipped a quarter it stands under the place (the client's y points down), turned it points north.
        streaks.parts[0].round = 90;
        let down = streaks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(close(down[0].corners[0].0[1], -66.8));
        (streaks.parts[0].round, streaks.parts[0].rise) = (0, 90);
        let north = streaks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(close(north[0].corners[0].0[2], 66.8) && close(north[0].corners[0].0[0], 0.0));
        // Any number past 2 is a pale blue, mixed in.
        streaks.parts[0].flags[0] = 3;
        let pale = streaks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((pale[0].colour, pale[0].adds), ([200, 200, 255], false));
    }
}
