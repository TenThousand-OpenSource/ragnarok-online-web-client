//! What is laid over the picture of the world once it is drawn: the pointer's mark and its picture, the name of
//! whoever it is on, the numbers of the blows, the bars of life and of casts, what was said over heads.

use super::*;

mod bars;
mod pointer;

/// Works those out for this frame and puts them on the page. `feet` is where the player's own character stands
/// and `casts` the casts under way (where, since when, for how long, which effect, at whom), as the frame found
/// them while drawing. Gives back the page's size, which what follows needs too.
pub(super) fn overlays(state: &mut State, view: &View, time: f64, feet: Vector, casts: &[((f32, f32), f64, f64, u32, Option<u32>)]) -> Option<(f32, f32)> {
    let size = state.gpu.as_ref().map(|gpu| (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32))?;
    let over = pointed(state, view)?;
    numbers(state, time);
    bars::shown(state, view, time, feet, casts, size);
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
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
    let over = state.pointer.and_then(|pointer| pointed_at(gpu, scene, view, pointer)).filter(|(id, _)| scene.units.contains_key(id));
    let unit = over.and_then(|(id, _)| Some((id, scene.units.get(&id)?)));
    let fought = state.chase.map(|(id, ..)| id);
    let pointed = unit.filter(|(id, unit)| unit.lively && Some(*id) != fought).map(|(id, _)| id);
    state.target_at = [fought, pointed].map(|id| id.and_then(|id| scene.units.get(&id)).filter(|unit| unit.dying.is_none()).map(|unit| on_ground(&scene.gat, unit.at)));
    state.over = unit.filter(|(_, unit)| matches!(unit.look, Look::Creature(_))).map(|(id, unit)| (id, unit.lively));
    Some(over)
}

/// The sprite of a critical blow's mark: the numbers', greyed.
const CRITICAL_MARK: &str = "이팩트/숫자#aaaaaa";

/// The numbers of the hits rise from the heads of those hit, and are gone in a moment.
fn numbers(state: &mut State, time: f64) {
    let Some(scene) = &state.scene else { return };
    for (target, damage, critical) in std::mem::take(&mut state.hits) {
        let (hurt, own) = match scene.units.get(&target) {
            Some(unit) => (unit, false),
            None => (&scene.player, true),
        };
        // The client sends the number off by the quarter the one struck faces (`MsgEffectDrift`).
        let drift = match hurt.facing {
            0 | 1 => (-0.8, 0.8),
            2 | 3 => (-0.8, -0.8),
            4 | 5 => (0.8, -0.8),
            _ => (0.8, 0.8),
        };
        // Blows taken by the player's own character are red; a critical blow is yellow whoever takes it (the
        // client's `CGameActor::SendMsg`: its number is made with 0xFFFFFF00).
        let sprite = if critical {
            "이팩트/숫자#ffff00"
        } else if own {
            "이팩트/숫자#ff0000"
        } else {
            "이팩트/숫자"
        };
        let pictures = if damage > 0 {
            damage.to_string().bytes().map(|digit| SpriteKey::Creature(sprite.to_owned(), 0, usize::from(digit - b'0'))).collect()
        } else {
            vec![SpriteKey::Creature("이팩트/msg".to_owned(), 0, 0)]
        };
        // A miss is shown only when the player has it so (the client's `/miss`).
        if damage > 0 || state.options.misses {
            let base = on_ground(&scene.gat, hurt.at);
            // With a critical blow goes its mark: the third action of the numbers' sprite, greyed
            // (`SpawnMsgEffect` of `MAKETYPE_CRITICAL_EFFECT`, its colour 0xFFAAAAAA).
            if critical {
                let mark = vec![SpriteKey::Creature(CRITICAL_MARK.to_owned(), 2, 0)];
                state.flying.push(Flying { began: time, base, drift, pictures: mark, flight: Flight::Mark });
            }
            let flight = if damage <= 0 {
                Flight::Miss
            } else if critical {
                Flight::Critical
            } else {
                Flight::Number
            };
            state.flying.push(Flying { began: time, base, drift, pictures, flight });
        }
    }
}

/// The name of whoever the pointer is on goes under their feet, as the original puts it; with nothing pointed
/// at, the name of the monster being fought stays (the client's lock-on).
fn named(state: &mut State, view: &View, over: Option<(u32, (f32, f32))>, (width, height): (f32, f32)) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    // 20 pixels of a screen 480 high under the feet, nearer when the camera is.
    let below = height * 20.0 / 480.0 * DISTANCE / scene.camera.distance;
    // Monsters' names are pink, as the original's.
    let colour = |unit: &Mover| if unit.lively { "#ffc3c3" } else { "#fff" };
    let fought = state.chase.and_then(|(id, ..)| Some((id, scene.units.get(&id)?))).filter(|(_, unit)| !unit.name.is_empty() && unit.dying.is_none());
    // A thing on the ground under the pointer, when nobody is: what it is and how many (`lying_name`).
    let lying = || {
        let id = state.pointer.and_then(|pointer| lying_at(gpu, scene, view, pointer))?;
        let thing = scene.drops.get(&id)?;
        let under = view.project(on_ground(&scene.gat, thing.at))?;
        Some((id, lying_name(thing)?, (under.0 * width, under.1 * height + below)))
    };
    let of_fought = || fought.and_then(|(id, unit)| view.project(on_ground(&scene.gat, unit.at)).map(|feet| (id, unit.name.clone(), colour(unit), (feet.0 * width, feet.1 * height + below))));
    let pointed = over.and_then(|(id, feet)| Some((id, scene.units.get(&id)?, feet))).filter(|(_, unit, _)| !unit.name.is_empty());
    // Whoever the pointer is on; else the thing on the ground it is on; else the monster being fought.
    let shown = match pointed {
        Some((id, unit, feet)) => Some((id, unit.name.clone(), colour(unit), (feet.0, feet.1 + below))),
        None => lying().filter(|_| over.is_none()).map(|(id, name, under)| (id, name, LYING_NAME, under)).or_else(of_fought),
    };
    match shown {
        Some((id, name, colour, at)) => {
            state.named = Some(id);
            name_shown(Some((&name, colour, at)));
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
