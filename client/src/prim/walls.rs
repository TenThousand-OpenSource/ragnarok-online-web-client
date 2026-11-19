//! Three more kinds made of walls of light round the place of the effect, each wall a band of twenty pieces
//! as the kind 60 is (`slots.rs`), drawn the same way (`Render3DCasting`, `Slot::band`), but living otherwise:
//!
//! - the kind 30 (`Walls::Spell`): walls that lean outwards more and more as they widen, and begin anew while
//!   their primitive has more than 30 ticks left (`CEffectPrim::OnProcess`, `case 30`);
//! - the kind 31 (`Walls::Spin`): walls that turn round and widen fast, come up over ten ticks and fade
//!   (`PrimWaveSpin`); those of a primitive whose `accel` is 99 turn and widen slower and lean further
//!   (`Walls::SlowSpin`);
//! - the kind 149 (`Walls::Wave`): walls that turn slowly, come up over thirty ticks and fade over the last 46
//!   of their primitive, their picture changing every tick (`PrimWaveHeights`, drawn by
//!   `RenderGroundArcGlow`).
//!
//! And one kind that is no wall but lives as these do, by the count and the alpha of its part:
//!
//! - the kind 88 (`Walls::Square`): a square of light lying level over the ground that widens without end,
//!   comes up over ten ticks and fades (`PrimGrowFollow`, drawn by `RenderSizedSquare`).
//!
//! A wall's `holds` is here the number the effect gave it (the client's `alphaT`), by which a kind tells its
//! walls apart.
//!
//! Not ported, as no effect ported uses them: of the kind 149 the walls of the number 10, whose picture changes
//! every third tick and is drawn once.

use super::*;

/// Which of the three kinds a primitive of walls is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Walls {
    Spell,
    Spin,
    /// The kind 31 of a primitive whose `accel` is 99.
    SlowSpin,
    /// `RenderGroundArcGlow` draws every piece twice, once white and mixed in at the wall's alpha and once in
    /// a colour, its light added, at that alpha taken `gain` times (255 at most). A primitive here is one of
    /// the two drawings: `gain` is 1 for the first.
    Wave { gain: i32 },
    /// The kind 88; `quick` for the primitive whose `size` is 113, which comes up three times as fast.
    Square { quick: bool },
}

impl Prim {
    /// One tick of the walls of a primitive of one of the three kinds.
    pub(super) fn tick_walls(&mut self, walls: Walls) {
        let duration = self.duration;
        for (index, slot) in self.slots.iter_mut().enumerate().filter(|(_, slot)| slot.live) {
            match walls {
                Walls::Spell => slot.cast_as_wall(index as i32, duration),
                Walls::Spin => slot.spin_as_wall(index as i32, false),
                Walls::SlowSpin => slot.spin_as_wall(index as i32, true),
                Walls::Wave { .. } => slot.wave_as_wall(index as i32, duration),
                Walls::Square { quick } => slot.grow_as_square(quick),
            }
        }
    }

    /// The quads of the walls now, round the primitive's middle.
    pub(super) fn lit_walls(&self, walls: Walls) -> Vec<Lit> {
        let middle = self.middle([0.0; 3]);
        let live = self.slots.iter().filter(|slot| slot.live);
        match walls {
            Walls::Spell | Walls::Spin | Walls::SlowSpin => live.flat_map(|slot| slot.band(self.picture, middle)).collect(),
            Walls::Wave { gain } => live.flat_map(|slot| slot.glow(self.pictures, gain, middle)).collect(),
            Walls::Square { .. } => live.map(|slot| slot.square(self.picture, middle)).collect(),
        }
    }
}

impl Slot {
    /// How high the edges of a wall of the kinds 30 and 149 stand: its height, and up to three tenths more or
    /// less as a sine goes along the wall, half a round from its first edge to its last, and as another goes
    /// with the ticks, each wall at its own pace.
    fn sway(&mut self, index: i32) {
        let angle = if index * 45 < 90 { self.process + index * 90 } else { (self.process + index * 45) * 2 };
        for (edge, height) in self.heights.iter_mut().enumerate() {
            *height = sine((edge * 9) as f32) * self.tall * 0.3 * sine((angle % 360) as f32) + self.tall;
        }
    }

    /// The kind 30, for the wall of that number of a primitive that lives `duration` ticks. A wall waits while
    /// its count is under one. Then every tick it stands 0.07 further out and leans a degree more outwards
    /// (once it rises at less than 10 degrees it shows nothing). Nearer than 4 it comes up by 10 a tick (by 5
    /// for the walls numbered 22); further out it goes down by 3, and having gone it begins anew, 3.37 out and
    /// rising at 74 degrees (64 for those numbered 22), unless its primitive has 30 ticks or fewer left.
    fn cast_as_wall(&mut self, index: i32, duration: i32) {
        self.process += 1;
        if self.process <= 0 {
            return;
        }
        self.out += 0.07;
        self.rise -= 1;
        if self.rise < 10 {
            (self.rise, self.alpha) = (10, 0);
        }
        if self.out < 4.0 {
            self.alpha += if self.holds == 22 { 5 } else { 10 };
        } else {
            self.alpha -= 3;
            if self.alpha < 1 {
                self.alpha = 0;
                if self.process < duration - 30 {
                    (self.out, self.rise) = (3.37, if self.holds == 22 { 64 } else { 74 });
                }
            }
        }
        self.sway(index);
    }

    /// `PrimWaveSpin`, for the wall of that number: every tick it turns 5 degrees, stands 0.3 further out and
    /// leans a degree more outwards, down to 40 (a `slow` one, of a primitive whose `accel` is 99: 3 degrees,
    /// 0.1 and down to 30). Over its first ten ticks it comes up by 20 a tick (by 10 for
    /// the walls numbered 1); after them it goes down by 8, 5, 5 or 4 a tick for those numbered 2, 3, 4 or 5,
    /// and by 2 for the others. Its edges stand as a sine goes along the wall, nothing at its first and last
    /// edge and its whole height in the middle, and up to 15 of a hundred more or less with the ticks.
    fn spin_as_wall(&mut self, index: i32, slow: bool) {
        self.process += 1;
        self.turned += if slow { 3 } else { 5 };
        if self.turned >= 360 {
            self.turned -= 360;
        }
        self.rise = (self.rise - 1).max(if slow { 30 } else { 40 });
        self.out += if slow { 0.1 } else { 0.3 };
        if self.process <= 10 {
            self.alpha += if self.holds == 1 { 10 } else { 20 };
        } else {
            let fades = match self.holds {
                2 => 8,
                3 | 4 => 5,
                5 => 4,
                _ => 2,
            };
            self.alpha = (self.alpha - fades).max(0);
        }
        let swing = sine(((index * 90 + self.process) % 360) as f32);
        for (edge, height) in self.heights.iter_mut().enumerate() {
            let base = sine((edge * 9) as f32) * self.tall;
            *height = base * 0.15 * swing + base;
        }
    }

    /// `PrimWaveHeights`, for the wall of that number of a primitive that lives `duration` ticks: every tick it
    /// turns 3 degrees; over its first 29 ticks it comes up by 3 a tick, to 100 at most, and over the last 46
    /// of its primitive it goes down by 2 a tick.
    fn wave_as_wall(&mut self, index: i32, duration: i32) {
        self.process += 1;
        if self.process <= 0 {
            return;
        }
        self.turned += 3;
        if self.turned >= 360 {
            self.turned -= 360;
        }
        if self.process < 30 {
            self.alpha = (self.alpha + 3).min(100);
        }
        if self.process > duration - 46 {
            self.alpha = (self.alpha - 2).max(0);
        }
        self.sway(index);
    }

    /// `RenderGroundArcGlow`, one of its two drawings of a wall: the band as `Render3DCasting` makes it, but
    /// standing further out or nearer by the sine of where round it begins, with the picture of its tick (they
    /// come one a tick, over and over), at its alpha taken `gain` times, 255 at most.
    fn glow(&self, pictures: &'static [&'static str], gain: i32, middle: [f32; 3]) -> Vec<Lit> {
        let Some(picture) = pictures.get(self.process.max(0) as usize % pictures.len().max(1)) else { return Vec::new() };
        let wall = Slot { out: sine(self.turned as f32) + self.out, alpha: (self.alpha * gain).min(255), ..self.clone() };
        wall.band(picture, middle)
    }

    /// `PrimGrowFollow`, for a part of the kind 88: it waits while its count is under one; then it reaches 7 of
    /// a hundred further every tick, comes up by 2 a tick over its first ten (by 6 if `quick`) and goes down
    /// by 1 a tick after them.
    fn grow_as_square(&mut self, quick: bool) {
        self.process += 1;
        if self.process <= 0 {
            return;
        }
        if self.process <= 10 {
            self.alpha += if quick { 6 } else { 2 };
        }
        self.out *= 1.07;
        if self.process > 10 {
            self.alpha = (self.alpha - 1).max(0);
        }
    }

    /// `RenderSizedSquare`, for a part of the kind 88 as the effects here launch it (not turned, `riseAngle` of
    /// nothing): a level square as high over the middle as the part was put (`at`), its corners nine tenths of
    /// its reach to the east, north, west and south; the picture's left bottom is on the first, and its right
    /// bottom, right top and left top on the next ones (`RenderTeiRect`).
    fn square(&self, picture: &'static str, middle: [f32; 3]) -> Lit {
        let reach = self.out * 0.1 + self.out * 0.8;
        let corner = |quarter: i32, u: f32, v: f32| {
            let angle = (self.turned + quarter * 90) as f32;
            ([middle[0] + cosine(angle) * reach, middle[1] + self.at[1], middle[2] + sine(angle) * reach], [u, v])
        };
        let (first, second, third, fourth) = (corner(0, 0.0, 1.0), corner(1, 1.0, 1.0), corner(2, 1.0, 0.0), corner(3, 0.0, 0.0));
        Lit { picture, corners: [first, second, third, fourth, third, first], alpha: self.alpha as f32 }
    }
}
