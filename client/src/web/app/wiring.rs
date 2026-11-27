//! Starting the client: the page's events (clicks, keys, drags, timers) are turned into actions for the app.

use super::*;

pub fn start() -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let ui = ui()?;
    ui.set_attribute("style", &format!("background-image:url('{}')", asset_url(WALLPAPER))).map_err(js_error)?;
    // The things in the windows are dragged: onto the hotkey bar, into another window, out onto the world.
    dragging::wire(&window, &ui)?;
    // The windows themselves are dragged by their title bars.
    drag_windows(&document)?;
    // The chat's tabs are pulled out of the window, and their own windows made larger by their corners.
    chat::drag_chat_tabs(&document)?;
    tooltip::wire()?;
    wire_presses(&ui)?;
    wire_wheel(&ui)?;
    scrollbars::wire(&ui, &document)?;
    wire_keys(&document)?;
    wire_page(&window, &document)?;
    restore(&window);
    check_music_soon();
    load_tables();
    Ok(())
}

/// What was pressed, if it or something around it has the attribute (or is a `button`).
fn pressed(event: &Event, attribute: &str) -> Option<Element> {
    let target = event.target().and_then(|target| target.dyn_into::<Element>().ok())?;
    let selector = if attribute == "button" { attribute.to_owned() } else { format!("[{attribute}]") };
    target.closest(&selector).ok().flatten()
}

/// A gesture of the pointer followed over the whole page, wherever it goes while it lasts: what is done when
/// a button goes down, when the pointer moves and when a button comes up. The listeners live as long as the
/// page.
pub(super) fn wire_gesture(
    document: &web_sys::Document,
    on_down: Closure<dyn FnMut(web_sys::MouseEvent)>,
    on_move: Closure<dyn FnMut(web_sys::MouseEvent)>,
    on_up: Closure<dyn FnMut()>,
) -> Result<(), String> {
    document.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref()).map_err(js_error)?;
    on_down.forget();
    on_move.forget();
    on_up.forget();
    Ok(())
}

/// Moves a list or a text that moves by steps (`parts::stepped`) so many notches, down for more than none:
/// by its own step and so many steps a notch, never past its ends (`crate::scrolling::moved`). The wheel
/// does it, and the arrows up and down for what an NPC said.
pub(super) fn step_list(list: &Element, notches: i32) {
    let number = |name: &str| list.get_attribute(name).and_then(|number| number.parse::<i32>().ok()).unwrap_or(0);
    let most = list.scroll_height() - list.client_height();
    list.set_scroll_top(crate::scrolling::moved(list.scroll_top(), number("data-step"), notches * number("data-notch"), most));
}

/// The wheel over a list or a text that moves by steps (`parts::stepped`): it is moved as the client moves
/// its own, by whole rows or lines and so many a notch (`crate::scrolling`), and the page does not scroll it
/// by pixels.
fn wire_wheel(ui: &Element) -> Result<(), String> {
    thread_local! {
        /// What is left of the wheel's turns that were less than a notch (a touchpad's).
        static TURNED: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
    }
    let on_wheel = Closure::<dyn FnMut(web_sys::WheelEvent)>::new(|event: web_sys::WheelEvent| {
        let Some(list) = pressed(&event, "data-step") else { return };
        event.prevent_default();
        // A wheel that counts in lines or pages gives a notch a turn.
        let turned = if event.delta_mode() == 0 { event.delta_y() } else { event.delta_y().signum() * crate::scrolling::NOTCH };
        let (whole, left) = crate::scrolling::notches(TURNED.get(), turned);
        TURNED.set(left);
        if whole != 0 {
            step_list(&list, whole);
        }
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_passive(false);
    ui.add_event_listener_with_callback_and_add_event_listener_options("wheel", on_wheel.as_ref().unchecked_ref(), &options).map_err(js_error)?;
    on_wheel.forget();
    Ok(())
}

/// The presses on the page: forms sent, clicks, double clicks, the right button and sliders.
fn wire_presses(ui: &Element) -> Result<(), String> {
    let on_submit = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        event.prevent_default();
        let form = event.target().and_then(|target| target.dyn_into::<Element>().ok());
        if let Some(form) = form.and_then(|form| form.get_attribute("data-submit")).and_then(|form| things::read(&form)) {
            dispatch(Action::Submit(form));
        }
    });
    thread_local! {
        /// The thing the last press was on that a double click does something with, as it is written.
        static USED: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    }
    let on_click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        glance(App::tune);
        // What the press is on is read before the press is handled: handling it may draw the thing anew.
        let used = pressed(&event, "data-use").and_then(|pressed| pressed.get_attribute("data-use"));
        if let Some(pressed) = pressed(&event, "data-click") {
            // A thing that asks where on it the press was (`parts::pressed_at`) is told that in the place of
            // what it was drawn with: the place in its own measure, however large the page draws it.
            let place = event.dyn_ref::<web_sys::MouseEvent>().filter(|_| pressed.has_attribute("data-place")).map(|mouse| {
                let on = pressed.get_bounding_client_rect();
                let scale = pressed.clone().dyn_into::<web_sys::HtmlElement>().map_or(1.0, |thing| on.width() / f64::from(thing.offset_width().max(1)));
                let scale = if scale > 0.0 { scale } else { 1.0 };
                format!("{},{}", ((f64::from(mouse.client_x()) - on.left()) / scale) as i32, ((f64::from(mouse.client_y()) - on.top()) / scale) as i32)
            });
            dispatch(Action::Click(
                pressed.get_attribute("data-click").unwrap_or_default(),
                place.unwrap_or_else(|| pressed.get_attribute("data-arg").unwrap_or_default())
            ));
        }
        // A double click on a thing (`things::doubled`): the second press on it, counted here, for the page
        // sends no double click to a thing its first press drew anew (a line of a menu, a row of a list).
        let presses = event.dyn_ref::<web_sys::MouseEvent>().map_or(0, |event| event.detail());
        let before = USED.replace(used.clone());
        if things::doubled(presses, before.as_deref(), used.as_deref()) {
            if let Some(what) = used.as_deref().and_then(things::read) {
                dispatch(Action::Use(what));
            }
        }
    });
    let on_double_click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        // A double click on a server enters it.
        if pressed(&event, "data-open-server").is_some() {
            dispatch(Action::Click("server-ok".to_owned(), String::new()));
        }
        if let Some(slot) = pressed(&event, "data-open").and_then(|pressed| pressed.get_attribute("data-open")).and_then(|slot| slot.parse().ok()) {
            dispatch(Action::Open(slot));
        }
    });
    // A slider moved (`data-slide`): its name and where it stands.
    let on_slide = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        let slider = event.target().and_then(|target| target.dyn_into::<HtmlInputElement>().ok());
        if let Some((name, level)) = slider.and_then(|slider| Some((slider.get_attribute("data-slide")?, slider.value().parse::<u8>().ok()?))) {
            dispatch(Action::Slid(name, level));
        }
    });
    let on_menu = Closure::<dyn FnMut(Event)>::new(right_pressed);
    ui.add_event_listener_with_callback("submit", on_submit.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dblclick", on_double_click.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("contextmenu", on_menu.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("input", on_slide.as_ref().unchecked_ref()).map_err(js_error)?;
    // These live as long as the page.
    on_submit.forget();
    on_click.forget();
    on_double_click.forget();
    on_menu.forget();
    on_slide.forget();
    Ok(())
}

/// The right button on the page: it empties a slot of the hotkey bar, with Alt moves all of a thing into the
/// storage or out of it, shows what an item or a skill is, opens the menu of a thing that has one.
fn right_pressed(event: Event) {
    if let Some(slot) = pressed(&event, "data-slot").and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok()) {
        event.prevent_default();
        dispatch(Action::Dropped(slot, None));
    }
    // The right button with Alt held on a thing that goes into the storage or comes out of it moves all
    // of it at once (the client's `UIItemWnd::OnRBtnDown`, `UIItemStoreWnd::OnRBtnDown`).
    let alt = event.dyn_ref::<web_sys::MouseEvent>().is_some_and(|event| event.alt_key());
    let stowed = pressed(&event, "data-use").filter(|_| alt).and_then(|thing| thing.get_attribute("data-use")).and_then(|what| things::read::<Use>(&what));
    if let Some(what @ (Use::Put(_) | Use::Take(_))) = stowed {
        event.prevent_default();
        return dispatch(Action::Stowed(what));
    }
    // The right button on an item shows what it is.
    if let Some((id, identified)) = pressed(&event, "data-info").and_then(|item| item.get_attribute("data-info")).and_then(
        |id| Some((id.trim_end_matches('?').parse().ok()?, !id.ends_with('?')))
    ) {
        event.prevent_default();
        dispatch(Action::Info(id, identified));
    }
    // The right button on a thing that has a menu opens it there (`data-menu`).
    let place = event.dyn_ref::<web_sys::MouseEvent>().map(|event| (event.client_x() as f32, event.client_y() as f32));
    if let (Some(what), Some(place)) = (
        pressed(&event, "data-menu").and_then(|thing| thing.get_attribute("data-menu")).and_then(|what| things::read(&what)),
        place
    ) {
        event.prevent_default();
        dispatch(Action::MenuOn(what, place));
    }
    // The right button on a skill shows its description.
    if let Some(id) = pressed(&event, "data-skill").and_then(|skill| skill.get_attribute("data-skill")).and_then(|id| id.parse().ok()) {
        event.prevent_default();
        dispatch(Action::SkillInfo(id));
    }
}

/// The keys: the game's own, and the first press that lets the music start.
fn wire_keys(document: &web_sys::Document) -> Result<(), String> {
    let on_key = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
        // Not everything a browser calls a key press has a key: filling a form in from its saved passwords, it
        // sends one with none. Asking such a one for its key fails in the browser itself, in the middle of
        // whatever asked; so the key and its place are read here, before the app is touched, and a press without
        // them is nobody's.
        let said = |name: &str| js_sys::Reflect::get(&event, &name.into()).ok().and_then(|value| value.as_string());
        let (Some(key), Some(code)) = (said("key"), said("code")) else { return };
        let in_game = glance(|app| matches!(app.screen, Screen::Game) && app.message.is_none() && app.ask.is_none()).unwrap_or(false);
        // A key pressed while something else is being handled is left to the browser: it cannot be told in time
        // whether the game uses it.
        let used = APP.with(|app| app.try_borrow_mut().is_ok_and(
            |mut app| app.key(&key, &code, crate::hotkeys::held(event.alt_key(), event.ctrl_key(), event.shift_key()))
        ));
        if used || (in_game && browsers_own(&event)) {
            event.prevent_default();
        }
        glance(App::tune);
    });
    // A browser plays nothing before the first press on the page, wherever it lands: the music starts with it.
    let on_press = Closure::<dyn FnMut()>::new(|| {
        glance(App::tune);
        check_music_soon();
    });
    document.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref()).map_err(js_error)?;
    for event in ["pointerdown", "keydown"] {
        document.add_event_listener_with_callback(event, on_press.as_ref().unchecked_ref()).map_err(js_error)?;
    }
    // These live as long as the page.
    on_key.forget();
    on_press.forget();
    Ok(())
}

/// What the page itself tells: its size changed, it was hidden or shown; and the beats that keep the
/// connection alive and watch it (`beats.rs`).
fn wire_page(window: &web_sys::Window, document: &web_sys::Document) -> Result<(), String> {
    let on_resize = Closure::<dyn FnMut()>::new(|| dispatch(Action::Resized));
    let on_visibility = Closure::<dyn FnMut()>::new(|| {
        glance(App::tune);
    });
    window.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("visibilitychange", on_visibility.as_ref().unchecked_ref()).map_err(js_error)?;
    beats::wire(window)?;
    // These live as long as the page.
    on_resize.forget();
    on_visibility.forget();
    Ok(())
}

/// What the browser remembers of the player (the language, the name, the sound, the keys, the settings) and
/// what the address asks for; then the first page is drawn.
fn restore(window: &web_sys::Window) {
    APP.with_borrow_mut(|app| {
        app.lang = stored("roweb.lang").and_then(|code| Lang::from_code(&code)).unwrap_or_default();
        crate::i18n::speak(app.lang);
        app.saved_id = stored("roweb.id").unwrap_or_default();
        app.save_id = !app.saved_id.is_empty();
        app.sound = stored("roweb.sound").as_deref() != Some("off");
        app.load_keys();
        app.load_settings();
        app.load_chat();
        app.load_shortcuts();
        app.load_tips();
        app.load_whispers();
        // A reset link from an e-mail: `/?reset=<token>`.
        let query = window.location().search().unwrap_or_default();
        // `/?mute` opens the page silent without changing what the player has chosen: for tests beside a player.
        if query.trim_start_matches('?').split('&').any(|pair| pair == "mute") {
            app.sound = false;
        }
        if let Some(token) = query.trim_start_matches('?').split('&').find_map(|pair| pair.strip_prefix("reset=")) {
            app.screen = Screen::Reset(token.to_owned());
        }
        app.render();
        // The music is asked for at once. Most browsers hold it back until the first press on a freshly loaded
        // page; some let it through.
        app.tune();
    });
}

/// The game's tables arrive a moment later; figures painted without them are painted again.
fn load_tables() {
    wasm_bindgen_futures::spawn_local(async {
        data::load_music().await;
        match data::load().await {
            Ok(()) => {
                figure::forget();
                dispatch(Action::Resized);
            }
            Err(error) => web_sys::console::warn_1(&error.into()),
        }
    });
}

/// Looks, a moment from now, whether the music plays: starting it takes the browser a moment.
pub(super) fn check_music_soon() {
    after(400, Action::CheckMusic);
}

/// Hands the app something that happened. One thing is handled at a time: what happens while another is being
/// handled (the browser tells of some things at once, in the middle of what caused them: a field losing the
/// keyboard as the page is drawn anew, a form sent by a key that is still being looked at) waits its turn and
/// is handled once the first is done. It waits for the browser's next turn, not for the end of this one: should
/// the app ever be left in the middle of something for good, the page stays alive.
pub(super) fn dispatch(action: Action) {
    APP.with(|app| match app.try_borrow_mut() {
        Ok(mut app) => app.handle(action),
        Err(_) => after(0, action),
    });
}

/// Looks at the app from a listener of the page, when it is not in the middle of handling something: then the
/// listener's small part (the music, mostly) is left to what is being handled, which does it itself.
fn glance<Seen>(look: impl FnOnce(&App) -> Seen) -> Option<Seen> {
    APP.with(|app| app.try_borrow().ok().map(|app| look(&app)))
}

/// Runs an action so many milliseconds from now.
pub(super) fn after(milliseconds: i32, action: Action) {
    let run = Closure::once_into_js(move || dispatch(action));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(run.unchecked_ref(), milliseconds);
    }
}

/// Runs an action after the current browser callback has returned.
pub(super) fn later(action: Action) {
    wasm_bindgen_futures::spawn_local(async move { dispatch(action) });
}
