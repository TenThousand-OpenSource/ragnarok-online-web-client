//! The kind `PP_GI_3` (34) of the client's primitives: up to four quarters of a wall of light, each round a
//! middle of its own, that rise once and fade (`PrimGI3`, drawn by `Render3DCasting2`). Grand Cross stands
//! four of them corner to corner, their hollow sides outwards, so that they wall in a cross.

use super::*;

impl Part {
    /// `PrimGI3`, for one part: every tick it stands a hundredth nearer its middle. Over its first nine ticks
    /// it comes up by 10 a tick; from the tenth it goes down by 1 a tick, and once it shows nothing it stands
    /// 20 out and upright, and after its 30th tick is gone. All its edges are as high as its height (`tall`)
    /// times the sine of its count, in degrees.
    pub(super) fn rise_as_quarter(&mut self) {
        self.process += 1;
        self.out -= 0.01;
        if self.process >= 10 {
            self.alpha -= 1;
            if self.alpha <= 0 {
                self.alpha = 0;
                if self.process > 30 {
                    self.live = false;
                }
                (self.out, self.rise) = (20.0, 90);
            }
        } else {
            self.alpha += 10;
        }
        self.heights = [self.tall * sine(self.process as f32); 21];
    }

    /// `Render3DCasting2`, for the part of that number: a wall in pieces, an edge every ninth of how far round
    /// it goes (`round`) from `turned` on, the last edge left out. An edge's foot is on the ground, `out` from
    /// the wall's middle, which is `flags[0]` east and `flags[1]` north of the effect's place; its top is its
    /// height from the foot, leaning as `rise` says (90 is upright) towards a way that is the edge's own
    /// angle mirrored: 450, 270, 90 and 630 degrees less it for the first, second, third and fourth part. A
    /// piece shows its 21st of the picture in the colour of the primitive's `size`.
    pub(super) fn quarter(&self, picture: &'static str, index: usize, size: f32) -> Vec<Rect> {
        let step = self.round / 9;
        if step <= 0 {
            return Vec::new();
        }
        let (east, north) = (f32::from(self.flags[0]), f32::from(self.flags[1]));
        let edge = |(order, offset): (i32, i32)| {
            let angle = (self.turned + offset) % 360;
            let height = self.heights[order as usize];
            let (lean, lift) = (cosine(self.rise as f32) * height, sine(self.rise as f32) * height);
            let mirrored = [450, 270, 90, 630][index.min(3)] - angle;
            let way = if mirrored >= 360 { mirrored - 360 } else { mirrored } as f32;
            let foot = [cosine(angle as f32) * self.out + east, 0.0, sine(angle as f32) * self.out + north];
            (foot, [foot[0] + cosine(way) * lean, -lift, foot[2] + sine(way) * lean])
        };
        Rect::band(picture, self.alpha, size, 20, edges(self.round, step, false, 20).map(edge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    /// The first quarter as `GrandCrossRings` launches it.
    fn first() -> Part {
        let mut flags = [0; 21];
        (flags[0], flags[1]) = (23, 23);
        Part { round: 90, tall: 120.0, out: 19.9, rise: 90, turned: 180, flags, ..Part::default() }
    }

    #[test]
    fn a_quarter_comes_up_fast_fades_slowly_and_is_gone() {
        let mut walls = Prim::new(Shape::Tei(Tei::Gi3), "ring_white.tga");
        (walls.duration, walls.size) = (999, 1.0);
        walls.parts.push(first());
        for _ in 0..9 {
            walls.tick(999);
        }
        assert_eq!(walls.parts[0].alpha, 90);
        assert!(close(walls.parts[0].out, 19.81));
        assert!(close(walls.parts[0].heights[20], 120.0 * 9.0_f32.to_radians().sin()));
        for _ in 0..89 {
            walls.tick(999);
        }
        assert!(walls.parts[0].live && walls.parts[0].alpha == 1);
        walls.tick(999);
        assert!(!walls.parts[0].live);
        assert!(walls.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
    }

    #[test]
    fn a_quarter_stands_round_its_own_middle_in_eight_pieces() {
        let mut part = first();
        part.alpha = 50;
        part.heights = [10.0; 21];
        let rects = part.quarter("ring_white.tga", 0, 1.0);
        assert_eq!(rects.len(), 8);
        // The first piece begins due west of the middle (23, 23), on the ground, and stands upright.
        let (foot, top) = (rects[0].corners[0].0, rects[0].corners[3].0);
        assert!(close(foot[0], 23.0 - 19.9) && close(foot[1], 0.0) && close(foot[2], 23.0));
        assert!(close(top[0], foot[0]) && close(top[1], 10.0) && close(top[2], foot[2]));
        // The last ends 80 degrees on, towards the south.
        let last = rects[7].corners[1].0;
        assert!(close(last[0], 23.0 + 19.9 * 260.0_f32.to_radians().cos()) && close(last[2], 23.0 + 19.9 * 260.0_f32.to_radians().sin()));
        // Each shows its 21st of the picture, in the pale red of the size 1, its light added.
        assert!(close(rects[1].corners[0].1[0], 1.0 / 21.0) && close(rects[1].corners[1].1[0], 2.0 / 21.0));
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([255, 175, 175], true, 50.0));
    }
}
