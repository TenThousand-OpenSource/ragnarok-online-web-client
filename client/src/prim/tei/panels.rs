//! The kinds 115 and `PP_SHIELDBOOMERANG3` (116) of the client's primitives: squares that face the screen,
//! sparks thrown every way and a picture that flies with a tail of itself.
//!
//! The client lays such a square flat, along its x and z, and turns it to the screen (`Tei_TowardScreen` with
//! the camera's turn and tip, which `FaceCamera` writes into the first two parts every tick): its x then lies
//! along the screen's right and its z down the screen (`Part::facing`).

use super::*;

impl Part {
    /// `PrimParticleSpread` (the kind 115), for one part, which is three sparks (`at[2]`, `at[0]`, `at[3]`):
    /// each goes `tall` a tick along its own way, a tip up from level (`heights[2]`, `[6]`, `[10]`) and a turn
    /// (`heights[0]`, `[4]`, `[8]`). After 25 ticks they slow by a tenth a tick and each goes down by 6 a tick
    /// (`heights[12]` to `[14]`).
    pub(super) fn spread_sparks(&mut self) {
        self.process += 1;
        let speed = self.tall;
        for (spark, tip, turn) in [(2, 2, 0), (0, 6, 4), (3, 10, 8)] {
            let (tip, turn) = (self.heights[tip].trunc(), self.heights[turn].trunc());
            self.at[spark][1] -= speed * sine(tip);
            self.at[spark][0] += speed * cosine(tip) * sine(turn);
            self.at[spark][2] += speed * cosine(tip) * cosine(turn);
        }
        if self.process <= 25 {
            return;
        }
        self.tall = speed * 0.9;
        for alpha in &mut self.heights[12..15] {
            *alpha = (*alpha - 6.0).max(0.0);
        }
    }

    /// `RenderSoulParticle`, for one part: each of its three sparks a square that faces the screen, `out` from
    /// its middle to a corner, at the spark's own alpha, its light added; one whose first corner is under the
    /// level of the effect's place is left out. `heights[15]` gives the colour, from a pale red (1) to a
    /// deeper one (3), or blue (0; of the kind 4 each spark has a blue of its own).
    pub(super) fn sparks(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let kind = self.heights[15];
        [(2, 12, 40), (0, 13, 10), (3, 14, 70)]
            .into_iter()
            .filter_map(|(spark, alpha, shade)| {
                let corners = Self::facing(self.at[spark], self.out, 45, right, up);
                let colour = if kind == 0.0 {
                    [80, 80, 255]
                } else if kind == 1.0 {
                    [255, 200, 200]
                } else if kind == 2.0 {
                    [255, 160, 160]
                } else if kind == 3.0 {
                    [255, 120, 120]
                } else if kind == 4.0 {
                    [shade, shade, 255]
                } else {
                    return None;
                };
                (corners[0][1] <= 0.0).then(|| Rect::new(picture, (self.heights[alpha] as i32).max(0), corners, (0.0, 1.0), colour, true))
            })
            .collect()
    }

    /// `RenderFieldPanel` as it draws a part of a primitive whose first part's `flags[2]` is 0: a square that
    /// faces the screen about `at[0]`, `out` from its middle to a corner and turned by `round`, white and
    /// mixed in. (On one map, `ecl_fild01`, the client holds such a square's alpha to 100; not done here.)
    pub(super) fn panel(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let [first, second, third, fourth] = Self::facing(self.at[0], self.out, self.round, right, up);
        vec![Rect::new(picture, self.alpha, [second, first, fourth, third], (0.0, 1.0), [255; 3], false)]
    }
}

impl Prim {
    /// `PrimShieldBoomerang3`: the first part flies `heights[1]` a tick along its way `turned`, turning 15
    /// degrees a tick against the clock; it comes up by 45 a tick for five ticks (to 255 at most) and goes
    /// down by 10 a tick from the fifteenth. Every third tick the other three take each the look of the one
    /// before, the fourth first: where it is and how it is turned, 25 fainter, the second 150 fainter than the
    /// first.
    pub(super) fn fly_with_a_tail(&mut self) {
        let Some((head, tail)) = self.head_and_tail() else { return };
        if head.process % 3 == 0 {
            Part::trail(head, tail, 150);
        }
        head.round -= 15;
        if head.round < 0 {
            head.round += 360;
        }
        if head.process <= 5 {
            head.alpha = (head.alpha + 45).min(255);
        }
        if head.process >= 15 {
            head.alpha = (head.alpha - 10).max(0);
        }
        head.at[0][0] += head.heights[1] * cosine(head.turned as f32);
        head.at[0][2] += head.heights[1] * sine(head.turned as f32);
    }
}
