//! Dragging the things in the windows (items, skills, what is on the hotkey bar), as the client does it.
//!
//! The client takes a thing up when the button goes down on it (`CGameMode::BeginDragItem`, `BeginDragSkill`)
//! and from then draws it under the pointer until the button is let go (`CGameMode::OnRender`): an item as its
//! own sprite, the one it has lying on the ground (`CSession::ItemDragResource`: `sprite/아이템/<resource>`, the
//! first picture of its first action, a touch see-through, 0xFA of 0xFF), with the game's pointer over it. Let
//! go over a slot of the hotkey bar it goes there; over the world it is thrown away; anywhere else nothing
//! happens. The browser's own dragging is not used: it would show its own picture and its own pointer.

use super::*;

thread_local! {
    /// What is being dragged (as its `data-drag` says), while the button is down.
    static DRAGGED: RefCell<Option<Drag>> = const { RefCell::new(None) };
}

/// Puts what follows the pointer on the page: `picture` with `origin` of it at the pointer, `scale` times its size.
fn show(picture: &str, origin: (f32, f32), scale: f64) {
    if let Some(ghost) = element("drag") {
        ghost.set_inner_html(&format!(
            r#"<img alt="" draggable="false" src="{picture}" style="position:absolute;left:{:.1}px;top:{:.1}px;transform:scale({scale:.3});transform-origin:0 0;image-rendering:pixelated">"#,
            -f64::from(origin.0) * scale,
            -f64::from(origin.1) * scale
        ));
    }
}

fn place(event: &web_sys::MouseEvent, shown: bool) {
    if let Some(ghost) = element("drag") {
        let _ = ghost.set_attribute("style", &format!("display:{};left:{}px;top:{}px", if shown { "block" } else { "none" }, event.client_x(), event.client_y()));
    }
}

/// Listens for the presses that take a thing up, the moves that carry it and the release that lets it go.
pub(super) fn wire(window: &web_sys::Window, ui: &Element) -> Result<(), String> {
    let on_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        let taken = event.target().and_then(|target| target.dyn_into::<Element>().ok()).and_then(|target| target.closest("[data-drag]").ok().flatten());
        let Some(taken) = taken.filter(|_| event.button() == 0) else { return };
        let Some(dragged) = taken.get_attribute("data-drag").and_then(|dragged| things::read::<Drag>(&dragged)) else { return };
        // The browser neither drags the picture itself nor marks text while the thing is carried.
        event.prevent_default();
        // The windows may be drawn larger than their own measures: what is carried is as much larger.
        let scale = taken.dyn_ref::<web_sys::HtmlElement>().filter(|taken| taken.offset_width() > 0).map_or(1.0, |html| taken.get_bounding_client_rect().width() / f64::from(html.offset_width()));
        // Until the item's sprite is read (and for a skill, which is dragged as its icon), the icon it has in
        // its window, its middle at the pointer.
        if let Some(icon) = taken.query_selector("img").ok().flatten().and_then(|icon| icon.get_attribute("src")) {
            show(&icon, (12.0, 12.0), scale);
        }
        place(&event, true);
        DRAGGED.set(Some(dragged.clone()));
        // An item: its sprite, by what the cell says it is (`data-info`: the item, with a `?` if not identified).
        let about = taken.get_attribute("data-info").filter(|_| matches!(dragged, Drag::Item(_)));
        if let Some((id, identified)) = about.and_then(|about| Some((about.trim_end_matches('?').parse::<u32>().ok()?, !about.ends_with('?')))) {
            wasm_bindgen_futures::spawn_local(async move {
                let resource = data::item_as(id, identified).map(|(_, resource)| resource).filter(|resource| !resource.is_empty()).unwrap_or_else(|| "사과".to_owned());
                if let Ok((picture, _, origin)) = figure::frame_picture(&format!("아이템/{resource}"), 0, 0).await {
                    if DRAGGED.with_borrow(|now| now.as_ref() == Some(&dragged)) {
                        show(&picture, origin, scale);
                    }
                }
            });
        }
    });
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        if DRAGGED.with_borrow(Option::is_some) {
            place(&event, true);
        }
    });
    let on_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        let Some(dragged) = DRAGGED.take() else { return };
        place(&event, false);
        let under = crate::web::document().and_then(|document| document.element_from_point(event.client_x() as f32, event.client_y() as f32));
        let Some(under) = under else { return };
        let slot = under.closest("[data-slot]").ok().flatten().and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok());
        match slot {
            Some(slot) => dispatch(Action::Dropped(slot, Some(dragged))),
            // Over the world itself, not over a window on it.
            None if under.id() == "game" => dispatch(Action::Thrown(dragged)),
            None => {}
        }
    });
    ui.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref()).map_err(js_error)?;
    window.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    window.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref()).map_err(js_error)?;
    on_down.forget();
    on_move.forget();
    on_up.forget();
    Ok(())
}
