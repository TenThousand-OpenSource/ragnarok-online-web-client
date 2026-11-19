//! The kind `PP_SHIELDBOOMERANG` (42) of the client's primitives: a square facing the screen that is thrown
//! at a place, turning, with three fainter ones behind it, and comes back or flies on (`PrimShieldBoomerang`,
//! drawn by `RenderFieldPanel`, `tei/field.rs`). The shield of Shield Boomerang and the spheres of Finger
//! Offensive are of it.

use super::*;

impl Prim {
    /// `PrimShieldBoomerang`: the first part flies `heights[1]` a tick along its way `turned` and `heights[2]`
    /// a tick down, turning 15 degrees a tick against the clock, until it has gone as far as `tall`. Then
    /// (`flags[1]`) a slow one (`flags[2]` not 1) turns 5 degrees a tick, still flying, until it points within
    /// 4 degrees of the way back to the place of its effect, and from then on goes down by 12 a tick and back
    /// up as it flies; a quick one (`flags[2]` of 1) flies on three times as fast, up to 45 degrees to a side
    /// drawn by lot, fading the same. A quick one comes up by 25 a tick over its first ten ticks. The other
    /// three parts take each the look of the one before, the fourth first, every third tick (a quick one's
    /// every tick): 25 fainter, the second 150 fainter than the first.
    ///
    /// The client measures the way to the place thrown at as the part's count reaches 1, from where the part
    /// is to the primitive's `deltaPos`; the part has not moved by then, and here the effect gives it that way
    /// as it launches it. The way back is to the place of the effect now; here to where the effect was as it
    /// launched the primitive.
    pub(super) fn fly_out_and_back(&mut self) {
        let Some((head, tail)) = self.head_and_tail() else { return };
        let quick = head.flags[2] == 1;
        if quick {
            if head.process <= 10 {
                head.alpha += 25;
            }
            head.alpha = head.alpha.min(255);
        }
        if head.process % if quick { 1 } else { 3 } == 0 {
            Part::trail(head, tail, 150);
        }
        head.round -= 15;
        if head.round < 0 {
            head.round += 360;
        }
        match head.flags[1] {
            0 => {
                if head.tall < head.heights[1] * head.process as f32 {
                    head.turn_at_the_far_end(quick);
                }
                head.at[0][1] += head.heights[2];
            }
            1 => {
                let back = angle_of(-head.at[0][0], -head.at[0][2]);
                if head.turned < back - 4 || head.turned > back + 4 {
                    head.turned += 5;
                    if head.turned >= 360 {
                        head.turned -= 360;
                    }
                } else {
                    head.flags[1] = 2;
                }
            }
            _ => {
                head.alpha = (head.alpha - 12).max(0);
                head.at[0][1] -= head.heights[2];
            }
        }
        head.fly_level();
    }
}

impl Part {
    /// What the first part of the kind 42 does on having gone as far as it was thrown: a slow one begins to
    /// turn back; a quick one flies on three times as fast, up to 44 degrees to the left or to the right.
    fn turn_at_the_far_end(&mut self, quick: bool) {
        if !quick {
            self.flags[1] = 1;
            return;
        }
        self.flags[1] = 2;
        self.heights[1] *= 3.0;
        if self.lot(2) == 0 {
            self.turned += self.lot(45);
            self.turned = under_a_round(self.turned);
        } else {
            self.turned -= self.lot(45);
            if self.turned < 0 {
                self.turned += 360;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::close;

    /// A shield as `TexturedShieldThrow` launches it at a place 20 to the east and level (`quick`: a sphere as
    /// `TANJI` does, without its wait).
    fn thrown(quick: bool) -> Prim {
        let mut shield = Prim::new(Shape::Tei(Tei::ShieldBoomerang), "shield_boomerang.bmp");
        shield.duration = 499;
        let mut head = Part { tall: 20.0, out: 7.0, alpha: if quick { 5 } else { 255 }, seed: 7, ..Part::default() };
        head.at[0] = [0.0, -8.0, 0.0];
        head.heights[1] = 2.0;
        head.flags[2] = i8::from(quick);
        shield.parts.push(head);
        shield.parts.extend((0..3).map(|_| Part::default()));
        shield
    }

    #[test]
    fn a_shield_flies_out_turns_round_and_fades_on_its_way_back() {
        let mut shield = thrown(false);
        for _ in 0..10 {
            shield.tick(499);
        }
        // Ten ticks out at 2 a tick, turning 15 degrees a tick against the clock.
        let head = &shield.parts[0];
        assert!(close(head.at[0][0], 20.0) && close(head.at[0][1], -8.0) && head.at[0][2].abs() < 0.001);
        assert_eq!((head.round, head.flags[1], head.alpha), (210, 0, 255));
        // The eleventh tick finds it past the place: it turns 5 degrees a tick until it points back (west).
        shield.tick(499);
        assert_eq!(shield.parts[0].flags[1], 1);
        let mut turning = 0;
        while shield.parts[0].flags[1] == 1 {
            shield.tick(499);
            turning += 1;
            assert!(turning < 100);
        }
        // It has flown a bow and points at where it was thrown from, within the few degrees of a step.
        let head = &shield.parts[0];
        let back = angle_of(-head.at[0][0], -head.at[0][2]);
        assert!((head.turned - back).abs() <= 10, "{} {back}", head.turned);
        assert_eq!(head.alpha, 255);
        for _ in 0..5 {
            shield.tick(499);
        }
        assert_eq!(shield.parts[0].alpha, 195);
        // The three behind it took its look every third tick, each fainter.
        assert!(shield.parts[1].at[0] != [0.0; 3] && shield.parts[3].at[0] != [0.0; 3]);
        let rects = shield.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(!rects.is_empty() && rects.iter().all(|rect| !rect.adds && rect.colour == [255; 3]));
    }

    #[test]
    fn the_tail_is_the_look_of_the_one_before_fainter() {
        let mut shield = thrown(false);
        for _ in 0..3 {
            shield.tick(499);
        }
        // At the third tick the second takes the first's look, 150 fainter.
        assert_eq!(shield.parts[1].alpha, 105);
        assert_eq!(shield.parts[1].round, 330);
        assert!(close(shield.parts[1].at[0][0], 4.0));
        assert_eq!(shield.parts[2].alpha, 0);
        for _ in 0..3 {
            shield.tick(499);
        }
        assert_eq!((shield.parts[2].alpha, shield.parts[1].alpha, shield.parts[3].alpha), (80, 105, 0));
    }

    #[test]
    fn a_sphere_comes_up_flies_on_faster_to_a_side_and_fades() {
        let mut sphere = thrown(true);
        for _ in 0..10 {
            sphere.tick(300);
        }
        assert_eq!((sphere.parts[0].alpha, sphere.parts[0].flags[1]), (255, 0));
        // Its tail follows every tick; it draws blue, its light added.
        assert!(sphere.parts[3].alpha > 0);
        let rects = sphere.rects([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(rects.iter().all(|rect| rect.adds && rect.colour == [150, 150, 250]));
        sphere.tick(300);
        let head = &sphere.parts[0];
        assert_eq!(head.flags[1], 2);
        assert!(close(head.heights[1], 6.0));
        assert!(head.turned < 45 || head.turned > 315);
        sphere.tick(300);
        assert_eq!(sphere.parts[0].alpha, 243);
    }
}
