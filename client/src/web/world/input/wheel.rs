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
    // With Shift held the wheel tilts the camera, 5 degrees a notch (`ProcessMouseWheel`): between 50 and 65
    // degrees outdoors, as flat as 25 with Ctrl held too; else it moves the camera in and out.
    let (shift, ctrl) = (event.shift_key(), event.ctrl_key());
    STATE.with_borrow_mut(|state| {
        let zoom_out = state.options.zoom_out;
        let Some(scene) = &mut state.scene else { return };
        // A notch of a mouse's wheel is about a hundred pixels, and one step of the camera. A touchpad sends
        // many small moves, a finger resting on it during a click among them: those move it as little.
        let pixels = event.delta_y() * [1.0, 16.0, 400.0][event.delta_mode().min(2) as usize];
        if pixels.abs() < 4.0 {
            return;
        }
        let notches = (pixels / 100.0).clamp(-1.0, 1.0) as f32;
        let tilts = scene.scenery.view.is_none().then(|| crate::scenery::views::wheel_tilts(scene.indoor, ctrl));
        scene.ask_camera(zoom_out, |camera| {
            if shift {
                // (The way of the wheel that moves the camera out lays it flatter.)
                camera.pitch -= notches * crate::scenery::views::WHEEL_TILT;
                if let Some([flattest, steepest]) = tilts {
                    camera.pitch = camera.pitch.clamp(flattest, steepest);
                }
            } else {
                camera.distance += notches * crate::scenery::views::WHEEL_STEP;
            }
        });
    });
}
