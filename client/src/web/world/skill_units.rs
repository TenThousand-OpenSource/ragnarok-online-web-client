//! The units skills leave on the ground (`crate::skill_units`, the client's `CSkill`) and the places skills are
//! used on: each launches the effect of its kind on itself, again and again or once, and its effects end with
//! it. An effect "on" a unit is an effect on its ID: `Scene::place_of` knows where the units and the places are
//! (`Scene::spots`).
//!
//! A unit's count goes up once a tick, and a tick is taken to be that of the client's coded effects
//! (`CAST_FRAME_MS`): `CSkill::OnProcess` runs once a pass of the client's loop, and how long that is was not
//! found. Of the units only a trap is pointed at, and only by a skill for traps (`traps.rs`; the client lets an Ice
//! Wall, a Blast Mine and a Claymore Trap be struck too).
//!
//! A trap is no effect but a model on its cell: `skill_units/traps.rs`.

use super::*;

mod traps;

pub use traps::{skill_unit_seen, skill_unit_strikes, skill_unit_used};
use crate::net::skill_units::Entry;
use crate::skill_units::{Pulse, Shows};

/// The pictures of the boxes of light of Sanctuary, Magnus Exorcismus and Fog Wall, among the client's pictures
/// of effects.
pub(super) const PICTURES: [&str; 3] = ["alpha_down.tga", "ring_red.tga", "ring_white.tga"];

/// One more than the most the client's `rand()` gives: under it is any number, for the rules that take one.
const RAND_MAX: u32 = 32768;

/// The IDs given to the places skills are used on, which the server gives none: from here up.
const FIRST_SPOT: u32 = 0xfff0_0000;

/// A unit standing on the ground.
struct Standing {
    cell: (f32, f32),
    /// What kind of unit it is (the client's `UNT_` numbers).
    unit: u32,
    /// The visit to a map it is of (`State::entered`): the units of a map left are forgotten.
    visit: u32,
    /// Its count towards launching its effect again (`m_effectTimer`), and up to when its ticks were counted.
    count: i32,
    ticked: f64,
    /// Whether the effect it launches once was launched, and its box of light if that is what it shows.
    launched: bool,
    pulse: Option<Pulse>,
    /// Whether it is to be seen (the client's `render.enabled`): one that is not shows neither its model nor
    /// the effect over a trap.
    seen: bool,
    /// The model it shows, if it is a trap (by its path among the client's models), and since when the model's
    /// animation plays: since the trap sprang (`traps.rs`).
    model: Option<&'static str>,
    sprung: Option<f64>,
}

/// The units on the map, by the server's ID, and the places skills were used on: where, of which visit to a map,
/// and until when the effect there may last.
#[derive(Default)]
pub(super) struct Grounds {
    units: HashMap<u32, Standing>,
    spots: HashMap<u32, ((f32, f32), u32, f64)>,
    spots_made: u32,
    /// The traps' models read so far, by their paths; nothing yet for one being read (`traps.rs`).
    models: HashMap<&'static str, Option<Rc<land::Raisable>>>,
}

/// The server shows a unit on the ground (ZC_SKILL_ENTRY5; the client's `AM_SKILLSTANDENTRY`): it stands there
/// from now on, and sounds if its kind has a sound for that. One already there is told its kind anew. A trap
/// shows its model if it is to be seen (`entry.visible`: by everyone, or by the player who laid it).
pub fn skill_unit_shown(entry: &Entry) {
    STATE.with_borrow_mut(|state| {
        let (visit, cell) = (state.entered, (f32::from(entry.cell.0), f32::from(entry.cell.1)));
        // The unit as it first stands.
        let fresh = || Standing {
            cell,
            unit: entry.unit,
            visit,
            count: crate::skill_units::COUNT_AT_FIRST,
            ticked: now(),
            launched: false,
            pulse: None,
            seen: entry.visible,
            model: None,
            sprung: None
        };
        let standing = state.grounds.units.entry(entry.id).and_modify(|standing| standing.cell = cell).or_insert(fresh());
        if standing.visit != visit {
            *standing = fresh();
        }
        if let (true, Some(scene)) = (standing.shows_as(entry.unit, entry.visible), &mut state.scene) {
            scene.models.lower(entry.id);
        }
    });
    if let Some(sound) = crate::skill_units::appears_with(entry.unit, lot(RAND_MAX)) {
        sound::effect(sound, 1.0);
    }
}

/// A cell of the map is of another type from now on (ZC_UPDATE_MAPINFO): whether it can be walked on follows,
/// for the ways the player's character and everyone else are shown walking.
pub fn cell_retyped(cell: (u16, u16), kind: u16) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            scene.gat.retype(i32::from(cell.0), i32::from(cell.1), i32::from(kind));
        }
    });
}

/// Whether a unit of that ID stands on the map (the client's `GetSkillByAID`): the server names one as the
/// striker of what a skill on the ground does.
pub fn skill_unit_stands(id: u32) -> bool {
    STATE.with_borrow(|state| state.grounds.units.contains_key(&id))
}

/// A unit is gone (ZC_SKILL_DISAPPEAR): its effects go with it.
pub fn skill_unit_gone(id: u32) {
    STATE.with_borrow_mut(|state| {
        if state.grounds.units.remove(&id).is_some() {
            ended(state, id);
        }
    });
}

/// A unit becomes another kind (ZC_SPRITE_CHANGE of its base, the client's `AM_SETJOB`: a Warp Portal opens, a
/// trap springs): its count is put far up again, so the new kind's effect is launched at once. A Warp Portal
/// that opens sounds (`UNT_WARP_WAITING`, 128). A trap made a used one launches the effect of its kind on
/// itself, and a trap's model plays from now on: the one it had, or its new kind's.
pub fn skill_unit_becomes(id: u32, unit: u32) {
    let known = STATE.with_borrow_mut(|state| {
        let standing = state.grounds.units.get_mut(&id)?;
        let effect = crate::skill_units::traps::sprung_with(standing.unit).filter(|_| unit == crate::skill_units::traps::USED);
        standing.count = crate::skill_units::COUNT_AT_FIRST;
        let anew = standing.shows_as(unit, standing.seen);
        if standing.model.is_some() {
            standing.sprung.get_or_insert_with(now);
        }
        if let (true, Some(scene)) = (anew, &mut state.scene) {
            scene.models.lower(id);
        }
        if let Some(effect) = effect {
            launch(state, effect, id, None, now());
        }
        Some(())
    });
    if known.is_some() && unit == 128 {
        sound::effect("effect/ef_portal.wav", 1.0);
    }
}

/// A skill of that level was used on a place (ZC_NOTIFY_GROUNDSKILL): the effect the client has for that skill
/// plays there (`CGameActor::PlayGroundSkillEffect`, `LaunchDummyEffect`), or on the one who used it
/// (`crate::skill_units::on_caster`).
pub fn ground_skill(skill: u16, level: i16, cell: (u16, u16), by: Option<u32>) {
    if let Some(effect) = crate::skill_units::on_caster(skill) {
        launch::burst(effect, by, None, 0.0);
    }
    STATE.with_borrow_mut(|state| {
        // The sounds the client's handlers start beside the effect (Gospel's, Basilica's, Gravitation's), heard
        // from the place. (The client plays them whole.)
        if let Some(scene) = &state.scene {
            for (_, sound) in crate::effect_tables::skill_sounds(skill, crate::effect_tables::Sounds::Placed) {
                earshot::heard_at(scene, sound, (f32::from(cell.0), f32::from(cell.1)));
            }
        }
        let Some((effect, ticks)) = crate::skill_units::ground_effect(skill, level) else { return };
        let time = now();
        let id = spot(state, (f32::from(cell.0), f32::from(cell.1)), ticks, time);
        // The effect is on the place and told who used the skill (none named: the player's own character).
        launch(state, effect, id, Some(Blow { striker: Some(id), place_of: by, hits: 0 }), time);
    });
}

/// A place of the map an effect is launched on by itself, for so many ticks (the client's
/// `CGameMode::LaunchDummyEffect`): where, in cells. Gives back the ID the place is known by.
pub(super) fn spot(state: &mut State, cell: (f32, f32), ticks: i32, time: f64) -> u32 {
    let id = FIRST_SPOT.wrapping_add(state.grounds.spots_made % 0x000f_0000);
    state.grounds.spots_made = state.grounds.spots_made.wrapping_add(1);
    state.grounds.spots.insert(id, (cell, state.entered, time + f64::from(ticks) * CAST_FRAME_MS));
    if let Some(scene) = &mut state.scene {
        scene.spots.insert(id, cell);
    }
    id
}

/// One of the client's effects launched on a place of the map by itself for so many ticks (the client's
/// `LaunchDummyEffect`), as the place one teleported away from has.
pub(super) fn dummy(state: &mut State, cell: (f32, f32), effect: u32, ticks: i32, time: f64) {
    let on = spot(state, cell, ticks, time);
    launch(state, effect, on, None, time);
}

/// Launches one of the client's effects on a unit or a place: its files, if it is played from files
/// (`launch::files`), with the sound a unit's effect starts with
/// when the client's table has none; and, unlike an effect on someone (`launch::burst`), what of it is made
/// of primitives besides, if that is ported: a Safety Wall is its file and its wall of light.
fn launch(state: &mut State, effect: u32, on: u32, blow: Option<Blow>, time: f64) {
    let played = launch::files(state, effect, Some(on), blow, time);
    if played && crate::effect_tables::effect_sounds(effect).is_empty() {
        state.sounds_due.extend(crate::skill_units::starts_with(effect, lot(RAND_MAX)).map(|sound| (time, sound.to_owned())));
    }
    prims::begin(effect, Some(on), blow, time, &mut state.bursts, &mut state.sounds_due);
}

/// The effects on a unit or a place end, and it is no place any more.
fn ended(state: &mut State, id: u32) {
    state.bursts.retain(|burst| burst.on != Some(id));
    state.effects.retain(|playing| playing.on != Some(id));
    state.effects_due.retain(|due| due.2 != Some(id));
    if let Some(scene) = &mut state.scene {
        scene.spots.remove(&id);
        scene.models.lower(id);
    }
}

/// One frame of the units: those of a map left and the places whose time is up are forgotten, the map is told
/// where the rest are, and each unit lives the ticks that have passed (`CSkill::OnProcess`), launching its
/// effect when it is due. (After a long pause the ticks missed are not lived through one by one.)
pub(super) fn step(state: &mut State, time: f64) {
    let visit = state.entered;
    let stale: Vec<u32> = state.grounds.units
        .iter()
        .filter(|(_, standing)| standing.visit != visit)
        .map(|(id, _)| *id)
        .chain(state.grounds.spots.iter().filter(|(_, spot)| spot.1 != visit || spot.2 <= time).map(|(id, _)| *id))
        .collect();
    for id in stale {
        state.grounds.units.remove(&id);
        state.grounds.spots.remove(&id);
        ended(state, id);
    }
    let Some(scene) = &mut state.scene else { return };
    for (id, cell) in state.grounds.units.iter().map(|(id, standing)| (*id, standing.cell)).chain(state.grounds.spots.iter().map(|(id, spot)| (*id, spot.0))) {
        scene.spots.insert(id, cell);
    }
    let mut launches = Vec::new();
    for (id, standing) in &mut state.grounds.units {
        let due = ((time - standing.ticked) / CAST_FRAME_MS) as i32;
        standing.ticked += f64::from(due) * CAST_FRAME_MS;
        for _ in 0..due.min(60) {
            // (The file over a Shockwave Trap plays only over one that is seen.)
            match crate::skill_units::shows(standing.unit).filter(|_| standing.seen || standing.unit != crate::skill_units::traps::SHOCKWAVE) {
                Some(Shows::Again(effect, every)) => {
                    let (count, launched) = crate::skill_units::tick(standing.count, every);
                    standing.count = count;
                    if launched {
                        launches.push((effect, *id));
                    }
                }
                Some(Shows::Kept(effect) | Shows::AtOnce(effect)) if !standing.launched => {
                    standing.launched = true;
                    standing.pulse = Pulse::of(effect, lot(RAND_MAX));
                    if standing.pulse.is_none() {
                        launches.push((effect, *id));
                    }
                }
                _ => {}
            }
            if let Some(pulse) = &mut standing.pulse {
                pulse.tick();
            }
        }
    }
    for (effect, id) in launches {
        launch(state, effect, id, None, time);
    }
    traps::step(state, time);
}

/// The boxes of light of the units that show one, each wall a quad standing on the unit's cell.
pub(super) fn drawn(state: &State, view: &View, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    for standing in state.grounds.units.values() {
        let Some(pulse) = &standing.pulse else { continue };
        let Some(binds) = gpu.effect_pictures.get(pulse.picture) else { continue };
        let place = on_ground(&scene.gat, standing.cell);
        let nearer = nearer(view, place, 25.0);
        for (alpha, wall) in pulse.walls() {
            let colour = [pulse.colour[0] / 255.0, pulse.colour[1] / 255.0, pulse.colour[2] / 255.0, alpha / 255.0];
            // `RenderTeiRect`'s two triangles: the first three corners, then the fourth, the third and the first.
            for index in [0, 1, 2, 3, 2, 0] {
                let (at, uv) = wall[index];
                quads.corner(view, nearer, [place[0] + at[0], place[1] + at[1], place[2] + at[2]], uv, colour);
            }
            quads.picture(binds, pulse.adds);
        }
    }
}
