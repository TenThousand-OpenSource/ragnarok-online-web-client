//! What lies on the ground while a skill is cast: the circles under the places skills are being cast on
//! (`crate::scope`, the client's `CGroundScope`), each a square of cells with the circle's picture turning over
//! it, until its cast is over; and the red planes over the area a skill warns of (`CWarningPlaneEffect`).

use super::*;

/// The picture of the circle, among the client's pictures of effects.
const PICTURE: &str = "magic_target.tga";

/// How far over the ground a circle lies, so that the ground does not cut through it.
const OVER: f32 = 0.4;

/// The picture of a plane that warns (`CWarningPlaneEffect::AddPlane`): it is not among the pictures of
/// effects but beside their folder, and is read when the first area is warned of.
const WARNING: &str = "warning_plane.tga";

/// A circle on the ground: the cell it is around, how many cells across it is, when it began and when the cast
/// it is of is over. A warned area is kept with the circles: its `side` is nought and `warned` has its cells
/// and who casts (it goes with that one, `CWarningPlaneEffect::IsAlive`).
pub(super) struct Scope {
    cell: (i32, i32),
    side: usize,
    began: f64,
    until: f64,
    warned: Option<(Vec<(i32, i32)>, Option<u32>)>,
}

/// Someone starts to cast a skill on a place (the client's `LaunchSkillCastEffect` for a cast aimed at no one):
/// the circle of that skill lies around the place for as long as the cast takes.
pub fn scope_shown(cell: (u16, u16), skill: u16, milliseconds: f64) {
    STATE.with_borrow_mut(|state| {
        let began = now();
        let cell = (i32::from(cell.0), i32::from(cell.1));
        state.scopes.push(Scope { cell, side: crate::scope::side(skill), began, until: began + milliseconds, warned: None });
    });
}

/// Someone starts to cast a skill that warns of its area (the client's `CGameMode::SpawnSkillWarningPlanes`):
/// a red plane lies on each cell of `across` cells around `cell` for as long as the cast takes, or until the
/// caster is gone. Nothing for a caster not in sight, as in the client. `who` is the caster by the server's
/// number, `own` the player's own.
pub fn area_warned(who: u32, own: u32, cell: (u16, u16), across: (i32, i32), milliseconds: f64) {
    let known = STATE.with_borrow_mut(|state| {
        if state.scene.as_ref().is_none_or(|scene| scene.one(who, own).is_none()) {
            return true;
        }
        let cell = (i32::from(cell.0), i32::from(cell.1));
        let (cells, began) = (crate::scope::warned_cells(cell, across), now());
        state.scopes.push(Scope { cell, side: 0, began, until: began + milliseconds, warned: Some((cells, (who != own).then_some(who))) });
        state.gpu.as_ref().is_none_or(|gpu| gpu.effect_pictures.contains_key(WARNING))
    });
    if !known {
        wasm_bindgen_futures::spawn_local(async {
            let Ok(image) = picture(WARNING).await else { return };
            STATE.with_borrow_mut(|state| {
                if let Some(gpu) = &mut state.gpu {
                    let binds = gpu.effect_binds(image.width, image.height, &image.rgba, &gpu.sampler);
                    gpu.effect_pictures.insert(WARNING, binds);
                }
            });
        });
    }
}

/// The planes of the warned areas of this frame (`CWarningPlaneEffect::OnProcess` and `Render`): each a red
/// square of one cell lying level at the height of its cell's middle, turned about its middle as the camera
/// is turned, showing more and less. A tick is taken as for the circles (`CAST_FRAME_MS`). Not found in the
/// client: how the plane's blending is set (it is drawn see-through here, adding no light) and which way up its
/// picture lies (its top is away from the eye here).
fn warned(state: &State, view: &View, time: f64, quads: &mut Quads) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let Some(binds) = gpu.effect_pictures.get(WARNING) else { return };
    let half = crate::scope::WARNED_HALF;
    let (right, ahead) = (view.right, [-view.right[2], 0.0, view.right[0]]);
    for scope in &state.scopes {
        let Some((cells, _)) = &scope.warned else { continue };
        let colour = [1.0, 0.0, 0.0, crate::scope::warned_alpha(((time - scope.began) / CAST_FRAME_MS) as u32)];
        for (column, row) in cells {
            let middle = on_ground(&scene.gat, (*column as f32, *row as f32));
            let corner = |across: f32, away: f32| [
                middle[0] + (right[0] * across + ahead[0] * away) * half,
                middle[1] + OVER,
                middle[2] + (right[2] * across + ahead[2] * away) * half
            ];
            let corners = [(corner(-1.0, 1.0), [0.0, 0.0]), (corner(1.0, 1.0), [1.0, 0.0]), (corner(-1.0, -1.0), [0.0, 1.0]), (corner(1.0, -1.0), [1.0, 1.0])];
            for index in [0, 1, 2, 2, 1, 3] {
                quads.corners.extend(corners[index].0);
                quads.corners.extend(corners[index].1);
                quads.corners.extend(colour);
            }
            quads.picture(binds, false);
        }
    }
}

/// The circles of this frame: each cell of each a quad lying on the ground, showing its part of the turned
/// picture. The client turns a circle a step each tick of its own; here a tick is a frame of its coded effects
/// (`CAST_FRAME_MS`), which is this project's reading of it.
pub(super) fn drawn(state: &mut State, view: &View, time: f64, quads: &mut Quads) {
    let scene = state.scene.as_ref();
    let gone = |who: &Option<u32>| scene.is_some_and(|scene| scene.mover(*who).is_none());
    state.scopes.retain(|scope| scope.until >= time && !scope.warned.as_ref().is_some_and(|(_, who)| gone(who)));
    warned(state, view, time, quads);
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let Some(binds) = gpu.effect_pictures.get(PICTURE) else { return };
    for scope in state.scopes.iter().filter(|scope| scope.warned.is_none()) {
        let angle = ((time - scope.began) / CAST_FRAME_MS) as f32 * crate::scope::turn(scope.side);
        let colour = [1.0, 1.0, 1.0, crate::scope::alpha(angle)];
        for ((column, row), shown) in crate::scope::cells(scope.cell, scope.side, angle) {
            // The cell's four corners on the ground, in the order the picture's parts are given in: its own
            // corner, the next cell along, the next row, and both.
            let corner = |east: f32, north: f32| {
                let at = on_ground(&scene.gat, (column as f32 - 0.5 + east, row as f32 - 0.5 + north));
                [at[0], at[1] + OVER, at[2]]
            };
            let corners = [corner(0.0, 0.0), corner(1.0, 0.0), corner(0.0, 1.0), corner(1.0, 1.0)];
            for index in [0, 1, 2, 2, 1, 3] {
                quads.corners.extend(corners[index]);
                quads.corners.extend(shown[index]);
                quads.corners.extend(colour);
            }
            quads.picture(binds, false);
        }
    }
}
