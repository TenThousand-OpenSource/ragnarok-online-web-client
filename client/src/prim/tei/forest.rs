//! `PP_FOREST` (66) of the client's primitives: shafts of light that slant down to a place from high over it,
//! each a tube of five sides between two places, one inside the other (`PrimForest`, drawn by
//! `RenderPentagon`): the light of Renovatio.
//!
//! A shaft's foot is the part's `vecTNow` (`at[1]`) and its head the part's `vecTPre` (`at[3]`); `tall` is how
//! far its sides stand from its middle.

use super::*;

impl Part {
    /// `PrimForest`, for the part of that number; `flags[0]` is the kind its effect gave it. Of the kinds 11
    /// and 12 it comes up by 1 a tick over its first 40 ticks and after its 140th goes down by 1; of the
    /// kind 13 it comes up so and stays; of any other its alpha is as the effect gave it. The second and the
    /// fourth shaft swell and shrink by half about the width they were given (`heights[0]`), as the sine of
    /// half a degree a tick goes.
    pub(super) fn shine_as_shaft(&mut self, index: i32) {
        self.process += 1;
        self.turned = under_a_round(self.turned);
        let kind = self.flags[0];
        if kind >= 11 {
            if self.process <= 40 {
                self.alpha += 1;
            } else if kind != 13 && self.process > 140 {
                self.alpha -= 1;
            }
            self.alpha = self.alpha.max(0);
        }
        if index == 1 || index == 3 {
            self.tall = sine((self.process % 720 / 2) as f32) * 0.5 + self.heights[0];
        }
    }

    /// `RenderPentagon`, for one part: five sides from the foot to the head, the first where `turned` says
    /// and each next a fifth of a round on, `tall` out from the middle at both ends; a pale green, or blue
    /// for the kind 13, their light added.
    pub(super) fn slanting_shaft(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let colour = if self.flags[0] == 13 { [80, 80, 255] } else { [230, 255, 230] };
        let edge = |point: i32| {
            let angle = ((self.turned + point * 72) % 360) as f32;
            let (east, north) = (self.tall * cosine(angle), self.tall * sine(angle));
            ([self.at[1][0] + east, self.at[1][1], self.at[1][2] + north], [self.at[3][0] + east, self.at[3][1], self.at[3][2] + north])
        };
        (1..=5)
            .map(|point| {
                let ((foot_before, head_before), (foot, head)) = (edge(point - 1), edge(point));
                Rect::new(picture, self.alpha, [foot_before, foot, head, head_before], (0.0, 1.0), colour, true)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shafts as `ForestLight` launches them for its flag 12.
    fn light() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Forest), "cloud11.tga");
        prim.duration = 999;
        prim.parts.extend((0..4).map(|index| {
            let wide = if index == 3 { 1.0 } else { index as f32 * 0.5 + 1.0 };
            let mut part = Part { process: if index == 3 { 180 } else { 0 }, alpha: 30, tall: wide, turned: index * 25, ..Part::default() };
            (part.heights[0], part.at[3], part.flags[0]) = (wide, [-70.0, -300.0, -70.0], 12);
            part
        }));
        prim
    }

    #[test]
    fn the_light_comes_up_holds_and_goes() {
        let mut prim = light();
        for _ in 0..40 {
            prim.tick(999);
        }
        assert_eq!(prim.parts[0].alpha, 70);
        // The second shaft has swollen by half the sine of 20 degrees; the first keeps its width.
        assert!((prim.parts[1].tall - 1.5 - 0.5 * sine(20.0)).abs() < 1e-4 && prim.parts[0].tall == 1.0);
        for _ in 40..140 {
            prim.tick(999);
        }
        assert_eq!(prim.parts[0].alpha, 70);
        for _ in 140..215 {
            prim.tick(999);
        }
        assert_eq!(prim.parts[0].alpha, 0);
        assert!(prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]).iter().all(|rect| rect.picture == "cloud11.tga"));
        // The fourth, which began at a count of 180, only ever goes down: it is gone after 30 ticks.
        assert_eq!(prim.parts[3].alpha, 0);
    }

    #[test]
    fn a_shaft_is_five_sides_from_the_ground_to_high_in_the_south_west() {
        let prim = light();
        let rects = prim.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(rects.len(), 20);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([230, 255, 230], true, 30.0));
        // The first side of the first shaft: its foot 1 to the east of the place, its head 300 up, 70 to the
        // west and south of that.
        assert_eq!((rects[0].corners[0].0, rects[0].corners[3].0), ([1.0, 0.0, 0.0], [-69.0, 300.0, -70.0]));
        // The next side begins a fifth of a round on.
        assert!((rects[1].corners[0].0[0] - cosine(72.0)).abs() < 1e-5 && (rects[1].corners[0].0[2] - sine(72.0)).abs() < 1e-5);
    }
}
