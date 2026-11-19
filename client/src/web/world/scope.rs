//! The circles on the ground under the places skills are being cast on (`crate::scope`, the client's
//! `CGroundScope`): each a square of cells with the circle's picture turning over it, until its cast is over.

use super::*;

/// The picture of the circle, among the client's pictures of effects.
const PICTURE: &str = "magic_target.tga";

/// How far over the ground a circle lies, so that the ground does not cut through it.
const OVER: f32 = 0.4;

/// A circle on the ground: the cell it is around, how many cells across it is, when it began and when the cast
/// it is of is over.
pub(super) struct Scope {
    cell: (i32, i32),
    side: usize,
    began: f64,
    until: f64,
}

/// Someone starts to cast a skill on a place (the client's `LaunchSkillCastEffect` for a cast aimed at no one):
/// the circle of that skill lies around the place for as long as the cast takes.
pub fn scope_shown(cell: (u16, u16), skill: u16, milliseconds: f64) {
    STATE.with_borrow_mut(|state| {
        let began = now();
        state.scopes.push(Scope { cell: (i32::from(cell.0), i32::from(cell.1)), side: crate::scope::side(skill), began, until: began + milliseconds });
    });
}

/// The circles of this frame: each cell of each a quad lying on the ground, showing its part of the turned
/// picture. The client turns a circle a step each tick of its own; here a tick is a frame of its coded effects
/// (`CAST_FRAME_MS`), which is this project's reading of it.
pub(super) fn drawn(state: &mut State, time: f64, quads: &mut Quads) {
    state.scopes.retain(|scope| scope.until >= time);
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let Some(binds) = gpu.effect_pictures.get(PICTURE) else { return };
    for scope in &state.scopes {
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
