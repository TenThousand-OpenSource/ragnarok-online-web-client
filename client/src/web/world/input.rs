//! The mouse over the world: walking, pressing someone, aiming a skill, turning and zooming the camera.

use super::*;

/// The mouse on the canvas: a left press walks, the right button turns the camera, the wheel moves it.
pub(super) fn listen() {
    if STATE.with_borrow_mut(|state| std::mem::replace(&mut state.listening, true)) {
        return;
    }
    let Ok(canvas) = canvas() else { return };
    let position = |event: &MouseEvent, canvas: &HtmlCanvasElement| {
        (event.offset_x() as f32 / canvas.client_width().max(1) as f32, event.offset_y() as f32 / canvas.client_height().max(1) as f32)
    };

    let pressed = {
        let canvas = canvas.clone();
        Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            let at = position(&event, &canvas);
            // The browser does nothing of its own with a press on the world: no text marked, no scrolling by the
            // middle button, no page left by the side buttons.
            event.prevent_default();
            match event.button() {
                0 if STATE.with_borrow(|state| state.aiming.is_some()) => {
                    // A skill is being aimed: the press picks whom, or where, it is used on. Whatever is
                    // pressed, the aiming is over with it, as in the client (a skill for someone pressed on the
                    // bare ground is not used).
                    let aimed = STATE.with_borrow_mut(|state| {
                        let ground = state.aiming.take() == Some(true);
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let view = scene.camera.view(on_ground(&scene.gat, scene.player.at), gpu.config.width as f32 / gpu.config.height as f32);
                        let pointer = (event.client_x() as f32, event.client_y() as f32);
                        let aimed = if ground {
                            let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
                            let cell = view.pick(&scene.gat, (pointer.0 / width.max(1.0), pointer.1 / height.max(1.0)))?;
                            (None, (cell.0.max(0) as u16, cell.1.max(0) as u16))
                        } else {
                            let (id, _) = pointed_at(gpu, scene, &view, pointer)?;
                            let unit = scene.units.get(&id)?;
                            (Some(id), (unit.cell.0.max(0) as u16, unit.cell.1.max(0) as u16))
                        };
                        // The skill is used from within its reach: the character walks there first if it must
                        // (see the frame's code).
                        state.chase = None;
                        state.fetch = None;
                        state.reach = Some((aimed.0, (i32::from(aimed.1.0), i32::from(aimed.1.1)), state.aim_range, 0.0));
                        Some(())
                    });
                    let _ = aimed;
                }
                // The right button lets go of a skill being aimed.
                2 if STATE.with_borrow(|state| state.aiming.is_some()) => STATE.with_borrow_mut(|state| state.aiming = None),
                0 => {
                    // A press on someone talks to them or fights them; a press on the ground walks there.
                    let pressed = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let view = scene.camera.view(on_ground(&scene.gat, scene.player.at), gpu.config.width as f32 / gpu.config.height as f32);
                        let (id, _) = pointed_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32))?;
                        let unit = scene.units.get(&id).filter(|unit| matches!(unit.look, Look::Creature(_)))?;
                        // A thing lying at an NPC's feet is picked up, not the NPC talked to.
                        if !unit.lively && lying_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32)).is_some() {
                            return None;
                        }
                        state.hooks.as_ref().map(|hooks| (hooks.press, id, unit.lively))
                    });
                    if let Some((press, id, monster)) = pressed {
                        // A monster is run after until it is down or the player presses somewhere else.
                        STATE.with_borrow_mut(|state| {
                            state.chase = monster.then(|| (id, now(), true));
                            state.strike_asked = now();
                            state.reach = None;
                        });
                        press(id, monster);
                        return;
                    }
                    // A press on a thing lying on the ground goes to it and picks it up.
                    let lying = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let view = scene.camera.view(on_ground(&scene.gat, scene.player.at), gpu.config.width as f32 / gpu.config.height as f32);
                        lying_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32))
                    });
                    if let Some(id) = lying {
                        STATE.with_borrow_mut(|state| {
                            state.chase = None;
                            state.fetch = Some((id, 0.0));
                        });
                        return;
                    }
                    STATE.with_borrow_mut(|state| {
                        state.fetch = None;
                        state.chase = None;
                        state.reach = None;
                        state.held = Some((event.client_x() as f32, event.client_y() as f32));
                        state.held_asked = now();
                    });
                    let request = STATE.with_borrow(|state| {
                        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
                        let player = &scene.player;
                        let feet = on_ground(&scene.gat, (player.cell.0 as f32, player.cell.1 as f32));
                        let view = scene.camera.view(feet, gpu.config.width as f32 / gpu.config.height as f32);
                        let picked = view.pick(&scene.gat, at)?;
                        // A press on a wall, or too far away, walks as far towards it as the way is open.
                        let goal = if scene.gat.path(player.cell, picked).is_empty() { scene.gat.reachable_towards(player.cell, picked)? } else { picked };
                        state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal))
                    });
                    if let Some((walk_to, goal)) = request {
                        walk_to(goal.0 as u16, goal.1 as u16);
                    }
                }
                2 => STATE.with_borrow_mut(|state| {
                    let pressed = now();
                    if pressed - state.right_pressed < 350.0 {
                        if let Some(scene) = &mut state.scene {
                            scene.camera = Camera { distance: scene.camera.distance, ..Camera::starting(scene.indoor) };
                            if !scene.indoor {
                        scene.camera.remember();
                    }
                        }
                    }
                    state.right_pressed = pressed;
                    state.turning = Some((event.client_x() as f32, event.client_y() as f32));
                }),
                _ => {}
            }
        })
    };
    let moved = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        // With no button held, the pointer shows the name of whoever it is on.
        if event.buttons() == 0 {
            let over_world = event.target().and_then(|target| target.dyn_into::<web_sys::Element>().ok()).is_some_and(|target| target.id() == "game");
            STATE.with_borrow_mut(|state| state.pointer = over_world.then(|| (event.client_x() as f32, event.client_y() as f32)));
            return;
        }
        STATE.with_borrow_mut(|state| {
            state.pointer = None;
            if state.held.is_some() {
                state.held = if event.buttons() & 1 != 0 { Some((event.client_x() as f32, event.client_y() as f32)) } else { None };
            }
        });
        STATE.with_borrow_mut(|state| {
            let (Some(last), Some(scene)) = (state.turning, &mut state.scene) else { return };
            // The right button was let go outside the page.
            if event.buttons() & 2 == 0 {
                state.turning = None;
                return;
            }
            let (x, y) = (event.client_x() as f32, event.client_y() as f32);
            let (pitches, distances) = Camera::limits(scene.indoor);
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
    });
    let released = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        if event.button() == 0 {
            STATE.with_borrow_mut(|state| state.held = None);
        }
        if event.button() == 2 {
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
    });
    let side = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| {
        if event.button() != 0 && event.button() != 2 && STATE.with_borrow(|state| state.scene.is_some()) {
            event.prevent_default();
        }
    });
    // Holding Ctrl, the wheel (and a pinch on a touchpad) would make the whole page larger or smaller.
    let page_zoom = Closure::<dyn FnMut(WheelEvent)>::new(|event: WheelEvent| {
        if event.ctrl_key() && STATE.with_borrow(|state| state.scene.is_some()) {
            event.prevent_default();
        }
    });
    let wheel = Closure::<dyn FnMut(WheelEvent)>::new(|event: WheelEvent| {
        event.prevent_default();
        STATE.with_borrow_mut(|state| {
            if let Some(scene) = &mut state.scene {
                // A notch of a mouse's wheel is about a hundred pixels, and one step of the camera. A touchpad sends
                // many small moves, a finger resting on it during a click among them: those move it as little.
                let pixels = event.delta_y() * [1.0, 16.0, 400.0][event.delta_mode().min(2) as usize];
                let step = (pixels * 0.15).clamp(-15.0, 15.0) as f32;
                if pixels.abs() >= 4.0 {
                    let (_, distances) = Camera::limits(scene.indoor);
                    scene.camera.distance = (scene.camera.distance + step).clamp(distances[0], distances[1]);
                    if !scene.indoor {
                        scene.camera.remember();
                    }
                }
            }
        });
    });
    // The right button is the camera's: the browser's menu stays away.
    let menu = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| event.prevent_default());

    let window = web_sys::window();
    let _ = canvas.add_event_listener_with_callback("mousedown", pressed.as_ref().unchecked_ref());
    let _ = canvas.add_event_listener_with_callback("contextmenu", menu.as_ref().unchecked_ref());
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    let _ = canvas.add_event_listener_with_callback_and_add_event_listener_options("wheel", wheel.as_ref().unchecked_ref(), &options);
    if let Some(window) = &window {
        let _ = window.add_event_listener_with_callback("mousemove", moved.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("mouseup", released.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("mousedown", side.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("auxclick", side.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback_and_add_event_listener_options("wheel", page_zoom.as_ref().unchecked_ref(), &options);
    }
    // These live as long as the page.
    pressed.forget();
    side.forget();
    page_zoom.forget();
    moved.forget();
    released.forget();
    wheel.forget();
    menu.forget();
}
