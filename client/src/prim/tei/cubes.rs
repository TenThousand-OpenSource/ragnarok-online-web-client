//! `PP_HERMODE` (114) of the client's primitives: a small box of light over the place of the effect that
//! turns, breathes and goes up and down (`CEffectPrim::OnProcess`, `case PP_HERMODE`; drawn by
//! `RenderHermode`): what stands on a cell of Hermode's Rod.

use super::*;

impl Part {
    /// The kind 114, for the first part: it comes up by 2 a tick to 140, and its breath (`rise`) and its turn
    /// (`turned`) go on a degree a tick.
    pub(super) fn turn_as_cube(&mut self) {
        if self.alpha < 140 {
            self.alpha += 2;
        }
        self.rise = under_a_round(self.rise + 1);
        self.turned = under_a_round(self.turned + 1);
    }

    /// `RenderHermode`, for one part: a box whose floor is a level square, its corners a quarter of a turn
    /// apart from where the part is turned to, each eight tenths of its reach (`out`) from the middle and up
    /// to two tenths more as its breath goes; the floor is `heights[0]` under the effect's place (over it for
    /// a height under nothing) and up to 2 more or less as the sine of its turn goes, and the box is as tall
    /// as the part reaches. Its six sides have their light added, each in a blue of its own (the floor and the
    /// lid 250, the walls 220, 190, 130 and 160) over as much red and green as `flags[0]` says.
    pub(super) fn cube(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let radius = self.out * 0.8 + self.out * 0.1 * (sine(self.rise as f32) + 1.0);
        let floor = self.heights[0] + sine(self.turned as f32) * 2.0;
        let feet: [[f32; 3]; 4] = std::array::from_fn(|quarter| {
            let angle = ((self.turned + quarter as i32 * 90) % 360) as f32;
            [cosine(angle) * radius, floor, sine(angle) * radius]
        });
        let tops = feet.map(|foot| [foot[0], foot[1] - self.out, foot[2]]);
        let dark = self.flags[0] as u8;
        let side = |corners: [[f32; 3]; 4], blue: u8| Rect::new(picture, self.alpha, corners, (0.0, 1.0), [dark, dark, blue], true);
        let wall = |from: usize, blue: u8| {
            let to = (from + 1) % 4;
            side([feet[from], feet[to], tops[to], tops[from]], blue)
        };
        vec![side(feet, 250), wall(0, 220), wall(1, 190), wall(2, 130), wall(3, 160), side(tops, 250)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP, close};

    /// The box as `TexturedSpin` launches it, its breath and its turn begun at nothing.
    fn hermode() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Hermode), "white02.bmp");
        (prim.duration, prim.size) = (29999, 5.0);
        let mut part = Part { out: 1.6, ..Part::default() };
        (part.heights[0], part.flags[0]) = (-12.0, 30);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn the_box_comes_up_turns_and_goes_up_and_down() {
        let mut prim = hermode();
        assert!(prim.rects(RIGHT, UP).is_empty());
        for _ in 0..90 {
            prim.tick(29999);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[0].turned, prim.parts[0].rise), (140, 90, 90));
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 6);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([30, 30, 250], true, 140.0));
        assert_eq!([1, 2, 3, 4, 5].map(|side| rects[side].colour[2]), [220, 190, 130, 160, 250]);
        // A quarter of a turn on, at the top of its breath: its first corner is its whole reach to the north,
        // and its floor is 2 lower than it was put, 10 over the ground.
        let first = rects[0].corners[0].0;
        assert!(first[0].abs() < 0.001 && close(first[1], 10.0) && close(first[2], 1.6));
        // The lid is as far over the floor as the part reaches.
        assert!(close(rects[5].corners[0].0[1], 11.6) && close(rects[1].corners[2].0[1], 11.6));
        for _ in 0..180 {
            prim.tick(29999);
        }
        // Half a turn on it is at the bottom of its breath and the top of its swing.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].turned), (140, 270));
        let floor = prim.rects(RIGHT, UP)[0].corners[0].0;
        assert!(close(floor[1], 14.0) && close(floor[2], -1.28));
    }
}
