//! The effects that the states of someone begin and end on it (`crate::state_effects`): as a state goes on or
//! off, as one comes into sight with its third option, and on the player's own character as it arrives on a
//! map.

use super::*;

/// A state of someone's goes on or off (none named: the player's own character): the effects that live for as
/// long as the state are ended, and with the state on begun anew (`crate::state_effects`), each from its file
/// if it is one of the client's files and else as it is drawn by code.
pub fn state_effects(on: Option<u32>, state: u32, holds: bool) {
    // (The night is a state of the player's own character: `scenery.rs`.)
    scenery::state_changed(on, state, holds);
    let change = crate::state_effects::changed(state, holds);
    // What begins only if another effect ran as the state changed.
    let after = change.after.filter(|(ran, _)| STATE.with_borrow(|state| running(state, *ran, on))).map(|(_, begun)| begun);
    for effect in change.ends {
        effect_ends(*effect, on);
    }
    for effect in change.begins.iter().copied().chain(after) {
        burst(effect, on, None, 0.0);
    }
    if !change.sound.is_empty() {
        sound::effect(change.sound, 1.0);
    }
}

/// The player's own character has arrived on a map, or elsewhere on this one, in those states: the effects of
/// each begin anew on it (`crate::state_effects::reapplied`, the client's `CPlayer::ReapplySkillEffects`),
/// those the arrival ended (`session.rs`). An effect that runs on it already is left as it is: the server may
/// have told a state while the map was still being read.
pub fn own_states_again(states: impl Iterator<Item = u32>) {
    for state in states {
        scenery::state_changed(None, state, true);
        let effects = crate::state_effects::reapplied(state);
        if effects.first().is_some_and(|first| !STATE.with_borrow(|state| running(state, *first, None))) {
            for effect in effects {
                burst(*effect, None, None, 0.0);
            }
        }
    }
}

/// Someone is seen in the states its third option tells (`crate::state_effects::seen`): the effects of those
/// states begin on it unless they run already, and a monster's that it is not in end. `bits` is the state it
/// is in, by which one who hides shows none; `character` whether it is of a player's job.
pub(super) fn third_option(state: &mut State, id: u32, third: u32, bits: u32, character: bool) {
    let (seen, on, time) = (crate::state_effects::seen(third, bits & 6 != 0, character), Some(id), now());
    state.bursts.retain(|burst| !seen.ends.iter().any(|effect| burst.is(*effect, on)));
    let begun: Vec<u32> = seen.begins.iter().filter(|(_, unless)| !running(state, *unless, on)).map(|(effect, _)| *effect).collect();
    for effect in begun {
        prims::begin(effect, on, None, time, &mut state.bursts, &mut state.sounds_due);
    }
}

/// The server tells the third option of someone in sight anew (ZC_NPC_SHOWEFST_UPDATE), with the state it is
/// in.
pub fn third_option_told(id: u32, bits: u32, third: u32) {
    STATE.with_borrow_mut(|state| {
        let Some(character) = state.scene.as_ref().and_then(|scene| scene.units.get(&id)).map(|unit| matches!(unit.look, Look::Character(_))) else { return };
        third_option(state, id, third, bits, character);
    });
}
