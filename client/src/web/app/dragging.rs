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
        ghost.set_inner_html(&[
            format!(
                r#"<img alt="" draggable="false" src="{picture}" style="position:absolute;left:{:.1}px;top:{:.1}px;"#,
                -f64::from(origin.0) * scale,
                -f64::from(origin.1) * scale
            ),
            format!(r#"transform:scale({scale:.3});transform-origin:0 0;image-rendering:pixelated">"#)
        ].concat());
    }
}

fn place(event: &web_sys::MouseEvent, shown: bool) {
    if let Some(ghost) = element("drag") {
        let _ = ghost.set_attribute(
            "style",
            &format!("display:{};left:{}px;top:{}px", if shown { "block" } else { "none" }, event.client_x(), event.client_y())
        );
    }
}

/// The button went down on a thing that can be dragged: it is taken up, and follows the pointer as its icon
/// until its sprite is read.
fn taken_up(event: web_sys::MouseEvent) {
    let taken = event.target().and_then(|target| target.dyn_into::<Element>().ok()).and_then(|target| target.closest("[data-drag]").ok().flatten());
    let Some(taken) = taken.filter(|_| event.button() == 0) else { return };
    let Some(dragged) = taken.get_attribute("data-drag").and_then(|dragged| things::read::<Drag>(&dragged)) else { return };
    // The browser neither drags the picture itself nor marks text while the thing is carried.
    event.prevent_default();
    // The windows may be drawn larger than their own measures: what is carried is as much larger.
    let scale = taken.dyn_ref::<web_sys::HtmlElement>().filter(|taken| taken.offset_width() > 0).map_or(
        1.0,
        |html| taken.get_bounding_client_rect().width() / f64::from(html.offset_width())
    );
    // Until the item's sprite is read (and for a skill, which is dragged as its icon), the icon it has in
    // its window, its middle at the pointer.
    if let Some(icon) = taken.query_selector("img").ok().flatten().and_then(|icon| icon.get_attribute("src")) {
        show(&icon, (12.0, 12.0), scale);
    }
    place(&event, true);
    DRAGGED.set(Some(dragged.clone()));
    // An item: its sprite, by what the cell says it is (`data-info`: the item, with a `?` if not identified).
    let about = taken.get_attribute("data-info").filter(|_| matches!(
        dragged,
        Drag::Item(_) | Drag::Worn(_) | Drag::Attached(_) | Drag::Offer(_) | Drag::Basket(_) | Drag::Stored(_)
        | Drag::Carted(_) | Drag::Stock(_) | Drag::Stalled(_) | Drag::Vended(_) | Drag::Wanted(_)
    ));
    if let Some((id, identified)) = about.and_then(|about| Some((about.trim_end_matches('?').parse::<u32>().ok()?, !about.ends_with('?')))) {
        wasm_bindgen_futures::spawn_local(async move {
            let resource = data::item_as(id, identified)
                .map(|(_, resource)| resource)
                .filter(|resource| !resource.is_empty())
                .unwrap_or_else(|| "사과".to_owned());
            if let Ok((picture, _, origin)) = figure::frame_picture(&format!("아이템/{resource}"), 0, 0).await {
                if DRAGGED.with_borrow(|now| now.as_ref() == Some(&dragged)) {
                    show(&picture, origin, scale);
                }
            }
        });
    }
}

/// The button was let go: what was carried goes to what lies under the pointer, if that takes it.
fn let_go(event: web_sys::MouseEvent) {
    let Some(dragged) = DRAGGED.take() else { return };
    place(&event, false);
    let under = crate::web::document().and_then(|document| document.element_from_point(event.client_x() as f32, event.client_y() as f32));
    let Some(under) = under else { return };
    // A member of the party taken out of the party window and let go anywhere but over that window gets a
    // small window (`UIDragMiniPartyWnd::OnLBtnUp`, `UIMessengerGroupWnd::DropMember`).
    if let Drag::Member(_) = dragged {
        if under.closest(&format!("#{}", wnd::PARTY.id)).ok().flatten().is_none() {
            dispatch(Action::Thrown(dragged));
        }
        return;
    }
    let slot = under.closest("[data-slot]").ok().flatten().and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok());
    // A window that takes things says so itself (`data-drop`: a shop's list and basket, the storage).
    let onto = under.closest("[data-drop]").ok().flatten().and_then(|onto| onto.get_attribute("data-drop")).and_then(|onto| things::read::<Drop>(&onto));
    match (slot, onto) {
        (Some(slot), _) => dispatch(Action::Dropped(slot, Some(dragged))),
        (None, Some(onto)) => dispatch(Action::LetGo(dragged, onto)),
        // Over the world itself, not over a window on it.
        (None, None) if under.id() == "game" => dispatch(Action::Thrown(dragged)),
        // Over the window of a deal between two players: it goes into the deal (the client's
        // `CUIExchangeUI::DragAndDrop`).
        (None, None) if under.closest("#exchange").ok().flatten().is_some() => dispatch(Action::Offered(dragged)),
        // Over a thing being forged: it goes into one of its slots (`UIMakeTargetProcessWnd::DragAndDrop`).
        (None, None) if under.closest("#choose-line").ok().flatten().is_some() => dispatch(Action::Forged(dragged)),
        // Over the letter being written: it goes into the letter (the client's `UIRodexComposeWnd::DragAndDrop`).
        (None, None) if under.closest("#mail-write").ok().flatten().is_some() => dispatch(Action::Mailed(dragged)),
        // A thing worn, let go over the inventory, is taken off (the client's item window takes what is
        // dragged out of `UIEquipWnd` so).
        (None, None) if under.closest("#inventory").ok().flatten().is_some() => {
            if let Drag::Worn(index) = dragged {
                dispatch(Action::Use(Use::Item(index)));
            }
            // A thing of a letter being written comes back out of it.
            if let Drag::Attached(index) = dragged {
                dispatch(Action::Unmailed(index));
            }
            // A thing of the storage comes out of it (the client's `UIItemWnd::DragAndDrop`).
            if let Drag::Stored(index) = dragged {
                dispatch(Action::Unstored(index));
            }
            // A thing of the cart comes out of it (the same function, for what was taken up in the cart).
            if let Drag::Carted(index) = dragged {
                dispatch(Action::Uncarted(index));
            }
        }
        (None, None) => {}
    }
}

/// Listens for the presses that take a thing up, the moves that carry it and the release that lets it go.
pub(super) fn wire(window: &web_sys::Window, ui: &Element) -> Result<(), String> {
    let on_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(taken_up);
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        if DRAGGED.with_borrow(Option::is_some) {
            place(&event, true);
        }
    });
    let on_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(let_go);
    ui.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref()).map_err(js_error)?;
    window.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    window.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref()).map_err(js_error)?;
    on_down.forget();
    on_move.forget();
    on_up.forget();
    Ok(())
}
