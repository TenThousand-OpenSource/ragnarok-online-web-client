//! The picture of the mouse pointer over the world: which of the game's pointers it is by what it is over, and
//! which of that pointer's pictures shows now (`crate::cursor`); and the reading of the game's pointers, once,
//! for the world and for everything pressable on the page.

use super::*;

/// The game's pointers, as actions of its pointer sprite: the plain one, over someone to talk to, over a monster,
/// over a portal, over a thing to pick up, while the camera is turned, while a skill is aimed, while a skill
/// is aimed at someone it can be used on, and over ground nobody can walk on.
const POINTERS: [usize; 9] = [0, 1, 5, 7, 9, 4, 10, 11, 13];

/// Picks the pointer for what the mouse is over (`over`: whoever it is on) and puts its picture on the canvas.
pub(super) fn shown(state: &mut State, view: &View, time: f64, over: Option<u32>, (width, height): (f32, f32)) {
    let (Some(gpu), Some(scene)) = (&state.gpu, &state.scene) else { return };
    let unit = over.and_then(|id| scene.units.get(&id));
    // Another player who is fought on this map has a monster's pointers: the sword, and the aimed skill's
    // other picture (the client's cursor actions 5 and 11 in `CGameMode::HandleActorPick`).
    let foe = over.zip(unit).is_some_and(|(id, unit)| state.pvp.foe(id, unit));
    let lying = || state.pointer.is_some_and(|pointer| lying_at(gpu, scene, view, pointer).is_some());
    // A portal is pointed at anywhere over its rings.
    let near = DISTANCE / scene.camera.distance * height / 480.0;
    let on_portal = state.pointer.is_some_and(|pointer| {
        scene.portals.values().any(|cell| {
            view.project(on_ground(&scene.gat, (cell.0 as f32, cell.1 as f32))).is_some_and(|middle| {
                let (across, up) = ((pointer.0 - middle.0 * width).abs(), middle.1 * height - pointer.1);
                across < 45.0 * near && (-25.0 * near..45.0 * near).contains(&up)
            })
        })
    });
    // Which of `POINTERS` it is.
    let pointing = match unit {
        _ if state.turning.is_some() => 5,
        // A skill being aimed: the pointer of a skill, and its other picture over a monster it can go to
        // (the client's cursor actions 10 and 11).
        // (`CGameMode::HandleActorPick`: 11 over a monster or a player the skill would go on as Shift stands.)
        Some(unit) if state.aiming == Some(false) && over.is_some_and(|id| state.aim_goes_on(id, unit)) => 7,
        _ if state.aiming.is_some() => 6,
        // Over a pet the pointer stays plain, as in the client.
        Some(unit) if unit.companion => 0,
        Some(unit) if unit.lively || foe => 2,
        // A thing lying at an NPC's feet is still picked up.
        _ if lying() => 4,
        Some(unit) if unit.look.creature() => 1,
        None if on_portal => 3,
        // Over ground nobody can walk on, or off the map (the client's action 13, when nothing else is pointed at).
        None if state.pointer.is_some_and(
            |pointer| view.pick(&scene.gat, (pointer.0 / width.max(1.0), pointer.1 / height.max(1.0))).is_none_or(
                |cell| !scene.gat.walkable(cell.0, cell.1)
            )
        ) => 8,
        _ => 0,
    };
    if pointing != state.pointing {
        state.pointing_since = time;
    }
    let pictures = state.pointers.get(pointing).map_or(0, |(pictures, _)| pictures.len());
    let pointer_frame = crate::cursor::frame(
        POINTERS.get(pointing).copied().unwrap_or(0),
        pictures,
        time - state.pointing_since,
        state.held.map(|_| time - state.held_since)
    );
    if pointing != state.pointing || pointer_frame != state.pointer_frame {
        state.pointing = pointing;
        state.pointer_frame = pointer_frame;
        if let Some(pointer) = state.pointers.get(pointing).and_then(|(pictures, _)| pictures.get(pointer_frame)) {
            let _ = gpu.canvas.style().set_property("cursor", pointer);
        }
    }
}

/// Over what can be pressed in the windows the pointer is the client's finger, and while the button is held on
/// it the finger goes down picture by picture (`crate::cursor::frame` for action 2) and stays down: the page's
/// `--pressed` is the picture of the moment, from the button going down.
fn finger_goes_down(finger: Vec<String>) {
    let Some(document) = crate::web::document() else { return };
    let pressed_at = Rc::new(std::cell::Cell::new(None::<f64>));
    let step = {
        let pressed_at = pressed_at.clone();
        Closure::<dyn FnMut()>::new(move || {
            let Some(since) = pressed_at.get() else { return };
            let Some(root) = crate::web::document().and_then(|document| document.document_element()).and_then(
                |root| root.dyn_into::<web_sys::HtmlElement>().ok()
            ) else { return };
            if let Some(picture) = finger.get(crate::cursor::frame(2, finger.len(), 0.0, Some(now() - since))) {
                let _ = root.style().set_property("--pressed", picture);
            }
        })
    };
    let down = {
        let pressed_at = pressed_at.clone();
        Closure::<dyn FnMut()>::new(move || pressed_at.set(Some(now())))
    };
    let up = Closure::<dyn FnMut()>::new(move || pressed_at.set(None));
    let _ = document.add_event_listener_with_callback("pointerdown", down.as_ref().unchecked_ref());
    let _ = document.add_event_listener_with_callback("pointerup", up.as_ref().unchecked_ref());
    // A tick of the client's pointer is 24 milliseconds.
    if let Some(window) = web_sys::window() {
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(step.as_ref().unchecked_ref(), 24);
    }
    // These live as long as the page.
    step.forget();
    down.forget();
    up.forget();
}

/// Reads the game's own mouse pointers, once.
pub(in crate::web::world) fn pointers() {
    if STATE.with_borrow(|state| !state.pointers.is_empty()) {
        return;
    }
    wasm_bindgen_futures::spawn_local(async {
        let mut read = Vec::new();
        for action in POINTERS {
            // The first picture tells how many there are.
            let Ok(first) = figure::cursor(action, 0).await else { return };
            let (count, shows) = figure::timing("cursors", action).unwrap_or((1, 100.0));
            let mut pictures = vec![first];
            for frame in 1..count.min(32) {
                match figure::cursor(action, frame).await {
                    Ok(picture) => pictures.push(picture),
                    Err(_) => break,
                }
            }
            read.push((pictures, shows.max(40.0)));
        }
        // Everything pressable on the page has the game's pointers too: the plain one, and the pressing one.
        if let (Some(root), Some(plain), Some(press)) = (
            crate::web::document().and_then(|document| document.document_element()).and_then(|root| root.dyn_into::<web_sys::HtmlElement>().ok()),
            read.first().and_then(|(pictures, _)| pictures.first()),
            figure::cursor(2, 0).await.ok(),
        ) {
            let _ = root.style().set_property("--point", plain);
            let _ = root.style().set_property("--press", &press);
            // The pointer of a skill being aimed (the client's cursor action 10), for over the windows.
            if let Ok(aim) = figure::cursor(10, 0).await {
                let _ = root.style().set_property("--aim", &aim);
            }
            // The finger down: the pictures of that action, which the client runs through once while the
            // button is held (`finger_goes_down`).
            let mut finger = vec![press];
            for frame in 1..figure::timing("cursors", 2).map_or(3, |(pictures, _)| pictures).min(16) {
                match figure::cursor(2, frame).await {
                    Ok(picture) => finger.push(picture),
                    Err(_) => break,
                }
            }
            if let Some(down) = finger.last() {
                let _ = root.style().set_property("--pressed", down);
            }
            finger_goes_down(finger);
        }
        // Every picture of every pointer is read by the browser ahead of its use and kept on the page, out of
        // sight: a pointer whose picture the browser has yet to read shows as the plain arrow for a moment, which
        // made the pointer flicker the first time it ran over a monster.
        if let Some(document) = crate::web::document() {
            if let (Ok(keep), Some(body)) = (document.create_element("div"), document.body()) {
                let _ = keep.set_attribute("style", "position:fixed;left:-99px;top:-99px;width:1px;height:1px;overflow:hidden;pointer-events:none");
                for picture in read.iter().flat_map(|(pictures, _)| pictures) {
                    let Some(url) = picture.strip_prefix("url(").and_then(|rest| rest.split(')').next()) else { continue };
                    if let Ok(image) = web_sys::HtmlImageElement::new() {
                        image.set_src(url);
                        let _ = wasm_bindgen_futures::JsFuture::from(image.decode()).await;
                        let _ = keep.append_child(&image);
                    }
                }
                let _ = body.append_child(&keep);
            }
        }
        STATE.with_borrow_mut(|state| {
            state.pointers = read;
            // Shown with the next frame.
            state.pointing = usize::MAX;
        });
    });
}
