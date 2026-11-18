//! The wheel over the world: the level of a skill being aimed, or the camera.

use super::*;

pub(super) fn turned(event: &WheelEvent) {
    event.prevent_default();
    // While a skill is aimed the wheel picks the level it is used at, up or down one a notch (the client's
    // `MM_USESKILL_LEVEL_PLUS` and `_MINUS`), and the camera stays where it is.
    // With Quick Skill 2 the wheel uses the eighth slot of the hotkey bar turned up and the seventh turned
    // down, and moves nothing (the client's `ProcessMouseWheel`).
    let quick = STATE.with_borrow(|state| state.hooks.as_ref().filter(|_| state.options.quick_wheel && state.aiming.is_none()).map(|hooks| hooks.quick));
    if let Some(quick) = quick {
        if event.delta_y() != 0.0 {
            quick(if event.delta_y() < 0.0 { 7 } else { 6 });
        }
        return;
    }
    let level = STATE.with_borrow(|state| state.aiming.and(state.hooks.as_ref()).map(|hooks| hooks.aim_level));
    if let Some(level) = level {
        if event.delta_y() != 0.0 {
            level(if event.delta_y() < 0.0 { 1 } else { -1 });
        }
        return;
    }
    STATE.with_borrow_mut(|state| {
        let zoom_out = state.options.zoom_out;
        if let Some(scene) = &mut state.scene {
            // A notch of a mouse's wheel is about a hundred pixels, and one step of the camera. A touchpad sends
            // many small moves, a finger resting on it during a click among them: those move it as little.
            let pixels = event.delta_y() * [1.0, 16.0, 400.0][event.delta_mode().min(2) as usize];
            let step = (pixels * 0.15).clamp(-15.0, 15.0) as f32;
            if pixels.abs() >= 4.0 {
                let (_, distances) = Camera::limits(scene.indoor, zoom_out);
                scene.camera.distance = (scene.camera.distance + step).clamp(distances[0], distances[1]);
                if !scene.indoor {
                    scene.camera.remember();
                }
            }
        }
    });
}
