//! Entering and leaving a map, and what the server says of the player's own character.

use super::*;

/// Enters a map: loads its ground and puts the character on it. `hooks.ready` is called when it shows.
pub fn enter(map: String, cell: (u16, u16), figure: Figure, hooks: Hooks) {
    let entered = STATE.with_borrow_mut(|state| {
        state.scene = None;
        state.early.clear();
        state.reading = Some(map.clone());
        state.start = None;
        state.hooks = Some(hooks);
        state.entered += 1;
        state.entered
    });
    if let Ok(canvas) = canvas() {
        let _ = canvas.style().set_property("display", "block");
    }
    wasm_bindgen_futures::spawn_local(async move {
        let loaded = load(&map, cell, figure).await;
        let ready = STATE.with_borrow_mut(|state| {
            if state.entered != entered {
                return None;
            }
            let ready = state.hooks.as_ref().map(|hooks| hooks.ready);
            match loaded {
                Ok((gpu, mut scene)) => {
                    if let Some(cell) = state.start.take() {
                        (scene.player.cell, scene.player.at) = (cell, (cell.0 as f32, cell.1 as f32));
                    }
                    state.reading = None;
                    state.gpu = Some(gpu);
                    state.scene = Some(scene);
                    ready.map(|ready| (ready, Ok(())))
                }
                Err(error) => ready.map(|ready| (ready, Err(error))),
            }
        });
        if let Some((ready, outcome)) = ready {
            let drawn = outcome.is_ok();
            ready(outcome);
            if drawn {
                for unit in STATE.with_borrow_mut(|state| std::mem::take(&mut state.early)) {
                    unit_shown(&unit);
                }
                listen();
                next_frame();
            }
        }
    });
}

/// Leaves the world: the canvas is hidden and nothing is drawn any more.
pub fn leave() {
    STATE.with_borrow_mut(|state| {
        state.scene = None;
        state.reading = None;
        state.entered += 1;
        state.turning = None;
        state.own = [0; 4];
        state.bars = false;
        state.named = None;
        state.pointer = None;
        state.held = None;
        state.chase = None;
        state.fetch = None;
        state.hits.clear();
        state.flying.clear();
        state.arrows.clear();
        state.bolts.clear();
        state.effects_due.clear();
        state.sounds_due.clear();
        state.effects.clear();
        state.landing.clear();
        state.bursts.clear();
    });
    // What the game drew over the page goes with it: the bars, the names, the numbers of blows.
    if let Some(document) = crate::web::document() {
        if let Some(bars) = document.get_element_by_id("bars") {
            bars.set_inner_html("");
        }
        if let Ok(numbers) = document.query_selector_all(".hit") {
            for index in 0..numbers.length() {
                if let Some(number) = numbers.item(index).and_then(|number| number.dyn_into::<web_sys::Element>().ok()) {
                    number.remove();
                }
            }
        }
    }
    name_shown(None);
    if let Ok(canvas) = canvas() {
        let _ = canvas.style().set_property("display", "none");
    }
}

/// The server says the character walks from one cell to another.
pub fn walk(from: (u16, u16), to: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let Some(scene) = &mut state.scene else { return };
        scene.player.walk(&scene.gat, (i32::from(from.0), i32::from(from.1)), (i32::from(to.0), i32::from(to.1)));
    });
}

/// The weapon the player's own character holds, as its item ID (0 for none): it is drawn in hand.
pub fn own_weapon(weapon: u32) {
    STATE.with_borrow_mut(|state| {
        // Another weapon reaches another way: the server says how far itself (`combat::weapon_reaches`).
        state.weapon = weapon;
        if let Some(Look::Character(figure)) = state.scene.as_mut().map(|scene| &mut scene.player.look) {
            figure.weapon = weapon;
        }
    });
}

/// The player's own life and spirit, left and most.
pub fn own_life(hp: i64, most_hp: i64, sp: i64, most_sp: i64) {
    STATE.with_borrow_mut(|state| state.own = [hp, most_hp, sp, most_sp]);
}

/// The player's own character dies, or is alive again.
pub fn own_dead(dead: bool) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            scene.player.dead = dead;
            scene.player.walk = None;
            scene.player.motion = None;
        }
        state.chase = None;
        state.fetch = None;
        state.held = None;
    });
}

/// The character is moved to another place of the map it is on (a Fly Wing, a warp within the map): the map
/// stays, and everyone seen at the old place is forgotten: the server shows who is at the new one.
pub fn jump(cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        let cell = (i32::from(cell.0), i32::from(cell.1));
        let Some(scene) = &mut state.scene else {
            // The map is still being read: the character will stand there when it shows. Whoever the server
            // has shown meanwhile stays: it shows them once.
            state.start = Some(cell);
            return;
        };
        // Right after the character comes in the server names the cell it already stands on: nobody is
        // forgotten then, for the server has shown them (a pet among them) and will not show them again.
        if scene.player.cell == cell && scene.player.walk.is_none() {
            return;
        }
        scene.player.cell = cell;
        scene.player.at = (cell.0 as f32, cell.1 as f32);
        scene.player.walk = None;
        scene.player.motion = None;
        scene.player.dead = false;
        scene.aim = None;
        scene.units.clear();
        scene.drops.clear();
        scene.falls.clear();
        scene.portals.clear();
        state.chase = None;
        state.fetch = None;
        state.reach = None;
        state.held = None;
        state.target_at = [None; 2];
        state.flying.clear();
        state.arrows.clear();
        state.bolts.clear();
        state.effects_due.clear();
        state.sounds_due.clear();
        state.effects.clear();
        state.landing.clear();
        state.bursts.clear();
    });
}

/// The server says the character stands on a cell.
pub fn stand(cell: (u16, u16)) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = &mut state.scene {
            scene.player.walk = None;
            scene.player.cell = (i32::from(cell.0), i32::from(cell.1));
        }
    });
}

/// The player's options the world goes by, as they are now.
pub fn options(options: Options) {
    STATE.with_borrow_mut(|state| {
        state.options = options;
        // A camera farther out than is now allowed comes in.
        if let Some(scene) = &mut state.scene {
            let (_, distances) = Camera::limits(scene.indoor, options.zoom_out);
            scene.camera.distance = scene.camera.distance.clamp(distances[0], distances[1]);
        }
    });
}

/// The player's own character turns an eighth of the way round where it stands, clockwise or the other way
/// (the client's `/bangbang` and `/bingbing`): which way it then faces, as the server counts directions (0
/// north, turning left). None while it walks or there is no map.
pub fn turn(clockwise: bool) -> Option<u8> {
    STATE.with_borrow_mut(|state| {
        let player = &mut state.scene.as_mut()?.player;
        if player.walk.is_some() {
            return None;
        }
        // The sprites count from the south, turning the other way.
        player.facing = (player.facing + if clockwise { 1 } else { 7 }) % 8;
        Some((12 - player.facing) % 8)
    })
}

/// Whether the camera stays fixed on the character (the client's `TT_FIXED_CAMERA_ON_OFF`, which its
/// `CameraState::Smooth` asks) or follows it a little at a time.
pub fn camera_fixed(fixed: bool) {
    STATE.with_borrow_mut(|state| state.camera_fixed = fixed);
}

/// The character's walking speed, in milliseconds per cell.
pub fn speed(speed: i32) {
    STATE.with_borrow_mut(|state| {
        if let Some(scene) = state.scene.as_mut().filter(|_| speed > 0) {
            scene.player.speed = f64::from(speed);
        }
    });
}

/// The size, in cells, of the map shown.
/// Whether this map is the one being read right now.
pub fn reading(map: &str) -> bool {
    STATE.with_borrow(|state| state.scene.is_none() && state.reading.as_deref() == Some(map))
}

pub fn map_size() -> Option<(usize, usize)> {
    STATE.with_borrow(|state| state.scene.as_ref().map(|scene| (scene.gat.width, scene.gat.height)))
}
