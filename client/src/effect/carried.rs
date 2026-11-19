//! An effect file that a class of the client's own plays and carries: moved along a way from one actor to
//! another, turned on the screen, put aside, begun late, begun anew when it is over.
//!
//! The client's `CEZ2STRParticle` (`Contents/Effects/Particles/Particle.cpp`) is one playing file. Each tick of
//! the effect that holds it, the effect gives it a place in the world and calls its `Process`, which counts its
//! delay (`m_delay`) down by 16 or moves it on by one frame; a file that loops (`m_loop`) begins anew when its
//! layers are through. `RenderScreen` then draws the layers across the screen about the place, the corners
//! turned about it by `m_angle` (and not at all by an angle not over nothing) and faded by `m_alpha`.
//!
//! What gives the place and the angle is the effect: `CCartCannon`, `CStormCannonEffect`, `CAcidifiedThrow`,
//! `CCrimsonArrowEffect`, `CSavageImpactEffect`, `CRandomHitEffect`, and `CEZ2STRScreenEffect` for the plain
//! files it turns towards someone. Their rules are the functions of `flights.rs`, named through this file; which effect is carried how is listed in
//! `crate::effect::played`, and `web/world/frame/effects.rs` draws them.
//!
//! Places are this client's: east, up, north, in the client's units (the client's own y points down).

/// A tick of the client's effects in milliseconds: what `CEZ2STRParticle::Process` takes off a delay each time.
pub const TICK_MS: i32 = 16;

/// The tick of its flight at which what `CAcidifiedThrow` throws has landed: its time runs from 1 down by 0.07
/// a tick, and is not over nothing after the fifteenth.
pub const BOW_LANDS: i32 = 14;

/// Where a carried file is, tick by tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Way {
    /// On the one the effect is on, wherever that one goes.
    Stays,
    /// On the other one the effect was told of, wherever that one goes (`CAcidifiedThrow`'s `impactPosition`).
    There,
    /// From where the one stood to where the other stood as it began, in ten steps (`CCartCannon`).
    Steps,
    /// From where the one stood towards where the other stood and on, fading once it is past (`CStormCannonEffect`).
    Straight,
    /// From the one to the other in a bow to the side (`CAcidifiedThrow`).
    Bowed,
}

/// How a carried file is turned on the screen (`m_angle`, degrees, clockwise).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    None,
    /// By so much.
    By(f32),
    /// By the angle on the screen of the way from the one to the other as it began (`screen_angle`), and so
    /// much more.
    Towards(f32),
    /// By the way of its flight over the ground and the camera's yaw as it began (`steps_turn`).
    Flight,
}

/// How one file of an effect is carried.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Carried {
    pub way: Way,
    /// How far from the place its way gives it stands: east, up, north.
    pub offset: [f32; 3],
    /// How many pixels up the screen its middle is from that place: 80 for a file `CEZ2STRScreenEffect` plays
    /// (`Process(0, -80, 0)`), nothing for the classes that carry theirs (`Process(0, 0, 0)`).
    pub lift: f32,
    pub turn: Turn,
    /// The milliseconds it waits before its first frame (`m_delay`, or the effect's own `delay`).
    pub delay: i32,
    /// Whether it begins anew when it is over (`m_loop`).
    pub looped: bool,
    /// How many times a tick it is moved on (`CRandomHitEffect::OnProcess` calls `Process` twice).
    pub pace: i32,
}

impl Carried {
    /// A file on that way, as a class of the client's that carries its own plays it.
    pub const fn new(way: Way) -> Self {
        Carried { way, offset: [0.0; 3], lift: 0.0, turn: Turn::None, delay: 0, looped: false, pace: 1 }
    }

    pub const fn offset(self, offset: [f32; 3]) -> Self {
        Carried { offset, ..self }
    }

    pub const fn lifted(self, lift: f32) -> Self {
        Carried { lift, ..self }
    }

    pub const fn turned(self, turn: Turn) -> Self {
        Carried { turn, ..self }
    }

    pub const fn delayed(self, delay: i32) -> Self {
        Carried { delay, ..self }
    }

    pub const fn looped(self) -> Self {
        Carried { looped: true, ..self }
    }

    pub const fn paced(self, pace: i32) -> Self {
        Carried { pace, ..self }
    }

    /// How many frames it has been moved on before the one it shows at a tick of its effect (the first tick is
    /// 0); nothing while it waits. A delay takes one call of `Process` for every 16 milliseconds begun.
    pub fn lived(&self, tick: i32) -> Option<i32> {
        let waits = if self.delay > 0 { (self.delay + TICK_MS - 1) / TICK_MS } else { 0 };
        let moved = (tick + 1) * self.pace - waits;
        (moved > 0).then_some(moved - 1)
    }

    /// The frame of its file it shows, having lived so many, the file's last frame being `last`; nothing once
    /// it is over. (The client ends a file, or begins it anew, the call after its layers are all through their
    /// keys; here that is taken to be the frame after the file's last.)
    pub fn frame(&self, lived: i32, last: i32) -> Option<i32> {
        if self.looped { Some(lived % (last + 1).max(1)) } else { (lived <= last).then_some(lived) }
    }
}

pub use super::flights::*;

#[cfg(test)]
mod tests;
