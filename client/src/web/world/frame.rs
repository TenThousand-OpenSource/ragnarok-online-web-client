//! One frame of the world: everyone is moved to where they are now, the effects advance, the picture is drawn,
//! and the page's overlays (names, bars, the pointer) follow.

use super::*;

/// How long one struck down takes to fade once it has fallen: the client's `CCorpse::OnProcess` takes 1 of 255
/// from it every 24 milliseconds.
const CORPSE_MS: f64 = 255.0 * 24.0;

mod effects;
mod marks;
mod sprites;

/// Draws one frame, and asks for the next while there is a world to draw.
pub(super) fn frame() {
    // The effects and sounds whose moment has come.
    let (effects, sounds) = STATE.with_borrow_mut(|state| {
        let time = now();
        let effects: Vec<_> = state.effects_due.extract_if(.., |due| due.0 <= time).collect();
        let sounds: Vec<_> = state.sounds_due.extract_if(.., |due| due.0 <= time).collect();
        (effects, sounds)
    });
    for (_, name, on) in effects {
        effect(&name, on);
    }
    for (_, name) in sounds {
        sound::effect(&name, 1.0);
    }
    let told = STATE.with_borrow_mut(|state| {
        let (Some(gpu), Some(scene)) = (&mut state.gpu, &mut state.scene) else { return None };
        gpu.fit();
        let (time, yaw) = (now(), scene.camera.yaw);
        let own = scene.player.now(time, yaw);
        let feet = on_ground(&scene.gat, own.0);
        if state.camera_fixed {
            scene.aim = None;
        }
        let aim = scene.follow(feet, time);
        let view = scene.camera.view(aim, gpu.config.width as f32 / gpu.config.height as f32);

        sprites::landings(state, time);
        let mut standing = sprites::standing(state, own, time, yaw);
        let (rings, marks) = marks::ground(state, &view, time);
        let numbers = marks::numbers(state, time);
        let mut quads = Quads::default();
        effects::files(state, &view, time, &mut quads);
        let casts = effects::casts(state, time, &mut quads);
        effects::aids(state, &view, time, &mut quads);
        effects::bursts(state, &view, time, &mut quads);
        effects::bolts(state, &view, time, &mut quads);
        effects::arrows(state, &view, time, &mut quads);
        let (Some(gpu), Some(scene)) = (&mut state.gpu, &state.scene) else { return None };
        gpu.draw(scene, &view, &mut standing, &rings, &marks, &numbers, &quads);

        let (width, height) = overlays(state, &view, time, feet, &casts)?;

        let Pursuit { follow, strike, take, used, once } = pursue(state, &view, (width, height), time);
        let scene = state.scene.as_mut()?;

        let cell = scene.player.cell;
        let changed = cell != scene.told;
        scene.told = cell;
        Some((state.hooks.as_ref().filter(|_| changed).map(|hooks| (hooks.moved, cell)), follow, strike, take, used, once))
    });
    if let Some((moved, follow, strike, take, used, once)) = told {
        if let Some((strike_once, id)) = once {
            strike_once(id);
        }
        if let Some((tell, whom, cell)) = used {
            tell(whom, cell);
        }
        if let Some((take, id)) = take {
            take(id);
        }
        if let Some((press, id)) = strike {
            press(id, true);
        }
        if let Some((moved, cell)) = moved {
            moved(cell.0.max(0) as u16, cell.1.max(0) as u16);
        }
        if let Some((walk_to, goal)) = follow {
            walk_to(goal.0 as u16, goal.1 as u16);
        }
    }
    if STATE.with_borrow(|state| state.scene.is_some()) {
        next_frame();
    }
}

pub(super) fn next_frame() {
    STATE.with_borrow_mut(|state| {
        let frame = state.frame.get_or_insert_with(|| Closure::<dyn FnMut()>::new(frame));
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(frame.as_ref().unchecked_ref());
        }
    });
}

/// Shows a name on the page, its middle at a place, or takes it away. Monsters' names are pink, as the original's.
pub(super) fn name_shown(named: Option<(&str, bool, (f32, f32))>) {
    let Some(label) = crate::web::by_id("unit-name") else { return };
    match named {
        Some((name, monster, (x, y))) => {
            if label.text_content().as_deref() != Some(name) {
                label.set_text_content(Some(name));
            }
            let colour = if monster { "#ffc3c3" } else { "#fff" };
            let _ = label.set_attribute("style", &format!("display:block;left:{x:.0}px;top:{y:.0}px;color:{colour}"));
        }
        None => {
            let _ = label.set_attribute("style", "display:none");
        }
    }
}
