//! The kinds 137 and 140 of the client's primitives:
//!
//! - 137: thin shafts of light of four sides that wait, shoot up and sink again (`PrimHeightRamp`, drawn by
//!   `RenderArcWalls`): the shafts of Tetra Vortex and of Hesperus Lit;
//! - 140: a picture facing the screen that runs through its pictures once and then fades (`PrimFrameFade`,
//!   drawn by `RenderTeiFacingSlots`, `Part::aura` in `tei/auras.rs`): the blast of Tetra Vortex.

use super::*;

impl Part {
    /// `PrimHeightRamp`, for one part: it turns a degree a tick. Until its count reaches 100 it stands
    /// nothing tall; over the 15 ticks after it comes up to its height (`tall`) as a sine does; from its
    /// 125th tick it goes down by 15 a tick and its height by 5.
    pub(super) fn ramp_up(&mut self) {
        self.process += 1;
        self.turned = (self.turned + 1) % 360;
        if self.process >= 125 {
            self.alpha = (self.alpha - 15).max(0);
            self.tall = (self.tall - 5.0).max(0.0);
        }
        let height = match self.process {
            ..100 => 0.0,
            100..=115 => sine((6 * self.process - 600) as f32) * self.tall,
            _ => self.tall,
        };
        self.heights = [height.max(0.0); 21];
    }

    /// One wall of `RenderArcWalls`: the feet and the tops of its five edges, a quarter of how far round it
    /// goes apart, from `from` degrees on, `out` from the part's own place (`at[0]`, moved by `at[3]`); as the
    /// client counts, the last edge of a whole round is where the part is turned to, whatever `from` is.
    fn shaft(&self, from: i32, out: f32) -> Vec<([f32; 3], [f32; 3])> {
        let step = (self.round / 4).max(1);
        let (lean, up) = (cosine(self.rise as f32), sine(self.rise as f32));
        (0..=self.round)
            .step_by(step as usize)
            .enumerate()
            .map(|(order, offset)| {
                let order = order.min(4);
                let angle = if self.round == 360 && order == 4 { self.turned } else { (from + offset) % 360 } as f32;
                let (cos, sin) = (cosine(angle), sine(angle));
                let height = self.heights[order];
                let base = [out * cos + self.at[0][0] + self.at[3][0], self.at[0][1] + self.at[3][1], out * sin + self.at[0][2] + self.at[3][2]];
                (base, [base[0] + height * lean * cos, base[1] - height * up, base[2] + height * lean * sin])
            })
            .collect()
    }

    /// `RenderArcWalls`, for one part: two shafts of four sides about its place. The inner one, `out` from
    /// the middle, is red and mixed in at a fifth of the part's alpha (the client hands its corners over in
    /// an order that folds each side: foot, foot, top, top). The outer one, half as far out again and an
    /// eighth of a turn on, is in the colour the primitive's `size` stands for, each side showing its 21st of
    /// the picture.
    pub(super) fn arc_walls(&self, picture: &'static str, size: f32) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let inner = self.shaft(self.turned, self.out);
        let outer = self.shaft(self.turned + 45, self.out * 1.5);
        let folded = inner.windows(2).map(|pair| Rect::new(picture, self.alpha / 5, [pair[0].0, pair[1].0, pair[0].1, pair[1].1], (0.0, 1.0), [255, 10, 10], false));
        let pieces = outer.windows(2).zip(0..).map(|(pair, piece)| Rect::piece(picture, piece, self.alpha, [pair[0].0, pair[1].0, pair[1].1, pair[0].1], size));
        folded.chain(pieces).collect()
    }

    /// `PrimFrameFade`, for one part: every third tick it shows its next picture (`flags[0]`); having shown
    /// the last of the `frames` it stays on it and goes down by 15 each time.
    pub(super) fn fade_by_frames(&mut self, frames: usize) {
        self.process += 1;
        if self.process % 3 == 0 {
            self.flags[0] += 1;
            if self.flags[0] as usize >= frames {
                self.flags[0] = frames.saturating_sub(1) as i8;
                self.alpha -= 15;
            }
        }
        self.alpha = self.alpha.clamp(0, 255);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    /// A shaft as `TexturedSpread` launches one for Tetra Vortex, 10 east of the effect's place.
    fn shaft(process: i32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::ArcWall), "ring_red.tga");
        (prim.duration, prim.size) = (600, 6.0);
        let mut part = Part { process, round: 360, turned: 0, alpha: 150, tall: 120.0, out: 0.2, rise: 90, ..Part::default() };
        part.at[0] = [10.0, 0.0, 0.0];
        prim.parts.push(part);
        prim
    }

    #[test]
    fn a_shaft_waits_shoots_up_and_sinks() {
        let mut prim = shaft(90);
        for _ in 0..9 {
            prim.tick(600);
        }
        assert_eq!(prim.parts[0].heights[0], 0.0);
        for _ in 0..6 {
            prim.tick(600);
        }
        // At its 105th tick it is the sine of 30 degrees of its height.
        assert!((prim.parts[0].heights[3] - 60.0).abs() < 1e-3);
        for _ in 0..20 {
            prim.tick(600);
        }
        // Its 125th tick is the first at which it goes down.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].tall, prim.parts[0].turned), (135, 115.0, 35));
        for _ in 0..30 {
            prim.tick(600);
        }
        assert!(prim.rects(RIGHT, UP).is_empty());
    }

    #[test]
    fn it_is_two_shafts_of_four_sides_the_inner_red_and_the_outer_in_the_colour_of_its_size() {
        let mut prim = shaft(116);
        prim.tick(600);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 8);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([255, 10, 10], false, 30.0));
        assert_eq!((rects[4].colour, rects[4].adds, rects[4].alpha), ([255, 100, 100], true, 150.0));
        // The inner shaft's first foot is 0.2 out at one degree, about the part's place; its third corner
        // is the top over that foot, the whole height up.
        let (foot, top) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert!((foot[0] - (10.0 + 0.2 * cosine(1.0))).abs() < 1e-3 && foot[1] == 0.0);
        assert!((top[1] - 120.0).abs() < 1e-3 && (top[0] - foot[0]).abs() < 1e-3);
        // A side of the outer shaft shows its 21st of the picture.
        assert!((rects[5].corners[0].1[0] - 1.0 / 21.0).abs() < 1e-5);
    }

    #[test]
    fn a_blast_runs_through_its_pictures_and_then_fades() {
        let mut prim = Prim::new(Shape::Tei(Tei::FrameFade), "tv-1.bmp");
        prim.duration = 600;
        prim.pictures = &["tv-1.bmp", "tv-2.bmp", "tv-3.bmp"];
        let mut part = Part { out: 30.0, round: 120, alpha: 200, ..Part::default() };
        part.heights[10] = 1.0;
        prim.parts.push(part);
        for _ in 0..6 {
            prim.tick(600);
        }
        assert_eq!((prim.parts[0].flags[0], prim.parts[0].alpha), (2, 200));
        assert_eq!(prim.rects(RIGHT, UP)[0].picture, "tv-3.bmp");
        for _ in 0..3 {
            prim.tick(600);
        }
        assert_eq!((prim.parts[0].flags[0], prim.parts[0].alpha), (2, 185));
        // White and mixed in; a figure taller than wide, its first corner at 120 degrees.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects[0].colour, rects[0].adds), ([255; 3], false));
        let (first, second) = (rects[0].corners[0].0, rects[0].corners[1].0);
        assert!((first[0] + 15.0).abs() < 1e-3 && (second[0] - 15.0).abs() < 1e-3 && (first[1] + 30.0 * sine(120.0)).abs() < 1e-3);
    }
}
