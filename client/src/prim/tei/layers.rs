//! `PP_ASURACASTING` (68) and the kind 147 of the client's primitives: a picture facing the screen over the
//! one the effect is on, drawn ten times over, each drawing larger than the one before and with an alpha of
//! its own, so that it flashes up large and settles (`PrimAsuraCasting`, `PrimRisingFan`; both drawn by
//! `RenderHaloLayers`): the letters of a soul link, the sign of a spell slammed down.
//!
//! A part's `heights[1]` to `[10]` are the alphas of the ten drawings, the largest first; `flags[4]` is the
//! way it lives and is drawn (the client's modes 0 to 7), `flags[0]` which of the primitive's pictures it
//! shows, and `heights[0]` how far to the right of the middle it stands.

use super::*;

impl Part {
    /// The ten drawings come up by `step` a tick, each to its most: `first` for the largest and `apart` more
    /// for each next.
    fn raise_layers(&mut self, step: f32, first: f32, apart: f32) {
        for layer in 1..=10 {
            self.heights[layer] = (self.heights[layer] + step).min(first + apart * (layer - 1) as f32);
        }
    }

    /// Drawings go down by so much a tick, to nothing.
    fn lower_layers(&mut self, layers: std::ops::RangeInclusive<usize>, step: f32) {
        for layer in layers {
            self.heights[layer] = (self.heights[layer] - step).max(0.0);
        }
    }

    /// The client's `ShrinkLast`: a drawing goes down by 5 a tick, and while anything of it shows the part
    /// reaches further by that `growth`.
    fn shrink_last(&mut self, layer: usize, growth: f32) {
        self.heights[layer] -= 5.0;
        if self.heights[layer] > 0.0 {
            self.out *= growth;
        }
        self.heights[layer] = self.heights[layer].max(0.0);
    }

    /// `PrimRisingFan` (the kind 147), for one part: it waits while its count is under one. Over its first 19
    /// ticks its drawings come up by 10 a tick (to 70 for the largest, 20 more for each next, 250 for the
    /// smallest); from then on all but the smallest go down by 4 a tick. Of the modes 6 and 7 the smallest
    /// goes down by 5 a tick after the 90th tick, the part reaching a hundredth further each tick it still
    /// shows. Over its first 49 ticks it reaches 0.2 less a tick.
    pub(super) fn rise_as_fan(&mut self) {
        if self.waits() {
            return;
        }
        if self.process < 20 {
            self.raise_layers(10.0, 70.0, 20.0);
        } else {
            self.lower_layers(1..=9, 4.0);
            if matches!(self.flags[4], 6 | 7) && self.process > 90 {
                self.shrink_last(10, 1.01);
            }
        }
        if self.process < 50 {
            self.out -= 0.2;
        }
    }

    /// `PrimAsuraCasting` (`PP_ASURACASTING`), for one part: it waits while its count is under one. Over its
    /// first 19 ticks its drawings come up by 13 a tick: of the mode 2 to 33 for the largest and 23 more for
    /// each next, of the others to 70 and 20 more. From then on, of the mode 2 all but the two smallest go
    /// down by 2 a tick and after the 100th tick those two by 5; of the others all but the smallest go down
    /// by 5 and the smallest after the 140th tick (the 25th of the mode 4). Of any mode but 2 and 4 it
    /// reaches 0.1 less a tick over its first 49 ticks.
    pub(super) fn flash_as_letter(&mut self) {
        if self.waits() {
            return;
        }
        let mode = self.flags[4];
        if mode == 2 {
            if self.process < 20 {
                self.raise_layers(13.0, 33.0, 23.0);
            } else {
                self.lower_layers(1..=8, 2.0);
                if self.process > 100 {
                    self.shrink_last(9, 1.005);
                    self.shrink_last(10, 1.005);
                }
            }
            return;
        }
        if self.process < 20 {
            self.raise_layers(13.0, 70.0, 20.0);
        } else {
            self.lower_layers(1..=9, 5.0);
            if self.process > if mode == 4 { 25 } else { 140 } {
                self.shrink_last(10, 1.01);
            }
        }
        if self.process < 50 && mode != 4 {
            self.out -= 0.1;
        }
    }

    /// `RenderHaloLayers`, for one part: its ten drawings, the smallest first as the client draws them. Each
    /// is a square that faces the screen (of the modes 6 and 7 a figure wider than tall), 30 over the
    /// effect's place (40 of the modes 2, 4, 6 and 7), `heights[0]` to the right of it; the smallest reaches
    /// `out` from its middle to a corner and each larger one 3 more (1 more of the modes 2, 4 and 5).
    ///
    /// It is white and mixed in; of the modes 6 and 7 its light is added; of the mode 3 it is all but black,
    /// the smallest drawn once more at half its alpha; of the mode 0 the client hands the drawing no colour
    /// at all, which is black. Of the modes 5, 6 and 7 every drawing is drawn once more a tenth larger, its
    /// light added, in blue (the mode 7: in red).
    pub(super) fn halo_layers(&self, pictures: &[&'static str], picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Vec<Rect> {
        let mode = self.flags[4];
        let picture = pictures.get(self.flags[0] as usize).copied().unwrap_or(picture);
        let wide = matches!(mode, 6 | 7);
        let angles = if wide { [145, 35, 325, 215] } else { [135, 45, 315, 225] };
        let lift = if matches!(mode, 2 | 4 | 6 | 7) { 40.0 } else { 30.0 };
        let figure = |reach: f32, lift: f32| angles.map(|angle| to_screen([0.0, -lift, 0.0], reach * cosine(angle as f32) + self.heights[0], reach * sine(angle as f32), right, up));
        let mut rects = Vec::new();
        for layer in (1..=10).rev() {
            let alpha = self.heights[layer] as i32;
            if alpha <= 0 {
                continue;
            }
            let step = (10 - layer) as f32 * if matches!(mode, 2 | 4 | 5) { 1.0 } else { 3.0 };
            let reach = step + self.out;
            let corners = figure(reach, lift);
            match mode {
                0 => rects.push(Rect::new(picture, alpha, corners, (0.0, 1.0), [0; 3], false)),
                3 => {
                    if layer == 10 {
                        rects.push(Rect::new(picture, alpha / 2, corners, (0.0, 1.0), [10; 3], false));
                    }
                    rects.push(Rect::new(picture, alpha, corners, (0.0, 1.0), [10; 3], false));
                }
                _ => rects.push(Rect::new(picture, alpha, corners, (0.0, 1.0), [255; 3], wide)),
            }
            if matches!(mode, 5..=7) {
                let colour = if mode == 7 { [255, 5, 5] } else { [5, 5, 255] };
                rects.push(Rect::new(picture, alpha, figure(reach * 1.1, if mode == 5 { 30.0 } else { 40.0 }), (0.0, 1.0), colour, true));
            }
        }
        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::prim::testing::{RIGHT, UP};

    fn one(kind: Tei, mode: i8, out: f32) -> Prim {
        let mut prim = Prim::new(Shape::Tei(kind), "priest_spell.bmp");
        prim.duration = 500;
        let mut part = Part { out, ..Part::default() };
        part.flags[4] = mode;
        prim.parts.push(part);
        prim
    }

    #[test]
    fn the_sign_of_a_slam_flashes_up_large_settles_and_goes_after_ninety_ticks() {
        let mut prim = one(Tei::RisingFan, 7, 21.0);
        for _ in 0..19 {
            prim.tick(500);
        }
        let part = prim.parts[0].clone();
        assert_eq!((part.heights[1], part.heights[5], part.heights[10]), (70.0, 150.0, 190.0));
        assert!((part.out - (21.0 - 19.0 * 0.2)).abs() < 1e-4);
        // Ten drawings, each twice: white and added, and a tenth larger in red.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!(rects.len(), 20);
        assert_eq!((rects[0].colour, rects[0].adds, rects[0].alpha, rects[1].colour, rects[1].adds), ([255; 3], true, 190.0, [255, 5, 5], true));
        // The smallest reaches as far as the part does, 40 over the place, wider than tall; the picture's
        // left bottom is down and to the left.
        let first = rects[0].corners[0].0;
        assert!((first[0] - part.out * cosine(145.0)).abs() < 1e-3 && (first[1] - (40.0 - part.out * sine(145.0))).abs() < 1e-3);
        // The largest reaches 27 further.
        assert!((rects[18].corners[1].0[0] - (part.out + 27.0) * cosine(35.0)).abs() < 1e-3);
        for _ in 19..90 {
            prim.tick(500);
        }
        let part = &prim.parts[0];
        assert_eq!((part.heights[1], part.heights[9], part.heights[10]), (0.0, 0.0, 190.0));
        for _ in 0..38 {
            prim.tick(500);
        }
        assert_eq!(prim.parts[0].heights[10], 0.0);
        assert!(prim.rects(RIGHT, UP).is_empty());
    }

    #[test]
    fn a_letter_of_a_soul_link_waits_comes_up_and_keeps_its_two_smallest_drawings() {
        let mut prim = one(Tei::AsuraCasting, 2, 5.0);
        (prim.parts[0].process, prim.parts[0].heights[0]) = (-20, -10.0);
        for _ in 0..20 {
            prim.tick(500);
        }
        assert!(prim.rects(RIGHT, UP).is_empty());
        for _ in 0..19 {
            prim.tick(500);
        }
        let part = prim.parts[0].clone();
        assert_eq!((part.heights[1], part.heights[2], part.heights[10], part.out), (33.0, 56.0, 240.0, 5.0));
        // White and mixed in, 10 to the left of the middle, a step of 1 from drawing to drawing.
        let rects = prim.rects(RIGHT, UP);
        assert_eq!((rects.len(), rects[0].colour, rects[0].adds, rects[0].alpha), (10, [255; 3], false, 240.0));
        assert!((rects[0].corners[1].0[0] - (5.0 * cosine(45.0) - 10.0)).abs() < 1e-3 && (rects[9].corners[1].0[0] - (14.0 * cosine(45.0) - 10.0)).abs() < 1e-3);
        for _ in 0..81 {
            prim.tick(500);
        }
        let part = prim.parts[0].clone();
        assert_eq!((part.heights[1], part.heights[8], part.heights[9], part.heights[10]), (0.0, 32.0, 217.0, 240.0));
        prim.tick(500);
        assert_eq!((prim.parts[0].heights[9], prim.parts[0].heights[10]), (212.0, 235.0));
        assert!((prim.parts[0].out - 5.0 * 1.005 * 1.005).abs() < 1e-4);
    }
}
