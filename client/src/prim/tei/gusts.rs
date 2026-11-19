//! The kind 119 of the client's primitives: a square of wind lying level that spins and flies off along the
//! ground the way it was sent, three fainter ones trailing it (`CEffectPrim::OnProcess`, `case 119`; drawn by
//! `RenderFloorSquare2`). What the kinds 122, 125 and 126 share with it is here too: the trail, and that
//! drawing.
//!
//! Only the first part of such a primitive moves; the other three are where it was three, six and nine ticks
//! ago. The client measures the way to the place its effect aims at as the first part takes its first tick
//! (`Get_Angle`); here the effect has measured it as it launched the part and put it into `turned`.

use super::*;

impl Prim {
    /// The kinds 119 and 126 (`ring`), which differ in how bright they get: the first part waits while its
    /// count is under one, then spins 30 degrees a tick and flies `heights[1]` a tick along the way it is
    /// turned. Its first tick it only sets off; over the nine after it comes up by 25 a tick (the kind 126:
    /// by 6) and flies 2 in a hundred faster each tick; from its eleventh it goes down by 5 a tick (by 3),
    /// flying 4 in a hundred faster, and is over once it would show less than nothing.
    pub(super) fn blow_as_gust(&mut self, ring: bool) {
        let Some((first, _)) = self.head_and_tail() else { return };
        first.rise = (first.rise + 30) % 360;
        if first.process == 1 {
        } else if first.process < 11 {
            first.alpha = if ring { (first.alpha + 6).min(60) } else { (first.alpha + 25).min(250) };
            first.heights[1] *= 1.02;
        } else {
            first.alpha -= if ring { 3 } else { 5 };
            first.heights[1] *= 1.04;
            first.over_once_gone();
        }
        first.fly_level();
        self.trail_parts();
    }

    /// Every third tick of the first part each part after it takes the place and the spin of the one before
    /// it, and eight tenths of its alpha.
    pub(super) fn trail_parts(&mut self) {
        if self.parts.first().is_none_or(|first| first.process % 3 != 0) {
            return;
        }
        for index in (1..self.parts.len().min(4)).rev() {
            let before = self.parts[index - 1].clone();
            let part = &mut self.parts[index];
            (part.at[0], part.alpha, part.rise) = (before.at[0], (before.alpha as f32 * 0.8) as i32, before.rise);
        }
    }

    /// All the parts go down by 5, each over once it would show less than nothing: how the kinds 122 and 125
    /// end.
    pub(super) fn fade_parts(&mut self) {
        for part in self.parts.iter_mut().take(4) {
            part.alpha -= 5;
            part.over_once_gone();
        }
    }
}

impl Part {
    /// `RenderFloorSquare2`, for one part: a level square about its place (`at[0]`), its corners `out` from
    /// it, the first a quarter of a turn on from its spin (`rise`) and each next a quarter on. `flags[3]` says
    /// how it is laid on: 0 in blue and 1 in green, their light added; 2 white and 3 dark grey, mixed in; any
    /// other is not drawn.
    pub(super) fn spinning_square(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (colour, adds) = match self.flags[3] {
            0 => ([80, 80, 255], true),
            1 => ([80, 255, 80], true),
            2 => ([255; 3], false),
            3 => ([70; 3], false),
            _ => return Vec::new(),
        };
        let corner = |quarter: i32| {
            let angle = (self.rise + quarter * 90).rem_euclid(360) as f32;
            [self.out * cosine(angle) + self.at[0][0], self.at[0][1], self.out * sine(angle) + self.at[0][2]]
        };
        vec![Rect::new(picture, self.alpha, [corner(1), corner(2), corner(3), corner(4)], (0.0, 1.0), colour, adds)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The squares as `TexturedSlashFan` launches them, sent east.
    fn gust() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Gust), "wind");
        prim.duration = 200;
        prim.parts = (0..4).map(|index| {
            let mut part = Part { process: index - 3, out: 20.0, rise: index, ..Part::default() };
            (part.at[0][1], part.heights[1]) = (-10.0, 0.8);
            part
        }).collect();
        prim
    }

    #[test]
    fn the_first_square_waits_sets_off_comes_up_and_flies_faster_and_faster() {
        let mut prim = gust();
        for _ in 0..3 {
            prim.tick(200);
        }
        assert_eq!((prim.parts[0].process, prim.parts[0].at[0][0]), (0, 0.0));
        prim.tick(200);
        // Its first tick it flies its speed and spins, showing nothing yet.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].rise), (0, 30));
        assert!((prim.parts[0].at[0][0] - 0.8).abs() < 1e-5);
        for _ in 0..9 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].alpha, 225);
        assert!((prim.parts[0].heights[1] - 0.8 * 1.02_f32.powi(9)).abs() < 1e-4);
        prim.tick(200);
        assert_eq!(prim.parts[0].alpha, 220);
        // The other parts do not count on: they are where the first was.
        assert_eq!(prim.parts[1].process, -2);
    }

    #[test]
    fn three_fainter_squares_trail_it_and_stay_when_it_is_gone() {
        let mut prim = gust();
        for _ in 0..12 {
            prim.tick(200);
        }
        // At its ninth tick the second took its place, spin and eight tenths of its 200, the third eight
        // tenths of what the second had since the sixth (100), the fourth of what the third had (32).
        assert_eq!([1, 2, 3].map(|index| prim.parts[index].alpha), [160, 80, 25]);
        assert!(prim.parts[1].at[0][0] > prim.parts[2].at[0][0] && prim.parts[2].at[0][0] > prim.parts[3].at[0][0]);
        for _ in 0..60 {
            prim.tick(200);
        }
        assert!(!prim.parts[0].live && prim.parts[1].live);
    }

    #[test]
    fn a_square_lies_level_ten_over_the_ground_blue_and_added() {
        let mut prim = gust();
        for _ in 0..6 {
            prim.tick(200);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // The first alone shows before the third tick of the trail... the second has its place by now.
        assert_eq!(rects.len(), 2);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([80, 80, 255], true, 50.0));
        // Spun to 90 degrees, its first corner is a half turn round: 20 to the west of its middle.
        let middle = prim.parts[0].at[0][0];
        assert!((rects[0].corners[0].0[0] - (middle - 20.0)).abs() < 1e-3 && rects[0].corners[0].0[1] == 10.0);
        prim.parts[0].flags[3] = 3;
        let dark = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((dark[0].colour, dark[0].adds), ([70; 3], false));
    }
}
