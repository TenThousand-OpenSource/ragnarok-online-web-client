//! The mouse over the world: walking, pressing someone, aiming a skill, turning and zooming the camera.

use super::*;

mod pointer;
mod press;
mod wheel;

/// The mouse on the canvas: a left press walks, the right button turns the camera, the wheel moves it.
pub(super) fn listen() {
    if STATE.with_borrow_mut(|state| std::mem::replace(&mut state.listening, true)) {
        return;
    }
    let Ok(canvas) = canvas() else { return };

    let pressed = {
        let canvas = canvas.clone();
        Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| press::pressed(&event, &canvas))
    };
    let moved = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| pointer::moved(&event));
    let released = Closure::<dyn FnMut(MouseEvent)>::new(|event: MouseEvent| pointer::released(&event));
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
    let wheel = Closure::<dyn FnMut(WheelEvent)>::new(|event: WheelEvent| wheel::turned(&event));
    // Shift going down or up changes whom the pointer picks and which pointer shows, with the mouse still.
    let shifted = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(|event: web_sys::KeyboardEvent| {
        STATE.with_borrow_mut(|state| state.picking.shift = event.shift_key());
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
        let _ = window.add_event_listener_with_callback("keydown", shifted.as_ref().unchecked_ref());
        let _ = window.add_event_listener_with_callback("keyup", shifted.as_ref().unchecked_ref());
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
    shifted.forget();
    menu.forget();
}

/// Writes the skill being aimed and its level by the pointer, or takes that away when nothing is aimed. The
/// pointer of a skill and these words are the client's mouse pointer itself, drawn over everything: they show
/// over the windows too (a skill aimed from the skill window is aimed with that window still open), which the
/// page does for a root marked `data-aiming`.
pub(super) fn aim_shown() {
    let aiming = STATE.with_borrow(|state| state.aiming.is_some());
    if let Some(root) = crate::web::document().and_then(|document| document.document_element()) {
        let _ = if aiming { root.set_attribute("data-aiming", "") } else { root.remove_attribute("data-aiming") };
    }
    let Some(label) = crate::web::by_id("aim") else { return };
    let shown = STATE.with_borrow(|state| state.aiming.and(state.mouse).filter(|_| !state.aim_words.is_empty()).map(|at| (at, state.aim_words.clone())));
    match shown {
        Some((at, words)) => {
            label.set_text_content(Some(&words));
            let _ = label.set_attribute("style", &format!("display:block;left:{:.0}px;top:{:.0}px", at.0 + 18.0, at.1 + 18.0));
        }
        None => {
            let _ = label.set_attribute("style", "display:none");
        }
    }
}
