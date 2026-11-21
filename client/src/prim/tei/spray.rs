//! The kind 80 of the client's primitives as the spray of a waterfall has it: four parts of four specks each,
//! squares facing the screen that come up out of the ground, drift as they rise and begin anew
//! (`PrimFallingPetals`, drawn by `RenderGroundFog`).
//!
//! Not ported, as no effect ported uses them: the alphas and colours `RenderGroundFog` has for parts whose
//! `flag1[0]` is not 2.

use super::*;

/// The specks of a part: which of its places each is (`vecBPre`, `vecBNow`, `vecTPre`, `vecTNow`, in the order
/// the client moves and draws them), the first of its four turns among the part's figures, and which way a
/// turn carries it east and north.
const SPECKS: [(usize, usize, f32, f32); 4] = [(2, 0, 1.0, 1.0), (0, 4, -1.0, -1.0), (3, 8, 1.0, -1.0), (1, 12, -1.0, 1.0)];

impl Part {
    /// A new place for a speck (`RespawnFallingPoint`; `first` as `CRagEffect::FrostScatter` draws it,
    /// which differs for a narrow part): up to 22 to either side (a narrow part, `flags[12]`: 12, at first
    /// 11), up to 29 under the ground and up to 5 before or behind; a part turned (`flags[11]` of 1) has the
    /// side and the depth changed over.
    fn speck(&mut self, first: bool) -> [f32; 3] {
        let side = match (self.flags[12], first) {
            (0, _) => self.lot(45) - 22,
            (_, true) => self.lot(23) - 11,
            _ => self.lot(25) - 12,
        };
        let (down, deep) = (self.lot(30), self.lot(11) - 5);
        if self.flags[11] == 1 { [deep as f32, down as f32, side as f32] } else { [side as f32, down as f32, deep as f32] }
    }

    /// The specks as `CRagEffect::FrostScatter` begins them: each at a place by lot, its four turns by lot.
    pub fn scatter_as_spray(&mut self) {
        for (speck, turn, ..) in SPECKS {
            self.at[speck] = self.speck(true);
            for figure in turn..turn + 4 {
                self.heights[figure] = self.lot(360) as f32;
            }
        }
    }

    /// `PrimFallingPetals`, for one part: its eight turns wander a degree a tick (`DriftSlotAngles`), and
    /// each speck rises by the part's `tall` a tick; once over the ground it drifts by a hundredth of the
    /// sines of two of the turns, and 8 over the ground it begins anew at a place by lot.
    pub(super) fn rise_as_spray(&mut self) {
        for pair in 0..8 {
            self.wander_by(pair * 2, 1.0, 1.0);
        }
        for (speck, turn, east, north) in SPECKS {
            if self.at[speck][1] < 0.0 {
                self.at[speck][0] += east * sine(self.heights[turn]) * 0.01;
                self.at[speck][2] += north * sine(self.heights[turn + 2]) * 0.01;
                if self.at[speck][1] < -8.0 {
                    self.at[speck] = self.speck(false);
                }
            }
            self.at[speck][1] -= self.tall;
        }
    }

    /// `RenderGroundFog`, for a part whose `flags[0]` is 2: each speck a square that faces the screen, `out`
    /// from its middle to a corner, a grey green whose light is added. It shows once its first corner (the
    /// lower right on the screen) is over the ground: at 50 of 255, less in its first half unit (a hundred
    /// times its height) and from 4 up, where it loses 15 a unit.
    pub(super) fn spray(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let speck = |(speck, ..): (usize, usize, f32, f32)| {
            let corners = Part::facing(self.at[speck], self.out, 45, right, up);
            let down = corners[0][1];
            let mut alpha = 50;
            if down < -4.0 {
                alpha = ((down + 4.0) * 15.0 + 50.0) as i32;
            }
            if down > -0.5 {
                alpha = (down * -100.0) as i32;
            }
            (down <= 0.0).then(|| Rect::new(picture, alpha.max(0), corners, (0.0, 1.0), [70, 120, 100], true))
        };
        SPECKS.into_iter().filter_map(speck).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A primitive of `CRagEffect::FrostScatter` with one part, its specks put by hand.
    fn spray(narrow: i8, turned: i8) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Spray), "freeze_a_small.bmp");
        prim.duration = 99_999_999;
        let mut part = Part { tall: 0.05, out: 4.5, seed: 7, ..Part::default() };
        (part.flags[0], part.flags[11], part.flags[12]) = (2, turned, narrow);
        prim.parts = vec![part];
        prim
    }

    #[test]
    fn specks_begin_under_the_ground_within_the_width_of_the_fall() {
        for (narrow, turned, wide) in [(0, 0, 22.0), (1, 0, 11.0), (0, 1, 22.0), (1, 1, 11.0)] {
            let mut prim = spray(narrow, turned);
            for seed in 0..200 {
                prim.parts[0].seed = seed;
                prim.parts[0].scatter_as_spray();
                for at in prim.parts[0].at {
                    let (side, deep) = if turned == 1 { (at[2], at[0]) } else { (at[0], at[2]) };
                    assert!(side.abs() <= wide && deep.abs() <= 5.0 && (0.0..30.0).contains(&at[1]), "{at:?}");
                }
                assert!(prim.parts[0].heights[..16].iter().all(|turn| (0.0..360.0).contains(turn)));
            }
        }
    }

    #[test]
    fn a_speck_rises_shows_over_the_ground_fades_and_begins_anew() {
        let mut prim = spray(0, 0);
        prim.parts[0].at = [[3.0, 1.0, 0.0]; 4];
        // Facing a camera that looks level from the south: the first corner is to the right and below.
        let rects = |prim: &Prim| prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let low = 4.5 * 45.0_f32.to_radians().sin();
        assert!(rects(&prim).is_empty());
        // 0.05 a tick: after 100 ticks the middle is 4 over the ground, its lower corners 0.82.
        for _ in 0..100 {
            prim.tick(99_999_999);
        }
        let shown = rects(&prim);
        assert_eq!(shown.len(), 4);
        assert!((shown[0].corners[0].0[1] - (4.0 - low)).abs() < 1e-3 && (shown[0].corners[2].0[1] - (4.0 + low)).abs() < 1e-3);
        assert_eq!((shown[0].colour, shown[0].adds, shown[0].alpha), ([70, 120, 100], true, 50.0));
        // The specks have drifted apart, a hundredth of a sine a tick at most.
        let part = &prim.parts[0];
        assert!(part.at.iter().all(|at| (at[0] - 3.0).abs() <= 0.81 && at[2].abs() <= 0.81));
        // Higher it fades, 15 a unit from where its first corner is 4 over the ground.
        for _ in 0..75 {
            prim.tick(99_999_999);
        }
        let fading = rects(&prim)[0].alpha;
        assert!((fading - (50.0 - (7.75 - low - 4.0) * 15.0).trunc()).abs() <= 1.0, "{fading}");
        // Past 8 over the ground it begins anew under it.
        for _ in 0..10 {
            prim.tick(99_999_999);
        }
        assert!(prim.parts[0].at.iter().all(|at| at[1] > -0.5 && at[1] < 30.0));
    }
}
