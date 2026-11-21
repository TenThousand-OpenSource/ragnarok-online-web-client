//! The pointer moves over the page, and a button is let go.

use super::*;

pub(super) fn moved(event: &MouseEvent) {
    STATE.with_borrow_mut(|state| {
        state.mouse = Some((event.client_x() as f32, event.client_y() as f32));
        state.picking.shift = event.shift_key();
    });
    // With no button held, the pointer shows the name of whoever it is on.
    if event.buttons() == 0 {
        let over_world = event.target().and_then(|target| target.dyn_into::<web_sys::Element>().ok()).is_some_and(|target| target.id() == "game");
        STATE.with_borrow_mut(|state| state.pointer = over_world.then(|| (event.client_x() as f32, event.client_y() as f32)));
        aim_shown();
        return;
    }
    STATE.with_borrow_mut(|state| {
        state.pointer = None;
        if state.held.is_some() {
            state.held = if event.buttons() & 1 != 0 { Some((event.client_x() as f32, event.client_y() as f32)) } else { None };
        }
    });
    STATE.with_borrow_mut(|state| {
        let zoom_out = state.options.zoom_out;
        let (Some(last), Some(scene)) = (state.turning, &mut state.scene) else { return };
        // The right button was let go outside the page.
        if event.buttons() & 2 == 0 {
            state.turning = None;
            return;
        }
        let (x, y) = (event.client_x() as f32, event.client_y() as f32);
        // Inside a building the camera does not turn, but on a map whose view the client's table sets.
        let turns = !scene.indoor || scene.scenery.view.is_some();
        let (shift, ctrl) = (event.shift_key(), event.ctrl_key());
        scene.ask_camera(zoom_out, |camera| {
            if shift {
                camera.pitch += (y - last.1) * 0.3;
            } else if ctrl {
                camera.distance += (y - last.1) * 1.5;
            } else if turns {
                camera.yaw += (x - last.0) * 0.4;
            }
        });
        state.turning = Some((x, y));
    });
}

pub(super) fn released(event: &MouseEvent) {
    if event.button() == 0 {
        // The button let go while a monster is fought only as long as it is held: the fight is called
        // off if a blow was asked for, or one blow is still to come if none was.
        let stop = STATE.with_borrow_mut(|state| {
            state.held = None;
            match (state.fight, state.chase) {
                (1, Some((_, _, true))) => {
                    (state.chase, state.fight) = (None, 0);
                    state.hooks.as_ref().map(|hooks| hooks.stop)
                }
                (1, Some(_)) => {
                    state.fight = 2;
                    None
                }
                _ => None,
            }
        });
        if let Some(stop) = stop {
            stop();
        }
    }
    if event.button() == 2 {
        // Let go about where it went down, on someone: their menu, if they have one (the client's
        // `OpenActorMenu`, on the right button's release). A hand moves the pointer a little between the two;
        // a press that turned the camera is told from one meant for the menu by having gone farther than a
        // few pixels. (That measure, 4 pixels, is this project's: the client's own was not found.)
        let menu = STATE.with_borrow_mut(|state| {
            let at = state.right_at.take().filter(|at| (at.0 - event.client_x() as f32).abs().max((at.1 - event.client_y() as f32).abs()) <= 4.0)?;
            let (gpu, scene) = (state.gpu.as_ref()?, state.scene.as_ref()?);
            let view = scene.view(gpu);
            let (id, _) = pointed_at(gpu, scene, &view, at, false)?;
            state.hooks.as_ref().map(|hooks| (hooks.menu, id, at))
        });
        if let Some((menu, id, at)) = menu {
            menu(id, at);
        }
        STATE.with_borrow_mut(|state| state.turning = None);
    }
    // The side buttons would leave the page for the one before or after it.
    if event.button() > 2 && STATE.with_borrow(|state| state.scene.is_some()) {
        event.prevent_default();
    }
}
