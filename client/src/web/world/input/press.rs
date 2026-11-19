//! A button of the mouse goes down on the world: a skill being aimed is used or let go, someone is pressed, the
//! character walks, the camera begins to turn.

use super::*;

/// Where on the canvas the pointer is, each way from 0 to 1.
fn place_on(event: &MouseEvent, canvas: &HtmlCanvasElement) -> (f32, f32) {
    (event.offset_x() as f32 / canvas.client_width().max(1) as f32, event.offset_y() as f32 / canvas.client_height().max(1) as f32)
}

pub(super) fn pressed(event: &MouseEvent, canvas: &HtmlCanvasElement) {
    let at = place_on(event, canvas);
    // The browser does nothing of its own with a press on the world: no text marked, no scrolling by the
    // middle button, no page left by the side buttons.
    event.prevent_default();
    match event.button() {
        0 if STATE.with_borrow(|state| state.aiming.is_some()) => aimed(event),
        // The right button lets go of a skill being aimed.
        2 if STATE.with_borrow(|state| state.aiming.is_some()) => {
            STATE.with_borrow_mut(|state| state.aiming = None);
            aim_shown();
        }
        // With Alt held the right button commands the homunculus and the left one the mercenary, when the
        // player has one out (`ai.rs`).
        2 if event.alt_key() && ai::commanded(event, at, 0) => {}
        0 if event.alt_key() && ai::commanded(event, at, 1) => {}
        0 => left(event, at),
        // The right button uses the ninth slot of the hotkey bar, when the player has it so (the client's
        // Quick Skill 1); the camera is not set back by two quick presses then, as in the client.
        2 if STATE.with_borrow(|state| state.options.quick_right) => {
            let quick = STATE.with_borrow_mut(|state| {
                state.turning = Some((event.client_x() as f32, event.client_y() as f32));
                state.right_at = state.turning;
                state.hooks.as_ref().map(|hooks| hooks.quick)
            });
            if let Some(quick) = quick {
                quick(8);
            }
        }
        2 => STATE.with_borrow_mut(|state| {
            let zoom_out = state.options.zoom_out;
            let pressed = now();
            if pressed - state.right_pressed < 350.0 {
                // Twice in a moment puts the camera back (`ProcessRightButtonDrag`, its double click): with Ctrl
                // its distance, with Shift its tilt (to 50 degrees), with neither its turn; not on a map the
                // client's table gives a view of its own.
                if let Some(scene) = state.scene.as_mut().filter(|scene| scene.scenery.view.is_none()) {
                    let (ctrl, shift, turns) = (event.ctrl_key(), event.shift_key(), !scene.indoor);
                    scene.ask_camera(zoom_out, |camera| {
                        if ctrl {
                            camera.distance = DISTANCE;
                        }
                        if shift {
                            camera.pitch = 50.0;
                        }
                        if turns && !ctrl && !shift {
                            camera.yaw = 0.0;
                        }
                    });
                }
            }
            state.right_pressed = pressed;
            state.turning = Some((event.client_x() as f32, event.client_y() as f32));
            state.right_at = state.turning;
        }),
        _ => {}
    }
}

/// The left button while a skill is aimed.
fn aimed(event: &MouseEvent) {
        // A skill is being aimed: the press picks whom, or where, it is used on. Whatever is
        // pressed, the aiming is over with it, as in the client (a skill for someone pressed on the
        // bare ground is not used).
        let aimed = STATE.with_borrow_mut(|state| {
            let ground = state.aiming.take() == Some(true);
            let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
            let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
            let pointer = (event.client_x() as f32, event.client_y() as f32);
            let aimed = if ground {
                let (width, height) = (gpu.canvas.client_width() as f32, gpu.canvas.client_height() as f32);
                let cell = view.pick(&scene.gat, (pointer.0 / width.max(1.0), pointer.1 / height.max(1.0)))?;
                (None, (cell.0.max(0) as u16, cell.1.max(0) as u16))
            } else {
                let (id, _) = pointed_at(gpu, scene, &view, pointer)?;
                // On a monster or another player only: the client asks for a skill on those (its
                // `CGameMode::RequestPlayerAction`), and a press on an NPC, a pet or the bare ground
                // only ends the aiming. The character walks on as it was.
                let unit = scene.units.get(&id).filter(|unit| unit.lively || matches!(unit.look, Look::Character(_)))?;
                // A skill of help goes on a monster only with Shift held, unless the player has the
                // client's No Shift option on.
                if unit.lively && state.aim_guarded && !event.shift_key() {
                    return None;
                }
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
        aim_shown();
}

/// With Shift held, or sitting, a press on the ground has the character look towards the place and not walk
/// (`poses::looks_at`): whether it was such a press.
fn looks(event: &MouseEvent, at: (f32, f32)) -> bool {
    let looks = STATE.with_borrow_mut(|state| {
        let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
        if !event.shift_key() && !scene.turns_only() {
            return None;
        }
        let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
        let turned = view.pick(&scene.gat, at).and_then(|picked| poses::looks_at(state, picked));
        state.held = None;
        Some((state.hooks.as_ref().map(|hooks| hooks.face), turned))
    });
    if let Some((Some(face), Some((head, direction)))) = looks {
        face(head, direction);
    }
    looks.is_some()
}

/// The left button on the world.
fn left(event: &MouseEvent, at: (f32, f32)) {
        // A press on someone talks to them or fights them; a press on the ground walks there.
        let pressed = STATE.with_borrow(|state| {
            let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
            let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
            // Whoever the last frame drew the pointer on is who is pressed: the pointer already
            // shows it, and the press must not see it otherwise (a picture a moment further in its
            // motion, the camera a step further along), or a press on a monster becomes a walk.
            let id = pointed_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32)).map(|(id, _)| id).or(state.over.map(|(id, _)| id))?;
            // A pet is neither fought nor talked to: a press on it is a press on the ground. So is a press on
            // another player, except on a map where players are fought: there one who is no friend is fought
            // as a monster is (the client's `CGameMode::HandleActorPick`; see `pvp.rs`).
            let foe = scene.units.get(&id).is_some_and(|unit| state.pvp.foe(id, unit));
            let unit = scene.units.get(&id).filter(|unit| foe || (matches!(unit.look, Look::Creature(_)) && !unit.companion))?;
            // A thing lying at an NPC's feet is picked up, not the NPC talked to.
            if !unit.lively && lying_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32)).is_some() {
                return None;
            }
            state.hooks.as_ref().map(|hooks| (hooks.press, id, unit.lively || foe))
        });
        if let Some((press, id, monster)) = pressed {
            // A monster is run after until it is down or the player presses somewhere else.
            // The blow is asked for once the character is near enough for its weapon, not before
            // (the client's `CPlayer::UpdatePendingAction`): the frame's code sees to it at once.
            // With Ctrl, or with the No Ctrl option, the monster is fought until it is down; else only
            // while the button is held (the client's `forcedAttack` and `AM_ATTACKCLICKUP`).
            let kept = event.ctrl_key();
            STATE.with_borrow_mut(|state| {
                state.chase = monster.then_some((id, 0.0, false));
                state.fight = u8::from(!(kept || state.options.no_ctrl));
                state.reach = None;
            });
            if !monster {
                press(id, monster);
            }
            return;
        }
        // A press on a thing lying on the ground goes to it and picks it up.
        let lying = STATE.with_borrow(|state| {
            let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
            let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
            lying_at(gpu, scene, &view, (event.client_x() as f32, event.client_y() as f32))
        });
        if let Some(id) = lying {
            STATE.with_borrow_mut(|state| {
                state.chase = None;
                state.fetch = Some((id, 0.0));
            });
            ready_to_pick_up();
            return;
        }
        STATE.with_borrow_mut(|state| {
            state.fetch = None;
            state.chase = None;
            state.reach = None;
            state.journey = None;
            state.held = Some((event.client_x() as f32, event.client_y() as f32));
            state.held_since = now();
            state.held_asked = now();
        });
        if looks(event, at) {
            return;
        }
        let request = STATE.with_borrow(|state| {
            let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return None };
            let player = &scene.player;
            // Seen as the last frame drew it: the place pressed is the place the player saw.
            let view = scene.camera.view(scene.looked_at(), gpu.config.width as f32 / gpu.config.height as f32);
            let picked = view.pick(&scene.gat, at)?;
            // A press on a wall, or too far away, walks as far towards it as the way is open.
            let goal = scene.goal_towards(player.cell, picked, state.options.far_walk)?;
            state.hooks.as_ref().map(|hooks| (hooks.walk_to, goal, picked))
        });
        if let Some((walk_to, goal, picked)) = request {
            // A press that comes too soon after the last walk asked for asks for none, as in the client
            // (`State::asks_walk`); the button held asks again by itself.
            let asks = STATE.with_borrow_mut(|state| {
                let asks = state.asks_walk(goal, now());
                // With the project's far walk switched on, a place farther than the server walks in one go
                // is walked to in stretches (`pursuit::journey`).
                state.journey = (asks && goal != picked && state.options.far_walk).then_some((picked, goal));
                asks
            });
            if asks {
                walk_to(goal.0 as u16, goal.1 as u16);
            }
        }
}
