//! `PP_KOUENKA` (124): a sprite that rises wandering from side to side and fades (`PrimKouenka`, drawn by
//! `RenderSegmentSprites2` as the other sprites are): the petals of fire of Kouenka.
//!
//! The client keeps its wandering in its first part: two pairs of angles (`heights[0]` and `[1]`, `[2]` and
//! `[3]`), each a turn that goes towards a mark and a mark drawn anew by lot when the turn has passed it; and
//! how fast it rises (`tall`). So it is here: the primitive has one part for those figures and its lots.

use super::*;

impl Prim {
    /// `PrimKouenka`: each of its two turns wanders (`Part::wander`, `tei/part.rs`). It rises
    /// by its part's `tall` a tick, drifts 0.15 a tick east and north by the sines of the two turns, and goes
    /// down by 3 a tick, to nothing once it is under 3.
    pub(super) fn drift_as_petal(&mut self) {
        let Some(part) = self.parts.first_mut().filter(|part| part.live) else { return };
        part.wander(0);
        part.wander(2);
        self.alpha -= 3.0;
        if self.alpha < 3.0 {
            self.alpha = 0.0;
        }
        self.moved = [self.moved[0] + 0.15 * sine(part.heights[0].trunc()), self.moved[1] + part.tall, self.moved[2] + 0.15 * sine(part.heights[2].trunc())];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A petal as `SakuraFireArrow` launches one, 8 east of the one struck.
    fn petal(turns: [f32; 4]) -> Prim {
        let mut prim = Prim::new(Shape::Kouenka, "이팩트/sakura01");
        (prim.duration, prim.alpha, prim.size) = (300, 250.0, 0.35);
        prim.moved = [8.0, 0.0, 0.0];
        let mut part = Part { tall: 0.45, seed: 3, ..Part::default() };
        part.heights[..4].copy_from_slice(&turns);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn it_rises_drifts_by_the_sines_of_its_turns_and_fades() {
        let mut prim = petal([90.0, 200.0, 0.0, 0.0]);
        prim.tick(300);
        // The first turn goes up to 93 towards its mark; the second is at nothing and its mark is drawn anew.
        let part = prim.parts[0].clone();
        assert_eq!((part.heights[0], part.heights[2], part.heights[3]), (93.0, 0.0, 359.0));
        assert_eq!(prim.alpha, 247.0);
        assert!((prim.moved[0] - (8.0 + 0.15 * sine(93.0))).abs() < 1e-4 && (prim.moved[1] - 0.45).abs() < 1e-5 && prim.moved[2] == 0.0);
        // It shows its sprite where it is, at its size.
        let seen = prim.seen();
        assert!(prim.shows_sprite() && seen.len() == 1 && seen[0].scale == 0.35);
        for _ in 0..82 {
            prim.tick(300);
        }
        assert_eq!(prim.alpha, 0.0);
    }

    #[test]
    fn a_turn_that_passes_its_mark_gets_a_new_one_under_itself() {
        let mut prim = petal([199.0, 200.0, 50.0, 10.0]);
        prim.tick(300);
        let part = &prim.parts[0];
        // 202 is past 200: the new mark is a lot under 202. The other turn goes down towards 10.
        assert!(part.heights[0] == 202.0 && part.heights[1] < 202.0);
        assert_eq!((part.heights[2], part.heights[3]), (48.0, 10.0));
    }
}
