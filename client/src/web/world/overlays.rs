//! What is laid over the picture of the world once it is drawn: the pointer's mark and its picture, the name of
//! whoever it is on, the numbers of the blows, the bars of life and of casts, what was said over heads.

use super::*;

mod bars;
mod pointer;

pub(super) use pointer::pointers;

/// Works those out for this frame and puts them on the page. `feet` is where the player's own character stands
/// and `casts` the casts under way (where, since when, for how long, which effect, at whom), as the frame found
/// them while drawing. Gives back the page's size, which what follows needs too.
pub(super) fn overlays(state: &mut State, view: &View, time: f64, feet: Vector, casts: &[((f32, f32), f64, f64, u32, Option<u32>)]) -> Option<(f32, f32)> {
    let size = state.gpu.as_ref().map(|gpu| (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32))?;
    let over = pointed(state, view)?;
    bars::shown(state, view, time, feet, casts, size);
    signs::shown(state, view, size);
    pointer::shown(state, view, time, over.map(|(id, _)| id), size);
    named(state, view, over, size);
    Some(size)
}

/// Whoever the pointer is on (and where on the page their feet are): the nearest of those whose picture it
/// touches. Kept with it: the places in the world of the turning marks, and whether it is a creature to press.
/// Nothing at all when there is no world.
///
/// The marks are the client's (the third of its pointers, at a monster's feet, never at an NPC's), but the
/// client has one: on whoever the pointer is on, or else on the monster being fought
/// (`CGameMode::UpdateHoverNameBalloon`). Here, by this project's owner's wish, the monster being fought keeps
/// its mark while the pointer is on another, which gets a second for as long as the pointer is there.
fn pointed(state: &mut State, view: &View) -> Option<Option<(u32, (f32, f32))>> {
    still_over_world(state);
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
    let over = state.pointer.and_then(|pointer| pointed_at(gpu, scene, view, pointer, state.players_first())).filter(|(id, _)| scene.units.contains_key(id));
    let unit = over.and_then(|(id, _)| Some((id, scene.units.get(&id)?)));
    let fought = state.chase.map(|(id, ..)| id);
    // A player who is fought on this map is marked and pressed as a monster is (`pvp.rs`).
    let foe = |id: u32, unit: &Mover| state.pvp.foe(id, unit);
    let pointed = unit.filter(|(id, unit)| (unit.lively || foe(*id, unit)) && Some(*id) != fought).map(|(id, _)| id);
    state.target_at = [fought, pointed].map(
        |id| id.and_then(|id| scene.units.get(&id)).filter(|unit| unit.dying.is_none()).map(|unit| on_ground(&scene.gat, unit.at))
    );
    state.over = unit.filter(|(id, unit)| unit.look.creature() || foe(*id, unit)).map(|(id, unit)| (id, unit.lively || foe(id, unit)));
    Some(over)
}

/// The name of whoever the pointer is on goes under their feet, as the original puts it; with nothing pointed
/// at, the name of the monster being fought stays (the client's lock-on).
fn named(state: &mut State, view: &View, over: Option<(u32, (f32, f32))>, (width, height): (f32, f32)) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    // 20 pixels of a screen 480 high under the feet, nearer when the camera is; with the player's `/font` on,
    // 100 over them (the client's `PlaceWindowAtActor`).
    let below = crate::bars::below(if state.options.name_above { -100.0 } else { 20.0 }, height, scene.camera.distance);
    let fought = state.chase.and_then(|(id, ..)| Some((id, scene.units.get(&id)?))).filter(|(_, unit)| !unit.name.is_empty() && unit.dying.is_none());
    // A thing on the ground under the pointer, when nobody is: what it is and how many (`lying_name`).
    let lying = || {
        let id = state.pointer.and_then(|pointer| lying_at(gpu, scene, view, pointer))?;
        let thing = scene.drops.get(&id)?;
        let under = view.project(on_ground(&scene.gat, thing.at))?;
        Some((id, lying_name(thing)?, (under.0 * width, under.1 * height + below)))
    };
    // What is written under someone in sight, its colour and the picture beside it (`names.rs`): an NPC's
    // name in pale blue, a monster's by its level, a title before it, a boss's mark.
    let of_unit = |id: u32, unit: &Mover, label: String, at: (f32, f32)| {
        let (label, colour, mark) = names::written(state, id, unit, label);
        (id, label, colour, at, mark)
    };
    let of_fought = || fought.and_then(|(id, unit)| view.project(on_ground(&scene.gat, unit.at)).map(
        |feet| of_unit(id, unit, unit.name.clone(), (feet.0 * width, feet.1 * height + below))
    ));
    let pointed = over.and_then(|(id, feet)| Some((id, scene.units.get(&id)?, feet))).filter(|(_, unit, _)| !unit.name.is_empty());
    // The player's own character under the pointer, when nobody else is: it has its name too (`guild.rs`).
    let own = || {
        let feet = touched(gpu, scene, view, &scene.player, state.pointer.filter(|_| over.is_none())?)?;
        let (id, name) = state.guilds.own_label()?;
        Some((id, names::titled(state, id, name), crate::names::PLAIN, (feet.0, feet.1 + below), None))
    };
    // Whoever the pointer is on; else the player's own character; else the thing on the ground it is on; else
    // the monster being fought.
    let shown = match pointed {
        // A player's name is written with the party's and the guild's (`guild.rs`).
        Some((id, unit, feet)) => Some(of_unit(id, unit, state.guilds.label(id, &unit.name), (feet.0, feet.1 + below))),
        None => own().or_else(|| lying().filter(|_| over.is_none()).map(|(id, name, under)| (id, name, LYING_NAME, under, None))).or_else(of_fought),
    };
    match shown {
        Some((id, name, colour, at, mark)) => {
            state.named = Some(id);
            name_shown(Some((&name, colour, at)));
            state.guilds.decorate(id, scene.units.get(&id), state.options.emblems);
            if let Some(mark) = mark {
                names::marked(&mark);
            }
        }
        None if state.named.take().is_some() => name_shown(None),
        None => {}
    }
}

/// The colour the client writes the name of a thing on the ground in (`CGameMode::UpdateHoverNameBalloon`: its
/// 0x95EFFF, blue first).
const LYING_NAME: &str = "#ffef95";

/// What the client writes under a thing on the ground the pointer is on: its message 183, "%s: %d ea.", with the
/// thing's name (that of what it passes for, while it is not identified) and how many lie there
/// (`CItem::SendMsg`). Nothing until the item's name is known.
fn lying_name(thing: &Lying) -> Option<String> {
    let (name, _) = data::item_as(thing.item, thing.identified)?;
    let line = data::message(183).unwrap_or_else(|| "%s: %d ea.".to_owned());
    Some(line.replacen("%s", &name, 1).replacen("%d", &thing.count.to_string(), 1))
}

/// Shows a name on the page in a colour, its middle at a place, or takes it away.
pub(super) fn name_shown(named: Option<(&str, &str, (f32, f32))>) {
    let Some(label) = crate::web::by_id("unit-name") else { return };
    match named {
        Some((name, colour, (x, y))) => {
            if label.text_content().as_deref() != Some(name) {
                label.set_text_content(Some(name));
            }
            let _ = label.set_attribute("style", &format!("display:block;left:{x:.0}px;top:{y:.0}px;color:{colour}"));
        }
        None => {
            let _ = label.set_attribute("style", "display:none");
        }
    }
}
