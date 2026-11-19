//! The kind 92 of the client's primitives: squares that lie level and slide along the ground. The kind 86 is the
//! same, moved and drawn by the same lines (`CEffectPrim::OnProcess`, `case 86`; `RenderFloorSquare`), but that
//! each part takes its way at its first tick: the way to the place its effect aims at, up to 15 degrees to
//! either side by lot (5 for a part whose `flags[1]` is 1). Here the effect that launches such a part gives it
//! that way, so a primitive of the kind 86 is one of the kind 92 (`Tei::FloorSquare`).

use super::*;

impl Part {
    /// The kind 92 in `CEffectPrim::OnProcess`, for one part: once its count is over nothing it slides
    /// `heights[1]` a tick along the way `turned`. From its second tick it comes up by 10 a tick and slides 2
    /// in a hundred faster each tick; from its eleventh it goes down by 5 a tick, sliding 4 in a hundred
    /// faster, and is over once nothing of it shows.
    pub(super) fn slide_as_square(&mut self) {
        let before = self.process;
        if self.waits() {
            return;
        }
        if before != 0 {
            if before < 10 {
                self.alpha += 10;
                self.heights[1] *= 1.02;
            } else {
                self.alpha -= 5;
                self.heights[1] *= 1.04;
                self.over_once_gone();
            }
        }
        self.fly_level();
    }

    /// `RenderFloorSquare`, for one part: a level square about its place (`at[0]`), its corners `out` from it,
    /// the first an eighth of a turn on from the way it slides and each next a quarter on; mixed in, in a pale
    /// blue, or green for a part whose `flags[1]` is 2. (A part whose `flags[1]` is 1 runs through four
    /// pictures there; no effect ported gives a part that, and it is not ported.)
    pub(super) fn level_square(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let corner = |quarter: i32| {
            let angle = ((self.turned + 45 + quarter * 90) % 360) as f32;
            [self.out * cosine(angle) + self.at[0][0], self.at[0][1], self.out * sine(angle) + self.at[0][2]]
        };
        vec![Rect::new(
            picture,
            self.alpha,
            [corner(0), corner(1), corner(2), corner(3)],
            (0.0, 1.0),
            if self.flags[1] == 2 { [0, 255, 0] } else { [200, 200, 255] },
            false
        )]
    }
}

impl Part {
    /// `PrimSpinFade` (the kind 103), for one part: it turns 3 to 5 degrees a tick, drawn by lot, and comes up
    /// by 1 a tick for forty ticks. `flags[5]` says when it goes: 1 goes down by 1 a tick after the 120th, 10
    /// never, any other after the sixtieth.
    pub(super) fn spin_and_fade(&mut self) {
        if self.waits() {
            return;
        }
        self.turned += self.lot(3) + 3;
        self.turned = under_a_round(self.turned);
        if self.flags[5] == 1 {
            if self.process > 120 {
                self.alpha -= 1;
            }
        } else if self.flags[5] != 10 && self.process > 60 {
            self.alpha -= 1;
        }
        if self.process <= 40 {
            self.alpha += 1;
        }
        self.alpha = self.alpha.max(0);
    }

    /// `RenderStackedRings`, for one part: `flags[0]` squares one under the other about the effect's place,
    /// the first `heights[0]` over it and `tall` from the middle to a corner, each next `heights[1]` lower,
    /// smaller (by `out` times 2.2 less a fifth for each square before it) and an eighth of a turn on. Two
    /// corners next to each other of every square stand 1 higher than the other two. `flags[4]` gives the
    /// colour and the blend; the kinds 8 and 11 show at a part of their alpha, and 11 is drawn twice.
    pub(super) fn stacked_squares(&self, picture: &'static str) -> Vec<Rect> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let looks: &[(i32, [u8; 3], bool)] = match self.flags[4] {
            0 => &[(1, [50, 50, 255], true)],
            1 => &[(1, [164, 136, 55], false)],
            2 => &[(1, [155, 255, 155], false)],
            3 => &[(1, [130, 130, 255], false)],
            4 => &[(1, [255, 50, 50], false)],
            5 => &[(1, [5; 3], false)],
            6 => &[(1, [255; 3], false)],
            7 => &[(1, [255, 125, 255], false)],
            8 => &[(10, [225, 225, 255], false)],
            10 => &[(1, [125, 125, 255], true)],
            11 => &[(3, [255; 3], false), (1, [255, 55, 55], true)],
            _ => &[],
        };
        let mut rects = Vec::new();
        let (mut down, mut radius) = (-self.heights[0], self.tall);
        for square in 0..i32::from(self.flags[0]) {
            if square > 0 {
                down += self.heights[1];
                radius -= (2.2 - square as f32 * 0.2) * self.out;
            }
            let corner = |quarter: i32, down: f32| {
                let angle = ((self.turned + 45 * square + quarter * 90) % 360) as f32;
                [radius * cosine(angle), down, radius * sine(angle)]
            };
            let corners = [corner(0, down - 1.0), corner(1, down - 1.0), corner(2, down), corner(3, down)];
            rects.extend(looks.iter().map(|&(share, colour, adds)| Rect::new(picture, self.alpha / share, corners, (0.0, 1.0), colour, adds)));
        }
        rects
    }
}
