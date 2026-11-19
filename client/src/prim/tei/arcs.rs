//! `PP_HITLINE3` (100) of the client's primitives: sparks that leap out from a place in a bow and come down,
//! each a thin streak of three sides with a tail.

use super::*;

impl Part {
    /// `PP_HITLINE3` in `CEffectPrim::OnProcess`, for one part: once its count is over nothing its head goes 6
    /// degrees further along its bow a tick, up to 180 (`heights[0]`), and the places it had before are kept
    /// behind it, as many as it has pieces (`at[1][1]`, the client's `vecTNow.y`); after 35 ticks it goes down
    /// by 5 a tick. It goes with the one its effect is on.
    pub(super) fn leap_as_spark(&mut self) {
        if self.waits() {
            return;
        }
        if self.process > 35 {
            self.alpha -= 5;
        }
        self.alpha = self.alpha.max(0);
        self.heights[0] = (self.heights[0] + 6.0).min(180.0);
        for piece in (1..=(self.at[1][1] as usize).min(20)).rev() {
            self.heights[piece] = self.heights[piece - 1];
            self.flags[piece] = self.flags[piece - 1];
        }
    }

    /// `RenderHitLine3`, for one part. A place of its bow, so many degrees along, is `out` times a 90th of the
    /// degrees from the effect's place along the way `rise`, and `out` times the sine of the degrees times
    /// `at[1][0]` (the client's `vecTNow.x`) over it, 5 higher still. Each piece is a tube of three sides
    /// between two such places next to each other, `tall` thick, its light added, each piece 10 fainter than
    /// the one before. `alpha_top` gives the colour, which changes from piece to piece; of the kind 3 the tube
    /// is grey and has a blue one four times as thick about it at half the alpha.
    pub(super) fn bow(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 || self.process <= 0 {
            return Vec::new();
        }
        let place = |piece: usize| {
            let degrees = self.heights[piece];
            let along = if degrees > 0.0 { degrees / 90.0 * self.out } else { 0.0 };
            [
                along * cosine(self.rise as f32) + self.at[0][0],
                -self.out * sine(degrees) * self.at[1][0] + self.at[0][1],
                along * sine(self.rise as f32) + self.at[0][2]
            ]
        };
        let mut rects = Vec::new();
        for piece in 0..(self.at[1][1] as usize).min(20) {
            let (head, tail) = (place(piece), place(piece + 1));
            let across = ((angle_of(head[0] - tail[0], head[2] - tail[2]) + 90) % 360) as f32;
            let step = piece as i32;
            let shade = |value: i32| value.rem_euclid(256) as u8;
            let colour = match self.alpha_top {
                0 => [255, 200, shade(150 - (5 - step) * 30)],
                1 => [shade(120 - (5 - step) * 22), shade(120 - (5 - step) * 22), 255],
                2 => [255, 200, shade(150 - (3 - step) * 50)],
                _ => [shade(255 - (5 - step) * 30); 3],
            };
            let mut tube = |thick: f32, alpha: i32, colour: [u8; 3]| {
                let edge = |third: i32| {
                    let wide = thick * cosine((third * 120) as f32);
                    let off = [wide * cosine(across), thick * sine((third * 120) as f32) - 5.0, wide * sine(across)];
                    (std::array::from_fn(|axis| head[axis] + off[axis]), std::array::from_fn(|axis| tail[axis] + off[axis]))
                };
                for third in 1..=3 {
                    let ((head_before, tail_before), (head_now, tail_now)): (([f32; 3], [f32; 3]), ([f32; 3], [f32; 3])) = (edge(third - 1), edge(third));
                    if alpha > 0 {
                        rects.push(Rect::new(picture, alpha, [head_now, tail_now, tail_before, head_before], (0.0, 1.0), colour, true));
                    }
                }
            };
            tube(self.tall, self.alpha - step * 10, colour);
            if self.alpha_top == 3 {
                tube(self.tall * 4.0, ((self.alpha - step * 10) as f32 * 0.5) as i32, [0, 0, 255]);
            }
        }
        rects
    }
}
