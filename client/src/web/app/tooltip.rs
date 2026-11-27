//! The game's own tooltip on the page (the client's `CMode::ShowTooltip`): the one box that says what the
//! thing under the pointer is. Its words and where it stands are the thing's own (`parts::tip` and the
//! others beside it); how they are cut into lines and kept on the screen is `crate::tooltip`.

use super::*;

thread_local! {
    /// Where the pointer was last, on the page.
    static POINTER: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

/// How often what is under a pointer that stands still is looked at again while a tooltip shows, in
/// milliseconds (this project's own: the client looks each frame).
const TIP_LOOK_MS: i32 = 200;

/// The tooltip is white words (or of the thing's own colour, `data-tip-colour`) on a see-through black box,
/// shown at once over whatever has one (`data-tip`) and gone when the pointer leaves.
///
/// The whole page is listened to, not the windows alone: the pointer leaves a thing for the world too, which
/// is no part of them, and for nowhere at all when it goes off the page. It leaves a thing also when it
/// stands still and a window opens under it, or the thing is drawn away: a page tells of neither until the
/// pointer moves. So while a tooltip shows, what is under the pointer is looked at again every `TIP_LOOK_MS`,
/// and the tooltip goes, or changes, with it (the client draws its tooltip anew each frame for the window
/// under the pointer). One that goes with the pointer is moved with it.
pub(super) fn wire() -> Result<(), String> {
    let on_over = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        // (The pointer may come over a thing without having been told of moving: a window drawn under it.)
        POINTER.set((event.client_x() as f32, event.client_y() as f32));
        let target = event.target().and_then(|target| target.dyn_into::<Element>().ok());
        tip_over(target.and_then(|target| target.closest("[data-tip]").ok().flatten()));
    });
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        POINTER.set((event.client_x() as f32, event.client_y() as f32));
        // One that goes with the pointer is put anew; the others stay where they were asked for.
        if let Some(tipped) = under_pointer().filter(|tipped| shown() && tipped.get_attribute("data-tip-at").as_deref() == Some("pointer")) {
            tip_over(Some(tipped));
        }
    });
    let on_look = Closure::<dyn FnMut()>::new(|| {
        if shown() {
            tip_over(under_pointer());
        }
    });
    let on_out = Closure::<dyn FnMut()>::new(|| tip_over(None));
    let page = crate::web::document().ok_or("no document")?;
    page.add_event_listener_with_callback("mouseover", on_over.as_ref().unchecked_ref()).map_err(js_error)?;
    page.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    page.add_event_listener_with_callback("mouseleave", on_out.as_ref().unchecked_ref()).map_err(js_error)?;
    if let Some(window) = web_sys::window() {
        window.set_interval_with_callback_and_timeout_and_arguments_0(on_look.as_ref().unchecked_ref(), TIP_LOOK_MS).map_err(js_error)?;
    }
    // These live as long as the page.
    on_over.forget();
    on_move.forget();
    on_out.forget();
    on_look.forget();
    Ok(())
}

/// Whether a tooltip shows now.
fn shown() -> bool {
    element("tip").is_some_and(|tip| tip.get_attribute("style").is_some_and(|style| style.starts_with("display:block")))
}

/// The thing with a tooltip that the pointer is on, if any.
fn under_pointer() -> Option<Element> {
    let (x, y) = POINTER.get();
    crate::web::document()?.element_from_point(x, y)?.closest("[data-tip]").ok().flatten()
}

/// Shows the tooltip of the thing under the pointer (`tipped`), or takes it away when there is none. It is
/// asked for 16 above the thing's top left corner, or where the thing says (`data-tip-at`: so far right and
/// down of that corner in the thing's own measure, or `pointer` for 10 left of and 20 above the pointer,
/// `UINewSkillListWnd::OnMouseHover`), and kept on the screen as the client keeps it.
fn tip_over(tipped: Option<Element>) {
    let Some(tip) = element("tip").and_then(|tip| tip.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
    let style = match tipped.and_then(|tipped| tipped.dyn_into::<web_sys::HtmlElement>().ok()) {
        Some(tipped) => {
            let said = |name: &str| tipped.get_attribute(name);
            let words = crate::tooltip::lines(&said("data-tip").unwrap_or_default()).join("\n");
            if tip.text_content().as_deref() != Some(words.as_str()) {
                tip.set_text_content(Some(&words));
            }
            // The windows may be drawn larger than their own measures: the tooltip is as much larger.
            let place = tipped.get_bounding_client_rect();
            let scale = if tipped.offset_width() > 0 { place.width() / f64::from(tipped.offset_width()) } else { 1.0 };
            let from = said("data-tip-at");
            let at = match from.as_deref().map(|from| from.split_once(',').and_then(|(x, y)| Some((x.parse::<f64>().ok()?, y.parse::<f64>().ok()?)))) {
                Some(Some((x, y))) => (place.left() + x * scale, place.top() + y * scale),
                Some(None) => (f64::from(POINTER.get().0) - 10.0 * scale, f64::from(POINTER.get().1) - 20.0 * scale),
                None => (place.left(), place.top() - 16.0 * scale),
            };
            let look = format!("font-size:{:.1}px;color:{}", 12.0 * scale, said("data-tip-colour").unwrap_or_else(|| "#fff".to_owned()));
            // It is measured as it will be drawn before it is put: the page draws nothing in between.
            let _ = tip.set_attribute("style", &format!("display:block;left:0;top:0;{look}"));
            let view = crate::web::page_size();
            let (x, y) = crate::tooltip::placed((at.0 as i32, at.1 as i32), (tip.offset_width(), tip.offset_height()), (view.0 as i32, view.1 as i32));
            format!("display:block;left:{x}px;top:{y}px;{look}")
        }
        None => "display:none".to_owned(),
    };
    if tip.get_attribute("style").as_deref() != Some(style.as_str()) {
        let _ = tip.set_attribute("style", &style);
    }
}
