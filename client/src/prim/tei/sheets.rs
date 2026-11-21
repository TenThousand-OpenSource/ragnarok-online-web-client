//! The kind 79 of the client's primitives: upright sheets of water whose three pictures run down them, the
//! sheets of a waterfall (`CEffectPrim::OnProcess`, `case 79`, drawn by `RenderScrollColumn`).

use super::*;

impl Part {
    /// The kind 79, for one part: it only counts its ticks.
    pub(super) fn run_as_sheet(&mut self) {
        self.process += 1;
    }

    /// `RenderScrollColumn`, for one part: a sheet as wide as its `out`, 160 high from the effect's place up
    /// and `at[0]`'s third figure before it, made of five pieces of 40 that show the primitive's three
    /// pictures one after the other. Every `tall` ticks the pieces have slid down by one piece (the lowest is
    /// cut off at the ground and the highest at the top, with the picture) and each shows the next picture.
    /// A sheet whose `flags[1]` is 1 or 10 runs the other way, blue (pink for the 10), its light added; any
    /// other is the pictures' own colours mixed in by its alpha. One whose `flags[0]` is 1 stands across
    /// north and south, not east and west; one whose `flags[2]` is over 10 is lower, by 10 over that number.
    /// (A sheet of no `tall` is one piece that does not run.)
    pub(super) fn sheet(&self, pictures: &[&'static str]) -> Vec<Rect> {
        if self.alpha <= 0 || pictures.len() < 3 {
            return Vec::new();
        }
        let (frames, cycle, mut scroll) = match self.tall as i32 {
            0 => (1, 0, 40.0),
            frames => (frames, self.process % (frames * 3), (self.process % frames) as f32 * 40.0 / frames as f32),
        };
        let back = matches!(self.flags[1], 1 | 10);
        if back {
            scroll = 40.0 - scroll;
        }
        let (colour, adds) = match self.flags[1] {
            1 => ([55, 55, 255], true),
            10 => ([255, 55, 255], true),
            _ => ([255; 3], false),
        };
        let (side, depth, shift, cut) = (self.out * 0.5, self.at[0][2], cycle / frames, scroll * 0.025);
        let piece = |number: usize| {
            let (mut low, mut high) = (scroll - self.heights[number], scroll - self.heights[number + 1]);
            if self.flags[2] > 10 {
                (low, high) = (low * 10.0 / f32::from(self.flags[2]), high * 10.0 / f32::from(self.flags[2]));
            }
            let picture = if back { (number as i32 % 3 - shift).rem_euclid(3) } else { (number as i32 % 3 + shift) % 3 };
            // The picture's lower edge is on the higher end of a piece.
            let (mut at_low, mut at_high) = (0.0, 1.0);
            if number == 0 {
                (low, at_low) = (low - (low - high) * cut, cut);
            } else if number == 4 {
                (high, at_high) = ((low - high) * (1.0 - cut) + high, cut);
            }
            let corner = |across: f32, down: f32| if self.flags[0] == 1 { [depth, down, across] } else { [across, down, depth] };
            let corners = [corner(-side, high), corner(side, high), corner(side, low), corner(-side, low)];
            Rect::new(pictures[picture as usize], self.alpha, corners, (0.0, 1.0), colour, adds).down(at_low, at_high)
        };
        (0..if frames == 1 { 1 } else { 5 }).map(piece).collect()
    }
}

impl Rect {
    /// The rectangle showing of its picture's height only from `top` (at its last two corners) to `bottom`
    /// (at its first two): the two figures `RenderTeiRect` is given for that (`ty1`, `ty2`).
    fn down(mut self, top: f32, bottom: f32) -> Self {
        for (_, shown) in &mut self.corners {
            shown[1] = if shown[1] == 0.0 { top } else { bottom };
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PICTURES: [&str; 3] = ["waterfall11.tga", "waterfall12.tga", "waterfall13.tga"];

    /// The first sheet of `CRagEffect::Waterfall`, and of `WaterfallSheets` if `blue`.
    fn sheets(blue: bool, across: i8) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Sheet), PICTURES[0]);
        (prim.duration, prim.pictures) = (99_999_999, &PICTURES);
        let mut part = Part { alpha: if blue { 120 } else { 80 }, out: 36.0, tall: 80.0, ..Part::default() };
        (part.flags[0], part.flags[1], part.flags[2], part.at[0][2]) = (across, i8::from(blue), if blue { 10 } else { 0 }, -1.0);
        part.heights[..20].iter_mut().enumerate().for_each(|(number, height)| *height = (number * 40) as f32);
        prim.parts = vec![part];
        prim
    }

    /// The lowest and the highest a rectangle reaches, and the part of its picture's height it shows.
    fn span(rect: &Rect) -> (f32, f32, f32, f32) {
        (rect.corners[2].0[1], rect.corners[0].0[1], rect.corners[2].1[1], rect.corners[0].1[1])
    }

    #[test]
    fn a_sheet_is_five_pieces_from_the_ground_up_that_slide_down_and_change_pictures() {
        let mut prim = sheets(false, 0);
        for _ in 0..20 {
            prim.tick(99_999_999);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 5);
        // A quarter of a piece down after 20 of 80 ticks: the lowest is cut at the ground, the highest at 160.
        assert_eq!(span(&rects[0]), (0.0, 30.0, 0.25, 1.0));
        assert_eq!(span(&rects[1]), (30.0, 70.0, 0.0, 1.0));
        assert_eq!(span(&rects[4]), (150.0, 160.0, 0.0, 0.25));
        assert_eq!(rects.iter().map(|rect| rect.picture).collect::<Vec<_>>(), [0, 1, 2, 0, 1].map(|picture| PICTURES[picture]));
        // 18 to either side, east and west, 1 to the south; its own colours, mixed in at 80.
        assert_eq!((rects[1].corners[0].0, rects[1].corners[1].0), ([-18.0, 70.0, -1.0], [18.0, 70.0, -1.0]));
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([255; 3], false, 80.0));
        // After a whole piece each shows the next picture.
        for _ in 0..60 {
            prim.tick(99_999_999);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((span(&rects[0]), rects[0].picture, rects[2].picture), ((0.0, 40.0, 0.0, 1.0), "waterfall12.tga", "waterfall11.tga"));
    }

    #[test]
    fn a_blue_sheet_runs_the_other_way_and_one_turned_stands_north_and_south() {
        let mut prim = sheets(true, 1);
        for _ in 0..20 {
            prim.tick(99_999_999);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(span(&rects[0]), (0.0, 10.0, 0.75, 1.0));
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([55, 55, 255], true, 120.0));
        assert_eq!((rects[1].corners[0].0, rects[1].corners[1].0), ([-1.0, 50.0, -18.0], [-1.0, 50.0, 18.0]));
        for _ in 0..60 {
            prim.tick(99_999_999);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.iter().map(|rect| rect.picture).collect::<Vec<_>>(), [2, 0, 1, 2, 0].map(|picture| PICTURES[picture]));
    }

    #[test]
    fn a_sheet_told_a_number_over_ten_is_lower_by_it() {
        let mut prim = sheets(false, 0);
        prim.parts[0].flags[2] = 20;
        prim.tick(99_999_999);
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((span(&rects[1]).0, span(&rects[1]).1, span(&rects[0]).0), (19.75, 39.75, 0.0));
    }
}
