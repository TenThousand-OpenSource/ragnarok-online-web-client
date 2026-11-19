//! The spirit spheres about a monk and the coins about a gunslinger, as the server tells how many someone has
//! (`crate::spirits`): the effect of them on that one is ended and begun anew, told the count. The effects
//! themselves are `prims/spirits.rs`.
//!
//! They go with the map as every effect does (`session.rs`), and with the one they are about when that one
//! goes out of sight (`spheres_gone`: the client's effects go with their actor).

use super::*;

/// The job and the effect state of someone in sight (none named: the player's own character); nothing for one
/// who is not there.
fn job_and_state(state: &State, on: Option<u32>) -> Option<(u16, u32)> {
    let scene = state.scene.as_ref()?;
    let mover = match on {
        None => &scene.player,
        Some(id) => scene.units.get(&id)?,
    };
    Some((mover.job, mover.state))
}

/// The effect begins on someone, told how many spheres it is (`LaunchEffect` and `AM_SETPOS` with the count
/// as the first figure).
fn begin(state: &mut State, effect: u32, on: Option<u32>, count: u16) {
    if let Some(spheres) = prims::begin(effect, on, None, now(), &mut state.bursts, &mut state.sounds_due) {
        spheres.tell([f32::from(count), 0.0, 0.0]);
    }
}

/// ZC_SPIRITS: someone has so many spheres now (none named: the player's own character).
/// `CGameMode::UpdateActorSpiritCount`: nothing for one who is not in sight; the effect of that one's job is
/// ended, and unless there are none begun anew, told the count. (The client launches it here on one who
/// hides or cloaks too, and whether or not effects are shown: only its handler of ZC_SPIRITS2 looks at
/// either.) The later jobs' arms of the handler are not ported: nothing is shown for them
/// (`crate::spirits::effect`).
pub fn spheres_told(on: Option<u32>, count: u16) {
    STATE.with_borrow_mut(|state| {
        let Some(effect) = job_and_state(state, on).and_then(|(job, _)| crate::spirits::effect(job)) else { return };
        state.bursts.retain(|burst| !burst.is(effect, on));
        if count != 0 {
            begin(state, effect, on, count);
        }
    });
}

/// ZC_SPIRITS2: someone who comes into sight has so many spheres. `CGameMode::Zc_Spirits2`: nothing while
/// effects are not shown (`/effect`), nor for one who is not in sight; on one who hides or cloaks the effect
/// of that one's job is ended; on any other it begins, told the count, unless it runs on that one already.
/// (As the client, it does not look whether the count is nothing: the effect then shows nothing.)
pub fn spheres_seen(on: Option<u32>, count: u16) {
    STATE.with_borrow_mut(|state| {
        if !state.options.effects {
            return;
        }
        let Some((job, bits)) = job_and_state(state, on) else { return };
        let Some(effect) = crate::spirits::effect(job) else { return };
        if crate::spirits::hidden(bits) {
            state.bursts.retain(|burst| !burst.is(effect, on));
        } else if !running(state, effect, on) {
            begin(state, effect, on, count);
        }
    });
}

/// Someone went out of sight: the spheres about that one go too, whichever of the effects they are. (In the
/// client an effect goes with the actor it is on.)
pub fn spheres_gone(id: u32) {
    STATE.with_borrow_mut(|state| state.bursts.retain(|burst| !crate::spirits::EFFECTS.iter().any(|effect| burst.is(*effect, Some(id)))));
}
