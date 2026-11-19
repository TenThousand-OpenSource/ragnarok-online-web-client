//! The kind 146 of the client's primitives: walls of light most of the way round a place, open towards
//! where they came from, that fly off along the ground one after the other, each pulsing up and gone and
//! then again lower and nearer (`PrimDriftingRings`, drawn by `RenderTrailingArc`): the waves of Sonic Wave,
//! the bubbles of the elementals.

use super::*;

impl Part {
    /// `PrimDriftingRings`, for one part: it waits while its count is under one. Every tick it has flown 1
    /// further (`at[1]`) along the way 15 degrees back from where its wall begins (`turned`). Over its
    /// first nine ticks it comes up by 25 a tick, to 250 at most, and then goes down by 10 a tick; having
    /// gone it begins anew, 20 degrees flatter (`rise`) and 0.4 nearer (`out`), and is over once it would lean
    /// at 40 degrees or less. Its edges stand its height (`tall`), the six at each end lower (four tenths at
    /// the very end, a tenth more at each next), and over its first 15 ticks all come up as a sine does.
    pub(super) fn drift_as_ring(&mut self) {
        if self.waits() {
            return;
        }
        let heading = (self.turned + 345).rem_euclid(360) as f32;
        self.at[1][0] += cosine(heading);
        self.at[1][2] += sine(heading);
        self.turned = self.turned.rem_euclid(360);
        if self.process < 10 {
            self.alpha = (self.alpha + 25).min(250);
        } else {
            self.alpha -= 10;
            if self.alpha < 0 {
                self.rise -= 20;
                (self.alpha, self.process) = (0, 0);
                self.out -= 0.4;
                if self.rise <= 40 {
                    (self.live, self.rise) = (false, 40);
                }
            }
        }
        for (point, height) in self.heights.iter_mut().enumerate() {
            let from_end = point.min(20 - point);
            let scale = if from_end < 6 { 0.4 + from_end as f32 * 0.1 } else { 1.0 };
            *height = scale * self.tall;
            if self.process <= 15 {
                *height *= sine((6 * self.process) as f32);
            }
            *height = height.max(0.0);
        }
    }

    /// `RenderTrailingArc`, for one part: its wall in pieces of a twentieth of how far round it goes
    /// (`round`), from `turned` on. An edge's foot is `out` from the middle and its top its own height from
    /// the foot, leaning outwards as `rise` says; the whole is moved back by as far as the first part has
    /// flown (`flown`), the feet by three quarters of that, so the wall leans into its flight. A piece shows
    /// its 21st of the picture in the colour the primitive's `size` stands for; the client sets that size from
    /// the part (`at[3][0]`) as it draws. (As the client counts, the last piece is drawn twice over, the
    /// second time on to the edge after the last.)
    pub(super) fn trailing_arc(&self, picture: &'static str, flown: [f32; 3]) -> Vec<Rect> {
        let step = self.round / 20;
        if step <= 0 || self.process <= 0 {
            return Vec::new();
        }
        let (lean, up) = (cosine(self.rise as f32), sine(self.rise as f32));
        let edge = |(order, offset): (i32, i32)| {
            let angle = if self.round == 360 && order == 20 { self.turned } else { (self.turned + offset) % 360 } as f32;
            let (cos, sin) = (cosine(angle), sine(angle));
            let height = self.heights[order as usize];
            let foot = [self.out * cos - flown[0] * 0.75, 0.0, self.out * sin - flown[2] * 0.75];
            (foot, [height * lean * cos + self.out * cos - flown[0], -(height * up), height * lean * sin + self.out * sin - flown[2]])
        };
        Rect::band(picture, self.alpha, self.at[3][0], 20, edges(self.round, step, true, 20).map(edge))
    }
}

impl Prim {
    /// The walls of the kind 146 now, each with the picture its count picks among the primitive's.
    pub(super) fn trailing_arcs(&self) -> Vec<Rect> {
        let flown = self.parts.first().map_or([0.0; 3], |first| first.at[1]);
        let picture = |part: &Part| if self.pictures.is_empty() { self.picture } else { self.pictures[part.process.max(0) as usize % self.pictures.len()] };
        self.parts.iter().filter(|part| part.live).flat_map(|part| part.trailing_arc(picture(part), flown)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The walls as `TexturedFanSweep` launches its narrow ones towards the east (the client's angle 0).
    fn waves() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::DriftRing), "son_wave_00.tga");
        prim.duration = 1678;
        prim.parts = (0..4)
            .map(|index| Part {
                process: index * -8,
                tall: 30.0,
                round: 315,
                turned: 195,
                out: 6.0 - index as f32 * 0.1,
                rise: (14 - index) * 5,
                ..Part::default()
            })
            .collect();
        prim
    }

    #[test]
    fn a_wall_pulses_twice_lower_and_nearer_and_is_over() {
        let mut prim = waves();
        for _ in 0..9 {
            prim.tick(1678);
        }
        // Nine ticks up by 25; the second part has only begun.
        assert_eq!((prim.parts[0].alpha, prim.parts[1].alpha, prim.parts[1].process), (225, 25, 1));
        // The way 15 degrees back from 195 is west: it has flown 9 to the west.
        assert!((prim.parts[0].at[1][0] + 9.0).abs() < 1e-3 && prim.parts[0].at[1][2].abs() < 1e-3);
        // The edges come up as a sine over 15 ticks; the ends are lower.
        assert!((prim.parts[0].heights[10] - 30.0 * sine(54.0)).abs() < 1e-3 && (prim.parts[0].heights[0] - 12.0 * sine(54.0)).abs() < 1e-3);
        for _ in 0..23 {
            prim.tick(1678);
        }
        // Down by 10 from its tenth tick: under nothing at its 32nd, and it begins anew.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].process, prim.parts[0].rise), (0, 0, 50));
        assert!((prim.parts[0].out - 5.6).abs() < 1e-4);
        for _ in 0..32 {
            prim.tick(1678);
        }
        assert!(!prim.parts[0].live);
        // The third begins at 60 degrees and is over after one pulse.
        assert!(!prim.parts[2].live || prim.parts[2].rise == 60);
    }

    #[test]
    fn it_is_drawn_in_pieces_round_the_place_moved_back_by_its_flight() {
        let mut prim = waves();
        for _ in 0..15 {
            prim.tick(1678);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // Two walls show, 21 pieces each: 315 degrees in steps of 15 are 22 edges.
        assert_eq!(rects.len(), 42);
        assert_eq!((rects[0].alpha, rects[0].colour, rects[0].adds), (165.0, [255; 3], false));
        // The first foot is 6 out at 195 degrees and three quarters of the 15 flown further east.
        let foot = rects[0].corners[0].0;
        assert!((foot[0] - (6.0 * cosine(195.0) + 11.25)).abs() < 1e-3 && foot[1] == 0.0);
        // Its top leans out at 70 degrees from the foot, and is moved by all that was flown.
        let top = rects[0].corners[3].0;
        let height = prim.parts[0].heights[0];
        assert!((top[1] - height * sine(70.0)).abs() < 1e-3);
        assert!((top[0] - (height * cosine(70.0) * cosine(195.0) + 6.0 * cosine(195.0) + 15.0)).abs() < 1e-3);
        // A part told the size 7 is green, its light added.
        prim.parts[0].at[3][0] = 7.0;
        assert_eq!((prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].colour, prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])[0].adds), ([100, 255, 100], true));
    }
}
