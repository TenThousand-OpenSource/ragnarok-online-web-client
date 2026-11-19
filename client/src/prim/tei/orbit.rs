//! The kind `PP_INTIMIDATE` (43) of the client's primitives: up to four squares that fly out from round a
//! place, each standing upright and spinning about the upright like a coin, or facing the screen and turning
//! (`PrimIntimidate`, drawn by `RenderOrbitSquare`): the coins of Steal Coin, the gear a Strip tears off.

use super::*;

impl Part {
    /// `PrimIntimidate`, for one part: once its count is over nothing it stands `heights[0]` further out
    /// every tick; over its first ten ticks it comes up by 25 a tick (by 15 for the number 0, `flags[0]`),
    /// and after the 25th goes down by 7 a tick. Every tick it spins 5 degrees on (`heights[2]`); the number 2
    /// spins 3 and turns 3 degrees on the screen (`turned`).
    pub(super) fn fly_as_coin(&mut self) {
        self.process += 1;
        if self.process > 0 {
            self.out += self.heights[0];
        }
        if self.process <= 10 && self.process > 0 {
            self.alpha += if self.flags[0] == 0 { 15 } else { 25 };
        } else if self.process > 25 {
            self.alpha = (self.alpha - 7).max(0);
        }
        if self.flags[0] == 2 {
            self.turned += 3;
        }
        self.turned = under_a_round(self.turned);
        self.heights[2] += if self.flags[0] == 2 { 3.0 } else { 5.0 };
        if self.heights[2] >= 360.0 {
            self.heights[2] -= 360.0;
        }
    }

    /// `RenderOrbitSquare`, for one part: a square whose middle is `out` from `at[0]`, up or down by `round`
    /// and round by `rise` (so that parts drawn by lot stand on a ball about the place), `heights[1]` from its
    /// middle to a corner, its first corner `turned` degrees round. It stands upright in a plane that
    /// `heights[2]` turns about the upright; of the numbers 2, 8 and 9 (`flags[0]`) it faces the screen
    /// instead. The number gives the colour; its light is added. Any number past 9 shows nothing.
    ///
    /// (The numbers 6 and 7 the client draws over everything; here they are drawn where they stand.)
    pub(super) fn coin(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Option<Rect> {
        let colour = match self.flags[0] {
            0 => [100, 150, 255],
            1 => [250, 250, 155],
            2 | 6 => [250, 100, 100],
            3 | 8 => [50, 50, 255],
            4 => [255, 50, 50],
            5 => [50, 255, 50],
            7 => [5, 5, 255],
            9 => [70; 3],
            _ => return None,
        };
        if self.alpha <= 0 {
            return None;
        }
        let level = cosine(self.round as f32) * self.out;
        let off = [level * cosine(self.rise as f32), sine(self.round as f32) * self.out, level * sine(self.rise as f32)];
        let middle: [f32; 3] = std::array::from_fn(|axis| self.at[0][axis] + off[axis]);
        let reach = self.heights[1];
        let corners = if matches!(self.flags[0], 2 | 8 | 9) {
            Self::facing(middle, reach, self.turned, right, up)
        } else {
            let (spin_cos, spin_sin) = (cosine(self.heights[2]), sine(self.heights[2]));
            std::array::from_fn(|quarter| {
                let angle = ((self.turned + quarter as i32 * 90) % 360) as f32;
                let across = reach * cosine(angle);
                [middle[0] + spin_cos * across, middle[1] + reach * sine(angle), middle[2] + spin_sin * across]
            })
        };
        Some(Rect::new(picture, self.alpha, corners, (0.0, 1.0), colour, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP, close};

    /// A coin as `TexturedSpray` launches one (`number` 1), or a thing as `TexturedSprayKind` does (2).
    fn coins(number: i8, wait: i32) -> Prim {
        let mut coins = Prim::new(Shape::Tei(Tei::Intimidate), "coin_a.bmp");
        coins.duration = 300;
        let mut part = Part { process: -wait, out: 12.0, ..Part::default() };
        (part.heights[0], part.heights[1], part.flags[0]) = (1.5, 3.5, number);
        coins.parts.push(part);
        coins
    }

    #[test]
    fn a_coin_waits_then_flies_out_comes_up_and_fades_spinning_all_the_while() {
        let mut flying = coins(1, 18);
        for _ in 0..18 {
            flying.tick(300);
        }
        let part = &flying.parts[0];
        assert_eq!((part.alpha, part.out, part.heights[2]), (0, 12.0, 90.0));
        assert!(flying.rects(RIGHT, UP).is_empty());
        for _ in 0..10 {
            flying.tick(300);
        }
        let part = &flying.parts[0];
        assert_eq!((part.alpha, part.out, part.turned), (250, 27.0, 0));
        for _ in 0..16 {
            flying.tick(300);
        }
        assert_eq!(flying.parts[0].alpha, 243);
        // A thing torn off comes up as fast and turns on the screen, spinning slower.
        let mut things = coins(2, 0);
        for _ in 0..4 {
            things.tick(300);
        }
        assert_eq!((things.parts[0].alpha, things.parts[0].turned, things.parts[0].heights[2]), (100, 12, 12.0));
    }

    #[test]
    fn a_coin_stands_upright_out_from_its_place_and_a_thing_faces_the_screen() {
        let mut part = Part { alpha: 200, out: 10.0, round: 0, rise: 90, ..Part::default() };
        (part.heights[1], part.flags[0]) = (2.0, 1);
        part.at[0] = [1.0, -3.0, 0.0];
        // Its middle is 10 north of the place; not spun, its plane lies east and west.
        let rect = part.coin("coin_a.bmp", RIGHT, UP).unwrap();
        assert_eq!((rect.colour, rect.adds, rect.alpha), ([250, 250, 155], true, 200.0));
        assert!(close(rect.corners[0].0[0], 3.0) && close(rect.corners[0].0[1], 3.0) && close(rect.corners[0].0[2], 10.0));
        assert!(close(rect.corners[1].0[0], 1.0) && close(rect.corners[1].0[1], 1.0) && close(rect.corners[1].0[2], 10.0));
        // Spun a quarter, it lies north and south.
        part.heights[2] = 90.0;
        let spun = part.coin("coin_a.bmp", RIGHT, UP).unwrap();
        assert!(close(spun.corners[0].0[0], 1.0) && close(spun.corners[0].0[2], 12.0));
        // Raised by `round`, the middle goes down the client's y for an angle over nothing.
        (part.round, part.flags[0]) = (270, 2);
        let facing = part.coin("sword.bmp", [0.0, 0.0, 1.0], UP).unwrap();
        assert_eq!(facing.colour, [250, 100, 100]);
        assert!(close(facing.corners[0].0[1], 13.0) && close(facing.corners[0].0[2], 2.0) && close(facing.corners[0].0[0], 1.0));
        part.flags[0] = 10;
        assert!(part.coin("sword.bmp", RIGHT, UP).is_none());
    }
}
