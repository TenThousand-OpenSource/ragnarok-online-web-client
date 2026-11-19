//! The effects a state of someone's keeps going for as long as it holds. The client has no skill start these:
//! the server puts a bit into the one's effect state (ZC_STATE_CHANGE; the client's `stateFlags`), and each
//! actor looks at its own bits as it is processed (`CGameActor::UpdateStateEffects`,
//! `Contents/Actors/GameActor.cpp`):
//!
//! - bit 0, Sight: `EF_SIGHT` (22) is launched, and again every time as many passes have gone by as that effect
//!   lives (270);
//! - bit 13, Ruwach: `EF_RUWACH` (33) in the same way, every 200.
//!
//! The actor has one count for both (`effectLaunchCnt`): it is put far up (36000) whenever no bit at all is set
//! and goes up by one a pass while any is, so an effect begins at once when its bit comes to one who had none.
//! Nothing ends a running effect when its bit goes: it lives its time out. As either bit comes on the client
//! plays `EF_Sight.wav` (`CGameActor::SetStatusStates`, `Contents/Actors/RenderObject.cpp`).
//!
//! (Whether the client counts passes of its loop or ticks there was not found; a tick is taken, as an effect's
//! own time is in ticks.)

use std::collections::HashMap;

/// The bits of the effect state that keep an effect going, each with its effect: Sight and Ruwach.
const KEPT: [(u32, u32); 2] = [(1, 22), (1 << 13, 33)];

/// The counts of everyone with such a bit set (none named: the player's own character): the bits seen last,
/// and the moment the count of `UpdateStateEffects` was at nothing.
#[derive(Default)]
pub struct Kept {
    counts: HashMap<Option<u32>, (u32, f64)>,
}

impl Kept {
    /// Looks at everyone's effect state at `time` (milliseconds, a tick being `tick` of them): the effects to
    /// begin now (which, and on whom) and the sounds to start. `lives` says how many ticks an effect lives (the
    /// client's `EffectLifetime` of it). Whoever is not among `states` any more is forgotten.
    pub fn look(&mut self, states: &[(Option<u32>, u32)], time: f64, tick: f64, lives: impl Fn(u32) -> i32) -> (Vec<(u32, Option<u32>)>, Vec<&'static str>) {
        self.counts.retain(|who, _| states.iter().any(|(other, bits)| other == who && *bits != 0));
        let (mut effects, mut sounds) = (Vec::new(), Vec::new());
        for &(who, bits) in states.iter().filter(|(_, bits)| *bits != 0) {
            let (before, since) = self.counts.entry(who).or_insert((0, time - 36000.0 * tick));
            for (bit, _) in KEPT {
                if bits & bit != 0 && *before & bit == 0 {
                    sounds.push("effect/ef_sight.wav");
                }
            }
            *before = bits;
            // Sight is looked at first, and having begun puts the count at nothing, so that Ruwach beside it
            // waits a round.
            let mut count = ((time - *since) / tick) as i32;
            for (bit, effect) in KEPT {
                if bits & bit != 0 && count >= lives(effect) {
                    (*since, count) = (time, 0);
                    effects.push((effect, who));
                }
            }
        }
        (effects, sounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lives(effect: u32) -> i32 {
        if effect == 22 { 270 } else { 200 }
    }

    #[test]
    fn sight_begins_at_once_and_again_when_it_has_lived_its_time() {
        let mut kept = Kept::default();
        let sees = [(None, 1), (Some(7), 0)];
        assert_eq!(kept.look(&sees, 1000.0, 16.0, lives), (vec![(22, None)], vec!["effect/ef_sight.wav"]));
        // Nothing more while it lives, and no sound while the bit stays.
        assert_eq!(kept.look(&sees, 1000.0 + 269.0 * 16.0, 16.0, lives), (Vec::new(), Vec::new()));
        assert_eq!(kept.look(&sees, 1000.0 + 270.0 * 16.0, 16.0, lives), (vec![(22, None)], Vec::new()));
    }

    #[test]
    fn ruwach_beside_sight_waits_a_round_and_one_without_bits_is_forgotten() {
        let mut kept = Kept::default();
        let both = [(Some(7), 1 | 1 << 13)];
        assert_eq!(kept.look(&both, 0.0, 16.0, lives), (vec![(22, Some(7))], vec!["effect/ef_sight.wav", "effect/ef_sight.wav"]));
        assert_eq!(kept.look(&both, 200.0 * 16.0, 16.0, lives), (vec![(33, Some(7))], Vec::new()));
        // The bits gone, the count is forgotten: they begin at once when they come again.
        assert_eq!(kept.look(&[(Some(7), 0)], 3300.0, 16.0, lives), (Vec::new(), Vec::new()));
        assert_eq!(kept.look(&[(Some(7), 1 << 13)], 3400.0, 16.0, lives).0, vec![(33, Some(7))]);
    }
}
