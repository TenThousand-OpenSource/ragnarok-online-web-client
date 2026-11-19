//! A part of a primitive of the kinds built corner by corner (`prim/tei.rs`): the client's
//! `EffectPrimitiveSlot` kept whole, and the motions several kinds share, each written once: the wait before a
//! part begins (`waits`), a turn kept under a whole round (`under_a_round`), the end of one that has faded
//! (`over_once_gone`), the angle that wanders (`wander`, the client's `WanderAngle`), a flight along the ground
//! (`fly_level`, `steer`), and a tail of parts that take the look of the one before (`follow`, `trail`, the
//! client's `FollowTrail`).

use super::*;

/// A part of a primitive: the client's `EffectPrimitiveSlot`, every field of it. What each means is the kind's
/// own, and said where the kind is.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    /// Whether it lives (`life` of 1), and its count (`process`), which an effect may start under nothing to
    /// have the part wait.
    pub live: bool,
    pub process: i32,
    /// `alphaB` and `alphaT`.
    pub alpha: i32,
    pub alpha_top: i32,
    /// `fullDisplayAngle`, `rotStart` and `riseAngle`, degrees.
    pub round: i32,
    pub turned: i32,
    pub rise: i32,
    /// `maxHeight` and `distance`.
    pub tall: f32,
    pub out: f32,
    /// `height` and `flag1`.
    pub heights: [f32; 21],
    pub flags: [i8; 21],
    /// `vecBNow`, `vecTNow`, `vecBPre`, `vecTPre`: places in the client's directions (x east, y down, z north)
    /// from the effect's place as the primitive was launched.
    pub at: [[f32; 3]; 4],
    /// Where the client's `rand()` stands for this part: the kinds that draw lots as they go draw them from
    /// here (`Part::lot`), the effect having given each part a number to start from.
    pub seed: u32,
}

impl Default for Part {
    /// A part as the client makes one (`EffectPrimitiveSlot::EffectPrimitiveSlot` leaves it at nothing), alive.
    fn default() -> Self {
        Self {
            live: true,
            process: 0,
            alpha: 0,
            alpha_top: 0,
            round: 0,
            turned: 0,
            rise: 0,
            tall: 0.0,
            out: 0.0,
            heights: [0.0; 21],
            flags: [0; 21],
            at: [[0.0; 3]; 4],
            seed: 0
        }
    }
}

impl Part {
    /// The client's `rand() % under` (Microsoft's `rand`: the seed times 214013 plus 2531011, of which the
    /// bits 16 to 30 are the lot).
    pub(in crate::prim) fn lot(&mut self, under: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
        ((self.seed >> 16 & 0x7fff) % under) as i32
    }

    /// Counts a tick of its life, and says whether it still waits: a part whose count the effect started
    /// under nothing does nothing until the count is over nothing.
    pub(super) fn waits(&mut self) -> bool {
        self.process += 1;
        self.process <= 0
    }

    /// Once it would show less than nothing it shows nothing and is over.
    pub(super) fn over_once_gone(&mut self) {
        if self.alpha < 0 {
            (self.alpha, self.live) = (0, false);
        }
    }

    /// The client's `WanderAngle`, for the pair of figures at `heights[turn]` and the one after it: a turn
    /// that goes 3 degrees a tick up towards its mark (to 359 at most), or 2 down (to nothing), and having
    /// passed the mark gets a new one by lot on the other side of itself.
    pub(in crate::prim) fn wander(&mut self, turn: usize) {
        self.wander_by(turn, 3.0, 2.0);
    }

    /// The same for a turn that goes `up` degrees a tick towards its mark or `down` back from it (the
    /// client's `DriftSlotAngles`, a pair of its figures).
    pub(in crate::prim) fn wander_by(&mut self, turn: usize, up: f32, down: f32) {
        let under = |part: &mut Part, turn: f32| if turn as i32 == 0 { 0.0 } else { part.lot(turn as u32) as f32 };
        if self.heights[turn + 1] > self.heights[turn] {
            self.heights[turn] = (self.heights[turn] + up).min(359.0);
            if self.heights[turn] > self.heights[turn + 1] {
                self.heights[turn + 1] = under(self, self.heights[turn]);
            }
        } else {
            self.heights[turn] = (self.heights[turn] - down).max(0.0);
            if self.heights[turn] <= self.heights[turn + 1] {
                self.heights[turn + 1] = 359.0 - under(self, self.heights[turn]);
            }
        }
    }

    /// Flies `heights[1]` along the ground the way it is turned.
    pub(super) fn fly_level(&mut self) {
        self.at[0][0] += self.heights[1] * cosine(self.turned as f32);
        self.at[0][2] += self.heights[1] * sine(self.turned as f32);
    }

    /// Turns by so many degrees, staying from 0 up to 360.
    pub(super) fn steer(&mut self, degrees: i32) {
        self.turned = (self.turned + degrees).rem_euclid(360);
    }

    /// `FollowTrail`: a part of a tail takes the look of the one before it, `fainter`: how it is turned, how
    /// far it reaches and swells, where it is and which picture it shows.
    pub(super) fn follow(&mut self, before: &Part, fainter: i32) {
        (self.round, self.out, self.heights[8]) = (before.round, before.out, before.heights[8]);
        self.alpha = (before.alpha - fainter).max(0);
        (self.at[0], self.flags[0]) = (before.at[0], before.flags[0]);
    }

    /// `FollowTrail` down a tail of three parts behind `head`: each takes the look of the one before it, the
    /// last first, 25 fainter, and the first that of the head, `fainter`; a part of the tail follows only
    /// once the head has lived more ticks than its place in the tail.
    pub(super) fn trail(head: &Part, tail: &mut [Part], fainter: i32) {
        if tail.len() != 3 {
            return;
        }
        if head.process > 3 {
            let before = tail[1].clone();
            tail[2].follow(&before, 25);
        }
        if head.process > 2 {
            let before = tail[0].clone();
            tail[1].follow(&before, 25);
        }
        if head.process > 1 {
            tail[0].follow(head, fainter);
        }
    }
}

/// A turn in whole degrees that has gone past a whole round, brought back under it: what the client does
/// after adding to one (`if (angle >= 360) angle -= 360`).
pub(super) fn under_a_round(angle: i32) -> i32 {
    if angle >= 360 { angle - 360 } else { angle }
}

impl Prim {
    /// The first part and the parts after it, the first counted a tick on: nothing if it is over, or still
    /// waits. How the kinds whose first part leads the others begin their tick.
    pub(super) fn head_and_tail(&mut self) -> Option<(&mut Part, &mut [Part])> {
        let [head, tail @ ..] = &mut self.parts[..] else { return None };
        (head.live && !head.waits()).then_some((head, tail))
    }
}
