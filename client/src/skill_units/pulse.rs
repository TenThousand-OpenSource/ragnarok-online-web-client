//! The box of light that stands on a cell of Sanctuary and of Magnus Exorcismus: the client's primitive of the
//! kind 64 (`PP_RECT_UP`), as `CRagEffect::TexturedPulseKind` launches it for `EF_BOTTOM_SANC` (its kind 1) and
//! `EF_BOTTOM_MAG` (its kind 0), moves it (`CEffectPrim::PrimPulseRing`) and draws it
//! (`CEffectPrim::RenderRectUp`): four walls on the sides of a square, whose height swings.
//!
//! The other kinds of `TexturedPulseKind` (2, 3, 4) are not here: no unit drawn here uses them. With the
//! client's option for fewer effects on, Magnus's box is 10 tall instead of 50; here the option is off.

use crate::prim::sine;

/// The box, with the figures of its one live part (the client's `EffectPrimitiveSlot`).
#[derive(Clone, Debug, PartialEq)]
pub struct Pulse {
    /// The picture of its walls, among the client's pictures of effects; their colour, each of 255; and whether
    /// their light is added to what is behind them (`RenderTeiRect` told 0 for its `pw`) or mixed in.
    pub picture: &'static str,
    pub colour: [f32; 3],
    pub adds: bool,
    /// How far each wall stands from the middle (`height[0]`, `height[1]`), and how tall they get (`maxHeight`).
    half: f32,
    tall: f32,
    /// Its alpha of 255 (`alphaB`), and the tick from which it goes down (`alphaT`).
    alpha: i32,
    holds: i32,
    /// Ticks lived (`process`), the angle its height swings by (`fullDisplayAngle`), and how tall the walls are
    /// now (`height[2]`).
    process: i32,
    angle: i32,
    height: f32,
}

impl Pulse {
    /// The box of an effect, its swing begun at `lot` degrees (the client draws them by lot, 0 to 359); nothing
    /// for an effect that is no such box.
    pub fn of(effect: u32, lot: u32) -> Option<Self> {
        let (picture, colour, adds, half, tall, alpha, holds) = match effect {
            super::EF_BOTTOM_MAG => ("ring_red.tga", [255.0; 3], true, 5.0, 50.0, 30, 864),
            super::EF_BOTTOM_SANC => ("alpha_down.tga", [235.0, 255.0, 235.0], false, 2.5, 16.0, 120, 1880),
            _ => return None,
        };
        Some(Self { picture, colour, adds, half, tall, alpha, holds, process: 0, angle: (lot % 360) as i32, height: 0.0 })
    }

    /// `PrimPulseRing`: the angle goes on a degree; past the tick it holds to, the alpha goes down by 2; the
    /// walls stand at 65 of 100 of their height and 35 more or less as the angle's sine goes, and over the
    /// first 90 ticks the whole rises as a sine does.
    pub fn tick(&mut self) {
        self.process += 1;
        self.angle = (self.angle + 1) % 360;
        if self.process > self.holds {
            self.alpha = (self.alpha - 2).max(0);
        }
        self.height = self.tall * sine(self.angle as f32) * 0.35 + self.tall * 0.65;
        if self.process < 90 {
            self.height *= sine(self.process as f32);
        }
    }

    /// Its alpha now, of 255.
    pub fn alpha(&self) -> f32 {
        self.alpha as f32
    }

    /// `RenderRectUp`: the four walls now, each as the four corners of `RenderTeiRect` (the two at its foot,
    /// then the two at its top, going back), as places from the unit's own (east, up, north) with where of
    /// the picture each shows. The feet stand 1 over the ground; the whole picture is on every wall, its bottom
    /// at the feet. Nothing while it does not show.
    pub fn walls(&self) -> Vec<[([f32; 3], [f32; 2]); 4]> {
        if self.alpha <= 0 {
            return Vec::new();
        }
        let half = self.half;
        let corners = [[half, half], [half, -half], [-half, -half], [-half, half]];
        (0..4)
            .map(|side| {
                let (from, to) = (corners[side], corners[(side + 1) % 4]);
                [
                    ([from[0], 1.0, from[1]], [0.0, 1.0]),
                    ([to[0], 1.0, to[1]], [1.0, 1.0]),
                    ([to[0], 1.0 + self.height, to[1]], [1.0, 0.0]),
                    ([from[0], 1.0 + self.height, from[1]], [0.0, 0.0])
                ]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanctuarys_box_rises_and_swings() {
        let mut light = Pulse::of(crate::skill_units::EF_BOTTOM_SANC, 0).unwrap();
        assert_eq!((light.picture, light.adds, light.alpha()), ("alpha_down.tga", false, 120.0));
        light.tick();
        // At its first tick: 65 of 100 of 16 and 35 of the sine of one degree, all by the sine of one degree.
        let first = (16.0 * 1f32.to_radians().sin() * 0.35 + 16.0 * 0.65) * 1f32.to_radians().sin();
        assert!((light.height - first).abs() < 1e-4);
        for _ in 1..90 {
            light.tick();
        }
        // After 90 ticks it has risen whole, and the swing is at its top: the full 16.
        assert!((light.height - 16.0).abs() < 1e-3);
        let walls = light.walls();
        assert_eq!(walls.len(), 4);
        assert_eq!(walls[0][0], ([2.5, 1.0, 2.5], [0.0, 1.0]));
        assert!((walls[0][2].0[1] - 17.0).abs() < 1e-3 && walls[0][2].1 == [1.0, 0.0]);
        // Each wall begins where the one before ended.
        assert_eq!(walls[1][0].0, walls[0][1].0);
    }

    #[test]
    fn the_box_fades_after_the_tick_it_holds_to() {
        let mut light = Pulse::of(crate::skill_units::EF_BOTTOM_MAG, 400).unwrap();
        assert_eq!((light.picture, light.adds, light.angle), ("ring_red.tga", true, 40));
        for _ in 0..864 {
            light.tick();
        }
        assert_eq!(light.alpha(), 30.0);
        light.tick();
        assert_eq!(light.alpha(), 28.0);
        for _ in 0..20 {
            light.tick();
        }
        assert!(light.walls().is_empty());
        assert_eq!(Pulse::of(25, 0), None);
    }
}
