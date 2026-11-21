//! The aura of the highest level, kept on those who show it: the player's own character and everyone in
//! sight. Who shows which effects is `crate::aura`'s rule; the effects are `prims/max_level.rs`.
//!
//! The client makes and takes away the aura at each thing that can change it: a job set
//! (`CGameActor::SetActorJob`), a level told with the packet that shows someone or tells its states anew
//! (`CGameMode::ApplyActorStatusEffects`), the player's own level gained and an option of the aura switched
//! (`CGameMode::RefreshActorJobEffects`), hiding, cloaking or being invisible begun or ended
//! (`CGameActor::SetStatusStates`), the arrival on a map (`CPlayer::ReapplySkillEffects`); and an effect goes
//! with the actor it is on. Each of those leaves the aura on exactly those the rule names. Here that is held
//! to once a frame, ahead of the effects' own step (`frame/effects.rs`), from what the world knows then: the
//! job and the state of each (`Mover`), the level the server told of it (`names.rs`), the player's options.
//!
//! With effects off (`/effect`) the client keeps the aura and does not draw it (`CEffect::UpdateVisibility`),
//! so that its lights are where they were when effects are on again; so here: `kept` names the effects that
//! are not drawn now. An option of the aura switched takes every aura away and makes it anew, its lights
//! beginning under the feet again (`ApplyServiceOptionActorRefresh` and `ApplyServiceOptionWithNotice`,
//! `Contents/Configuration/ServiceOptions.cpp`, both through `RefreshActorJobEffects`): `switched`.
//!
//! What differs from the client by this:
//!
//! - an aura that stays is not begun anew where the client takes it away and makes it again for one actor
//!   alone: a job set anew on one who keeps the same aura (`SetActorJob`);
//! - the client remembers of each actor whether an aura was ever asked for it (`hasLevelAura`, set by
//!   `AM_MAKE_LEVEL99_EFFECT`), and makes the aura of another character anew only with that set
//!   (`RefreshActorJobEffects`, `SetStatusStates`): one who came into sight while the auras were hidden
//!   (`/aura2`) stays without an aura there when they are shown again, until it comes into sight anew, and so
//!   does one who came into sight invisible (`StateFlag06`) and then shows. Here both show the aura at once;
//! - one of these effects that something else began on someone who shows no aura (a script's effect by its
//!   number) is ended here at the next frame, where the client would keep it while that one is in sight;
//! - the client neither moves nor draws an effect farther than 130 from the player's own character
//!   (`CEffect::CheckRange`: 26 cells, farther than the server shows anyone); here every aura in sight lives.
//!
//! Not done: `CEffect::IsVisible`, by which the client leaves the effects on another character undrawn when
//! that character is not (`EffectVisible`: one who hides or cloaks, who shows no aura as it is); the level in
//! ZC_NPC_SHOWEFST_UPDATE, which rAthena sends for monsters and NPCs alone, is not read
//! (`net::actors::third_option`), so the level of one in sight is the one told as it came into sight.

use super::*;

/// The auras are as the rule has them now: those that should not be are ended (those of one gone out of
/// sight among them), those that should be and are not begin. Gives back the effects that are not to be
/// drawn now: the aura's with effects off, none with them on.
pub(super) fn kept(state: &mut State, time: f64) -> Vec<u32> {
    let Some(scene) = &state.scene else { return Vec::new() };
    let table = data::aura_table();
    let options = crate::aura::Options { shown: state.options.auras, full: state.options.aura_full };
    let cards = &state.cards;
    let own = std::iter::once((None, &scene.player, state.own_level));
    let others = scene.units.iter().map(|(id, unit)| (Some(*id), unit, cards.get(id).map_or(0, |card| card.level)));
    let mut wanted: BTreeSet<(Option<u32>, u32)> = own.chain(others)
        .flat_map(|(on, mover, level)| crate::aura::shown(mover.job, level, mover.state, &table, options).into_iter().flatten().map(
            move |effect| (on, effect)
        ))
        .collect();
    let all = table.effects();
    // (One that runs and is wanted is left as it is, and is wanted no more.)
    state.bursts.retain(|burst| match all.iter().find(|effect| burst.is(**effect, burst.on)) {
        Some(effect) => wanted.remove(&(burst.on, *effect)),
        None => true,
    });
    for (on, effect) in wanted {
        prims::begin(effect, on, None, time, &mut state.bursts, &mut state.sounds_due);
    }
    if state.options.effects { Vec::new() } else { all }
}

/// One of the aura's two options was switched (`/aura2`, `/aura`): every aura is taken away
/// (`CGameActor::ResetJobEffects`), and `kept` makes those that still show anew at the next frame.
pub(super) fn switched(state: &mut State) {
    let all = data::aura_table().effects();
    state.bursts.retain(|burst| !all.iter().any(|effect| burst.is(*effect, burst.on)));
}
