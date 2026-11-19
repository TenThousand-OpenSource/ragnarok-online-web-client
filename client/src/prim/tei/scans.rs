//! The kind 135 of the client's primitives: a thin line of light from the place of the effect that sweeps
//! round it, with sheets of light trailing behind it (`PrimTrailShift`, drawn by `RenderJitterSweeps`): the
//! beam of Infrared Scan. And `PP_ELECTRIC` (101), which is such a line that reaches out further and further
//! and does not sweep (`CEffectPrim::OnProcess`, `case PP_ELECTRIC`; drawn by `RenderLightningArc`): the arcs
//! of Magnetic Field.

use super::*;

impl Part {
    /// `PrimTrailShift`, for the part of that number: it stays 10 over the effect's place. Once its count is
    /// over nothing and over ten times its number, the way it points (`rise`) turns 2 degrees a tick, with
    /// the clock for a part whose `alpha_top` is 1 and against it for any other; after its sixtieth tick it
    /// goes down by 6 a tick. At its first tick its line is laid out (`heights`): how far along the sweep
    /// each of its points is, 180 at the first two and 9 less at each next, as many points as `at[1][1]`
    /// says.
    pub(super) fn shift_as_trail(&mut self, index: i32) {
        self.process += 1;
        self.at[0] = [0.0, -10.0, 0.0];
        if self.process < 1 {
            return;
        }
        if index * 10 < self.process {
            self.rise += if self.alpha_top == 1 { 2 } else { -2 };
        }
        self.rise = self.rise.rem_euclid(360);
        if self.process > 60 {
            self.alpha -= 6;
        }
        self.alpha = self.alpha.max(0);
        if self.process == 1 {
            self.lay_out_line();
        }
    }

    /// The line of a part laid out, as the kinds 101 and 135 do at a part's first tick (`heights`): how far
    /// along the sweep each of its points is, 180 at the first two and 9 less at each next, as many points
    /// as `at[1][1]` says.
    fn lay_out_line(&mut self) {
        for _ in 0..21 {
            self.heights[0] += 9.0;
            if self.heights[0] > 180.0 {
                self.heights[0] = 180.0;
                continue;
            }
            for point in (1..=(self.at[1][1] as usize).min(20)).rev() {
                self.heights[point] = self.heights[point - 1];
                self.flags[point] = self.flags[point - 1];
            }
        }
    }

    /// The line of one part as `RenderJitterSweeps` lays it out: the two ends of the piece of that number. A
    /// point is as far out along the part's way (`rise`) as its place along the sweep says (a ninetieth of
    /// the part's reach, `out`, to a degree), lifted by 2 and by the sine of that place taken `at[1][0]`
    /// times the reach, and shaken by up to 0.3 every way. The client draws the shake by lot at every
    /// drawing; here it is drawn from the part's own number and count, so it changes once a tick.
    fn piece_of_line(&self, piece: usize) -> ([f32; 3], [f32; 3]) {
        let mut lots = Part { seed: self.seed ^ (self.process as u32).wrapping_mul(2_654_435_761) ^ (piece as u32).wrapping_mul(40_503), ..Part::default() };
        let mut point = |along: f32| {
            let shake: [f32; 3] = std::array::from_fn(|_| lots.lot(7) as f32 * 0.1 - 0.3);
            let far = if along > 0.0 { along / 90.0 * self.out } else { 0.0 };
            [
                cosine(self.rise as f32) * far + shake[0] + self.at[0][0],
                -self.out * sine(along) * self.at[1][0] + shake[1] - 2.0 + self.at[0][1],
                sine(self.rise as f32) * far + shake[2] + self.at[0][2]
            ]
        };
        (point(self.heights[piece]), point(self.heights[piece + 1]))
    }
}

impl Part {
    /// The piece of that number of a part's line as a tube of three sides, `tall` thick: its four edges (the
    /// last is the first again), each as the two ends of the piece moved out from the line, a third of a
    /// turn apart about it.
    fn line_tube(&self, piece: usize) -> [([f32; 3], [f32; 3]); 4] {
        let (start, end) = self.piece_of_line(piece);
        let facing = ((angle_of(start[0] - end[0], start[2] - end[2]) + 90) % 360) as f32;
        [0.0, 120.0, 240.0, 360.0].map(|turn: f32| {
            let (lift, wide) = (self.tall * sine(turn), self.tall * cosine(turn));
            let side = [wide * cosine(facing), lift, wide * sine(facing)];
            (std::array::from_fn(|axis| start[axis] + side[axis]), std::array::from_fn(|axis| end[axis] + side[axis]))
        })
    }

    /// `PP_ELECTRIC` (101), for one part: it waits while its count is under one. Every tick it reaches 0.4
    /// further (`out`), and further than 15 goes down by 2 a tick; over its first 25 ticks it comes up by 5
    /// to 8 a tick, by lot. At its first tick its line is laid out as the kind 135 lays its own
    /// (`shift_as_trail`).
    pub(super) fn arc_as_lightning(&mut self) {
        if self.waits() {
            return;
        }
        self.out += 0.4;
        if self.out > 15.0 {
            self.alpha -= 2;
        }
        if self.process <= 25 {
            self.alpha += self.lot(4) + 5;
        }
        self.alpha = self.alpha.max(0);
        if self.process == 1 {
            self.lay_out_line();
        }
    }

    /// `RenderLightningArc`, for one part: its line as the kind 135 has it (out along the way the part
    /// points, lifted in a bow, shaken), the second to the nineteenth piece each a tube of three sides,
    /// blue and paler the further along, each piece fainter by 5 than the one before, its light added. (For
    /// a part whose `alpha_top` is 3 the client draws the colour by lot; no effect ported gives it that.)
    pub(super) fn lightning_arc(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 || self.process <= 0 {
            return Vec::new();
        }
        let mut rects = Vec::new();
        for piece in 1..(self.at[1][1] as usize).min(19) {
            let alpha = self.alpha - piece as i32 * 5;
            if alpha > 0 {
                let pale = (piece * 6) as u8;
                rects.extend(self.line_tube(piece).windows(2).map(
                    |pair| Rect::new(picture, alpha, [pair[1].0, pair[1].1, pair[0].1, pair[0].0], (0.0, 1.0), [pale, pale, 255], true)
                ));
            }
        }
        rects
    }
}

impl Prim {
    /// `RenderJitterSweeps`: the first part is the line, drawn piece by piece (the second to the nineteenth)
    /// as a tube of three sides, `tall` wide, red and paler the further along, each piece fainter by 5 than
    /// the one before, its light added. Each other part is a sheet of red light between its own line and the
    /// first part's, at a third of its alpha for the second part and a half for the third.
    pub(super) fn jitter_sweeps(&self) -> Vec<Rect> {
        let mut rects = Vec::new();
        let mut first: [([f32; 3], [f32; 3]); 20] = [([0.0; 3], [0.0; 3]); 20];
        for (layer, part) in self.parts.iter().enumerate().take(4) {
            if !part.live || part.alpha <= 0 || part.process <= 0 {
                continue;
            }
            for piece in 1..(part.at[1][1] as usize).min(19) {
                let edges = part.line_tube(piece);
                let alpha = part.alpha - piece as i32 * 5;
                if alpha > 0 && layer == 0 {
                    let pale = (piece * 3) as u8;
                    rects.extend(edges.windows(2).map(|pair| Rect::new(
                        self.picture,
                        alpha,
                        [pair[1].0, pair[1].1, pair[0].1, pair[0].0],
                        (0.0, 1.0),
                        [255, pale, pale],
                        true
                    )));
                }
                let last = edges[3];
                if layer == 0 {
                    first[piece] = last;
                } else {
                    let alpha = match layer {
                        1 => part.alpha / 3,
                        2 => part.alpha / 2,
                        _ => part.alpha,
                    };
                    rects.push(Rect::new(self.picture, alpha, [last.0, last.1, first[piece].1, first[piece].0], (0.0, 1.0), [255, 0, 0], true));
                }
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_arc_of_lightning_reaches_out_comes_up_by_lot_and_is_a_blue_tube() {
        let mut prim = Prim::new(Shape::Tei(Tei::Electric), "elec1.tga");
        prim.duration = 300;
        let mut part = Part { tall: 0.1, rise: 45, round: 3, seed: 11, ..Part::default() };
        (part.at[0], part.at[1]) = ([0.0, -5.0, 0.0], [0.05, 20.0, 0.0]);
        prim.parts.push(part);
        for _ in 0..25 {
            prim.tick(300);
        }
        let part = prim.parts[0].clone();
        assert!((part.out - 10.0).abs() < 1e-4 && (125..=200).contains(&part.alpha));
        assert_eq!((part.heights[1], part.heights[20]), (180.0, 9.0));
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 18 * 3);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([6, 6, 255], true, (part.alpha - 5) as f32));
        // Further out than 15 it goes down by 2 a tick, and comes up no more after its 25th tick.
        for _ in 0..13 {
            prim.tick(300);
        }
        assert_eq!(prim.parts[0].alpha, part.alpha - 2);
    }

    /// The beam as `ElectricFan` launches it for one who faces south (the client's angle 0).
    fn beam(fade: i32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::JitterSweep), "elec1.tga");
        prim.duration = 300;
        prim.parts = (0..4)
            .map(|index| {
                let mut part = Part {
                    tall: 0.1,
                    alpha: if index == 0 { 250 } else { 140 - index * 30 },
                    rise: 270,
                    out: 20.0,
                    alpha_top: fade,
                    seed: index as u32,
                    ..Part::default()
                };
                part.at[1] = [0.05, 20.0, 0.0];
                part
            })
            .collect();
        prim
    }

    #[test]
    fn the_line_is_laid_out_at_the_first_tick_and_the_parts_turn_one_after_the_other() {
        let mut prim = beam(0);
        prim.tick(300);
        let heights = prim.parts[0].heights;
        assert_eq!((heights[0], heights[1], heights[2], heights[20]), (180.0, 180.0, 171.0, 9.0));
        // The first part turns from its first tick, against the clock; the second waits ten ticks.
        assert_eq!((prim.parts[0].rise, prim.parts[1].rise), (268, 270));
        for _ in 0..14 {
            prim.tick(300);
        }
        assert_eq!((prim.parts[0].rise, prim.parts[1].rise, prim.parts[2].rise), (240, 260, 270));
        let mut other = beam(1);
        other.tick(300);
        assert_eq!(other.parts[0].rise, 272);
        // After sixty ticks they go down by 6 a tick.
        for _ in 0..50 {
            prim.tick(300);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[3].alpha), (250 - 30, 50 - 30));
    }

    #[test]
    fn the_line_is_a_tube_of_three_sides_and_the_other_parts_are_sheets_to_it() {
        let mut prim = beam(0);
        for _ in 0..5 {
            prim.tick(300);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // Eighteen pieces: three sides each for the line, one sheet each for the three other parts.
        assert_eq!(rects.len(), 18 * 3 + 3 * 18);
        assert_eq!((rects[0].colour, rects[0].alpha, rects[0].adds), ([255, 3, 3], 245.0, true));
        // The first piece drawn is 40 to 38 out along the line's way, about 12 over the ground.
        let at = rects[0].corners[0].0;
        let far = (at[0] * at[0] + at[2] * at[2]).sqrt();
        assert!((far - 40.0).abs() < 0.6 && (at[1] - 12.0).abs() < 0.6);
        let sheet = &rects[18 * 3];
        assert_eq!((sheet.colour, sheet.alpha), ([255, 0, 0], (110 / 3) as f32));
    }
}
