//! The kind 60 of the client's primitives as it lives when its columns are not of the first kind
//! (`PrimPulsingColumns` with a `flag1` of 1, 2 or 3): walls of light round the place of the effect that turn.
//! (The walls whose columns are of the first kind, which swing up and down and do not turn, are
//! `Shape::Casting3d` in `prim/slots.rs`; those of the kind 33, which the client turns with the camera, are
//! not ported.)

use super::*;

impl Part {
    /// `PrimPulsingColumns`, for the part of that number. It turns by its number and 8 degrees a tick if its
    /// first column is of the kind 1, by its number and 6 if of the kind 3, and of the kind 2 by its number
    /// and 4 for the first two parts and its number and 2 for the others. Until the tick it holds to
    /// (`alpha_top`; under nothing: for ever) it comes up over its first 15 ticks, by 5 a tick (3 for the
    /// kind 2, 2 for the kind 3), to 180 at most; from then on it goes down by 2 a tick. A column of the
    /// kind 1 or 3 stands the wall's whole height, which it comes up to as a sine does over 90 ticks; one of
    /// the kind 2 rises and falls once as a sine does over 45 ticks and is then gone.
    pub(super) fn turn_as_columns(&mut self, index: i32) {
        self.process += 1;
        self.turned += match self.flags[0] {
            1 => index + 8,
            2 if index < 2 => index + 4,
            2 => index + 2,
            3 => index + 6,
            _ => 0,
        };
        self.turned = under_a_round(self.turned);
        if self.process < self.alpha_top || self.alpha_top < 0 {
            if self.process < 16 {
                self.alpha += match self.flags[0] {
                    2 => 3,
                    3 => 2,
                    _ => 5,
                };
                self.alpha = self.alpha.min(180);
            }
        } else {
            self.alpha = (self.alpha - 2).max(0);
        }
        for column in 0..21 {
            match self.flags[column] {
                1 | 3 => self.heights[column] = if self.process < 91 { self.tall * sine(self.process as f32) } else { self.tall },
                2 => self.heights[column] = if self.process < 46 { self.tall * sine((self.process * 4) as f32) } else { 0.0 },
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The walls of a Warp Portal as `VioletRingCloud` launches them, each begun at no turn.
    fn portal() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Columns), "magic_violet.tga");
        (prim.duration, prim.size) = (9999, 4.0);
        prim.parts = [4.0, 3.0, 2.0].into_iter().map(|out| Part { round: 360, tall: 50.0, out, rise: 90, alpha_top: 1400, flags: [1; 21], ..Part::default() }).collect();
        prim
    }

    #[test]
    fn the_walls_of_a_portal_turn_each_at_its_pace_and_rise_over_ninety_ticks() {
        let mut prim = portal();
        for _ in 0..30 {
            prim.tick(9999);
        }
        assert_eq!([0, 1, 2].map(|index| prim.parts[index].turned), [240, 270, 300]);
        assert_eq!(prim.parts[0].alpha, 75);
        assert!((prim.parts[0].heights[20] - 25.0).abs() < 0.001);
        for _ in 0..70 {
            prim.tick(9999);
        }
        assert_eq!((prim.parts[0].alpha, prim.parts[0].heights[3]), (75, 50.0));
        // Upright bands of twenty pieces, blue and added for a primitive of size 4.
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 60);
        assert_eq!((rects[0].colour, rects[0].adds), ([100, 100, 255], true));
        let (foot, top) = (rects[0].corners[0].0, rects[0].corners[3].0);
        assert!((top[1] - foot[1] - 50.0).abs() < 0.001 && (top[0] - foot[0]).abs() < 0.001);
    }

    #[test]
    fn they_go_down_from_the_tick_they_hold_to_and_columns_of_the_second_kind_fall_again() {
        let mut prim = portal();
        prim.parts[0].alpha_top = 20;
        prim.parts[1].flags = [2; 21];
        for _ in 0..22 {
            prim.tick(9999);
        }
        assert_eq!(prim.parts[0].alpha, 75 - 3 * 2);
        // The second kind: 3 a tick, and a sine of four degrees a tick.
        assert_eq!(prim.parts[1].alpha, 45);
        assert!((prim.parts[1].heights[0] - 50.0 * 88.0_f32.to_radians().sin()).abs() < 0.001);
        for _ in 0..30 {
            prim.tick(9999);
        }
        assert_eq!(prim.parts[1].heights[0], 0.0);
    }
}
