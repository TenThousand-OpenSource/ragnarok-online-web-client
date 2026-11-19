//! The kind 167 of the client's primitives: a sprite drawn four times, once at each of the primitive's four
//! parts, which slide apart over the ground while its frames play (`Prim3DSlotParticle`, drawn by
//! `RenderSegmentSprites`): the ring of fire of an elemental's Fire Wave, the ring of poison of Poison Buster.
//!
//! `Prim3DSlotParticle` is `Prim3DParticle` with one thing before it: every part that lives goes
//! `heights[0]` a tick the way its `turned` says (its `vecBNow`, `at[0]`). `RenderSegmentSprites` draws the
//! sprite's frame about each part's place, every layer of it twice, its light added: once as it is, and once
//! 4 pixels larger on every side. With the primitive's `pattern & 0x200` (`Prim::own_colours`), which the one
//! effect that launches the kind sets, the colours are not the primitive's but made of the layer's own, by the
//! part's `flags[0]` (`Rim`):
//!
//! - 0: the layer's colour at 0.85 of its alpha, and the larger drawing in the layer's red with a little
//!   green and blue (55 of 255 each) at half its alpha;
//! - 1: both in blue (55, 55, 255), at 0.85 and at half of the layer's alpha.
//!
//! - 4 (Water Screw): the client's function writes no colour at all before it draws (what is drawn is what
//!   was left in its memory). Here it is drawn as 0 is, in the layer's own colours: this project's
//!   assumption, since the client's code does not say.
//!
//! For any other number the client's function writes no colour either: such a part is not drawn here. Without the pattern both drawings are in the
//! primitive's colour; no effect launches the kind so, and that is not ported.

use super::*;

/// One of the two drawings `RenderSegmentSprites` makes of a layer: the part's `flags[0]`, and whether it is
/// the larger one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rim {
    pub look: i8,
    pub outer: bool,
}

impl Rim {
    /// The colour of a layer whose file gives it `layer` (red, green, blue, alpha, each of 255), each part of
    /// 1; nothing for a look the client has no colour for.
    pub fn colour(&self, layer: [u8; 4]) -> Option<[f32; 4]> {
        let alpha = (f32::from(layer[3]) * if self.outer { 0.5 } else { 0.85 }).trunc();
        let [red, green, blue] = match (self.look, self.outer) {
            // (4: no colour in the client; taken as 0 here, see above.)
            (0 | 4, false) => [layer[0], layer[1], layer[2]],
            (0 | 4, true) => [layer[0], 0x37, 0x37],
            (1, _) => [0x37, 0x37, 0xff],
            _ => return None,
        };
        Some([f32::from(red) / 255.0, f32::from(green) / 255.0, f32::from(blue) / 255.0, alpha / 255.0])
    }

    /// The corners of a layer's quad (`crate::sprite::LayerQuad::corners`: a place and where of the picture
    /// it shows) for this drawing: as they are, or for the larger one each 4 further out along both of the
    /// quad's own sides. The client moves the quad's left edge 4 to the left and its right edge 4 to the
    /// right before it turns the quad, so a mirrored layer, whose left and right have changed places, gets
    /// narrower instead.
    pub fn widened(&self, corners: [[f32; 4]; 6]) -> [[f32; 4]; 6] {
        if !self.outer {
            return corners;
        }
        // The quad's two sides, as steps of one: from its picture's left to its right and from top to bottom.
        let step = |from: [f32; 4], to: [f32; 4]| {
            let (x, y) = (to[0] - from[0], to[1] - from[1]);
            let length = f32::hypot(x, y).max(f32::MIN_POSITIVE);
            [x / length, y / length]
        };
        let (across, down) = (step(corners[0], corners[1]), step(corners[0], corners[2]));
        let mirrored = if across[0] * down[1] - across[1] * down[0] < 0.0 { -1.0 } else { 1.0 };
        corners.map(|[x, y, u, v]| {
            let (wider, taller) = ((2.0 * u - 1.0) * 4.0 * mirrored, (2.0 * v - 1.0) * 4.0);
            [x + across[0] * wider + down[0] * taller, y + across[1] * wider + down[1] * taller, u, v]
        })
    }
}

impl Prim {
    /// Whether it is of the kind 167.
    pub fn is_fourfold(&self) -> bool {
        self.kind == Shape::Fourfold
    }

    /// `Prim3DSlotParticle`: its parts slide apart, and it flies as `Prim3DParticle` does (which, launched
    /// with no speed, is not at all).
    pub(super) fn spread_fourfold(&mut self) {
        for part in self.parts.iter_mut().filter(|part| part.live) {
            part.at[0][0] += part.heights[0] * cosine(part.turned as f32);
            part.at[0][2] += part.heights[0] * sine(part.turned as f32);
        }
        self.fly();
    }

    /// `RenderSegmentSprites`: every drawing of the sprite now, for each of the four parts (living or not, as
    /// in the client) and each place the primitive keeps (`Prim::seen`): where the sprite's origin is from
    /// the effect's place (east, up, north), how many times its own size it is, and which of a layer's two
    /// drawings it is. Nothing for any other kind.
    pub fn fourfold(&self) -> Vec<([f32; 3], f32, Rim)> {
        if !self.is_fourfold() || !self.own_colours {
            return Vec::new();
        }
        let mut drawings = Vec::new();
        for part in &self.parts {
            for seen in self.seen() {
                let at = [seen.at[0] + part.at[0][0], seen.at[1] - part.at[0][1], seen.at[2] + part.at[0][2]];
                drawings.extend([false, true].map(|outer| (at, seen.scale, Rim { look: part.flags[0] as i8, outer })));
            }
        }
        drawings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The primitive as `SpriteRing` launches its first: four parts 5 out and 5 up, an eighth of a round
    /// apart, that slide outwards 0.3 a tick.
    fn ring(look: i8) -> Prim {
        let mut prim = Prim::new(Shape::Fourfold, "이팩트/m_ef03");
        (prim.duration, prim.own_colours, prim.adds, prim.repeats) = (500, true, true, false);
        prim.parts.extend((0..4).map(|index| {
            let mut part = Part { turned: index * 45, ..Part::default() };
            let turned = part.turned as f32;
            (part.heights[0], part.flags[0]) = (0.3, look);
            part.at[0] = [cosine(turned) * 5.0, -5.0, sine(turned) * 5.0];
            part
        }));
        prim
    }

    #[test]
    fn the_four_slide_apart_while_the_frames_play_and_end_with_the_last() {
        let mut prim = ring(0);
        assert!(prim.shows_sprite() && prim.is_fourfold());
        // A sprite of three frames, one every fourth tick: the count 0 brings the second.
        assert!(prim.tick_on(&mut Master::left(500), 3, &|_, _| 0.0));
        assert_eq!(prim.motion, 1);
        for _ in 0..3 {
            assert!(prim.tick_on(&mut Master::left(500), 3, &|_, _| 0.0));
        }
        // Four ticks: each part 1.2 further out the way it is turned.
        assert!((prim.parts[0].at[0][0] - 6.2).abs() < 1e-4 && (prim.parts[2].at[0][2] - 6.2).abs() < 1e-4);
        assert!(prim.tick_on(&mut Master::left(500), 3, &|_, _| 0.0));
        for _ in 0..3 {
            prim.tick_on(&mut Master::left(500), 3, &|_, _| 0.0);
        }
        assert!(!prim.tick_on(&mut Master::left(500), 3, &|_, _| 0.0));
    }

    #[test]
    fn each_part_is_drawn_twice_at_its_own_place() {
        let mut prim = ring(1);
        prim.tick(500);
        let drawings = prim.fourfold();
        assert_eq!(drawings.len(), 8);
        // The second part: to the north east, 5 over the ground, at the sprite's own size.
        let (at, scale, rim) = drawings[2];
        let out = 5.3 * cosine(45.0);
        assert!((at[0] - out).abs() < 1e-4 && at[1] == 5.0 && (at[2] - 5.3 * sine(45.0)).abs() < 1e-4 && scale == 1.0);
        assert_eq!((rim, drawings[3].2), (Rim { look: 1, outer: false }, Rim { look: 1, outer: true }));
        // The ordinary drawing of a sprite is left to this one, and a primitive of another kind has none.
        assert!(Prim::new(Shape::Particle3d, "").fourfold().is_empty());
    }

    #[test]
    fn a_layer_is_drawn_in_its_own_colour_and_larger_in_red_or_both_in_blue() {
        let layer = [200, 100, 50, 255];
        let near = |colour: Option<[f32; 4]>, wanted: [f32; 4]| colour.is_some_and(|colour| (0..4).all(|part| (colour[part] - wanted[part]).abs() < 1e-6));
        assert!(near(Rim { look: 0, outer: false }.colour(layer), [200.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0, 216.0 / 255.0]));
        assert!(near(Rim { look: 0, outer: true }.colour(layer), [200.0 / 255.0, 55.0 / 255.0, 55.0 / 255.0, 127.0 / 255.0]));
        assert!(near(Rim { look: 1, outer: false }.colour(layer), [55.0 / 255.0, 55.0 / 255.0, 1.0, 216.0 / 255.0]));
        assert!(near(Rim { look: 1, outer: true }.colour(layer), [55.0 / 255.0, 55.0 / 255.0, 1.0, 127.0 / 255.0]));
        // The client has no colour for any other look.
        // Water Screw's, which the client gives no colour: as the first here.
        assert_eq!(Rim { look: 4, outer: false }.colour(layer), Rim { look: 0, outer: false }.colour(layer));
        assert_eq!(Rim { look: 2, outer: false }.colour(layer), None);
    }

    #[test]
    fn the_larger_drawing_is_four_pixels_out_on_every_side() {
        // A quad 10 wide and 6 tall as `layer_quads` gives one: left top, right top, left bottom, and again.
        let quad = |left: f32, right: f32| [
            [left, 0.0, 0.0, 0.0], [right, 0.0, 1.0, 0.0], [left, 6.0, 0.0, 1.0], [left, 6.0, 0.0, 1.0],
            [right, 0.0, 1.0, 0.0], [right, 6.0, 1.0, 1.0]
        ];
        let plain = quad(0.0, 10.0);
        assert_eq!(Rim { look: 0, outer: false }.widened(plain), plain);
        let wide = Rim { look: 0, outer: true }.widened(plain);
        assert_eq!((wide[0], wide[5]), ([-4.0, -4.0, 0.0, 0.0], [14.0, 10.0, 1.0, 1.0]));
        // A mirrored one, its picture's left on the right, gets narrower and taller.
        let narrow = Rim { look: 0, outer: true }.widened(quad(10.0, 0.0));
        assert_eq!((narrow[0], narrow[5]), ([6.0, -4.0, 0.0, 0.0], [4.0, 10.0, 1.0, 1.0]));
    }
}
