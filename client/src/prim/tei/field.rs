//! `RenderFieldPanel`, whole: how the client draws a part of the kinds that are squares facing the screen
//! (`PP_SHIELDBOOMERANG`, `PP_TEIHIT2`, the kind 87 and more). The first part's `flags[2]` picks the colour,
//! the blend and the order of the corners for all the parts of the primitive.

use super::*;

/// What `RenderFieldPanel` makes of the first part's `flags[2]` (and, for its 9, of that part's `flags[3]`):
/// the colour, whether the light is added, and whether the picture is mirrored (its corners taken second,
/// first, fourth, third); nothing for a number it does not draw.
fn look(kind: i8, sub: i8) -> Option<([u8; 3], bool, bool)> {
    Some(match kind {
        0 => ([255; 3], false, true),
        1 => ([150, 150, 250], true, true),
        2 => ([130, 130, 250], true, false),
        3 => ([200, 200, 100], true, false),
        4 => ([255; 3], false, false),
        7 | 18 | 23 | 25 => ([255; 3], true, false),
        8 => ([135, 255, 135], false, false),
        10 => ([250, 250, 150], true, false),
        11 => ([220, 250, 220], true, false),
        12 => ([252, 171, 143], false, false),
        13 => ([250, 100, 100], true, false),
        14 => ([94, 0, 0], false, false),
        15 => ([255, 200, 255], false, true),
        16 => ([0; 3], false, false),
        17 => ([255, 180, 180], false, false),
        24 => ([63, 72, 204], true, false),
        9 => match sub {
            1 => ([255, 135, 135], false, false),
            2 => ([255, 205, 135], false, false),
            3 => ([255, 255, 135], false, false),
            4 => ([255; 3], false, false),
            5 => ([135, 135, 255], false, false),
            6 => ([255; 3], true, false),
            _ => return None,
        },
        _ => return None,
    })
}

impl Part {
    /// `RenderFieldPanel`, for one part of a primitive whose first part's `flags[2]` and `flags[3]` are `kind`
    /// and `sub`: a square that faces the screen about `at[0]`, turned by `round`, from its middle to a
    /// corner `out` and up to a twentieth more or less as the sine of `heights[8]` says. A part whose
    /// `heights[9]` is 10 stands 4 to the right on the screen. The kind 18 shows at half its alpha, and the
    /// kind 25 stands 110 higher. `flags[0]` says which of the primitive's pictures it shows.
    ///
    /// (On one map, `ecl_fild01`, the client holds such a square's alpha to 100; not done here.)
    pub(super) fn field_panel(&self, pictures: &[&'static str], (kind, sub): (i8, i8), right: [f32; 3], up: [f32; 3]) -> Option<Rect> {
        let (colour, adds, mirrored) = look(kind, sub).filter(|_| self.alpha > 0)?;
        let picture = *pictures.get(self.flags[0].max(0) as usize)?;
        let swing = if (0.0..360.0).contains(&self.heights[8]) { self.heights[8] } else { 0.0 };
        let reach = self.out * sine(swing) * 0.05 + self.out;
        let mut place = self.at[0];
        if self.heights[9] == 10.0 {
            place = [place[0] + right[0] * 4.0, place[1] - right[1] * 4.0, place[2] + right[2] * 4.0];
        }
        if kind == 25 {
            place[1] -= 110.0;
        }
        let [first, second, third, fourth] = Self::facing(place, reach, self.round, right, up);
        let corners = if mirrored { [second, first, fourth, third] } else { [first, second, third, fourth] };
        Some(Rect::new(picture, if kind == 18 { self.alpha / 2 } else { self.alpha }, corners, (0.0, 1.0), colour, adds))
    }
}

impl Prim {
    /// The squares of a primitive drawn by `RenderFieldPanel`: its pictures are `pictures`, or `picture` alone.
    pub(super) fn field_panels(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let Some(first) = self.parts.first() else { return Vec::new() };
        let look = (first.flags[2], first.flags[3]);
        let pictures = if self.pictures.is_empty() { std::slice::from_ref(&self.picture) } else { self.pictures };
        self.parts.iter().filter(|part| part.live).filter_map(|part| part.field_panel(pictures, look, right, up)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 1.0, 0.0];

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    fn square(kind: i8) -> Part {
        let mut flags = [0; 21];
        flags[2] = kind;
        Part { alpha: 200, out: 2.0, at: [[5.0, -8.0, 3.0], [0.0; 3], [0.0; 3], [0.0; 3]], flags, ..Part::default() }
    }

    #[test]
    fn the_first_parts_number_gives_the_colour_the_blend_and_the_order_of_the_corners() {
        let plain = square(2).field_panel(&["a"], (2, 0), RIGHT, UP).unwrap();
        assert_eq!((plain.colour, plain.adds, plain.alpha), ([130, 130, 250], true, 200.0));
        // Not turned, its first corner is to the right of its middle, 8 over the ground.
        assert!(close(plain.corners[0].0[0], 7.0) && close(plain.corners[0].0[1], 8.0) && close(plain.corners[0].0[2], 3.0));
        // The next is a quarter on, down the screen.
        assert!(close(plain.corners[1].0[0], 5.0) && close(plain.corners[1].0[1], 6.0));
        let mirrored = square(0).field_panel(&["a"], (0, 0), RIGHT, UP).unwrap();
        assert_eq!((mirrored.colour, mirrored.adds), ([255; 3], false));
        assert_eq!((mirrored.corners[0].0, mirrored.corners[1].0), (plain.corners[1].0, plain.corners[0].0));
        // The kind 18 shows at half its alpha, the kind 25 stands 110 higher, the 9 goes by a second number.
        assert_eq!(square(18).field_panel(&["a"], (18, 0), RIGHT, UP).unwrap().alpha, 100.0);
        assert!(close(square(25).field_panel(&["a"], (25, 0), RIGHT, UP).unwrap().corners[0].0[1], 118.0));
        assert_eq!(square(9).field_panel(&["a"], (9, 5), RIGHT, UP).unwrap().colour, [135, 135, 255]);
        assert!(square(9).field_panel(&["a"], (9, 0), RIGHT, UP).is_none());
        assert!(square(5).field_panel(&["a"], (5, 0), RIGHT, UP).is_none());
    }

    #[test]
    fn a_square_swells_stands_aside_and_shows_the_picture_of_its_number() {
        let mut part = square(2);
        (part.heights[8], part.heights[9], part.flags[0]) = (90.0, 10.0, 1);
        let rect = part.field_panel(&["a", "b"], (2, 0), RIGHT, UP).unwrap();
        assert_eq!(rect.picture, "b");
        // A twentieth wider, and 4 to the right.
        assert!(close(rect.corners[0].0[0], 5.0 + 4.0 + 2.1));
        // One that shows nothing, or a picture the primitive has not, is not drawn.
        part.flags[0] = 2;
        assert!(part.field_panel(&["a", "b"], (2, 0), RIGHT, UP).is_none());
        (part.flags[0], part.alpha) = (0, 0);
        assert!(part.field_panel(&["a", "b"], (2, 0), RIGHT, UP).is_none());
    }
}
