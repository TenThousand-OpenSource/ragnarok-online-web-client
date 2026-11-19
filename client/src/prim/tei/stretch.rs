//! The kind 141 of the client's primitives: a bar of light across the one the effect is on, facing the
//! screen, that is drawn out from one end to twice its width and then fades (`PrimRiseFollow`, drawn by
//! `RenderTaperedQuad`): the strokes of Body Painting.

use super::*;

impl Part {
    /// `PrimRiseFollow`, for one part: it waits while its count is under one. It is drawn out by 5 a tick
    /// (`turned`) to 100; from then on it goes down by 2 a tick.
    pub(super) fn draw_out(&mut self) {
        if self.waits() {
            return;
        }
        self.turned += 5;
        if self.turned > 100 {
            (self.alpha, self.turned) = (self.alpha - 2, 100);
        }
        self.alpha = self.alpha.clamp(0, 255);
    }

    /// `RenderTaperedQuad`, for one part: a bar that faces the screen about its place (`at[0]`), as wide as
    /// the part reaches (`out`). One end is `out` from the middle the way the part points across the screen
    /// (`round`, from the screen's right towards its bottom); the other begins there too and goes back
    /// through the middle as the bar is drawn out, a fiftieth of `out` for each step of `turned`, to as far
    /// on the other side. Pale blue, its light added. (One that still waits has no length and shows nothing.)
    pub(super) fn tapered_quad(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (way, side) = (self.round.rem_euclid(360) as f32, (self.round - 90).rem_euclid(360) as f32);
        let moving = self.out - self.turned as f32 * self.out / 50.0;
        let corner = |along: f32, half: f32| to_screen(self.at[0], half * cosine(side) + along * cosine(way), half * sine(side) + along * sine(way), right, up);
        let half = self.out * 0.5;
        let corners = [corner(moving, half), corner(moving, -half), corner(self.out, -half), corner(self.out, half)];
        vec![Rect::new(picture, self.alpha, corners, (0.0, 1.0), [100, 100, 255], true)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 1.0, 0.0];

    /// The four bars as `TexturedDescendingRings` launches them.
    fn bars() -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::TaperedQuad), "freezing_stick.bmp");
        prim.duration = 200;
        prim.parts = (0..4)
            .map(|index| {
                let mut part = Part { process: index * -10, out: 20.0, round: 180, alpha: 255, ..Part::default() };
                part.at[0][1] = (index * 10 - 30) as f32;
                part
            })
            .collect();
        prim
    }

    #[test]
    fn a_bar_is_drawn_out_for_twenty_ticks_and_then_fades() {
        let mut prim = bars();
        for _ in 0..10 {
            prim.tick(200);
        }
        assert_eq!((prim.parts[0].turned, prim.parts[0].alpha, prim.parts[1].turned), (50, 255, 0));
        // Pointing left across the screen, it reaches from 20 to the left of the middle back to the middle,
        // 20 wide, 30 over the ground.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha), ([100, 100, 255], true, 255.0));
        let (moving, fixed) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert!(moving[0].abs() < 1e-3 && (fixed[0] + 20.0).abs() < 1e-3);
        // (The first corner is the one a quarter of a turn back from the way: under the bar's middle line.)
        assert!((moving[1] - 20.0).abs() < 1e-3 && (fixed[1] - 40.0).abs() < 1e-3);
        for _ in 0..11 {
            prim.tick(200);
        }
        assert_eq!((prim.parts[0].turned, prim.parts[0].alpha), (100, 253));
        // Drawn out whole it reaches as far to the right.
        assert!((prim.rects(RIGHT, UP)[0].corners[0].0[0] - 20.0).abs() < 1e-3);
        for _ in 0..127 {
            prim.tick(200);
        }
        assert!(prim.parts[0].alpha == 0 && prim.parts[3].alpha > 0);
    }
}
