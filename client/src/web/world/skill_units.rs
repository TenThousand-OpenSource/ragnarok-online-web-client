//! The units skills leave on the ground (`crate::skill_units`, the client's `CSkill`) and the places skills are
//! used on: each launches the effect of its kind on itself, again and again or once, and its effects end with
//! it. An effect "on" a unit is an effect on its ID: `Scene::place_of` knows where the units and the places are
//! (`Scene::spots`).
//!
//! A unit's count goes up once a tick, and a tick is taken to be that of the client's coded effects
//! (`CAST_FRAME_MS`): `CSkill::OnProcess` runs once a pass of the client's loop, and how long that is was not
//! found. Nothing of a unit can be pressed or pointed at (the client lets an Ice Wall and the traps be).

use super::*;
use crate::net::skill_units::Entry;
use crate::skill_units::{Pulse, Shows};

/// The pictures of the boxes of light of Sanctuary and Magnus Exorcismus, among the client's pictures of effects.
pub(super) const PICTURES: [&str; 2] = ["alpha_down.tga", "ring_red.tga"];

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
}

/// The units on the map, by the server's ID, and the places skills were used on: where, of which visit to a map,
/// and until when the effect there may last.
#[derive(Default)]
pub(super) struct Grounds {
    units: HashMap<u32, Standing>,
    spots: HashMap<u32, ((f32, f32), u32, f64)>,
    spots_made: u32,
}

/// The server shows a unit on the ground (ZC_SKILL_ENTRY5; the client's `AM_SKILLSTANDENTRY`): it stands there
/// from now on, and sounds if its kind has a sound for that. One already there is told its kind anew.
pub fn skill_unit_shown(entry: &Entry) {
    STATE.with_borrow_mut(|state| {
        let (visit, cell) = (state.entered, (f32::from(entry.cell.0), f32::from(entry.cell.1)));
        match state.grounds.units.get_mut(&entry.id).filter(|standing| standing.visit == visit) {
            Some(standing) => (standing.cell, standing.unit) = (cell, entry.unit),
            None => {
                state.grounds.units.insert(entry.id, Standing { cell, unit: entry.unit, visit, count: crate::skill_units::COUNT_AT_FIRST, ticked: now(), launched: false, pulse: None });
            }
        }
    });
    if let Some(sound) = crate::skill_units::appears_with(entry.unit) {
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
/// that opens sounds (`UNT_WARP_WAITING`, 128). Gives back nothing for an ID that is no unit's.
pub fn skill_unit_becomes(id: u32, unit: u32) {
    let known = STATE.with_borrow_mut(|state| {
        let standing = state.grounds.units.get_mut(&id)?;
        (standing.unit, standing.count) = (unit, crate::skill_units::COUNT_AT_FIRST);
        Some(())
    });
    if known.is_some() && unit == 128 {
        sound::effect("effect/ef_portal.wav", 1.0);
    }
}

/// A skill was used on a place (ZC_NOTIFY_GROUNDSKILL): the effect the client has for that skill plays there
/// (`CGameActor::PlayGroundSkillEffect`, `LaunchDummyEffect`).
pub fn ground_skill(skill: u16, cell: (u16, u16), by: Option<u32>) {
    let Some((effect, ticks)) = crate::skill_units::ground_effect(skill) else { return };
    STATE.with_borrow_mut(|state| {
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

/// Launches one of the client's effects on a unit or a place: its files, if it is played from files
/// (`launch::files`; Pneuma's, picked by lot, is the units' own), with the sound a unit's effect starts with
/// when the client's table has none; and, unlike an effect on someone (`launch::burst`), what of it is made
/// of primitives besides, if that is ported: a Safety Wall is its file and its wall of light.
fn launch(state: &mut State, effect: u32, on: u32, blow: Option<Blow>, time: f64) {
    let mut played = launch::files(state, effect, Some(on), blow, time);
    if let Some(file) = crate::skill_units::picked_file(effect, lot(RAND_MAX)).filter(|_| !played) {
        state.effects_due.push((time, file, Some(on), None));
        played = true;
    }
    if played && crate::effect_tables::effect_sound(effect).is_none() {
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
    }
}

/// One frame of the units: those of a map left and the places whose time is up are forgotten, the map is told
/// where the rest are, and each unit lives the ticks that have passed (`CSkill::OnProcess`), launching its
/// effect when it is due. (After a long pause the ticks missed are not lived through one by one.)
pub(super) fn step(state: &mut State, time: f64) {
    let visit = state.entered;
    let stale: Vec<u32> = state.grounds.units.iter().filter(|(_, standing)| standing.visit != visit).map(|(id, _)| *id).chain(state.grounds.spots.iter().filter(|(_, spot)| spot.1 != visit || spot.2 <= time).map(|(id, _)| *id)).collect();
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
            match crate::skill_units::shows(standing.unit) {
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
}

/// The boxes of light of the units that show one, each wall a quad standing on the unit's cell.
pub(super) fn drawn(state: &State, view: &View, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    for standing in state.grounds.units.values() {
        let Some(pulse) = &standing.pulse else { continue };
        let Some(binds) = gpu.effect_pictures.get(pulse.picture) else { continue };
        let place = on_ground(&scene.gat, standing.cell);
        let nearer = nearer(view, place, 25.0);
        let colour = [pulse.colour[0] / 255.0, pulse.colour[1] / 255.0, pulse.colour[2] / 255.0, pulse.alpha() / 255.0];
        for wall in pulse.walls() {
            // `RenderTeiRect`'s two triangles: the first three corners, then the fourth, the third and the first.
            for index in [0, 1, 2, 3, 2, 0] {
                let (at, uv) = wall[index];
                quads.corner(view, nearer, [place[0] + at[0], place[1] + at[1], place[2] + at[2]], uv, colour);
            }
            quads.picture(binds, pulse.adds);
        }
    }
}
