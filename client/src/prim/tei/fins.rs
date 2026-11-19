//! The kinds 154 and 156 of the client's primitives:
//!
//! - 154: two thin wedges of light from a place towards another, 25 degrees to each side of the way there,
//!   that flash up and fade (`PrimFlashFade`, drawn by `RenderBladeFins`): the lines of Great Echo and of
//!   Metallic Sound;
//! - 156: a picture facing the screen that hovers up and down over the one the effect is on, its corners
//!   never quite still (`PrimHoverFollow`, drawn by `RenderJitterQuads`): the note of Reverberation.

use super::*;

impl Part {
    /// `PrimFlashFade`, for one part: over its first 15 ticks it comes up by 13 a tick, to 255 at most; after
    /// its 20th it goes down by 3 a tick and is over once nothing of it shows.
    pub(super) fn flash_and_fade(&mut self) {
        self.process += 1;
        if self.process <= 15 {
            self.alpha = (self.alpha + 13).min(255);
        } else if self.process > 20 {
            self.alpha -= 3;
            self.over_once_gone();
        }
    }

    /// `RenderBladeFins`, for one part: two wedges 6 over the effect's place, each from that place to a point
    /// `heights[0]` away, 25 degrees to one side of the way the part points (`rise`), and `tall` wide there.
    /// Each is drawn twice: mixed in at a quarter of the alpha, and with its light added at the whole of it;
    /// `flags[0]` gives the two colours: pink and red (1), white and white (2), white and blue (any other).
    /// (The client draws them over everything; here they are drawn as the other effects are.)
    pub(super) fn fins(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let (pale, bright) = match self.flags[0] {
            1 => ([255, 89, 182], [255, 25, 25]),
            2 => ([255; 3], [255; 3]),
            _ => ([255; 3], [25, 25, 255]),
        };
        let base = [0.0, -6.0, 0.0];
        [-25, 25]
            .into_iter()
            .flat_map(|side| {
                let angle = (self.rise + side).rem_euclid(360);
                let inner = [self.heights[0] * cosine(angle as f32), -6.0, self.heights[0] * sine(angle as f32)];
                let across = (angle - 90).rem_euclid(360) as f32;
                let outer = [self.tall * cosine(across) + inner[0], -6.0, self.tall * sine(across) + inner[2]];
                let corners = [base, outer, inner, base];
                [Rect::new(picture, self.alpha / 4, corners, (0.0, 1.0), pale, false), Rect::new(picture, self.alpha, corners, (0.0, 1.0), bright, true)]
            })
            .collect()
    }

    /// `PrimHoverFollow`, for the first part: its swell (`heights[8]`) goes on 5 degrees a tick, and it
    /// hovers as a sine does, a degree a tick (`turned`), up to 3 over and under a place 6 over the effect's.
    pub(super) fn hover(&mut self) {
        self.process += 1;
        self.heights[8] = (self.heights[8] + 5.0) % 360.0;
        self.turned = (self.turned + 1) % 360;
        self.at[0] = [0.0, sine(self.turned as f32) * 3.0 - 6.0, 0.0];
    }

    /// `RenderJitterQuads`, for one part: a figure of four corners that faces the screen about its place,
    /// each corner as far out as the part reaches (`out`, a twentieth more or less as its swell goes), the
    /// first `round` degrees from the screen's right towards its bottom and each next 85 to 100 degrees on,
    /// by lot. The client draws those lots at every drawing; here they are drawn from the part's own number
    /// and count, so they change once a tick. `look` (`flags[1]` of the first part) says how it is laid on:
    /// 1 white and mixed in with four blue copies at half the alpha, their light added, 0.2 east, west, north
    /// and south of it; 2 the same all white with its light added; any other white and mixed in with four
    /// white copies added.
    pub(super) fn jitter_quad(&self, picture: &'static str, look: i8, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let mut lots = Part { seed: self.seed ^ (self.process as u32).wrapping_mul(2_654_435_761), ..Part::default() };
        let reach = self.out * sine(self.heights[8].trunc()) * 0.05 + self.out;
        let mut angle = self.round;
        let corners: [[f32; 3]; 4] = std::array::from_fn(|corner| {
            if corner > 0 {
                angle = (angle + lots.lot(16) + 85).rem_euclid(360);
            }
            to_screen(self.at[0], reach * cosine(angle as f32), reach * sine(angle as f32), right, up)
        });
        let figure = |east: f32, north: f32| [corners[1], corners[0], corners[3], corners[2]].map(|at| [at[0] + east, at[1], at[2] + north]);
        let copies = if look == 1 { [5, 5, 255] } else { [255; 3] };
        let mut rects = vec![Rect::new(picture, self.alpha, figure(0.0, 0.0), (0.0, 1.0), [255; 3], look == 2)];
        rects.extend([(0.2, 0.0), (-0.2, 0.0), (0.0, 0.2), (0.0, -0.2)].map(|(east, north)| Rect::new(picture, self.alpha / 2, figure(east, north), (0.0, 1.0), copies, true)));
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
    const UP: [f32; 3] = [0.0, 1.0, 0.0];

    #[test]
    fn the_lines_of_great_echo_flash_up_and_fade_as_two_wedges_to_each_side_of_the_way() {
        let mut prim = Prim::new(Shape::Tei(Tei::BladeFins), "line3.tga");
        prim.duration = 300;
        let mut part = Part { tall: 5.0, out: 100.0, rise: 90, ..Part::default() };
        (part.heights[0], part.heights[2], part.flags[0]) = (50.0, 80.0, 2);
        prim.parts.push(part);
        for _ in 0..15 {
            prim.tick(300);
        }
        assert_eq!(prim.parts[0].alpha, 195);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 4);
        assert_eq!((rects[0].alpha, rects[0].adds, rects[1].alpha, rects[1].adds, rects[1].colour), (48.0, false, 195.0, true, [255; 3]));
        // The first wedge reaches 50 out at 65 degrees, 6 over the ground, from the effect's place.
        let (base, inner) = (rects[0].corners[0].0, rects[0].corners[2].0);
        assert_eq!(base, [0.0, 6.0, 0.0]);
        assert!((inner[0] - 50.0 * cosine(65.0)).abs() < 1e-3 && (inner[2] - 50.0 * sine(65.0)).abs() < 1e-3 && inner[1] == 6.0);
        // The other is at 115 degrees.
        assert!((rects[2].corners[2].0[0] - 50.0 * cosine(115.0)).abs() < 1e-3);
        for _ in 15..86 {
            prim.tick(300);
        }
        assert!(!prim.parts[0].live);
    }

    #[test]
    fn the_note_hovers_and_is_drawn_five_times_with_corners_that_shake() {
        let mut prim = Prim::new(Shape::Tei(Tei::Hover), "melody_a.bmp");
        prim.duration = 150;
        let mut part = Part { out: 6.0, alpha: 120, turned: 89, round: 45, seed: 5, ..Part::default() };
        part.flags[1] = 2;
        prim.parts.push(part);
        prim.tick(150);
        assert!((prim.parts[0].at[0][1] + 3.0).abs() < 1e-4 && prim.parts[0].heights[8] == 5.0);
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 5);
        assert_eq!((rects[0].alpha, rects[0].adds, rects[1].alpha, rects[1].colour), (120.0, true, 60.0, [255; 3]));
        // The second corner handed over is the figure's first: at 45 degrees, down and to the right of a
        // place 3 over the ground.
        let reach = 6.0 * sine(5.0) * 0.05 + 6.0;
        let first = rects[0].corners[1].0;
        assert!((first[0] - reach * cosine(45.0)).abs() < 1e-3 && (first[1] - (3.0 - reach * sine(45.0))).abs() < 1e-3);
        // A copy is 0.2 to the east.
        assert!((rects[1].corners[1].0[0] - first[0] - 0.2).abs() < 1e-4);
        // The corners are others at the next tick.
        let before = rects[0].corners[0].0;
        prim.tick(150);
        assert_ne!(prim.rects(RIGHT, UP)[0].corners[0].0, before);
    }
}
