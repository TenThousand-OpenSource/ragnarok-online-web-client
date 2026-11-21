//! The kind `PP_ATTACKENERGY2` (106) of the client's primitives: up to four bands of light on a dome before
//! the one the effect is on, each a ring that runs from the dome's tip back to its rim and begins anew
//! (`CEffectPrim::OnProcess`, `case PP_ATTACKENERGY2`; drawn by `RenderAttackEnergy2`): the light off which
//! Kaite throws a spell back.
//!
//! The dome's tip points the way `turned` and a quarter says (`rotStart`), which the client makes anew every
//! tick from the way the one the effect is on faces then, half a turn on (`master->state.parent->render.angle`).
//! A part here is not told of that one: the effect gives the turn as it launches the part, and it stays.

use super::*;

impl Part {
    /// `PP_ATTACKENERGY2` in `CEffectPrim::OnProcess`, for one part: once its count is over nothing its ring
    /// goes a degree a tick from the tip towards the rim (`heights[1]`) and comes up by 10 a tick, to 60 at
    /// the most; at the rim it begins again at the tip, at 10. After 30 ticks it is no more than four times
    /// what is left to the 45th. Where its ring begins goes a degree round a tick (`rise`); and the way the
    /// dome points is made of its turn (`at[1]`, the client's `vecTNow`).
    pub(super) fn run_as_dome(&mut self) {
        if self.waits() {
            return;
        }
        if self.heights[1] >= 90.0 {
            (self.alpha, self.heights[1]) = (10, 0.0);
        } else {
            self.heights[1] += 1.0;
            self.alpha = (self.alpha + 10).min(60);
        }
        if self.process > 30 && (45 - self.process) * 4 <= self.alpha {
            self.alpha = ((45 - self.process) * 4).max(0);
        }
        self.rise += 1;
        if self.rise >= 360 {
            self.rise -= 360;
        }
        let ahead = (self.turned + if self.turned < 270 { 90 } else { -270 }) as f32;
        (self.at[1][0], self.at[1][2]) = (cosine(ahead), sine(ahead));
    }

    /// `RenderAttackEnergy2`, for one part: a band in twenty pieces round the dome's axis, between the ring so
    /// many degrees from the tip as `heights[1]` says and the ring 10 degrees further (at the rim at the most).
    /// A ring is `out` times the sine of its degrees wide, upright and across the way the dome points, its
    /// middle `tall` over the ground and 3 and `tall` times the cosine of its degrees along that way. A piece
    /// shows its 21st of the picture in the colour of the primitive's `size`; the client draws each twice.
    pub(super) fn dome(&self, picture: &'static str, size: f32) -> Vec<Rect> {
        let step = self.round / 20;
        if step <= 0 || self.alpha <= 0 {
            return Vec::new();
        }
        let across = self.turned as f32;
        let ring = |degrees: f32, angle: f32| {
            let wide = self.out * sine(degrees);
            let (aside, along) = (sine(angle) * wide, 3.0 + cosine(degrees) * self.tall);
            [
                cosine(across) * aside + self.at[1][0] * along,
                cosine(angle) * wide - self.tall,
                sine(across) * aside + self.at[1][2] * along
            ]
        };
        let (near, far) = (self.heights[1].trunc(), (self.heights[1] + 10.0).trunc().min(90.0));
        let edge = |(_, offset): (i32, i32)| {
            let turned = self.rise + offset;
            let angle = if self.round == 360 && offset == 360 { self.rise } else if turned >= 360 { turned - 360 } else { turned } as f32;
            (ring(near, angle), ring(far, angle))
        };
        let band = Rect::band(picture, self.alpha, size, 20, edges(self.round, step, true, 20).map(edge));
        [band.clone(), band].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::close;

    /// The parts `AttackEnergy2` launches, the dome pointing south (its turn half a turn on from one who
    /// faces south, 0).
    fn domes() -> Prim {
        let mut domes = Prim::new(Shape::Tei(Tei::AttackEnergy2), "cloud11.tga");
        (domes.duration, domes.size) = (100, 4.0);
        domes.parts.extend((0..4).map(|index| {
            let mut part = Part { tall: 8.0, out: 8.0, round: 360, rise: index * 45, turned: 180, ..Part::default() };
            (part.heights[0], part.heights[1]) = (10.0, (index * 22) as f32);
            part
        }));
        domes
    }

    #[test]
    fn a_ring_runs_from_the_tip_to_the_rim_begins_anew_and_is_gone_by_its_45th_tick() {
        let mut domes = domes();
        domes.tick(100);
        assert_eq!((domes.parts[0].alpha, domes.parts[0].heights[1], domes.parts[1].heights[1]), (10, 1.0, 23.0));
        assert_eq!((domes.parts[0].rise, domes.parts[3].rise), (1, 136));
        for _ in 1..25 {
            domes.tick(100);
        }
        // The fourth began 66 degrees from the tip: at the rim after 24 ticks, at the 25th it begins anew.
        assert_eq!((domes.parts[0].alpha, domes.parts[3].alpha, domes.parts[3].heights[1]), (60, 10, 0.0));
        for _ in 25..40 {
            domes.tick(100);
        }
        assert_eq!(domes.parts[0].alpha, 20);
        for _ in 40..45 {
            domes.tick(100);
        }
        assert!(domes.parts.iter().all(|part| part.alpha == 0) && domes.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).is_empty());
    }

    #[test]
    fn a_band_stands_before_the_one_it_is_on_across_the_way_that_one_faces() {
        let mut domes = domes();
        domes.parts.truncate(1);
        (0..30).for_each(|_| drop(domes.tick(100)));
        let rects = domes.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // Twenty pieces, each drawn twice, blue and added.
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (40, [100, 100, 255], true, 60.0));
        assert_eq!(rects[0], rects[20]);
        // Its near ring is 30 degrees from the tip: 4 wide round a middle 8 over the ground and 3 and 8 times
        // the cosine of 30 degrees to the south. Its first edge is 30 degrees round from straight down.
        let near = rects[0].corners[0].0;
        let south = 3.0 + 8.0 * 30.0_f32.to_radians().cos();
        assert!(close(near[2], -south) && close(near[1], 8.0 - 4.0 * 30.0_f32.to_radians().cos()));
        assert!(close(near[0].abs(), 4.0 * 30.0_f32.to_radians().sin()));
        // The far ring is 10 degrees further from the tip: wider, and nearer the one it is on.
        let far = rects[0].corners[3].0;
        assert!(close(far[2], -(3.0 + 8.0 * 40.0_f32.to_radians().cos())));
        assert!(close((far[0].powi(2) + (far[1] - 8.0).powi(2)).sqrt(), 8.0 * 40.0_f32.to_radians().sin()));
    }
}
