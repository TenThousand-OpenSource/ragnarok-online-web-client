//! `PP_RECT_UP2` (90) of the client's primitives: walls of light round a square (`PrimFadingColumns`, drawn
//! by `RenderRectUp2`). Its first two parts are the walls, all four sides of the square; its last two are
//! lights, a stretch of wall that runs round the square for ever. All rise as a sine does over 90 ticks, stand,
//! and sink again once their time is up: the walls of Basilica.
//!
//! For a part whose `heights[4]` is 20 or more the client brings the time it sinks at forward once its effect
//! has moved from where the primitive is; a part does not know that here, and it is not ported (no effect
//! ported launches such a part).

use super::*;

impl Part {
    /// `PrimFadingColumns`, for the part of that number: its count stops at 29000. One of the last two goes
    /// half a unit a tick along the square (`out`), whose way round is eight times its half width
    /// (`heights[0]`). Its height now (`heights[2]`): the sine of its count over the first 90 ticks, then its
    /// whole height (`tall`); past the tick it holds to (`alpha_top`) it sinks as a sine does over 90 ticks,
    /// which begin at a tick its kind (`heights[4]`) gives: the 2415th for the kind 0, the 2385th for the
    /// others under 10, the tick it holds to for those from 10, and for those from 20 over 18 ticks from it.
    pub(super) fn stand_as_fence(&mut self, index: i32) {
        self.process += 1;
        if self.process > 29000 {
            self.process = 29000;
        } else if self.process <= 0 {
            return;
        }
        if index >= 2 {
            self.out += 0.5;
            if self.out > self.heights[0] * 8.0 {
                self.out -= self.heights[0] * 8.0;
            }
        }
        let kind = self.heights[4];
        self.heights[2] = if self.process > self.alpha_top {
            let angle = if kind >= 20.0 {
                (self.alpha_top - self.process + 18) * 5
            } else if kind >= 10.0 {
                self.alpha_top - self.process + 90
            } else if kind == 0.0 {
                2505 - self.process
            } else {
                2475 - self.process
            };
            sine(angle.clamp(0, 90) as f32) * self.tall
        } else if self.process < 90 {
            sine(self.process as f32) * self.tall
        } else {
            self.tall
        };
    }

    /// `RenderRectUp2`, for the part of that number: upright walls from 1 over the effect's place to its
    /// height (`heights[2]`) more, along the sides of a square `heights[0]` to the east and west of the
    /// middle and `heights[1]` to the north and south, their light added. One of the first two parts is all
    /// four sides, pale green (blue for a kind from 10). One of the last two is one stretch `heights[3]` long
    /// that begins `out` along the way round (from the north-east corner south, then west, north and east
    /// again) and goes on round the next corner when it reaches it; a deeper green (a deeper blue).
    pub(super) fn fence(&self, picture: &'static str, number: usize) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let later = self.heights[4] >= 10.0;
        let (w, h, top) = (self.heights[0], self.heights[1], -1.0 - self.heights[2]);
        // The client's `vecBPre`, `vecBNow`, `vecTNow`, `vecTPre`: the feet of the four corners.
        let mut feet = [[w, -1.0, h], [w, -1.0, -h], [-w, -1.0, -h], [-w, -1.0, h]];
        let wall = |feet: &[[f32; 3]; 4], side: usize, colour: [u8; 3]| {
            let (from, to) = (feet[side], feet[(side + 1) % 4]);
            Rect::new(picture, self.alpha, [from, to, [to[0], top, to[2]], [from[0], top, from[2]]], (0.0, 1.0), colour, true)
        };
        if number < 2 {
            let colour = if later { [100, 100, 250] } else { [200, 250, 200] };
            return (0..4).map(|side| wall(&feet, side, colour)).collect();
        }
        let colour = if later { [50, 50, 250] } else { [150, 250, 150] };
        let (along, long) = (self.out, self.heights[3]);
        let side = [2.0, 4.0, 6.0].iter().filter(|sides| along >= w * **sides).count();
        // How much of the stretch is past the corner at its end.
        let mut left = 0.0;
        match side {
            0 => {
                feet[0][2] = h - along;
                feet[1][2] = feet[0][2] - long;
                if feet[1][2] < -h {
                    left = -h - feet[1][2];
                    (feet[1][2], feet[2][0]) = (-h, feet[1][0] - left);
                }
            }
            1 => {
                feet[1][0] = w - (along - w * 2.0);
                feet[2][0] = feet[1][0] - long;
                if feet[2][0] < -w {
                    left = -w - feet[2][0];
                    (feet[2][0], feet[3][2]) = (-w, feet[2][2] + left);
                }
            }
            2 => {
                feet[2][2] = along - w * 4.0 - h;
                feet[3][2] = feet[2][2] + long;
                if feet[3][2] > h {
                    left = feet[3][2] - h;
                    (feet[3][2], feet[0][0]) = (h, feet[3][0] + left);
                }
            }
            _ => {
                feet[3][0] = along - w * 6.0 - w;
                feet[0][0] = feet[3][0] + long;
                if feet[0][0] > w {
                    left = feet[0][0] - w;
                    (feet[0][0], feet[1][2]) = (w, feet[0][2] - left);
                }
            }
        }
        let mut walls = vec![wall(&feet, side, colour)];
        if left > 0.0 {
            walls.push(wall(&feet, (side + 1) % 4, colour));
        }
        walls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP, close};

    /// The walls as `TexturedFourRing` launches them with its kind 0, for Basilica.
    fn basilica() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::RectUp2), "alpha_down.tga");
        prim.duration = 29999;
        prim.parts.extend((0..2).map(|index| {
            let mut part = Part { tall: index as f32 + 30.0, alpha: 65, alpha_top: 2415, ..Part::default() };
            (part.heights[0], part.heights[1]) = (index as f32 * 0.4 + 12.5, index as f32 * 0.4 + 12.5);
            part
        }));
        prim.parts.extend((2..4).map(|index| {
            let mut part = Part { tall: index as f32 + 30.0, out: (index - 2) as f32 * 53.2, alpha: 15, alpha_top: 2415, ..Part::default() };
            (part.heights[0], part.heights[1], part.heights[3]) = (13.3, 13.3, 26.6);
            part
        }));
        prim
    }

    #[test]
    fn the_walls_rise_over_ninety_ticks_stand_and_sink_when_their_time_is_up() {
        let mut prim = basilica();
        for _ in 0..30 {
            prim.tick(29999);
        }
        assert!(close(prim.parts[0].heights[2], 15.0) && close(prim.parts[3].heights[2], 16.5));
        for _ in 0..60 {
            prim.tick(29999);
        }
        assert_eq!((prim.parts[0].heights[2], prim.parts[1].heights[2]), (30.0, 31.0));
        let rects = prim.rects(RIGHT, UP);
        // Four sides for each of the two walls, and a stretch for each of the two lights, both of which are
        // going round a corner now.
        assert_eq!(rects.len(), 12);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([200, 250, 200], true, 65.0));
        // The first side goes from the north-east corner south, from 1 over the ground to 31.
        assert_eq!((rects[0].corners[0].0, rects[0].corners[1].0), ([12.5, 1.0, 12.5], [12.5, 1.0, -12.5]));
        assert_eq!(rects[0].corners[2].0, [12.5, 31.0, -12.5]);
        assert_eq!((rects[8].colour, rects[8].alpha), ([150, 250, 150], 15.0));
        prim.parts.iter_mut().for_each(|part| part.process = 2459);
        prim.tick(29999);
        // 45 ticks past the one they hold to they are down to the sine of 45 degrees.
        assert!(close(prim.parts[0].heights[2], 30.0 * sine(45.0)));
        prim.parts.iter_mut().for_each(|part| part.process = 28999);
        prim.tick(29999);
        prim.tick(29999);
        assert_eq!((prim.parts[0].process, prim.parts[0].heights[2]), (29000, 0.0));
    }

    #[test]
    fn a_light_runs_round_the_square_and_on_round_its_corners() {
        let mut prim = basilica();
        prim.tick(29999);
        // Half a unit a tick; the second light is half the way round from the first.
        assert_eq!((prim.parts[2].out, prim.parts[3].out), (0.5, 53.7));
        let stretch = prim.parts[2].fence("a", 2);
        // The whole east side less the half unit it has gone, and that half unit round the corner.
        assert_eq!(stretch.len(), 2);
        assert!(close(stretch[0].corners[0].0[2], 12.8) && close(stretch[0].corners[1].0[2], -13.3));
        assert!(close(stretch[1].corners[0].0[0], 13.3) && close(stretch[1].corners[1].0[0], 12.8));
        // The other is on the west side, going north.
        let other = prim.parts[3].fence("a", 3);
        assert!(close(other[0].corners[0].0[0], -13.3) && close(other[0].corners[0].0[2], -12.8));
        // Past the whole way round it begins again.
        prim.parts[2].out = 106.2;
        prim.tick(29999);
        assert!(close(prim.parts[2].out, 0.3));
        // A kind from 10 is blue.
        prim.parts[0].heights[4] = 10.0;
        assert_eq!(prim.parts[0].fence("a", 0)[0].colour, [100, 100, 250]);
    }
}
