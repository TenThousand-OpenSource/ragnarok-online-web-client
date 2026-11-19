//! The kind 63 of the client's primitives: rings of light that widen from the place of the effect over and
//! over (`PrimRippleRings`), each a band of twenty pieces as `Render3DCasting` draws one. That drawing, for a
//! part kept whole, is here too (`Part::casting`): the kinds 36 and 60 of this folder use it.

use super::*;

impl Prim {
    /// `PrimRippleRings`: the first three parts are the rings, and the fourth only counts the ticks
    /// (`process`) and says which rings they are (`alpha_top`). A ring waits while its count is under one.
    ///
    /// Of the kind 3 it closes in by a quarter a tick, comes up by 12 a tick over its first 19 ticks (to 240
    /// at most), and nearer than 7 goes down by 7 a tick; having gone it begins anew 14 out, or is over once
    /// the fourth part has counted past 1400.
    ///
    /// Of any other kind it widens by a half a tick, comes up by 24 a tick over its first 9 ticks (to 240 at
    /// most; of the kind 5 to 120; of the kind 4 by 4 a tick to 40), and further out than 7 goes down by 15
    /// a tick (of the kind 4 by 5, of the kind 5 by 8); having gone it begins anew at the middle, or is over
    /// once the fourth part has counted past 1400, or past 120 for the kinds 1, 4 and 5.
    ///
    /// Every edge of a ring stands its whole height, which it comes up to as a sine does over its first ten
    /// ticks.
    pub(super) fn ripple_rings(&mut self) {
        let Some((control, rings)) = self.parts.split_last_mut() else { return };
        for (index, ring) in rings.iter_mut().enumerate().take(3).filter(|(_, ring)| ring.live) {
            ring.process += 1;
            if index == 0 {
                control.process += 1;
            }
            if ring.process <= 0 {
                continue;
            }
            ring.ripple(control.alpha_top, control.process);
            let height = if ring.process < 11 { ring.tall * sine((ring.process * 9) as f32) } else { ring.tall };
            ring.heights = [height; 21];
        }
    }
}

impl Part {
    /// One tick of a ring of the kind 63 that does not wait: `kind` and `counted` are what the fourth part
    /// says and has counted.
    fn ripple(&mut self, kind: i32, counted: i32) {
        if kind == 3 {
            self.out -= 0.25;
            if self.out < 7.0 {
                self.out = self.out.max(0.0);
                self.alpha -= 7;
                if self.alpha < 0 {
                    self.alpha = 0;
                    if counted > 1400 {
                        self.live = false;
                    } else {
                        (self.process, self.out) = (0, 14.0);
                    }
                }
            }
            if self.process < 20 {
                self.alpha = (self.alpha + 12).min(240);
            }
            return;
        }
        self.out += 0.5;
        if self.out > 7.0 {
            self.alpha -= match kind {
                4 => 5,
                5 => 8,
                _ => 15,
            };
            if self.alpha < 0 {
                self.alpha = 0;
                if (counted > 120 && matches!(kind, 1 | 4 | 5)) || counted > 1400 {
                    self.live = false;
                } else {
                    (self.process, self.out) = (0, 0.0);
                }
            }
        }
        if self.process < 10 {
            self.alpha = match kind {
                4 => (self.alpha + 4).min(40),
                5 => (self.alpha + 24).min(120),
                _ => (self.alpha + 24).min(240),
            };
        }
    }

    /// `Render3DCasting` (`pieces` of 20) and `Render3DCasting_LowPolygon` (of 10), for one part: a band
    /// round the effect's place, from `turned` on for `round` degrees in that many pieces (the last edge of a
    /// whole round is the first again). An edge's foot is `out` from the middle; its top is its own height
    /// (`heights`) from the foot, leaning outwards as `rise` says (90 is upright). `Render3DCasting` lifts a
    /// band whose `tall` is 6.001 by 2 and one of 24.001 by 1. A piece shows its 21st of the picture, in the
    /// colour the primitive's `size` stands for (`Rect::piece`). (`Render3DCasting_LowPolygon` moves a part
    /// whose `vecBNow.y` is 1234.5 aside; no effect ported gives a part that, and it is not ported.)
    pub(super) fn casting(&self, picture: &'static str, size: f32, pieces: i32) -> Vec<Rect> {
        let step = self.round / pieces;
        if step <= 0 || self.alpha <= 0 {
            return Vec::new();
        }
        let lift = match (pieces, self.tall) {
            (20, tall) if tall == 6.001 => 2.0,
            (20, tall) if tall == 24.001 => 1.0,
            _ => 0.0,
        };
        let edge = |(order, count): (i32, i32)| {
            let angle = if self.round == 360 && order == pieces { self.turned } else { (self.turned + count) % 360 } as f32;
            let (cos, sin) = (cosine(angle), sine(angle));
            let foot = [cos * self.out, -lift, sin * self.out];
            let height = self.heights[order as usize];
            let (lean, up) = (cosine(self.rise as f32) * height, sine(self.rise as f32) * height);
            (foot, [foot[0] + cos * lean, -(up + lift), foot[2] + sin * lean])
        };
        Rect::band(picture, self.alpha, size, pieces, edges(self.round, step, true, pieces).map(edge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rings as `BlueRingWave` launches them (`kind` of 1) and `VioletRingCloud` (of 0).
    fn rings(kind: i32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Ripple), "ring");
        (prim.duration, prim.size) = (9999, 5.0);
        prim.parts = [(0, 0, 2), (-10, 25, 3), (-20, 50, 4)]
            .into_iter()
            .map(|(process, turned, rise)| Part { process, round: 360, tall: 6.001, turned, rise, alpha_top: 100, ..Part::default() })
            .collect();
        prim.parts.push(Part { live: false, alpha_top: kind, ..Part::default() });
        prim
    }

    #[test]
    fn a_ring_comes_up_widens_fades_past_seven_and_begins_anew() {
        let mut prim = rings(0);
        for _ in 0..9 {
            prim.tick(9999);
        }
        // Nine ticks: 24 a tick, half a cell a tick; the second ring still waits.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].out, prim.parts[1].alpha, prim.parts[1].process), (216, 4.5, 0, -1));
        assert!((prim.parts[0].heights[7] - 6.001 * 81.0_f32.to_radians().sin()).abs() < 0.001);
        for _ in 0..6 {
            prim.tick(9999);
        }
        // Past 7 out it goes down by 15 a tick.
        assert_eq!((prim.parts[0].alpha, prim.parts[0].out), (216 - 15, 7.5));
        for _ in 0..14 {
            prim.tick(9999);
        }
        // Gone at its 29th tick, it begins anew at the middle and comes up at once.
        assert_eq!((prim.parts[0].process, prim.parts[0].out, prim.parts[0].alpha, prim.parts[0].live), (0, 0.0, 24, true));
        assert_eq!(prim.parts[3].process, 29);
    }

    #[test]
    fn the_rings_of_a_portal_that_is_readied_are_over_after_120_ticks() {
        let mut prim = rings(1);
        for _ in 0..200 {
            prim.tick(9999);
        }
        assert!(prim.parts[..3].iter().all(|ring| !ring.live));
        assert!(prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
    }

    #[test]
    fn a_ring_is_a_band_of_twenty_pieces_lifted_by_two() {
        let mut prim = rings(0);
        for _ in 0..12 {
            prim.tick(9999);
        }
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // The first ring shows, and the second, two ticks old; the third waits.
        assert_eq!(rects.len(), 40);
        let first = &rects[0];
        // 6 out, its foot 2 over the ground at the east; its top leans outwards at 2 degrees of rise.
        assert_eq!(first.corners[0].0, [6.0, 2.0, 0.0]);
        let top = first.corners[3].0;
        assert!((top[0] - (6.0 + 6.001 * 2.0_f32.to_radians().cos())).abs() < 0.001 && (top[1] - (2.0 + 6.001 * 2.0_f32.to_radians().sin())).abs() < 0.001);
        // White and added for a primitive of size 5; the last piece ends where the first began.
        assert_eq!((first.colour, first.adds, first.alpha), ([255; 3], true, 216.0));
        assert_eq!(rects[19].corners[1].0, first.corners[0].0);
    }
}
