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
/// touches. Kept with it: the place in the world of the client's turning mark (at the feet of that one if it is
/// a monster, or else of the monster being fought, never at an NPC's), and whether it is a creature to press.
/// Nothing at all when there is no world.
fn pointed(state: &mut State, view: &View) -> Option<Option<(u32, (f32, f32))>> {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
    let over = state.pointer.and_then(|pointer| pointed_at(gpu, scene, view, pointer)).filter(|(id, _)| scene.units.contains_key(id));
    let unit = over.and_then(|(id, _)| Some((id, scene.units.get(&id)?)));
    let marked = unit.filter(|(_, unit)| unit.lively).map(|(id, _)| id).or(state.chase.map(|(id, ..)| id)).and_then(|id| scene.units.get(&id)).filter(|unit| unit.dying.is_none());
    state.target_at = marked.map(|unit| on_ground(&scene.gat, unit.at));
    state.over = unit.filter(|(_, unit)| matches!(unit.look, Look::Creature(_))).map(|(id, unit)| (id, unit.lively));
    Some(over)
}

/// The numbers of the hits rise from the heads of those hit, and are gone in a moment.
fn numbers(state: &mut State, time: f64) {
    let Some(scene) = &state.scene else { return };
    for (target, damage) in std::mem::take(&mut state.hits) {
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
        // Blows taken by the player's own character are red.
        let sprite = if own { "이팩트/숫자#ff0000" } else { "이팩트/숫자" };
        let pictures = if damage > 0 {
            damage.to_string().bytes().map(|digit| SpriteKey::Creature(sprite.to_owned(), 0, usize::from(digit - b'0'))).collect()
        } else {
            vec![SpriteKey::Creature("이팩트/msg".to_owned(), 0, 0)]
        };
        // A miss is shown only when the player has it so (the client's `/miss`).
        if damage > 0 || state.options.misses {
            state.flying.push(Flying { began: time, base: on_ground(&scene.gat, hurt.at), drift, pictures, miss: damage <= 0 });
        }
    }
}

/// The name of whoever the pointer is on goes under their feet, as the original puts it; with nothing pointed
/// at, the name of the monster being fought stays (the client's lock-on).
fn named(state: &mut State, view: &View, over: Option<(u32, (f32, f32))>, (width, height): (f32, f32)) {
    let Some(scene) = &state.scene else { return };
    // 20 pixels of a screen 480 high under the feet, nearer when the camera is.
    let below = height * 20.0 / 480.0 * DISTANCE / scene.camera.distance;
    let fought = state.chase.and_then(|(id, ..)| Some((id, scene.units.get(&id)?))).filter(|(_, unit)| !unit.name.is_empty() && unit.dying.is_none());
    match (over.and_then(|(id, feet)| Some((id, scene.units.get(&id)?, feet))), fought) {
        (Some((id, unit, feet)), _) if !unit.name.is_empty() => {
            state.named = Some(id);
            name_shown(Some((&unit.name, unit.lively, (feet.0, feet.1 + below))));
        }
        (_, Some((id, unit))) => {
            if let Some(feet) = view.project(on_ground(&scene.gat, unit.at)) {
                state.named = Some(id);
                name_shown(Some((&unit.name, unit.lively, (feet.0 * width, feet.1 * height + below))));
            }
        }
        _ if state.named.take().is_some() => name_shown(None),
        _ => {}
    }
}
