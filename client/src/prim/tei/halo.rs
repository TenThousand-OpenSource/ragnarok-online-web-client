//! The kind 39 of the client's primitives: a square of light lying level over the ground that breathes or
//! turns (`CEffectPrim::OnProcess`, `case 39`), drawn by `RenderSizedSquare`. That drawing, for a part kept
//! whole, is here (`Part::sized_square`).

use super::*;

impl Part {
    /// The kind 39, for one part; `heights[4]` says which square it is. Of the kind 4 it turns 6 degrees a
    /// tick, of the kinds 10, 11 and 12 by 3. Of the kind 2 its breath (`rise`) goes on a degree a tick; of
    /// the kind 10 it draws in by 0.3 a tick over the first half of its turn and out again over the second;
    /// of the kinds 11 and 12 neither; of every other its breath goes on 3 degrees a tick.
    pub(super) fn breathe_as_halo(&mut self) {
        let kind = self.heights[4];
        if kind == 4.0 {
            self.turned += 6;
        } else if kind == 10.0 || kind == 11.0 || kind == 12.0 {
            self.turned += 3;
        }
        self.turned = under_a_round(self.turned);
        if kind == 2.0 {
            self.rise += 1;
        } else if kind == 10.0 {
            self.out += if self.turned < 181 { -0.3 } else { 0.3 };
        } else if kind != 11.0 && kind != 12.0 {
            self.rise += 3;
        }
        self.rise = under_a_round(self.rise);
    }

    /// `RenderSizedSquare`, for the part of that number of a primitive of that `size`: a level square
    /// `heights[0]` under the effect's place (over it for a height under nothing), its four corners a quarter
    /// of a turn apart, each eight tenths of the part's reach (`out`) from the middle and up to two tenths
    /// more as the sine of its breath (`rise`) says. The first corner is where the part is turned to
    /// (`turned`) for a primitive of size 0, 10, 100, 110 or more, or of size 4 with a part of the kind 4;
    /// else 23 degrees on for each part before it. The colour and the blend are the size's: white and added
    /// unless said here (the sizes 100 and 101 are drawn twice).
    pub(super) fn sized_square(&self, picture: &'static str, size: f32, number: i32) -> Vec<Rect> {
        let by_turn = size == 100.0 || size >= 110.0 || (size == 4.0 && self.heights[4] == 4.0) || size == 10.0 || size == 0.0;
        let first = if by_turn { self.turned } else { number * 23 };
        let reach = (sine(self.rise as f32) + 1.0) * (self.out * 0.1) + self.out * 0.8;
        let corner = |quarter: i32| {
            let angle = (first + quarter * 90).rem_euclid(360) as f32;
            [cosine(angle) * reach, self.heights[0], sine(angle) * reach]
        };
        let corners = [corner(0), corner(1), corner(2), corner(3)];
        let (colour, adds, times) = match size as i32 {
            101 => ([255, 0, 0], true, 2),
            102 => ([255, 255, 0], true, 1),
            111 => ([55, 55, 255], true, 1),
            112 => ([105, 225, 105], true, 1),
            100 => ([255; 3], true, 2),
            110 | 0 => ([255; 3], false, 1),
            113 => ([5, 5, 255], true, 1),
            114 => ([155, 5, 5], true, 1),
            15 => ([35, 255, 35], true, 1),
            17 => ([105, 105, 255], true, 1),
            18 => ([35, 35, 255], true, 1),
            _ => ([255; 3], true, 1),
        };
        vec![Rect::new(picture, self.alpha, corners, (0.0, 1.0), colour, adds); times]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The square of Land Protector as `TexturedHalo` launches it with its kind 1, its breath begun at nothing.
    fn halo(kind: f32, size: f32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Halo), "aaa copy.bmp");
        (prim.duration, prim.size) = (29999, size);
        let mut heights = [0.0; 21];
        (heights[0], heights[4]) = (-2.0, kind);
        prim.parts.push(Part { out: 7.0, alpha: 100, heights, ..Part::default() });
        prim
    }

    #[test]
    fn the_square_of_land_protector_breathes_between_eight_and_ten_tenths_of_its_reach() {
        let mut prim = halo(1.0, 5.0);
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 1);
        // Its corners to the east, north, west and south, 2 over the ground, nine tenths of 7 out at first.
        assert!((rects[0].corners[0].0[0] - 6.3).abs() < 0.001 && rects[0].corners[0].0[1] == 2.0);
        assert!((rects[0].corners[1].0[2] - 6.3).abs() < 0.001 && (rects[0].corners[2].0[0] + 6.3).abs() < 0.001);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([255; 3], true, 100.0));
        for _ in 0..30 {
            prim.tick(29999);
        }
        // Three degrees a tick: a quarter of its breath on, it reaches its whole 7.
        assert_eq!((prim.parts[0].rise, prim.parts[0].turned), (90, 0));
        assert!((prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].corners[0].0[0] - 7.0).abs() < 0.001);
    }

    #[test]
    fn the_other_squares_turn_and_are_coloured_by_their_primitives_size() {
        let mut prim = halo(4.0, 4.0);
        for _ in 0..15 {
            prim.tick(29999);
        }
        assert_eq!((prim.parts[0].turned, prim.parts[0].rise), (90, 45));
        // Of size 4 and kind 4 its first corner is where it is turned to: the north.
        assert!(prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].corners[0].0[0].abs() < 0.001);
        let mut drawn_in = halo(10.0, 10.0);
        for _ in 0..10 {
            drawn_in.tick(29999);
        }
        assert!((drawn_in.parts[0].out - 4.0).abs() < 0.001 && drawn_in.parts[0].rise == 0);
        assert_eq!(halo(1.0, 111.0).rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].colour, [55, 55, 255]);
        assert_eq!(halo(1.0, 100.0).rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).len(), 2);
        assert!(!halo(1.0, 0.0).rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].adds);
    }
}
