//! The kind 82 of the client's primitives: one picture over the one the effect is on, facing the screen, that
//! comes up and goes as its kind says and swells and shrinks a little all the while (`PrimAuraGlow`, drawn by
//! `RenderTeiFacingSlots`): the sign over one whose casting is slowed, the mark of Analyze, the flash of a hit.
//!
//! The client writes the camera's turn and tip into the first two parts every tick and turns the square by
//! them (`Tei_TowardScreen`); here the square is laid along the screen's two directions as it is drawn.

use super::*;

impl Part {
    /// `PrimAuraGlow`, for the first part; `flags[4]` is the kind its effect gave it.
    ///
    /// Over its first 299 ticks: of the kinds 1 and 3 it comes up by 10 a tick to 220 and from its 200th
    /// tick goes down by 5; of the kind 22 it comes up by 10 to 220, of the kind 20 by 5 to 200, of the kind
    /// 0 by 1; of the kinds 16 to 18 it is at 100, of 10 to 12 at 150, of 14 at 50, of any other from 10 up
    /// at 250. The kind 2 shows nothing for 40 ticks, comes up by 8 a tick until its 61st and goes down by 8
    /// after, 0.7 wider every tick it shows.
    ///
    /// The kinds 16 and 17, 11 and 12 turn a degree against the clock every other tick; the kinds 10, 13, 14,
    /// 15 and 19 with it. The kinds under 10 but 2 go down by 1 a tick after their 300th, as the kind 2
    /// does. Its swell (`heights[8]`) goes on 2 degrees a tick for a kind under 30.
    pub(super) fn glow_as_aura(&mut self) {
        self.process += 1;
        let kind = self.flags[4];
        if self.process < 300 {
            match kind {
                1 | 3 if self.process < 200 => self.alpha = (self.alpha + 10).min(220),
                1 | 3 => self.alpha = (self.alpha - 5).max(0),
                22 => self.alpha = (self.alpha + 10).min(220),
                0 => self.alpha += 1,
                20 => self.alpha = (self.alpha + 5).min(200),
                16..=18 => self.alpha = 100,
                10..=12 => self.alpha = 150,
                14 => self.alpha = 50,
                10.. => self.alpha = 250,
                _ => {}
            }
        }
        let fades_late = |part: &mut Part| {
            if part.process > 300 && part.alpha > 0 {
                part.alpha -= 1;
            }
        };
        match kind {
            2 => {
                if self.process > 40 {
                    self.alpha += if self.process >= 62 { -8 } else { 8 };
                    if self.alpha > 0 {
                        self.out += 0.7;
                    }
                }
                fades_late(self);
            }
            11 | 12 | 16 | 17 if self.process % 2 == 0 => self.round = (self.round - 1).rem_euclid(360),
            10 | 13..=15 | 19 if self.process % 2 == 0 => self.round = (self.round + 1).rem_euclid(360),
            ..=9 => fades_late(self),
            _ => {}
        }
        self.alpha = self.alpha.clamp(0, 255);
        if kind < 30 {
            self.heights[8] += 2.0;
        }
        if self.heights[8] >= 360.0 {
            self.heights[8] -= 360.0;
        }
        // (It follows its effect: its place stays the effect's, as high over it as it was put, `heights[15]`.)
        self.at[0] = [0.0, -self.heights[15], 0.0];
    }

    /// `RenderTeiFacingSlots`, for one part: a figure of four corners about its place that faces the screen,
    /// each corner as far out as the part reaches (`out`, a twentieth more or less as its swell goes). The
    /// first corner is `round` degrees from the screen's right towards its bottom; the next three follow
    /// against that turn, a quarter apart for a square (`heights[10]` of 0), or 60, 120 and 60 degrees
    /// (1), or 130, 50 and 130 (2), which make it wider than tall or the other way.
    ///
    /// `flags[2]` says how it is laid on. 0 and 3: white, mixed in (3 a little right and down of its place;
    /// 17 to 31 a little down). The others have their light added, the picture turned over: 15 in teal, 17
    /// white, 18 in blue, 19 in green and 20 and 32 in red (18 to 20 twice, the second time paler), 16 white
    /// and mixed in. For 2, 4, 5, 10 and 12 the client's function runs on from one into the next: 2 is drawn
    /// in pale blue, white, pale green, grey, dark grey and teal, one over the other, and each later one of
    /// the row in what is left of it.
    pub(super) fn aura(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let reach = self.out * sine(self.heights[8].trunc()) * 0.05 + self.out;
        let steps = match self.heights[10] {
            shape if shape == 0.0 => [0, 90, 180, 270],
            shape if shape == 1.0 => [0, 60, 180, 240],
            shape if shape == 2.0 => [0, 130, 180, 310],
            _ => [0; 4],
        };
        let look = self.flags[2];
        let (across, down) = match look {
            3 => (3.1, 6.0),
            17..=31 => (0.0, 6.0),
            _ => (0.0, 0.0),
        };
        let corners: [[f32; 3]; 4] = steps.map(|step| {
            let angle = (self.round - step).rem_euclid(360) as f32;
            to_screen(self.at[0], reach * cosine(angle) + across, reach * sine(angle) + down, right, up)
        });
        let turned_over = [corners[1], corners[0], corners[3], corners[2]];
        let added = |colour: [u8; 3]| Rect::new(picture, self.alpha, turned_over, (0.0, 1.0), colour, true);
        const ROW: [(i8, [u8; 3]); 6] = [(2, [130, 130, 250]), (4, [255; 3]), (5, [125, 255, 125]), (10, [150; 3]), (12, [50; 3]), (15, [51, 153, 153])];
        match look {
            0 | 3 => vec![Rect::new(picture, self.alpha, corners, (0.0, 1.0), [255; 3], false)],
            16 => vec![Rect::new(picture, self.alpha, turned_over, (0.0, 1.0), [255; 3], false)],
            17 => vec![added([255; 3])],
            18 => vec![added([5, 5, 255]), added([155, 155, 255])],
            19 => vec![added([5, 255, 5]), added([155, 255, 155])],
            20 => vec![added([255, 5, 5]), added([255, 155, 155])],
            32 => vec![added([255, 5, 5])],
            _ => ROW.iter().skip_while(|(first, _)| *first != look).map(|(_, colour)| added(*colour)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    /// The picture as `TexturedAura` launches it for its kind: how far it reaches, how high it is, how it is
    /// laid on, and the kind the primitive lives by.
    fn aura(reach: f32, lift: f32, look: i8, kind: i8, round: i32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(Tei::Aura), "freeze_a.bmp");
        prim.duration = 400;
        let mut part = Part { out: reach, round, ..Part::default() };
        (part.heights[15], part.flags[2], part.flags[4]) = (lift, look, kind);
        prim.parts.push(part);
        prim
    }

    #[test]
    fn the_flash_of_a_hit_waits_comes_up_widening_and_goes() {
        let mut prim = aura(10.0, 10.0, 2, 2, 135);
        for _ in 0..40 {
            prim.tick(400);
        }
        assert!(prim.rects(RIGHT, UP).is_empty());
        for _ in 0..21 {
            prim.tick(400);
        }
        assert_eq!(prim.parts[0].alpha, 168);
        assert!((prim.parts[0].out - (10.0 + 21.0 * 0.7)).abs() < 1e-3);
        // Six drawings one over the other, the picture turned over, their light added.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 6);
        assert_eq!((rects[0].colour, rects[5].colour, rects[0].adds), ([130, 130, 250], [51, 153, 153], true));
        for _ in 0..21 {
            prim.tick(400);
        }
        assert_eq!(prim.parts[0].alpha, 0);
        // It widens no more once nothing of it shows.
        let wide = prim.parts[0].out;
        prim.tick(400);
        assert_eq!(prim.parts[0].out, wide);
    }

    #[test]
    fn the_sign_of_slow_casting_comes_up_and_goes_after_two_hundred_ticks() {
        let mut prim = aura(6.0, 30.0, 0, 1, 135);
        for _ in 0..30 {
            prim.tick(400);
        }
        assert_eq!(prim.parts[0].alpha, 220);
        for _ in 30..199 {
            prim.tick(400);
        }
        assert_eq!(prim.parts[0].alpha, 220);
        prim.tick(400);
        assert_eq!(prim.parts[0].alpha, 215);
        // White and mixed in, a square 30 over the feet whose first corner is down and to the left.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds), (1, [255; 3], false));
        let reach = 6.0 * sine(40.0) * 0.05 + 6.0;
        let first = rects[0].corners[0].0;
        assert!((first[0] - reach * cosine(135.0)).abs() < 1e-3 && (first[1] - (30.0 - reach * sine(135.0))).abs() < 1e-3 && first[2] == 0.0);
        // The next corner is a quarter round against the turn: down and to the right.
        let second = rects[0].corners[1].0;
        assert!((second[0] - reach * cosine(45.0)).abs() < 1e-3 && (second[1] - first[1]).abs() < 1e-3);
    }

    #[test]
    fn the_mark_of_analyze_shows_at_once_and_does_not_turn() {
        let mut prim = aura(24.0, 10.0, 4, 21, 90);
        prim.tick(400);
        assert_eq!((prim.parts[0].alpha, prim.parts[0].round), (250, 90));
        // From white on down the client's row: five drawings.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour), (5, [255; 3]));
        let mut turning = aura(70.0, 0.0, 10, 10, 20);
        turning.tick(400);
        turning.tick(400);
        assert_eq!((turning.parts[0].alpha, turning.parts[0].round), (150, 21));
    }
}
