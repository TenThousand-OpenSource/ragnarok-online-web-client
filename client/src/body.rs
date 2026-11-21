//! What an effect makes of the body of the one it is on: the client's effects (`CRagEffect`) reach their parent
//! and colour it (`CRenderObject::SetColor`), turn it (`render.angle`), and have it drawn in other ways for as
//! long as they say so (`render.flags`, with the counts `effectPhase`, `tintPhase`, `shrinkPhase`,
//! `growPhase` and `outlineAlpha` beside them). This file has what an effect does at a tick (`Touch`) and what a
//! body keeps of it (`Body`), the colour its states give it among that; `body/drawn.rs` has what is drawn of a
//! body (`Body::quads`, `Quad`). `web/world/ailments.rs` keeps a body for everyone and `web/world/gpu/draw.rs`
//! draws it.
//!
//! How the client has it:
//!
//! - The flags last one pass: every actor puts them back to nothing as it is processed, once it has been
//!   drawn with them (`CGameActor::UpdateStateEffects`, `flagsResetPending`), and an effect sets its flags
//!   again at every tick it wants them. Here an effect says at each tick what it does to its parent (`Touch`),
//!   and a body has the flags of the effects that run on it now.
//! - The colour is one field of the actor (`render.colorChannels`), the same one the body and health states
//!   write (`CGameActor::SetStatusStates`). An effect writes over it whatever holds; every pass the actor
//!   brings a colour that is not white 8 nearer to white (`ModifyArgb(8, 8, 8, 8)` in its `OnProcess`), unless
//!   a state has locked it (`colorLocked`). So the colour of an effect fades when the effect is over, the
//!   colour of a state stays, and an effect on one whose colour a state has locked leaves its last colour
//!   there until the state is told again. That is kept here as it is.
//! - The alpha of that colour is written and comes back in the same way (`SetColor` with an alpha alone:
//!   `EF_CLOAKING`). It is the alpha the body itself is drawn at; the copies and the lights that flags lay
//!   about it keep the alphas the client gives them. (One who hides or cloaks by its state is not drawn at
//!   all, `crate::pose::hidden`: the two do not meet.)
//! - The turn is the actor's own way of facing: it stays turned when the effect is over.
//!
//! The client draws a character a layer at a time (body, head, what it wears) and a monster as one picture;
//! here everyone is one picture, and what the client does to each layer is done to the whole. The one layer
//! the client treats by itself is the weapon's (`CPc::ApplyLayerEffects`, its layer 5): with the flag 2 its
//! light is added to what is behind it, with the flag 4 it is drawn once more over itself, its light added, at
//! every other drawing. For that a character has a second picture of the same frame that holds the weapon
//! alone (`web/figure/weapon.rs`), drawn over the first as `Body::weapon` says; with the flag 2 the first is
//! the character without its weapon. (So the weapon lies over the head where the two meet, whichever of them
//! the client's order of layers has in front.) A monster with the flag 2 is blue with two blue copies behind
//! it (`CRenderObject::DrawSpriteEffects`).
//!
//! The flag 0x8000 leaves footprints behind a character (`CPc::UpdateFootprints`): `Body::stepped` says when
//! and how each is turned, and the world launches the effect of one where the character is. Not here:
//! 0x80000 and 0x100000.
//!
//! A pixel of the client's screen at its usual distance (`400 / desiredDistance` of one otherwise) is a pixel
//! of the sprite here. The counts that the client moves on at every drawing (`wavePhase`, `trailPhase`) move
//! on at every tick here.

/// The colour of a body nothing colours.
pub const WHITE: [u8; 3] = [255; 3];

/// The flags of `render.flags` drawn here, by what each does.
pub mod flag {
    /// The body's light is added to what is behind it.
    pub const ADDS: u32 = 1;
    /// The light of a character's weapon is added to what is behind it; a monster is blue, with two blue
    /// copies a touch larger behind it (as see-through as `outlineAlpha`).
    pub const WEAPON_ADDS: u32 = 2;
    /// A character's weapon is drawn once more over itself, its light added, at every other drawing.
    pub const WEAPON_TWICE: u32 = 4;
    /// A pale copy of the body a little larger behind it, that swells and shrinks.
    pub const OUTLINE: u32 = 8;
    /// It shrinks to half its size (by `shrinkPhase`).
    pub const SHRINKS: u32 = 0x10;
    /// It is pressed down from the top (by `effectPhase`).
    pub const PRESSED: u32 = 0x20;
    /// It turns about its middle, once round (by `effectPhase`).
    pub const SPUN: u32 = 0x40;
    /// It is thrown up and comes down (by `effectPhase`).
    pub const KICKED: u32 = 0x80;
    /// A white flash over it (by `effectPhase`).
    pub const FLASH: u32 = 0x100;
    /// Four pale copies behind it that widen and fade, over and over; the body itself is see-through.
    pub const AURA: u32 = 0x200;
    /// It grows to one and a half times its size (by `growPhase`).
    pub const GROWS: u32 = 0x400;
    /// Blue copies a touch larger behind a character (as see-through as `outlineAlpha`); ten white ones that
    /// widen behind a monster (by `effectPhase`).
    pub const SHADOWS: u32 = 0x800;
    /// Four copies of it that fly apart, left, right, up and down, and fade (by `effectPhase`).
    pub const BLUR: u32 = 0x1000;
    /// Its edges jump about.
    pub const JITTER: u32 = 0x2000;
    /// A blue light over it that comes and goes (by `tintPhase`).
    pub const BLUE: u32 = 0x4000;
    /// A character leaves footprints behind it (from the eighth tick of the effect that says so:
    /// `runStartTick`).
    pub const RUNS: u32 = 0x8000;
    /// It flies up and fades (by `effectPhase`).
    pub const RISES: u32 = 0x10000;
    /// It comes down from above, coming into sight (by `tintPhase`).
    pub const LANDS: u32 = 0x20000;
    /// A red light over it that comes and goes (by `tintPhase`).
    pub const RED: u32 = 0x40000;
}

/// What an effect does to its parent at one tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Touch {
    /// The flags it sets (`flag`).
    pub flags: u32,
    /// The colour it writes (`SetColor(-1, red, green, blue)`).
    pub colour: Option<[u8; 3]>,
    /// The alpha it writes (`SetColor(alpha, -1, -1, -1)`): how much of the body is left to see, of 255.
    pub alpha: Option<u8>,
    /// The counts it writes.
    pub effect_phase: Option<i16>,
    pub tint_phase: Option<u8>,
    pub shrink_phase: Option<u8>,
    pub grow_phase: Option<u8>,
    pub outline_alpha: Option<u8>,
    /// How far it turns its parent, the client's degrees (what it adds to `render.angle`).
    pub turn: f32,
    /// The tick it is at as it sets the flag of running, a byte (`runStartTick`).
    pub run_tick: Option<u8>,
    /// The picture it holds its parent in: an action's first number and the frame of it (`SetFrameSequence`
    /// with one frame for the first and the last).
    pub held: Option<(u8, u8)>,
    /// The colour it writes, and locks, on a character whose health state says it is poisoned (the bit 1).
    pub if_poisoned: Option<[u8; 3]>,
}

impl Touch {
    /// `SetColor(-1, red, green, blue)`; a figure out of a byte's range is cut to its low byte, as the client's
    /// cast does.
    pub fn colour(&mut self, red: i32, green: i32, blue: i32) {
        self.colour = Some([red as u8, green as u8, blue as u8]);
    }

    /// Sets flags and writes a colour with them: what an effect does that has its parent lit (`flag::ADDS`),
    /// outlined (`flag::OUTLINE`) or both in a colour, for the ticks it says.
    pub fn flagged(&mut self, flags: u32, red: i32, green: i32, blue: i32) {
        self.flags |= flags;
        self.colour(red, green, blue);
    }

    /// Sets a flag and the `effectPhase` that goes with it.
    pub fn phased(&mut self, flag: u32, phase: i32) {
        self.flags |= flag;
        self.effect_phase = Some(phase as i16);
    }

    /// Sets a flag and the `tintPhase` that goes with it.
    pub fn tinted(&mut self, flag: u32, phase: i32) {
        self.flags |= flag;
        self.tint_phase = Some(phase as u8);
    }

    /// An effect flashing its parent between a colour and white, as many of the client's do: at an even tick
    /// its light is added to what is behind it and it is in the colour, at an odd one it is white.
    pub fn flashes(&mut self, tick: i32, colour: [i32; 3]) {
        if tick % 2 == 0 {
            self.flagged(flag::ADDS, colour[0], colour[1], colour[2]);
        } else {
            self.colour(255, 255, 255);
        }
    }
}

/// The body of someone, as effects and states have left it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// The colour it is drawn in (`render.colorChannels`), and whether a state keeps it from fading back to
    /// white (`colorLocked`).
    colour: [u8; 3],
    locked: bool,
    /// The alpha it is drawn at (the alpha of `render.colorChannels`): what an effect wrote, coming back to
    /// the whole 8 a pass as the colour does.
    alpha: u8,
    /// The colour its states gave it last (`crate::ailments::change`): what the next telling of its states
    /// goes on from, whatever effects have written over the colour it is drawn in since.
    stated: [u8; 3],
    flags: u32,
    effect_phase: i16,
    tint_phase: u8,
    shrink_phase: u8,
    grow_phase: u8,
    outline_alpha: u8,
    wave_phase: u8,
    trail_phase: u8,
    /// How far effects have turned it that has not been taken yet (`turned`), the client's degrees.
    turn: f32,
    /// Whether the second drawing of the weapon was left out last (`blinkSkipped`).
    blink_skipped: bool,
    /// The tick the effect of running was at (`runStartTick`), the count towards the next footprint
    /// (`footprintCounter`) and where the last one was looked at (`footprintPosition`), east and north.
    run_tick: u8,
    steps: i32,
    stepped_from: Option<[f32; 2]>,
    /// The picture an effect holds it in now: an action's first number and the frame of it.
    held: Option<(u8, u8)>,
}

/// A footprint left where a character is: of which foot, and how it is turned (the client's degrees, as the
/// effect of a footprint is told them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Print {
    pub left: bool,
    pub turned: i32,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            colour: WHITE,
            locked: false,
            alpha: 255,
            stated: WHITE,
            flags: 0,
            effect_phase: 0,
            tint_phase: 0,
            shrink_phase: 0,
            grow_phase: 0,
            outline_alpha: 0,
            wave_phase: 0,
            trail_phase: 0,
            turn: 0.0,
            blink_skipped: false,
            run_tick: 0,
            steps: 0,
            stepped_from: None,
            held: None
        }
    }
}

impl Body {
    /// A state writes its colour (`SetStatusStates`): any but white is locked.
    pub fn stated(&mut self, colour: [u8; 3]) {
        (self.colour, self.locked, self.stated) = (colour, colour != WHITE, colour);
    }

    /// An effect writes a colour and locks it as a state's is (`EF_BLACKBODY` on one poisoned); what its states
    /// gave it stays what it was.
    pub fn locked(&mut self, colour: [u8; 3]) {
        (self.colour, self.locked) = (colour, colour != WHITE);
    }

    /// The colour its states gave it last.
    pub fn state_colour(&self) -> [u8; 3] {
        self.stated
    }

    /// So many passes of the actor go by (`OnProcess`): each brings a colour that is not locked 8 nearer to
    /// white and its alpha 8 nearer to the whole (`ModifyArgb(8, 8, 8, 8)`), and the counts of the outline and
    /// the aura move on.
    pub fn pass(&mut self, ticks: u32) {
        let ticks = ticks.min(255);
        if !self.locked {
            self.colour = self.colour.map(|part| part.saturating_add((ticks * 8).min(255) as u8));
            self.alpha = self.alpha.saturating_add((ticks * 8).min(255) as u8);
        }
        if self.flags & flag::OUTLINE != 0 {
            self.wave_phase = ((u32::from(self.wave_phase) + ticks) % 181) as u8;
        }
        if self.flags & flag::AURA != 0 {
            self.trail_phase = ((u32::from(self.trail_phase) + ticks) % 200) as u8;
        }
        // (The client turns this over at every drawing of the weapon; here at every tick.)
        if self.flags & flag::WEAPON_TWICE != 0 && ticks % 2 == 1 {
            self.blink_skipped = !self.blink_skipped;
        }
    }

    /// The flags are put back to nothing, before the effects that run now set theirs again; so is the picture
    /// an effect holds it in.
    pub fn clear(&mut self) {
        (self.flags, self.held) = (0, None);
    }

    /// Whether it leaves footprints now (the flag 0x8000).
    pub fn runs(&self) -> bool {
        self.flags & flag::RUNS != 0
    }

    /// One pass of a character that runs, being at `at` (east and north, in any measure): the footprint it
    /// leaves there, if any (`CPc::UpdateFootprints` with its kind 2). A count goes up to 14 and begins anew;
    /// at 7 a left foot is due and at 14 a right one, turned by the way gone since the last was looked at
    /// (135 degrees on from that way, whole degrees as the client's table of cosines gives them); none for
    /// one that has not moved, and none while the effect is not past its seventh tick.
    ///
    /// (The client's count and place are the actor's from its making, the place the corner of the map at
    /// first; here the place is where the body was at its first pass.)
    pub fn stepped(&mut self, at: [f32; 2]) -> Option<Print> {
        const EVERY: i32 = 7;
        self.steps = if self.steps >= EVERY * 2 { 1 } else { self.steps + 1 };
        let from = self.stepped_from.get_or_insert(at);
        if self.steps != EVERY && self.steps != EVERY * 2 {
            return None;
        }
        let (east, north) = (at[0] - from[0], at[1] - from[1]);
        *from = at;
        if east == 0.0 && north == 0.0 {
            return None;
        }
        let length = (east * east + north * north).sqrt();
        let way = (0..=180)
            .find(|degrees| east / length >= crate::prim::cosine(*degrees as f32))
            .map_or(180, |degrees| if north / length >= 0.0 { degrees } else { 360 - degrees });
        (i32::from(self.run_tick) > EVERY).then_some(Print { left: self.steps == EVERY, turned: (way + 135) % 360 })
    }

    /// The picture an effect holds it in now, if any: an action's first number and the frame of it.
    pub fn held(&self) -> Option<(usize, usize)> {
        self.held.map(|(action, frame)| (usize::from(action), usize::from(frame)))
    }

    /// An effect that runs on it does this to it. `fresh` is whether the effect has lived a tick since it was
    /// last asked: its colour is written at its ticks, and its turn made at them; its flags hold meanwhile.
    /// (Of two effects that set one flag at once the client keeps the count of the first; here the last one's
    /// is kept.)
    pub fn touch(&mut self, touch: &Touch, fresh: bool) {
        self.flags |= touch.flags;
        self.effect_phase = touch.effect_phase.unwrap_or(self.effect_phase);
        self.tint_phase = touch.tint_phase.unwrap_or(self.tint_phase);
        self.shrink_phase = touch.shrink_phase.unwrap_or(self.shrink_phase);
        self.grow_phase = touch.grow_phase.unwrap_or(self.grow_phase);
        self.outline_alpha = touch.outline_alpha.unwrap_or(self.outline_alpha);
        self.run_tick = touch.run_tick.unwrap_or(self.run_tick);
        self.held = touch.held.or(self.held);
        if fresh {
            self.colour = touch.colour.unwrap_or(self.colour);
            self.alpha = touch.alpha.unwrap_or(self.alpha);
            self.turn += touch.turn;
        }
    }

    /// Takes the whole eighths of a turn that effects have turned it by: so many steps round, from the south
    /// by the west (more of the client's `render.angle` is that way round; `Get8Dir` shows an eighth for
    /// every 45 degrees). What is left of the turn stays for the next time.
    pub fn turned(&mut self) -> i32 {
        let eighths = (self.turn / 45.0).trunc();
        self.turn -= eighths * 45.0;
        eighths as i32
    }
}

pub mod copies;
mod drawn;
#[cfg(test)]
mod tests;

pub use drawn::Quad;
