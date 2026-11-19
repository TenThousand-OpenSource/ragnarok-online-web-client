//! `PP_CHEMICALPROTECTION` (54, `Cast::Rays`): up to four rays of light from the place where the primitive was
//! launched, lying in the flat of the screen (`PrimChemical`, drawn by `RenderChemical`).
//!
//! A part's figures, as the client names them: `riseAngle` is the way the ray points, in whole degrees
//! (`rise`), and `distance` how long it is (`out`).
//!
//! `RenderChemical` draws every ray twice, each time a triangle from the place out to 3 or to 5 degrees on each
//! side of its way: the narrow one white at half the part's alpha, the wide one in a pale blue (130, 130, 255)
//! at all of it, both added to what is behind them. A primitive here is one of the two drawings, as it has one
//! colour: `wide` says which, and the effect gives the wide one its colour.

use super::*;

impl Slot {
    /// A ray as `ShockwaveRing` launches one: pointing that way, 100 long, nothing of it showing yet.
    pub fn ray(way: i32) -> Self {
        Slot::wall(0, 100.0, way, 0.0, 0)
    }

    /// `PrimChemical`: a ray comes up by 18 a tick over its first ten ticks, holds for ten, and goes down by 15
    /// a tick after.
    pub(super) fn shine_as_ray(&mut self) {
        self.process += 1;
        if self.process <= 10 {
            self.alpha += 18;
        } else if self.process > 20 {
            self.alpha = (self.alpha - 15).max(0);
        }
    }

    /// `RenderChemical`, one of its two drawings of a ray: a triangle from `middle` to the two points the ray's
    /// length away, 3 degrees to each side of its way (5 for the wide drawing). The client lays the triangle
    /// flat, x and z, and turns it to the screen (`Tei_TowardScreen` with the camera's pitch and yaw): its x
    /// then lies along the screen's `right` and its z down the screen. The picture's left bottom is at the
    /// place, its right edge across the far end (`RenderTeiRect`, whose second triangle has no room in it).
    pub(super) fn beam(&self, picture: &'static str, wide: bool, middle: [f32; 3], right: [f32; 3], up: [f32; 3]) -> Lit {
        let spread = if wide { 5 } else { 3 };
        let end = |turn: i32| {
            let angle = (self.rise + turn).rem_euclid(360) as f32;
            let (across, down) = (cosine(angle) * self.out, sine(angle) * self.out);
            std::array::from_fn(|axis| middle[axis] + right[axis] * across - up[axis] * down)
        };
        let (near, first, second) = ((middle, [0.0, 1.0]), (end(-spread), [1.0, 1.0]), (end(spread), [1.0, 0.0]));
        let alpha = if wide { self.alpha as f32 } else { (self.alpha as f32 * 0.5).trunc() };
        Lit { picture, corners: [near, first, second, (middle, [0.0, 0.0]), second, near], alpha }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 0.6, -0.8];

    fn rays(wide: bool) -> Prim {
        let mut rays = Prim::new(Shape::Cast(Cast::Rays { wide }), "a");
        rays.duration = 300;
        rays.slots = (0..4).map(|index| Slot::ray(index * 90 + 30)).collect();
        rays
    }

    #[test]
    fn a_ray_comes_up_holds_and_goes() {
        let mut rays = rays(true);
        assert!(rays.lit(RIGHT, UP).is_empty());
        for _ in 0..10 {
            rays.tick(300);
        }
        assert_eq!(rays.slots[0].alpha, 180);
        for _ in 0..10 {
            rays.tick(300);
        }
        assert_eq!(rays.slots[0].alpha, 180);
        rays.tick(300);
        assert_eq!(rays.slots[3].alpha, 165);
        for _ in 0..11 {
            rays.tick(300);
        }
        assert_eq!(rays.slots[0].alpha, 0);
        assert!(rays.lit(RIGHT, UP).is_empty());
    }

    #[test]
    fn a_ray_is_a_triangle_in_the_flat_of_the_screen_the_narrow_one_at_half_its_alpha() {
        let (mut wide, mut narrow) = (rays(true), rays(false));
        wide.tick(300);
        narrow.tick(300);
        let (wide, narrow) = (wide.lit(RIGHT, UP), narrow.lit(RIGHT, UP));
        assert_eq!((wide.len(), wide[0].alpha, narrow[0].alpha, wide[0].picture), (4, 18.0, 9.0, "a"));
        // The second ray points at 120 degrees: its wide drawing ends at 115 and at 125, 100 away, x along
        // the screen's right and z down it.
        let far = wide[1].corners[1].0;
        let (across, down) = (115.0f32.to_radians().cos() * 100.0, 115.0f32.to_radians().sin() * 100.0);
        assert!((far[0] - across).abs() < 0.01 && (far[1] + 0.6 * down).abs() < 0.01 && (far[2] - 0.8 * down).abs() < 0.01);
        assert_eq!(wide[1].corners[0], ([0.0; 3], [0.0, 1.0]));
        // The narrow one ends at 117.
        let far = narrow[1].corners[1].0;
        assert!((far[0] - 117.0f32.to_radians().cos() * 100.0).abs() < 0.01);
    }
}
