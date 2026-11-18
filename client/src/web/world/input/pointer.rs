//! The pointer moves over the page, and a button is let go.

use super::*;

pub(super) fn moved(event: &MouseEvent) {
    STATE.with_borrow_mut(|state| state.mouse = Some((event.client_x() as f32, event.client_y() as f32)));
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
        let (pitches, distances) = Camera::limits(scene.indoor, zoom_out);
        let indoor = scene.indoor;
        let camera = &mut scene.camera;
        if event.shift_key() {
            camera.pitch = (camera.pitch + (y - last.1) * 0.3).clamp(pitches[0], pitches[1]);
        } else if event.ctrl_key() {
            camera.distance = (camera.distance + (y - last.1) * 1.5).clamp(distances[0], distances[1]);
        } else if !indoor {
            camera.yaw = (camera.yaw + (x - last.0) * 0.4).rem_euclid(360.0);
        }
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
        // Let go where it went down, on someone: their menu, if they have one (the client's
        // `OpenActorMenu`, which wants the pointer not to have moved).
        let menu = STATE.with_borrow_mut(|state| {
            let at = state.right_at.take().filter(|at| *at == (event.client_x() as f32, event.client_y() as f32))?;
            let (gpu, scene) = (state.gpu.as_ref()?, state.scene.as_ref()?);
            let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
            let (id, _) = pointed_at(gpu, scene, &view, at)?;
            state.hooks.as_ref().map(|hooks| (hooks.menu, id, at))
        });
        if let Some((menu, id, at)) = menu {
            menu(id, at);
        }
        STATE.with_borrow_mut(|state| {
            state.turning = None;
            if let Some(scene) = &state.scene {
                if !scene.indoor {
                    scene.camera.remember();
                }
            }
        });
    }
    // The side buttons would leave the page for the one before or after it.
    if event.button() > 2 && STATE.with_borrow(|state| state.scene.is_some()) {
        event.prevent_default();
    }
}
