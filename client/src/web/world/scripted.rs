//! What an NPC's script shows in the world: the bar that fills over someone (`progressbar`; the client's
//! `AM_MAKE_SKILL_RECHARGEGAGE`, the same `UIRechargeGage` a cast has).

use super::*;

/// The bars filling now.
#[derive(Default)]
pub(super) struct Scripted {
    /// The visit to a map they are of (`State::entered`).
    visit: u32,
    /// By whom the bar is over (none is the player's own character): since when it fills, and for how long.
    gauges: HashMap<Option<u32>, (f64, f64)>,
}

/// A bar begins to fill over someone for so many milliseconds, or the one there is taken away. `own` is the
/// player's own ID. A bar that is full goes by itself (`CGameActor::UpdateGaugePositions`).
pub fn gauge(who: u32, own: u32, lasts: Option<f64>) {
    STATE.with_borrow_mut(|state| {
        let time = now();
        if state.scripted.visit != state.entered {
            state.scripted = Scripted { visit: state.entered, gauges: HashMap::new() };
        }
        state.scripted.gauges.retain(|_, (began, lasts)| time <= *began + *lasts);
        let key = (who != own).then_some(who);
        match lasts {
            Some(lasts) => state.scripted.gauges.insert(key, (time, lasts)),
            None => state.scripted.gauges.remove(&key),
        };
    });
}

/// The bars of this frame, as the bars of casts are told (`overlays/bars.rs` draws both alike): where the one
/// it is over stands, since when, and for how long.
pub(super) fn gauges<'a>(scripted: &'a Scripted, scene: &'a Scene, entered: u32, time: f64) -> impl Iterator<Item = ((f32, f32), f64, f64)> + 'a {
    scripted
        .gauges
        .iter()
        .filter(move |(_, (began, lasts))| scripted.visit == entered && time <= began + lasts)
        .filter_map(|(who, (began, lasts))| Some((scene.place_of(*who)?, *began, *lasts)))
}

/// A sound a script plays from where someone stands (`CGameMode::ReceiveSound`), `own` being the player's
/// own ID. One who is not in sight has no place: the sound is then played whole, unless it is one that
/// repeats (`in_sight_only`), which is not played at all while its NPC is out of sight
/// (`CGameMode::UpdateLoopSounds`).
pub fn sounded(name: &str, who: u32, own: u32, in_sight_only: bool) {
    STATE.with_borrow(|state| {
        let on = (who != own).then_some(who);
        match &state.scene {
            Some(scene) if in_sight_only && scene.place_of(on).is_none() => {}
            Some(scene) => earshot::heard(scene, name, on),
            None => sound::effect(name, 1.0),
        }
    });
}
