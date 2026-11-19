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
//! What differs from the client by this:
//!
//! - an aura that stays is not begun anew where the client takes it away and makes it again (a level gained
//!   at the highest level, `/aura` switched), so its lights keep their places;
//! - with effects off (`/effect`) the client keeps the aura and does not draw it (`CEffect::UpdateVisibility`);
//!   here it is ended, and begun when they are on again;
//! - one of these effects that something else began on someone who shows no aura (a script's effect by its
//!   number) is ended here at the next frame, where the client would keep it while that one is in sight.
//!
//! Not done: `CEffect::IsVisible`, by which the client leaves the effects on another character undrawn when
//! that character is not (`EffectVisible`); the level in ZC_NPC_SHOWEFST_UPDATE, which rAthena sends for
//! monsters and NPCs alone, is not read (`net::actors::third_option`), so the level of one in sight is the one
//! told as it came into sight.

use super::*;

/// The auras are as the rule has them now: those that should not be are ended (those of one gone out of
/// sight among them), those that should be and are not begin.
pub(super) fn kept(state: &mut State, time: f64) {
    let Some(scene) = &state.scene else { return };
    let table = data::aura_table();
    let options = crate::aura::Options { shown: state.options.auras && state.options.effects, full: state.options.aura_full };
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
}
