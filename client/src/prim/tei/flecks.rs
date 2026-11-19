//! The kind `PP_TEIHIT2` (45) of the client's primitives: up to four small squares facing the screen that
//! wait, then fly from a place level along a way of their own, rising or sinking, turn and fade
//! (`PrimTeiHit2`, drawn by `RenderFieldPanel`, `tei/field.rs`): the sparks struck out of the one hit by
//! Investigate and Back Stab.

use super::*;

impl Part {
    /// `PrimTeiHit2`, for one part: it waits while its count is under one. At its first tick it takes its
    /// way: up to 30 degrees to either side, by lot, of the way from the one its effect is on to the place the
    /// effect aims at. Over the nine ticks after it comes up by 15 a tick, and from then on goes down by 3 a
    /// tick, ending at nothing. Every tick it turns 5 degrees against the clock, flies `heights[1]` along its
    /// way and sinks by `heights[2]`.
    ///
    /// (The client measures the way to the place aimed at as the part takes it; here the effect has measured
    /// it as it launched the part, and it is in `turned`.)
    pub(super) fn fly_as_fleck(&mut self) {
        if self.waits() {
            return;
        }
        self.round -= 5;
        if self.round < 0 {
            self.round += 360;
        }
        if self.process == 1 {
            self.turned += self.lot(60) - 30;
            self.turned = under_a_round(self.turned);
            if self.turned < 0 {
                self.turned += 360;
            }
        } else if self.process <= 10 {
            self.alpha += 15;
        } else {
            self.alpha -= 3;
            self.over_once_gone();
        }
        self.at[0][1] += self.heights[2];
        self.at[0][0] += self.heights[1] * cosine(self.turned as f32);
        self.at[0][2] += self.heights[1] * sine(self.turned as f32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    /// A spark as `TEIHIT2` launches one from a place 10 to the east of the striker, 8 over the ground.
    fn spark(wait: i32) -> Prim {
        let mut sparks = Prim::new(Shape::Tei(Tei::TeiHit2), "thunder_center.bmp");
        sparks.duration = 300;
        let mut part = Part { process: -wait, out: 0.5, seed: 3, ..Part::default() };
        part.at[0] = [10.0, -8.0, 0.0];
        (part.heights[1], part.heights[2], part.flags[2]) = (1.0, -0.5, 2);
        sparks.parts.push(part);
        sparks
    }

    #[test]
    fn a_spark_waits_takes_a_way_near_the_blows_and_flies() {
        let mut sparks = spark(30);
        for _ in 0..30 {
            sparks.tick(300);
        }
        assert_eq!(sparks.parts[0].at[0], [10.0, -8.0, 0.0]);
        assert!(sparks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
        sparks.tick(300);
        // The way of the blow is east (0): its own is within 30 degrees of that.
        let way = sparks.parts[0].turned;
        assert!(way < 30 || way >= 330, "{way}");
        assert_eq!((sparks.parts[0].alpha, sparks.parts[0].round), (0, 355));
        for _ in 0..9 {
            sparks.tick(300);
        }
        let part = &sparks.parts[0];
        assert_eq!(part.alpha, 135);
        // Ten ticks of flight: 10 along its way, 5 higher.
        assert!(close(part.at[0][0], 10.0 + 10.0 * (way as f32).to_radians().cos()));
        assert!(close(part.at[0][1], -13.0));
        let rects = sparks.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (1, [130, 130, 250], true));
        // It goes down by 3 a tick and is gone.
        for _ in 0..45 {
            sparks.tick(300);
        }
        assert!(sparks.parts[0].live && sparks.parts[0].alpha == 0);
        sparks.tick(300);
        assert!(!sparks.parts[0].live);
    }
}
