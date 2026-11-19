//! The kinds 155 and 170 of the client's primitives: lights that fly off from a place, each a square facing the
//! screen whose way wanders as it goes (`PrimWanderingFlames`, `PrimWanderingFlames2`; both drawn by
//! `RenderTwinGlowSquares`): the sparks of Metallic Sound, the notes of Deep Sleep Lullaby and of
//! Reverberation.
//!
//! A part is two lights. The first is at `at[2]` (`vecBPre`) and goes by `heights[0]` to `[4]`, its alpha
//! `heights[12]`; the second at `at[0]` (`vecBNow`), by `heights[5]` to `[9]` and `heights[13]`. Of the five
//! figures of a light the first is the way it was sent, the second and third the turn it is making for and
//! the turn it has, the fourth and fifth the same for its tip, which stays near 180 degrees: nearly level.

use super::*;

/// The client's `StepTowardAngle`: a turn goes a step towards a mark the short way round; gives back whether
/// it has come to it.
fn step_toward(angle: &mut f32, mark: f32, step: f32) -> bool {
    let apart = *angle - mark;
    if (0.0..180.0).contains(&apart) {
        *angle -= step;
        return mark >= *angle;
    }
    if apart < -180.0 {
        *angle -= step;
        return mark - 360.0 >= *angle;
    }
    *angle += step;
    *angle >= mark
}

impl Part {
    /// `PrimWanderingFlames` (the kind 155) and, `wide`, `PrimWanderingFlames2` (the kind 170), for one part.
    /// Each light turns 2 degrees a tick towards its mark and, come to it, gets a new one by lot: within 45
    /// degrees of the way it was sent, or for the kind 170 anywhere round. Its tip goes a degree a tick
    /// towards a mark of 175 to 185. The lights come up by 25 a tick (to 255) over the first 9 ticks, or 49
    /// for the kind 170, and after the 20th, or the 60th, go down by 3; the part is over once one would go
    /// under nothing. Both go `tall` a tick along their turn; the first sinks by its tip's sine and the
    /// second rises by its own.
    pub(super) fn wander_as_flames(&mut self, wide: bool) {
        self.process += 1;
        for light in [0, 5] {
            let (turn_mark, tip_mark) = (self.heights[light + 1], self.heights[light + 3]);
            if step_toward(&mut self.heights[light + 2], turn_mark, 2.0) {
                self.heights[light + 1] = if wide { self.lot(360) as f32 + self.heights[light] } else { self.heights[light] - 45.0 + self.lot(91) as f32 };
            }
            if step_toward(&mut self.heights[light + 4], tip_mark, 1.0) {
                self.heights[light + 3] = (self.lot(11) + 175) as f32;
            }
        }
        for turn in &mut self.heights[..10] {
            if *turn < 0.0 {
                *turn += 360.0;
            }
            if *turn >= 360.0 {
                *turn -= 360.0;
            }
        }
        let (up_to, from) = if wide { (50, 60) } else { (10, 20) };
        for alpha in 12..14 {
            if self.process < up_to {
                self.heights[alpha] = (self.heights[alpha] + 25.0).min(255.0);
            } else if self.process > from {
                self.heights[alpha] -= 3.0;
                if self.heights[alpha] < 0.0 {
                    (self.live, self.heights[alpha]) = (false, 0.0);
                }
            }
        }
        let speed = self.tall;
        self.at[2][0] += speed * cosine(self.heights[2]);
        self.at[2][2] += speed * sine(self.heights[2]);
        self.at[2][1] -= speed * sine(self.heights[4]);
        self.at[0][0] += speed * cosine(self.heights[7]);
        self.at[0][2] += speed * sine(self.heights[7]);
        self.at[0][1] += speed * sine(self.heights[9]);
    }

    /// `RenderTwinGlowSquares`, for one part: each of its two lights a square that faces the screen about its
    /// place, `out` from its middle to a corner. `flags[0]` says how: 1 white and mixed in at a third of the
    /// light's alpha and over it red, its light added; 2 white with its light added, the picture turned over;
    /// 3 as 2, and the second light not at all; any other as 1 in blue. (The client draws them over
    /// everything; here they are drawn as the other effects are.)
    pub(super) fn twin_glows(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let look = self.flags[0];
        let mut rects = Vec::new();
        for (place, alpha) in [(2, 12), (0, 13)] {
            let alpha = (self.heights[alpha] as i32).max(0);
            let [first, second, third, fourth] = Self::facing(self.at[place], self.out, 45, right, up);
            match look {
                3 if place == 0 => {}
                2 | 3 => rects.push(Rect::new(picture, alpha, [second, first, fourth, third], (0.0, 1.0), [255; 3], true)),
                _ => {
                    let colour = if look == 1 { [255, 50, 50] } else { [50, 50, 255] };
                    rects.push(Rect::new(picture, alpha / 3, [first, second, third, fourth], (0.0, 1.0), [255; 3], false));
                    rects.push(Rect::new(picture, alpha, [first, second, third, fourth], (0.0, 1.0), colour, true));
                }
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    /// A part as `TexturedForkBolt` launches one towards a place to the north (90 degrees), 5 over the feet.
    fn bolt(look: i8) -> Part {
        let mut part = Part { tall: 0.8, out: 3.0, rise: 90, seed: 11, ..Part::default() };
        part.heights[..10].copy_from_slice(&[90.0, 100.0, 80.0, 180.0, 180.0, 90.0, 60.0, 120.0, 180.0, 180.0]);
        (part.at[2], part.at[0], part.flags[0]) = ([0.0, -5.0, 0.0], [0.0, -5.0, 0.0], look);
        part
    }

    #[test]
    fn a_turn_goes_the_short_way_round_to_its_mark() {
        let mut angle = 80.0;
        assert!(!step_toward(&mut angle, 100.0, 2.0) && angle == 82.0);
        let mut angle = 120.0;
        assert!(!step_toward(&mut angle, 60.0, 2.0) && angle == 118.0);
        // From 10 to 350 is down through nothing.
        let mut angle = 10.0;
        assert!(!step_toward(&mut angle, 350.0, 2.0) && angle == 8.0);
        let mut angle = 99.0;
        assert!(step_toward(&mut angle, 100.0, 2.0) && angle == 101.0);
    }

    #[test]
    fn two_sparks_fly_off_each_by_its_own_turn_come_up_and_go() {
        let mut prim = Prim::new(Shape::Tei(Tei::WanderingFlames), "met_particle_1.tga");
        prim.duration = 300;
        prim.parts.push(bolt(2));
        prim.tick(300);
        let part = prim.parts[0].clone();
        // The first has turned from 80 to 82 and the second from 120 to 118; each tip has gone from 180 to
        // 179, which is its mark passed, and has a new mark of 175 to 185.
        assert_eq!((part.heights[2], part.heights[7], part.heights[4], part.heights[12], part.heights[13]), (82.0, 118.0, 179.0, 25.0, 25.0));
        assert!((175.0..=185.0).contains(&part.heights[3]) && (175.0..=185.0).contains(&part.heights[8]));
        assert!(
            (part.at[2][0] - 0.8 * cosine(82.0)).abs() < 1e-5
                && (part.at[2][2] - 0.8 * sine(82.0)).abs() < 1e-5
                && (part.at[2][1] + 5.0 + 0.8 * sine(179.0)).abs() < 1e-5
        );
        assert!((part.at[0][0] - 0.8 * cosine(118.0)).abs() < 1e-5 && (part.at[0][1] + 5.0 - 0.8 * sine(179.0)).abs() < 1e-5);
        // White, the picture turned over, added: one square to a light.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (2, [255; 3], true, 25.0));
        for _ in 1..9 {
            prim.tick(300);
        }
        assert_eq!(prim.parts[0].heights[12], 225.0);
        // A new mark is within 45 degrees of the way the spark was sent.
        for _ in 9..20 {
            prim.tick(300);
            assert!((45.0..=135.0).contains(&prim.parts[0].heights[1]));
        }
        for _ in 20..96 {
            prim.tick(300);
        }
        assert!(!prim.parts[0].live);
    }

    #[test]
    fn the_notes_wander_anywhere_and_last_longer() {
        let mut prim = Prim::new(Shape::Tei(Tei::WanderingFlames2), "melody_a.bmp");
        prim.duration = 200;
        prim.parts.push(bolt(1));
        for _ in 0..49 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].heights[12], 255.0);
        for _ in 49..60 {
            prim.tick(200);
        }
        assert_eq!(prim.parts[0].heights[12], 255.0);
        // Red over white: two squares to a light, the white at a third.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 4);
        assert_eq!((rects[0].alpha, rects[0].adds, rects[1].alpha, rects[1].colour, rects[1].adds), (85.0, false, 255.0, [255, 50, 50], true));
        prim.tick(200);
        assert_eq!(prim.parts[0].heights[12], 252.0);
        // A note of Reverberation shows its first light alone.
        prim.parts[0].flags[0] = 3;
        assert_eq!(prim.rects(RIGHT, UP).len(), 1);
    }
}
