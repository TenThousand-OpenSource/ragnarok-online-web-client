//! The drawings of a body's picture, as the flags and the counts that effects have set on it say
//! (`crate::body`): the client's `CPc::ApplyLayerEffects` for a character and
//! `CRenderObject::DrawSpriteEffects` for a monster, with what `PrepareSprite`, `ScaleByPulse`,
//! `DrawStatusOverlays` and `DrawTintedQuad` do to the picture.

use super::{Body, flag};
use crate::prim::sine;

/// One drawing of a body's picture. The picture stands on its feet as it always does, and is then changed in
/// this order: made `scale` times its size about its feet; its edges moved (`edges`: left, top, right, bottom,
/// in pixels of the sprite, to the right and DOWN being more, as on the client's screen; `of_height`: top and
/// bottom, in parts of its height); turned about its middle by `turn`, the client's degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    /// The colour its own colours are multiplied by, and its alpha, each of 255.
    pub colour: [u8; 4],
    /// Whether its light is added to what is behind it (the client's blend 5).
    pub adds: bool,
    pub edges: [f32; 4],
    pub of_height: [f32; 2],
    pub scale: f32,
    pub turn: f32,
    /// Whether it lies under every body, however near: a shadow (the client draws the shadows of all before
    /// the sprite of any: a nearer one's shadow does not lie over the legs of one farther off).
    pub under: bool,
}

impl Quad {
    /// A picture as it is.
    pub const PLAIN: Quad = Quad { colour: [255; 4], adds: false, edges: [0.0; 4], of_height: [0.0; 2], scale: 1.0, turn: 0.0, under: false };

    /// Whether it is the picture as it is.
    pub fn plain(&self) -> bool {
        *self == Self::PLAIN
    }

    /// This one in another colour, wider by so much at every side.
    fn around(self, spread: f32, colour: [u8; 4], adds: bool) -> Self {
        Quad { colour, adds, edges: [self.edges[0] - spread, self.edges[1] - spread, self.edges[2] + spread, self.edges[3] + spread], ..self }
    }

    /// This one in another colour, moved so far right and down.
    fn moved(self, right: f32, down: f32, colour: [u8; 4], adds: bool) -> Self {
        Quad { colour, adds, edges: [self.edges[0] + right, self.edges[1] + down, self.edges[2] + right, self.edges[3] + down], ..self }
    }
}

impl Body {
    /// How a character's weapon is drawn by itself over its body now, if it is, and whether the body under it
    /// is to be the character without its weapon: with the flag 2 the weapon's light is added to what is
    /// behind it in place of the weapon itself; with the flag 4 the weapon is drawn a second time over
    /// itself, its light added, at every other drawing (`CPc::ApplyLayerEffects`, for its layer 5). It is in
    /// the body's colour and of its size; what moves the body's edges is not done to it.
    pub fn weapon(&self) -> Option<(Quad, bool)> {
        let alone = self.flags & flag::WEAPON_ADDS != 0;
        let twice = self.flags & flag::WEAPON_TWICE != 0 && !self.blink_skipped;
        (alone || twice).then(|| (
            Quad { colour: [self.colour[0], self.colour[1], self.colour[2], self.alpha], adds: true, scale: self.scale(), ..Quad::PLAIN },
            alone
        ))
    }

    /// How many times its size it is drawn (`ScaleByPulse`): the flag 0x10 brings it down to half over 45 of
    /// its count, the flag 0x400 up to one and a half.
    pub fn scale(&self) -> f32 {
        let mut scale = 1.0;
        let swing = |phase: u8| {
            let index = u32::from(phase) * 4 + 90;
            (index < 270).then(|| (sine(index as f32) + 1.0) * 0.25)
        };
        if self.flags & flag::SHRINKS != 0 {
            scale *= swing(self.shrink_phase).map_or(0.5, |swing| swing + 0.5);
        }
        if self.flags & flag::GROWS != 0 {
            scale *= swing(self.grow_phase).map_or(1.5, |swing| 1.5 - swing);
        }
        scale
    }

    /// The drawings of its picture now, the first behind the rest. `character` tells a character from a
    /// monster, which the client draws by different functions (`CPc::ApplyLayerEffects`,
    /// `CRenderObject::DrawSpriteEffects`); `height` is how tall the picture is, in its pixels; `lot` is the
    /// client's `rand() % under`.
    pub fn quads(&self, character: bool, height: f32, lot: &mut dyn FnMut(u32) -> u32) -> Vec<Quad> {
        let has = |flag: u32| self.flags & flag != 0;
        let phase = i32::from(self.effect_phase);
        let mut main = Quad { colour: [self.colour[0], self.colour[1], self.colour[2], self.alpha], adds: has(flag::ADDS), scale: self.scale(), ..Quad::PLAIN };
        self.shape(&mut main, lot);
        // A character that flies up or comes down has neither outline nor copies (`flags & 0x30000`).
        let bare = character && has(flag::RISES | flag::LANDS);
        let mut quads = Vec::new();
        let white = |alpha: i32| [255, 255, 255, alpha.clamp(0, 255) as u8];
        if has(flag::OUTLINE) && !bare {
            let wave = sine(f32::from(self.wave_phase));
            if character {
                quads.extend([main.around(wave + 3.0, white(100), true), main.around(wave + wave + 5.0, white(120), true)]);
            } else {
                quads.push(main.around(wave * 1.5 + 5.0, main.colour, true));
            }
        }
        if has(flag::SHADOWS) && character && !has(flag::LANDS) {
            // (The client also draws these a little longer downwards than the body; that is left out.)
            quads.extend([1.0, 2.0].map(|step: f32| Quad {
                colour: [0x37, 0x37, 0xff, self.outline_alpha],
                adds: true,
                scale: main.scale * (1.0 + step * 0.01),
                ..main
            }));
        }
        if has(flag::SHADOWS) && !character {
            quads.extend((1..=10).map(|step| {
                let alpha = if phase > 130 { 245 - step * 10 - phase } else { phase - (step * 5 + 50) };
                main.around((step * 3) as f32, white(alpha.min(200)), true)
            }));
        }
        if has(flag::AURA) && !bare {
            for step in [5.0, 10.0, 15.0, 20.0] {
                let mut spread = f32::from(self.trail_phase) * 0.1 + step;
                if spread >= 20.0 {
                    spread -= 20.0;
                }
                quads.push(main.around(spread, white((100.0 - spread * 5.0) as i32), main.adds));
            }
            main.colour = white(150);
        }
        if has(flag::JITTER) {
            // (`ApplySpriteJitter`: the top, the bottom, the left and the right, in that order of lots.)
            let (spread, base) = if character { (11, 5.0) } else { (21, 10.0) };
            let [top, bottom, left, right] = [(); 4].map(|()| lot(spread) as f32 - base);
            main.edges = [left, top, right, bottom];
        }
        if has(flag::RISES) && character {
            let rise = (phase * 20) as f32;
            (main.edges[1], main.edges[3], main.colour) = (-rise, -rise, white(255 - phase * 20));
        }
        if has(flag::LANDS) && character {
            let tint = i32::from(self.tint_phase);
            let rise = ((25 - tint) * 20).max(0) as f32;
            (main.edges[1], main.edges[3], main.colour) = (-rise, -rise, white((tint - 10) * 17));
        }
        if has(flag::WEAPON_ADDS) && !character {
            // (`DrawSpriteEffects`: the copies 1 and 2 wider at every side, and the body itself blue at 100.)
            let blue = |alpha: u8| [0x37, 0x37, 0xff, alpha];
            quads.extend([1.0, 2.0].map(|spread| main.around(spread, blue(self.outline_alpha), true)));
            main.colour = blue(100);
        }
        quads.push(main);
        quads.extend(self.lights(main));
        if has(flag::BLUR) && !bare {
            // A character's copies go 4 a count, a monster's a twentieth of its height; they are mixed in, in
            // front of the body.
            let far = phase as f32 * if character { 4.0 } else { height * 0.05 };
            let colour = white((30 - phase) * 5);
            let still = Quad { edges: [0.0; 4], ..main };
            quads.extend([(far, 0.0), (-far, 0.0), (0.0, far), (0.0, -far)].map(|(right, down)| still.moved(right, down, colour, false)));
        }
        quads
    }

    /// What `CRenderObject::PrepareSprite` does to where the picture stands: thrown up (0x80), pressed down
    /// (0x20), turned (0x40).
    fn shape(&self, main: &mut Quad, lot: &mut dyn FnMut(u32) -> u32) {
        let phase = i32::from(self.effect_phase);
        if self.flags & flag::KICKED != 0 {
            let up = match phase {
                ..=18 => sine((phase * 5) as f32) * 0.3,
                19..=20 => 0.3,
                21..=23 => (23 - phase) as f32 * 0.1,
                _ => 0.0,
            };
            main.of_height = [-up, -up];
        }
        if self.flags & flag::PRESSED != 0 {
            let left = match phase {
                ..=3 => (100.0 - phase as f32 * 20.0) * 0.01,
                4..=33 => 0.4 - lot(6) as f32 * 0.01,
                _ => 1.0,
            };
            main.of_height[0] += 1.0 - left;
        }
        if self.flags & flag::SPUN != 0 && phase <= 36 {
            let turn = 180.0 - sine((90 + phase * 5) as f32) * 180.0;
            main.turn = if turn >= 360.0 { turn - 360.0 } else { turn };
        }
    }

    /// The lights laid over the body, each added to it: the white flash (0x100; `DrawStatusOverlays`) and the
    /// blue and the red light (0x4000, 0x40000; `DrawTintedQuad`, which draws its light twice).
    fn lights(&self, main: Quad) -> Vec<Quad> {
        let mut lights = Vec::new();
        let phase = i32::from(self.effect_phase);
        if self.flags & flag::FLASH != 0 {
            let alpha = match phase {
                ..=5 => Some(phase * 50),
                6..=8 => Some(255),
                9..=11 => Some(895 - phase * 80),
                _ => None,
            };
            // (The alpha is a byte of the colour: only its low eight bits show.)
            lights.extend(alpha.map(|alpha| Quad { colour: [255, 255, 255, alpha as u8], adds: true, ..main }));
        }
        let tint = i32::from(self.tint_phase);
        let alpha = match tint {
            ..=10 => Some(tint * 15),
            11..=20 => Some(0xa0),
            21..=50 => Some((51 - tint) * 5),
            _ => None,
        };
        for (flag, [red, green, blue]) in [(flag::BLUE, [5, 5, 255]), (flag::RED, [255, 5, 5])] {
            if let Some(alpha) = alpha.filter(|_| self.flags & flag != 0) {
                lights.extend([Quad { colour: [red, green, blue, alpha as u8], adds: true, ..main }; 2]);
            }
        }
        lights
    }
}
