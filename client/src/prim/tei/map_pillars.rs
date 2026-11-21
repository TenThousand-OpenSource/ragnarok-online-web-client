//! The kind 41 of the client's primitives: walls of light round the place of the effect that turn slowly, stand
//! up after a long wait and sink again, the pillars of light a map places (`CEffectPrim::OnProcess`,
//! `case 41`, drawn by `Render3DCasting`: `Part::casting`).

use super::*;

impl Part {
    /// The kind 41, for one part: it turns a degree a tick. For its first 199 ticks its edges stand nothing
    /// high; over the next 90 they come up to the wall's height as a sine does, and stay there. From its 800th
    /// tick the wall is 1 lower a tick, down to nothing. One numbered 1 (`alpha_top`) goes down by 1 a tick
    /// after its 250th tick.
    pub(super) fn stand_as_map_pillar(&mut self) {
        self.process += 1;
        self.turned = under_a_round(self.turned + 1);
        if self.alpha_top == 1 && self.process > 250 {
            self.alpha = (self.alpha - 1).max(0);
        }
        if self.process >= 800 {
            self.tall = (self.tall - 1.0).max(0.0);
        }
        let height = match self.process {
            ..200 => 0.0,
            200..=290 => sine((self.process - 200) as f32) * self.tall,
            _ => self.tall,
        };
        self.heights = [height.max(0.0); 21];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The walls of `EF_MAPPILLAR` as `TexturedCrossStaged` launches them.
    fn pillar() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::MapPillar), "ring_blue.tga");
        (prim.duration, prim.size) = (999, 4.0);
        prim.parts = (0..4)
            .map(|index| Part { process: index * 30, round: 360, tall: 120.0, turned: index * 90, alpha: 50, rise: 89, ..Part::default() })
            .map(|part| Part { out: part.process as f32 / 60.0 + 2.0, ..part })
            .collect();
        prim
    }

    #[test]
    fn the_walls_wait_two_hundred_ticks_and_come_up_over_ninety_one_after_the_other() {
        let mut prim = pillar();
        for _ in 0..199 {
            prim.tick(999);
        }
        // The first has lived 199 ticks and shows nothing; the fourth began at 90 and is at its 289th.
        assert_eq!((prim.parts[0].heights[0], prim.parts[0].turned, prim.parts[3].turned), (0.0, 199, 270 + 199 - 360));
        assert!((prim.parts[3].heights[20] - 120.0 * 89.0_f32.to_radians().sin()).abs() < 0.001);
        assert!((prim.parts[1].heights[7] - 120.0 * 29.0_f32.to_radians().sin()).abs() < 0.001);
        for _ in 0..100 {
            prim.tick(999);
        }
        assert_eq!((prim.parts[0].heights[5], prim.parts[0].alpha), (120.0, 50));
        // Upright bands of twenty pieces, blue and added for a primitive of size 4.
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 80);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([100, 100, 255], true, 50.0));
    }

    #[test]
    fn they_sink_from_their_eight_hundredth_tick_and_only_those_numbered_one_fade() {
        let mut prim = pillar();
        prim.parts[1].alpha_top = 1;
        for _ in 0..810 {
            prim.tick(999);
        }
        assert_eq!((prim.parts[0].process, prim.parts[0].tall, prim.parts[0].heights[0]), (810, 109.0, 109.0));
        assert_eq!((prim.parts[3].tall, prim.parts[0].alpha), (120.0 - 101.0, 50));
        // The second began at 30: its 840th tick, 590 of them after its 250th.
        assert_eq!(prim.parts[1].alpha, 0);
        for _ in 0..189 {
            prim.tick(999);
        }
        assert_eq!(prim.parts.iter().map(|part| part.heights[0]).collect::<Vec<_>>(), [0.0; 4]);
    }
}
