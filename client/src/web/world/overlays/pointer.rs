//! The picture of the mouse pointer over the world: which of the game's pointers it is by what it is over, and
//! which of that pointer's pictures shows now (`crate::cursor`).

use super::*;

/// Picks the pointer for what the mouse is over (`over`: whoever it is on) and puts its picture on the canvas.
pub(super) fn shown(state: &mut State, view: &View, time: f64, over: Option<u32>, (width, height): (f32, f32)) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let unit = over.and_then(|id| scene.units.get(&id));
    // Another player who is fought on this map has a monster's pointers: the sword, and the aimed skill's
    // other picture (the client's cursor actions 5 and 11 in `CGameMode::HandleActorPick`).
    let foe = over.zip(unit).is_some_and(|(id, unit)| state.pvp.foe(id, unit));
    let lying = || state.pointer.is_some_and(|pointer| lying_at(gpu, scene, view, pointer).is_some());
    // A portal is pointed at anywhere over its rings.
    let near = DISTANCE / scene.camera.distance * height / 480.0;
    let on_portal = state.pointer.is_some_and(|pointer| {
        scene.portals.values().any(|cell| {
            view.project(on_ground(&scene.gat, (cell.0 as f32, cell.1 as f32))).is_some_and(|middle| {
                let (across, up) = ((pointer.0 - middle.0 * width).abs(), middle.1 * height - pointer.1);
                across < 45.0 * near && (-25.0 * near..45.0 * near).contains(&up)
            })
        })
    });
    // Which of `POINTERS` it is.
    let pointing = match unit {
        _ if state.turning.is_some() => 5,
        // A skill being aimed: the pointer of a skill, and its other picture over a monster it can go to
        // (the client's cursor actions 10 and 11).
        Some(unit) if state.aiming == Some(false) && (unit.lively || foe) => 7,
        _ if state.aiming.is_some() => 6,
        // Over a pet the pointer stays plain, as in the client.
        Some(unit) if unit.companion => 0,
        Some(unit) if unit.lively || foe => 2,
        // A thing lying at an NPC's feet is still picked up.
        _ if lying() => 4,
        Some(unit) if matches!(unit.look, Look::Creature(_)) => 1,
        None if on_portal => 3,
        // Over ground nobody can walk on, or off the map (the client's action 13, when nothing else is pointed at).
        None if state.pointer.is_some_and(|pointer| view.pick(&scene.gat, (pointer.0 / width.max(1.0), pointer.1 / height.max(1.0))).is_none_or(|cell| !scene.gat.walkable(cell.0, cell.1))) => 8,
        _ => 0,
    };
    if pointing != state.pointing {
        state.pointing_since = time;
    }
    let pictures = state.pointers.get(pointing).map_or(0, |(pictures, _)| pictures.len());
    let pointer_frame = crate::cursor::frame(POINTERS.get(pointing).copied().unwrap_or(0), pictures, time - state.pointing_since, state.held.map(|_| time - state.held_since));
    if pointing != state.pointing || pointer_frame != state.pointer_frame {
        state.pointing = pointing;
        state.pointer_frame = pointer_frame;
        if let Some(pointer) = state.pointers.get(pointing).and_then(|(pictures, _)| pictures.get(pointer_frame)) {
            let _ = gpu.canvas.style().set_property("cursor", pointer);
        }
    }
}
