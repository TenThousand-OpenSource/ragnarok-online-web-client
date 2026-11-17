//! Starting the client: the page's events (clicks, keys, drags, timers) are turned into actions for the app.

use super::*;

pub fn start() -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let ui = ui()?;
    ui.set_attribute("style", &format!("background-image:url('{}')", asset_url(WALLPAPER))).map_err(js_error)?;
    let on_submit = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        event.prevent_default();
        let form = event.target().and_then(|target| target.dyn_into::<Element>().ok());
        if let Some(action) = form.and_then(|form| form.get_attribute("data-submit")) {
            dispatch(Action::Submit(action));
        }
    });
    fn pressed(event: &Event, attribute: &str) -> Option<Element> {
        let target = event.target().and_then(|target| target.dyn_into::<Element>().ok())?;
        let selector = if attribute == "button" { attribute.to_owned() } else { format!("[{attribute}]") };
        target.closest(&selector).ok().flatten()
    }
    let on_click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        APP.with_borrow(App::tune);
        if let Some(pressed) = pressed(&event, "data-click") {
            dispatch(Action::Click(pressed.get_attribute("data-click").unwrap_or_default(), pressed.get_attribute("data-arg").unwrap_or_default()));
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
        if let Some(what) = pressed(&event, "data-use").and_then(|pressed| pressed.get_attribute("data-use")) {
            dispatch(Action::Use(what));
        }
    });
    // Icons are dragged from the inventory and the skill list onto the hotkey bar; a right click empties a slot.
    let on_drag = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        let dragged = pressed(&event, "data-drag").and_then(|dragged| dragged.get_attribute("data-drag"));
        if let (Some(dragged), Some(transfer)) = (dragged, event.data_transfer()) {
            let _ = transfer.set_data("text/plain", &dragged);
        }
    });
    let on_drag_over = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        if pressed(&event, "data-slot").is_some() {
            event.prevent_default();
        }
    });
    let on_drop = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        let slot = pressed(&event, "data-slot").and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok());
        let dragged = event.data_transfer().and_then(|transfer| transfer.get_data("text/plain").ok());
        if let (Some(slot), Some(dragged)) = (slot, dragged) {
            event.prevent_default();
            dispatch(Action::Dropped(slot, dragged));
        }
    });
    // A slider moved (`data-slide`): its name and where it stands.
    let on_slide = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        let slider = event.target().and_then(|target| target.dyn_into::<HtmlInputElement>().ok());
        if let Some((name, level)) = slider.and_then(|slider| Some((slider.get_attribute("data-slide")?, slider.value().parse::<u8>().ok()?))) {
            dispatch(Action::Slid(name, level));
        }
    });
    let on_menu = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        if let Some(slot) = pressed(&event, "data-slot").and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok()) {
            event.prevent_default();
            dispatch(Action::Dropped(slot, String::new()));
        }
        // The right button on an item shows what it is.
        if let Some((id, identified)) = pressed(&event, "data-info").and_then(|item| item.get_attribute("data-info")).and_then(|id| Some((id.trim_end_matches('?').parse().ok()?, !id.ends_with('?')))) {
            event.prevent_default();
            dispatch(Action::Info(id, identified));
        }
        // The right button on a skill shows its description.
        if let Some(id) = pressed(&event, "data-skill").and_then(|skill| skill.get_attribute("data-skill")).and_then(|id| id.parse().ok()) {
            event.prevent_default();
            dispatch(Action::SkillInfo(id));
        }
    });
    let on_key = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
        let in_game = APP.with_borrow(|app| matches!(app.screen, Screen::Game) && app.message.is_none() && app.ask.is_none());
        if APP.with_borrow_mut(|app| app.key(&event.key(), &event.code(), hotkeys::held(event.alt_key(), event.ctrl_key(), event.shift_key()))) || (in_game && browsers_own(&event)) {
            event.prevent_default();
        }
        APP.with_borrow(App::tune);
    });
    // A browser plays nothing before the first press on the page, wherever it lands: the music starts with it.
    let on_press = Closure::<dyn FnMut()>::new(|| {
        APP.with_borrow(App::tune);
        check_music_soon();
    });
    // A thing dragged out of the inventory and let go over the world is dropped there.
    if let Some(world) = document.get_element_by_id("game") {
        let over = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| event.prevent_default());
        let thrown = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
            event.prevent_default();
            if let Some(dragged) = event.data_transfer().and_then(|transfer| transfer.get_data("text/plain").ok()) {
                dispatch(Action::Thrown(dragged));
            }
        });
        world.add_event_listener_with_callback("dragover", over.as_ref().unchecked_ref()).map_err(js_error)?;
        world.add_event_listener_with_callback("drop", thrown.as_ref().unchecked_ref()).map_err(js_error)?;
        over.forget();
        thrown.forget();
    }
    drag_windows(&document)?;
    let on_resize = Closure::<dyn FnMut()>::new(|| dispatch(Action::Resized));
    let on_visibility = Closure::<dyn FnMut()>::new(|| APP.with_borrow(App::tune));
    let on_timer = Closure::<dyn FnMut()>::new(|| dispatch(Action::KeepAlive));
    // The game's own tooltip (the client's `CMode::ShowTooltip`): white words on a see-through black box, shown
    // at once over whatever has one (`data-tip`), 16 above its top left corner, and gone when the pointer leaves.
    let on_over = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        let Some(tip) = element("tip") else { return };
        let tipped = pressed(&event, "data-tip").and_then(|tipped| tipped.dyn_into::<web_sys::HtmlElement>().ok());
        let style = match tipped {
            Some(tipped) => {
                let words = tipped.get_attribute("data-tip").unwrap_or_default();
                if tip.text_content().as_deref() != Some(words.as_str()) {
                    tip.set_text_content(Some(&words));
                }
                // The windows may be drawn larger than their own measures: the tooltip is as much larger.
                let place = tipped.get_bounding_client_rect();
                let scale = if tipped.offset_width() > 0 { place.width() / f64::from(tipped.offset_width()) } else { 1.0 };
                format!("display:block;left:{:.0}px;top:{:.0}px;font-size:{:.1}px", place.left().max(0.0), (place.top() - 16.0 * scale).max(0.0), 12.0 * scale)
            }
            None => "display:none".to_owned(),
        };
        let _ = tip.set_attribute("style", &style);
    });
    ui.add_event_listener_with_callback("mouseover", on_over.as_ref().unchecked_ref()).map_err(js_error)?;
    on_over.forget();
    ui.add_event_listener_with_callback("submit", on_submit.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dblclick", on_double_click.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dragstart", on_drag.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dragover", on_drag_over.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("drop", on_drop.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("contextmenu", on_menu.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("input", on_slide.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref()).map_err(js_error)?;
    for event in ["pointerdown", "keydown"] {
        document.add_event_listener_with_callback(event, on_press.as_ref().unchecked_ref()).map_err(js_error)?;
    }
    window.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("visibilitychange", on_visibility.as_ref().unchecked_ref()).map_err(js_error)?;
    window.set_interval_with_callback_and_timeout_and_arguments_0(on_timer.as_ref().unchecked_ref(), KEEP_ALIVE_MS).map_err(js_error)?;
    // These live as long as the page.
    on_submit.forget();
    on_click.forget();
    on_double_click.forget();
    on_drag.forget();
    on_drag_over.forget();
    on_drop.forget();
    on_menu.forget();
    on_slide.forget();
    on_key.forget();
    on_press.forget();
    on_resize.forget();
    on_visibility.forget();
    on_timer.forget();

    APP.with_borrow_mut(|app| {
        app.lang = stored("roweb.lang").and_then(|code| Lang::from_code(&code)).unwrap_or_default();
        crate::i18n::speak(app.lang);
        app.saved_id = stored("roweb.id").unwrap_or_default();
        app.save_id = !app.saved_id.is_empty();
        app.sound = stored("roweb.sound").as_deref() != Some("off");
        app.load_keys();
        app.load_settings();
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
    check_music_soon();
    // The game's tables arrive a moment later; figures painted without them are painted again.
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
    Ok(())
}

/// Looks, a moment from now, whether the music plays: starting it takes the browser a moment.
pub(super) fn check_music_soon() {
    let check = Closure::once_into_js(|| dispatch(Action::CheckMusic));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(check.unchecked_ref(), 400);
    }
}

pub(super) fn dispatch(action: Action) {
    APP.with_borrow_mut(|app| app.handle(action));
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
