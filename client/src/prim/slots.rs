//! The kinds made of parts. The client gives every primitive four `EffectPrimitiveSlot`s, and some kinds are
//! nothing but those: each part lives by a count of its own (`process`) and has its own alpha, and the
//! primitive only lasts its time. Two are ported:
//!
//! - `PP_PARTICLE_UP` (59): up to four motes, squares that face the screen, turn and rise (`PrimParticleUp`,
//!   drawn by `RenderFieldPanel`);
//! - the kind 60: up to four walls of light standing round the place of the effect, each a band of twenty
//!   pieces whose tops swing up and down (`PrimPulsingColumns`, drawn by `Render3DCasting`).
//!
//! Not ported, as no effect ported uses them: of the walls, the kinds of column other than the first
//! (`flag1` of 1, 2, 3, 33: walls that turn, or rise once and fall), and the lifts `Render3DCasting` gives walls
//! of a height of 6.001 or 24.001; of the motes, the swelling (`height[8]`), and the colours and orders of
//! corners `RenderFieldPanel` has for kinds other than 0 and 11 (`flag1[2]`).

use super::*;

/// A part of a primitive made of parts: the fields of the client's `EffectPrimitiveSlot` the two kinds use.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    /// Whether it still lives (`life`), and how many ticks it has lived (`process`).
    pub live: bool,
    pub process: i32,
    /// Its alpha of 255 (`alphaB`).
    pub alpha: i32,
    /// How far it stands, or reaches, from its middle (`distance`): a wall's radius, a mote's half diagonal.
    pub out: f32,
    /// Of a wall: where round it begins (`rotStart`), how far round it goes (`fullDisplayAngle`), how steeply
    /// it rises (`riseAngle`: 90 is upright, less leans outwards), all in degrees; how tall it gets
    /// (`maxHeight`); the tick from which it fades (`alphaT`); and how high each of its 21 edges stands now
    /// (`height`).
    pub turned: i32,
    pub round: i32,
    pub rise: i32,
    pub tall: f32,
    pub holds: i32,
    pub heights: [f32; 21],
    /// Of a mote: where its middle is, from where the effect was as the primitive was launched (east, up,
    /// north; `vecBNow`); how far it is turned on the screen (`fullDisplayAngle`, kept in `round`); how far it
    /// rises a tick (`height[2]`); whether it shows its second picture for two ticks on the way
    /// (`flag1[1]`), and whether it shows it now (`flag1[0]`).
    pub at: [f32; 3],
    pub climbs: f32,
    /// Of a mote: how far it drifts level a tick (`height[1]`), along the way `turned` (the client's
    /// `rotStart`, which it draws by lot at the mote's first tick; here the effect gives it).
    pub drifts: f32,
    pub flashes: bool,
    pub flashing: bool,
}

impl Slot {
    /// Counts a tick of its life, and says whether it still waits: a part whose count the effect started
    /// under nothing does nothing until the count is over nothing (as `tei::Part::waits`).
    pub(super) fn waits(&mut self) -> bool {
        self.process += 1;
        self.process <= 0
    }

    /// A wall as the effects launch one: all the way round, nothing of it showing yet.
    pub fn wall(turned: i32, out: f32, rise: i32, tall: f32, holds: i32) -> Self {
        Self {
            live: true,
            process: 0,
            alpha: 0,
            out,
            turned,
            round: 360,
            rise,
            tall,
            holds,
            heights: [0.0; 21],
            at: [0.0; 3],
            climbs: 0.0,
            drifts: 0.0,
            flashes: false,
            flashing: false
        }
    }

    /// A mote as the effects launch one: not turned, nothing of it showing yet.
    pub fn mote(at: [f32; 3], out: f32, climbs: f32, flashes: bool) -> Self {
        Self {
            live: true,
            process: 0,
            alpha: 0,
            out,
            turned: 0,
            round: 0,
            rise: 0,
            tall: 0.0,
            holds: 0,
            heights: [0.0; 21],
            at,
            climbs,
            drifts: 0.0,
            flashes,
            flashing: false
        }
    }
}

/// One quad of a primitive made of parts: which of the primitive's pictures, the six corners of its two
/// triangles as places from the effect's (east, up, north) with where of the picture each shows, and its
/// alpha of 255.
#[derive(Clone, Debug, PartialEq)]
pub struct Lit {
    pub picture: &'static str,
    pub corners: [([f32; 3], [f32; 2]); 6],
    pub alpha: f32,
}

impl Prim {
    /// Whether it is of a kind made of parts.
    pub fn has_slots(&self) -> bool {
        matches!(self.kind, Shape::ParticleUp | Shape::Casting3d | Shape::Cast(_) | Shape::Walls(_) | Shape::Tei(_) | Shape::Streak { .. })
    }

    /// One tick of the parts of a kind made of them.
    pub(super) fn tick_slots(&mut self) {
        match self.kind {
            Shape::ParticleUp => self.slots.iter_mut().enumerate().filter(|(_, slot)| slot.live).for_each(|(index, slot)| slot.rise_as_mote(index as i32)),
            Shape::Walls(walls) => self.tick_walls(walls),
            Shape::Cast(cast) => self.tick_casts(cast),
            Shape::Streak { quick } => self.tick_streaks(quick),
            Shape::Tei(kind) => self.tick_parts(kind),
            _ => self.slots.iter_mut().enumerate().filter(|(_, slot)| slot.live).for_each(|(index, slot)| slot.pulse_as_wall(index as i32)),
        }
    }

    /// The quads of a kind made of parts now. `right` and `up` are the screen's two directions in the world,
    /// for the parts that face it.
    pub fn lit(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Lit> {
        let live = self.slots.iter().filter(|slot| slot.live);
        match self.kind {
            Shape::ParticleUp => live.filter(|slot| slot.alpha > 0)
                .map(|slot| slot.panel(self.pictures.get(usize::from(slot.flashing)).copied().unwrap_or(self.picture), right, up))
                .collect(),
            Shape::Casting3d => live.flat_map(|slot| slot.band(self.picture, self.middle([0.0; 3]))).collect(),
            Shape::Walls(walls) => self.lit_walls(walls),
            Shape::Streak { .. } => self.lit_streaks(),
            Shape::Texture3d => self.plate(right, up),
            Shape::Cast(cast) => self.lit_casts(cast, right, up),
            _ => Vec::new(),
        }
    }
}

impl Slot {
    /// `PrimParticleUp`, for the part of that number: it waits while its count is under one; then it turns 5
    /// degrees a tick against the clock, rises, and drifts along its way;
    /// its first tick it only begins, the nine after it comes up by 15 a tick, and from then on it goes down by
    /// 3 a tick, ending at nothing. One that flashes shows its second picture at two ticks, the 35th and 36th
    /// for the first part and 7 earlier for each part after.
    fn rise_as_mote(&mut self, index: i32) {
        if self.waits() {
            return;
        }
        self.round -= 5;
        if self.round < 0 {
            self.round += 360;
        }
        if self.process > 10 {
            if self.flashes {
                let flash = 36 - index * 7;
                self.flashing = self.process == flash - 1 || self.process == flash;
            }
            self.alpha -= 3;
            if self.alpha < 0 {
                (self.alpha, self.live) = (0, false);
            }
        } else if self.process > 1 {
            self.alpha += 15;
        }
        self.at[1] += self.climbs;
        self.at[0] += self.drifts * cosine(self.turned as f32);
        self.at[2] += self.drifts * sine(self.turned as f32);
    }

    /// `PrimPulsingColumns`, for the part of that number, its columns all of the first kind: until the tick it
    /// holds to, it comes up by 5 a tick over its first 15 ticks (to 180 at most), and from then on goes down
    /// by 2 a tick. Each edge stands three quarters of the wall's height and a quarter more or less as a sine
    /// goes round, 34 degrees on from edge to edge and 4 degrees a tick (3 for the second part; for the others
    /// twice 17 from edge to edge and 2 a tick); over the first 90 ticks the whole rises as a sine does.
    fn pulse_as_wall(&mut self, index: i32) {
        self.process += 1;
        if self.process < self.holds || self.holds < 0 {
            if self.process < 16 {
                self.alpha = (self.alpha + 5).min(180);
            }
        } else {
            self.alpha = (self.alpha - 2).max(0);
        }
        for (column, height) in self.heights.iter_mut().enumerate() {
            let column = column as i32;
            let phase = match index {
                0 => column * 34 + self.process * 4,
                1 => column * 34 + self.process * 3,
                _ => (self.process + column * 17) * 2,
            };
            *height = self.tall * 0.25 * sine((phase % 360) as f32) + self.tall * 0.75;
            if self.process < 91 {
                *height *= sine(self.process as f32);
            }
            *height = height.max(0.0);
        }
    }

    /// `RenderFieldPanel`, as it draws a part of its kind 11: a square that faces the screen, its corners its
    /// reach from its middle, the first where its turn points and each next a quarter on. The client lays the
    /// square flat, x and z, and turns it to the screen (`Tei_TowardScreen` with the camera's pitch and yaw):
    /// its x then lies along the screen's right and its z down the screen. The picture's left bottom is on the
    /// first corner, and its right bottom, right top and left top on the next ones (`RenderTeiRect`).
    fn panel(&self, picture: &'static str, right: [f32; 3], up: [f32; 3]) -> Lit {
        let corner = |quarter: i32, u: f32, v: f32| {
            let angle = (self.round + quarter * 90) as f32;
            let (across, down) = (self.out * cosine(angle), self.out * sine(angle));
            (std::array::from_fn(|axis| self.at[axis] + right[axis] * across - up[axis] * down), [u, v])
        };
        let (first, second, third, fourth) = (corner(0, 0.0, 1.0), corner(1, 1.0, 1.0), corner(2, 1.0, 0.0), corner(3, 0.0, 0.0));
        Lit { picture, corners: [first, second, third, fourth, third, first], alpha: self.alpha as f32 }
    }

    /// `Render3DCasting`: the wall in twenty pieces round `middle`, an edge every twentieth of how far round it
    /// goes (the last edge of a whole round is the first again). An edge's foot is on the ground, `out` from
    /// the middle; its top is its height from the foot, leaning outwards as the wall's rise says. The picture
    /// goes once round, a twenty-first to a piece (`RenderTeiRect`), its bottom at the feet.
    pub(super) fn band(&self, picture: &'static str, middle: [f32; 3]) -> Vec<Lit> {
        let step = self.round / 20;
        let edge = |order: usize| {
            let angle = if self.round == 360 && order == 20 { self.turned } else { (self.turned + order as i32 * step) % 360 } as f32;
            let (cos, sin) = (cosine(angle), sine(angle));
            let (lean, lift) = (cosine(self.rise as f32) * self.heights[order], sine(self.rise as f32) * self.heights[order]);
            let foot = [middle[0] + cos * self.out, middle[1], middle[2] + sin * self.out];
            (foot, [foot[0] + cos * lean, foot[1] + lift, foot[2] + sin * lean])
        };
        (1..=20)
            .map(|order| {
                let ((before_foot, before_top), (foot, top)) = (edge(order - 1), edge(order));
                let (left, right) = ((order - 1) as f32 / 21.0, order as f32 / 21.0);
                let corners = [
                    (before_foot, [left, 1.0]), (foot, [right, 1.0]), (top, [right, 0.0]), (before_top, [left, 0.0]),
                    (top, [right, 0.0]), (before_foot, [left, 1.0])
                ];
                Lit { picture, corners, alpha: self.alpha as f32 }
            })
            .collect()
    }
}
