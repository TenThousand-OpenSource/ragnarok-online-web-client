//! The kinds 122 and `PP_STIN4` (125) of the client's primitives: a square of wind lying level that spins and
//! flies in a bow round to the one its effect was told, three fainter ones trailing it (`PrimSTIN3`,
//! `PrimSTIN4`; drawn by `RenderFloorSquare2`, `Part::spinning_square` in `tei/gusts.rs`).
//!
//! The client's primitive looks the one it flies to up at every tick, by the account number its effect was
//! told. Here its effect tells it every tick where that one is: the place is in `at[1]` of the first part,
//! from the effect's place as the primitive was launched, and `flags[5]` is 1 while that one is in sight.

use super::*;

impl Part {
    /// The way from a place to another on the ground, the client's degrees (`Get_Angle`).
    fn way(from: [f32; 3], to: [f32; 3]) -> i32 {
        angle_of(to[0] - from[0], to[2] - from[2])
    }

    /// Whether it points within 3 degrees of a way.
    fn points_along(&self, way: i32) -> bool {
        self.turned >= way - 3 && self.turned <= way + 3
    }

    /// Turns by so many degrees to the side it was told to bow to: against the clock for a part whose
    /// `flags[2]` is 0, with it for any other.
    fn bow_by(&mut self, degrees: i32) {
        self.steer(if self.flags[2] == 0 { -degrees } else { degrees });
    }
}

impl Prim {
    /// `PrimSTIN3` (the kind 122). The first part spins 30 degrees a tick and flies `heights[1]` a tick the
    /// way it is turned. At its first tick it sets off a quarter of a turn to one side of the way to the one
    /// it flies to; over the 19 after it comes up by 15 a tick, to 250 at most. From its sixth tick it turns
    /// 5 degrees a tick towards that one until it points within 3 degrees of it. After its 140th tick (its
    /// 90th for one told so, `flags[4]`) all four parts go down by 5 a tick while it turns on a degree a
    /// tick. With no one to fly to it turns back to where it came from, and goes after its 50th tick.
    pub(super) fn chase_as_gust(&mut self) {
        let Some((first, _)) = self.head_and_tail() else { return };
        let target = (first.flags[5] == 1).then_some(first.at[1]);
        if target.is_none() && first.flags[1] < 10 {
            first.flags[1] = 20;
            if first.process == 1 {
                return;
            }
        }
        first.rise = (first.rise + 30) % 360;
        if first.process == 1 {
            first.turned = Part::way([0.0; 3], target.unwrap_or_default());
            first.bow_by(-90);
        } else if first.process <= 20 {
            first.alpha = (first.alpha + 15).min(250);
        } else if first.process > 140 || (first.process > 90 && first.flags[4] == 1) {
            first.flags[1] = 10;
        }
        let mut fades = false;
        if first.process > 5 {
            match (first.flags[1], target) {
                (0, Some(target)) if !first.points_along(Part::way(first.at[0], target)) => first.bow_by(5),
                (10, _) => {
                    fades = true;
                    first.bow_by(1);
                }
                (20, _) => {
                    if !first.points_along(Part::way(first.at[0], [0.0; 3])) {
                        first.bow_by(5);
                    }
                    if first.process > 50 {
                        first.flags[1] = 10;
                    }
                }
                _ => {}
            }
        }
        if fades {
            self.fade_parts();
        }
        self.parts[0].fly_level();
        self.trail_parts();
    }

    /// `PrimSTIN4` (the kind 125). As the kind 122, but it sets off only 25 to 45 degrees to the side of the
    /// way, by lot, comes up by 8 a tick to 120 at most, and goes (all four parts down by 5 a tick, turning
    /// on a degree a tick) as soon as it points within 3 degrees of the one it flies to, or after its 20th
    /// tick, or with no one to fly to.
    pub(super) fn swerve_as_gust(&mut self) {
        let Some((first, _)) = self.head_and_tail() else { return };
        let target = (first.flags[5] == 1).then_some(first.at[1]);
        if target.is_none() {
            first.flags[1] = 10;
            if first.process == 1 {
                return;
            }
        }
        first.rise = (first.rise + 30) % 360;
        if first.process == 1 {
            first.turned = Part::way([0.0; 3], target.unwrap_or_default());
            let swing = first.lot(21) + 25;
            first.bow_by(-swing);
        } else if first.process <= 20 {
            first.alpha = (first.alpha + 8).min(120);
        } else {
            first.flags[1] = 10;
        }
        let mut fades = false;
        if first.process > 5 {
            match (first.flags[1], target) {
                (0, Some(target)) if first.points_along(Part::way(first.at[0], target)) => first.flags[1] = 10,
                (0, Some(_)) => first.bow_by(5),
                (10, _) => {
                    fades = true;
                    first.bow_by(1);
                }
                _ => {}
            }
        }
        if fades {
            self.fade_parts();
        }
        self.parts[0].fly_level();
        self.trail_parts();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The squares as `TexturedSlashLevel` launches them, told one who stands 40 to the east; `side` is the
    /// side they bow to.
    fn chaser(kind: Tei, side: i8) -> Prim {
        let mut prim = Prim::new(Shape::Tei(kind), "wind");
        prim.duration = 200;
        prim.parts = (0..4)
            .map(|index| {
                let mut part = Part { out: 25.0, rise: index, seed: 7, ..Part::default() };
                (part.at[0][1], part.heights[1], part.flags[2]) = (-10.0, 2.0, side);
                part
            })
            .collect();
        (prim.parts[0].at[1], prim.parts[0].flags[5]) = ([40.0, 0.0, 0.0], 1);
        prim
    }

    #[test]
    fn it_sets_off_a_quarter_to_the_side_and_turns_to_the_one_it_flies_to() {
        let mut prim = chaser(Tei::GustChase, 0);
        prim.tick(200);
        // The way east is 0 degrees; a part that bows against the clock sets off at 90, which is north.
        assert_eq!((prim.parts[0].turned, prim.parts[0].alpha), (90, 0));
        assert!((prim.parts[0].at[0][2] - 2.0).abs() < 1e-4);
        for _ in 0..4 {
            prim.tick(200);
        }
        assert_eq!((prim.parts[0].turned, prim.parts[0].alpha), (90, 60));
        prim.tick(200);
        assert_eq!(prim.parts[0].turned, 85);
        let mut other = chaser(Tei::GustChase, 1);
        other.tick(200);
        assert_eq!(other.parts[0].turned, 270);
        // It goes round and comes to the one it flies to: after some ticks it is nearer than it set off.
        for _ in 0..30 {
            prim.tick(200);
        }
        let at = prim.parts[0].at[0];
        assert!(((at[0] - 40.0).powi(2) + at[2].powi(2)).sqrt() < 40.0);
        assert_eq!(prim.parts[0].alpha, 250);
        // Past its 140th tick all four parts go down by 5 a tick.
        for _ in 36..141 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].alpha, 245);
        for _ in 0..60 {
            prim.tick(200);
        }
        assert!(prim.parts.iter().all(|part| !part.live));
    }

    #[test]
    fn with_no_one_to_fly_to_it_turns_back_and_goes_after_fifty_ticks() {
        let mut prim = chaser(Tei::GustChase, 0);
        prim.tick(200);
        prim.parts[0].flags[5] = 0;
        for _ in 0..50 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].flags[1], 10);
        let alpha = prim.parts[0].alpha;
        prim.tick(200);
        assert_eq!(prim.parts[0].alpha, alpha - 5);
    }

    #[test]
    fn the_kind_125_sets_off_less_to_the_side_is_fainter_and_goes_after_twenty_ticks() {
        let mut prim = chaser(Tei::GustSwerve, 1);
        prim.tick(200);
        // 25 to 45 degrees with the clock from the way east.
        assert!((315..=335).contains(&prim.parts[0].turned));
        for _ in 0..19 {
            prim.tick(200);
        }
        assert!(prim.parts[0].alpha <= 120);
        for _ in 0..40 {
            prim.tick(200);
        }
        assert!(prim.parts.iter().all(|part| !part.live));
        // Told no one at its first tick it does not set off at all.
        let mut lost = chaser(Tei::GustSwerve, 0);
        lost.parts[0].flags[5] = 0;
        lost.tick(200);
        assert_eq!((lost.parts[0].flags[1], lost.parts[0].at[0][0], lost.parts[0].rise), (10, 0.0, 0));
    }
}
