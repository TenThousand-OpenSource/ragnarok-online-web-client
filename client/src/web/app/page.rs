//! The page itself: its elements and form fields, what the browser keeps between visits, the fade and the loading
//! screen, windows dragged about.

use super::*;

/// Now, in seconds since 1970.
pub(in super::super) fn epoch() -> f64 {
    js_sys::Date::now() / 1000.0
}

thread_local! {
    /// How far each of the game's windows was dragged from its place, by its element's ID; and the drag under
    /// way: the window, where the pointer was, and how far the window had been moved then.
    static MOVED: RefCell<std::collections::HashMap<String, (f64, f64)>> = RefCell::new(std::collections::HashMap::new());
    static DRAG: RefCell<Option<(String, (f64, f64), (f64, f64))>> = const { RefCell::new(None) };
}

/// Puts every window that was dragged where it was dragged to: a page drawn anew has them at their own places.
pub(super) fn place_moved() {
    MOVED.with_borrow(|moved| {
        for (id, (x, y)) in moved {
            if let Some(window) = element(id).and_then(|window| window.dyn_into::<web_sys::HtmlElement>().ok()) {
                let _ = window.style().set_property("transform", &format!("translate({x}px,{y}px)"));
            }
        }
    });
}

/// A window that opens anew at a place of its own is no longer where its element was last dragged to.
pub(super) fn forget_moved(id: &str) {
    MOVED.with_borrow_mut(|moved| moved.remove(id));
}

/// A window that was just put on the page under the pointer is held by it from there on, as if it had been
/// pressed: a tab of the chat torn out into a window of its own. `from` is where the pointer is.
pub(super) fn drag_begun(id: &str, from: (f64, f64)) {
    forget_moved(id);
    DRAG.set(Some((id.to_owned(), from, (0.0, 0.0))));
}

/// Gives the player a file to keep: the browser saves what is at that address under that name.
pub(super) fn give_file(name: &str, address: &str) {
    let Some(link) = crate::web::document().and_then(|document| document.create_element("a").ok()) else { return };
    let _ = link.set_attribute("href", address);
    let _ = link.set_attribute("download", name);
    if let Ok(link) = link.dyn_into::<web_sys::HtmlElement>() {
        link.click();
    }
}

thread_local! {
    /// Whether a window being moved docks to what is near it (the player's `/window`).
    static SNAP: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Sets whether a window being moved docks to what is near it.
pub(super) fn snap_windows(on: bool) {
    SNAP.set(on);
}

/// How far the window being moved is to go, on the screen, to dock (`crate::snap`): to the edges of the
/// screen and to the other windows that show. None when it docks to nothing.
fn docked(id: &str, scale: f64) -> Option<(f64, f64)> {
    let rect = |window: &Element| {
        let rect = window.get_bounding_client_rect();
        [rect.left(), rect.top(), rect.right(), rect.bottom()]
    };
    let moved = rect(&element(id)?);
    let windows = crate::web::document()?.query_selector_all(".hud:not([hidden])").ok()?;
    let others: Vec<crate::snap::Rect> = (0..windows.length())
        .filter_map(|at| windows.item(at)?.dyn_into::<Element>().ok())
        .filter(|window| window.id() != id)
        .map(|window| rect(&window))
        .filter(|rect| rect[2] > rect[0] && rect[3] > rect[1])
        .collect();
    let by = crate::snap::shift(moved, &others, crate::web::page_size(), crate::snap::NEAR * scale);
    (by != (0.0, 0.0)).then_some(by)
}

/// The windows of the game are dragged by their title bars (the top strip of each), as the original's are.
pub(super) fn drag_windows(document: &web_sys::Document) -> Result<(), String> {
    let on_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        if event.button() != 0 {
            return;
        }
        let Some(target) = event.target().and_then(|target| target.dyn_into::<Element>().ok()) else { return };
        // A window pressed anywhere comes over the others.
        if let Some(window) = target.closest(".hud").ok().flatten() {
            dispatch(Action::WindowPressed(window.id()));
        }
        // Buttons, fields and the things in the windows keep their own presses.
        if target.closest("button, input, select, [data-click], [data-use], [data-drag], .scroll").ok().flatten().is_some() {
            return;
        }
        let Some(window) = target.closest(".hud").ok().flatten().filter(|window| !window.id().is_empty()) else { return };
        // Only the top strip moves a window; the chat and the windows marked so (`data-whole`: a tab of the
        // chat in a window of its own, a party member's small window) move by all of them, unless their lock
        // is closed.
        let whole = window.id() == "chat" || window.has_attribute("data-whole");
        if whole && window.get_attribute("data-locked").as_deref() == Some("1") {
            return;
        }
        if !whole && f64::from(event.client_y()) - window.get_bounding_client_rect().top() > 20.0 {
            return;
        }
        let id = window.id();
        let was = MOVED.with_borrow(|moved| moved.get(&id).copied().unwrap_or((0.0, 0.0)));
        DRAG.set(Some((id, (f64::from(event.client_x()), f64::from(event.client_y())), was)));
        event.prevent_default();
    });
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        let Some((id, from, was)) = DRAG.with_borrow(Clone::clone) else { return };
        if event.buttons() & 1 == 0 {
            return DRAG.set(None);
        }
        // The stage may be enlarged: the window moves as far as the pointer does on the screen.
        let scale = crate::web::stage_scale();
        let to = (was.0 + (f64::from(event.client_x()) - from.0) / scale, was.1 + (f64::from(event.client_y()) - from.1) / scale);
        MOVED.with_borrow_mut(|moved| moved.insert(id.clone(), to));
        place_moved();
        // With the player's `/window` on, a window docks to the screen's edges and to the other windows.
        // (Not one the client keeps out of the docking: `data-apart`, a party member's small window.)
        if SNAP.get() && !element(&id).is_some_and(|window| window.has_attribute("data-apart")) {
            if let Some(by) = docked(&id, scale) {
                MOVED.with_borrow_mut(|moved| moved.insert(id.clone(), (to.0 + by.0 / scale, to.1 + by.1 / scale)));
                place_moved();
            }
        }
        let to = MOVED.with_borrow(|moved| moved.get(&id).copied().unwrap_or(to));
        // No window leaves the screen: one pushed past an edge is put back against it.
        if let Some(window) = element(&id) {
            let (wide, tall) = crate::web::page_size();
            let rect = window.get_bounding_client_rect();
            let back = |low: f64, high: f64, most: f64| if low < 0.0 { -low } else if high > most { (most - high).max(-low) } else { 0.0 };
            let (dx, dy) = (back(rect.left(), rect.right(), wide), back(rect.top(), rect.bottom(), tall));
            if dx != 0.0 || dy != 0.0 {
                MOVED.with_borrow_mut(|moved| moved.insert(id, (to.0 + dx / scale, to.1 + dy / scale)));
                place_moved();
            }
        }
    });
    // A window let go is told to the app: some do something where they are let go (`App::window_dropped`).
    let on_up = Closure::<dyn FnMut()>::new(|| {
        if let Some((id, ..)) = DRAG.take() {
            dispatch(Action::WindowDropped(id));
        }
    });
    document.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref()).map_err(js_error)?;
    on_down.forget();
    on_move.forget();
    on_up.forget();
    Ok(())
}

pub(super) fn fade(dark: bool) {
    let Some(cover) = element("fade").and_then(|cover| cover.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
    let style = cover.style();
    if !dark {
        // It comes back from full dark, whatever it was.
        let _ = style.set_property("transition", "none");
        let _ = style.set_property("opacity", "1");
        let _ = cover.offset_width();
    }
    let _ = style.set_property("transition", &format!("opacity {FADE_MS}ms linear"));
    let _ = style.set_property("opacity", if dark { "1" } else { "0" });
}

/// Runs an action once the screen has gone dark.
pub(super) fn after_fade(action: Action) {
    let run = Closure::once_into_js(move || dispatch(action));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(run.unchecked_ref(), FADE_MS);
    }
}

/// The address of one of the loading screen's pictures: they are taken in turn.
fn loading_picture(picture: usize) -> String {
    crate::web::parts::picture(&format!("loading{:02}.jpg", picture % LOADING_PICTURES))
}

/// Shows the client's loading screen with one of its pictures (`loading00.jpg` and on, in turn), or takes it away.
pub(super) fn loading(picture: Option<usize>) {
    let Some(screen) = element("loading").and_then(|screen| screen.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
    let style = screen.style();
    match picture {
        Some(picture) => {
            let scale = crate::web::stage_scale();
            let _ = style.set_property("--s", &scale.to_string());
            let _ = style.set_property("background-image", &format!("url('{}')", loading_picture(picture)));
            let _ = style.set_property("display", "block");
        }
        None => {
            let _ = style.set_property("display", "none");
        }
    }
}

/// Has the browser fetch a loading picture ahead of its screen, so that it is there when the screen shows.
pub(super) fn loading_ahead(picture: usize) {
    if let Ok(image) = web_sys::HtmlImageElement::new() {
        image.set_src(&loading_picture(picture));
        // The browser keeps what it fetched; the element itself is not needed.
        std::mem::forget(image);
    }
}

pub(super) fn ui() -> Result<Element, String> {
    element("ui").ok_or_else(|| "no #ui element".to_owned())
}

pub(super) fn element(id: &str) -> Option<Element> {
    crate::web::by_id(id)
}

/// What is typed into (or picked in) the form field with that ID.
/// What was typed into a field of the account's forms, without the spaces around it: a name, a password or an
/// address has none of its own (the rules for each forbid them), so one before or after is a slip of the hand
/// or of a paste, and is not held against the player.
pub(super) fn typed(id: &str) -> String {
    field(id).trim().to_owned()
}

pub(super) fn field(id: &str) -> String {
    let Some(element) = element(id) else { return String::new() };
    match element.dyn_into::<HtmlInputElement>() {
        Ok(input) => input.value(),
        Err(element) => element.dyn_into::<HtmlSelectElement>().map(|select| select.value()).unwrap_or_default(),
    }
}

/// What stands in a field of the page now (of one line or of many), if the field is there: for a window that
/// is drawn anew while something is typed into it, and takes what was typed from the page it replaces.
pub(in crate::web) fn written(id: &str) -> Option<String> {
    js_sys::Reflect::get(element(id)?.as_ref(), &"value".into()).ok()?.as_string()
}

pub(super) fn set_field(id: &str, value: &str) {
    match element(id).map(|element| element.dyn_into::<HtmlInputElement>()) {
        Some(Ok(input)) => input.set_value(value),
        Some(Err(element)) => {
            if let Ok(select) = element.dyn_into::<HtmlSelectElement>() {
                select.set_value(value);
            }
        }
        None => {}
    }
}

/// Whether a key pressed in the game is one the browser would do something of its own with, in the game's way:
/// the keys the game uses or will use (function keys, Alt and a letter, Tab, the keys that scroll), and the
/// browser's page commands. Typing, copying and reloading stay the browser's.
pub(super) fn browsers_own(event: &KeyboardEvent) -> bool {
    let (key, code) = (event.key(), event.code());
    let writing = event.target().and_then(|target| target.dyn_into::<web_sys::Element>().ok()).is_some_and(
        |target| matches!(target.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
    );
    // F11 is the full screen, which suits the game.
    if code.starts_with('F') && code.len() > 1 && code[1..].parse::<u8>().is_ok_and(|number| number != 11) {
        return true;
    }
    if event.alt_key() && !event.ctrl_key() && !event.meta_key() {
        return true;
    }
    if event.ctrl_key() || event.meta_key() {
        return matches!(code.as_str(), "KeyS" | "KeyF" | "KeyG" | "KeyP" | "KeyD" | "KeyO" | "KeyE" | "KeyU" | "KeyH" | "KeyJ" | "KeyL");
    }
    key == "Tab" || (!writing && matches!(
        key.as_str(),
        " " | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "PageUp" | "PageDown" | "Home" | "End" | "Backspace" | "/" | "'"
    ))
}

/// What this browser remembers under a name. Storage may be switched off: then nothing is remembered.
pub(super) fn stored(name: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(name).ok()?
}

pub(super) fn store(name: &str, value: Option<&str>) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|window| window.local_storage()) {
        let _ = match value {
            Some(value) => storage.set_item(name, value),
            None => storage.remove_item(name),
        };
    }
}

/// The client's clock, in milliseconds, as the servers count it: the page's (`web::now`) in whole ones.
pub(in super::super) fn now() -> u32 {
    crate::web::now() as u32
}

/// Text made safe to put into HTML: names come from the server, and so from other players.
pub(in super::super) fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}
