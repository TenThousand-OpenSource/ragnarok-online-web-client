//! The client's scroll bar on the page (`UIScrollBar`), one for every list and text that has one: its handle
//! is sized and stood by the list beside it, and a press on an arrow, on the way above or below the handle
//! and a drag of the handle move that list as the client's bar moves its own (`crate::scrolling::Bar`).
//!
//! A bar is what `hud/lists.rs scroll_bar` writes right after its list: the list is the element before it.
//! The list itself is scrolled by the page (the wheel in `wiring.rs`); the bar follows whatever moved it.

use super::*;
use crate::scrolling::{self, Bar, Part};

/// The attribute of a bar: how many pixels a step of its list is.
const BAR: &str = "data-bar";

/// The handle being dragged: its bar, where on the page it was taken, and how many steps the list has moved
/// since (`UIScrollBar::pressY` and `dragAmount`).
struct Held {
    bar: Element,
    taken: f64,
    dragged: i32,
}

thread_local! {
    static HELD: RefCell<Option<Held>> = const { RefCell::new(None) };
}

/// A bar with its list as they are measured now: the bar as rules, the list, the pixels of a step, how
/// much larger than its own measure the page draws the bar, and for a list that begins at its end (the
/// chat's, which a page counts from its end up: `data-reversed`) how far its end is from its top.
struct Measured {
    bar: Bar,
    list: Element,
    unit: i32,
    scale: f64,
    from_end: Option<i32>,
}

fn measured(bar: &Element) -> Option<Measured> {
    let list = bar.previous_element_sibling()?;
    let unit = bar.get_attribute(BAR)?.parse::<i32>().ok()?.max(1);
    let drawn = bar.clone().dyn_into::<web_sys::HtmlElement>().ok()?;
    let length = drawn.offset_height();
    let scale = if length > 0 { bar.get_bounding_client_rect().height() / f64::from(length) } else { 1.0 };
    let from_end = bar.has_attribute("data-reversed").then(|| (list.scroll_height() - list.client_height()).max(0));
    Some(Measured {
        bar: Bar::of(length, unit, list.client_height(), list.scroll_height(), list.scroll_top() + from_end.unwrap_or(0)),
        list,
        unit,
        scale: if scale > 0.0 { scale } else { 1.0 },
        from_end,
    })
}

/// Stands a bar's handle where its list is, as long as the list asks; a bar with nowhere to scroll is not
/// drawn and takes no press (`UIScrollBar::Paint`).
fn fit(bar: &Element) {
    let Some(Measured { bar: rules, .. }) = measured(bar) else { return };
    let shown = if rules.shown() { "visible" } else { "hidden" };
    let Ok(drawn) = bar.clone().dyn_into::<web_sys::HtmlElement>() else { return };
    if drawn.style().get_property_value("visibility").as_deref() != Ok(shown) {
        let _ = drawn.style().set_property("visibility", shown);
    }
    if let Some(handle) = bar.last_element_child().and_then(|handle| handle.dyn_into::<web_sys::HtmlElement>().ok()) {
        let style = format!("top:{}px;height:{}px", rules.thumb_at(), rules.thumb());
        if handle.get_attribute("style").as_deref() != Some(style.as_str()) {
            let _ = handle.set_attribute("style", &style);
        }
    }
}

/// Fits every bar within a part of the page that was just written.
pub(in crate::web) fn fit_within(root: &Element) {
    let Ok(bars) = root.query_selector_all(&format!("[{BAR}]")) else { return };
    for index in 0..bars.length() {
        if let Some(bar) = bars.item(index).and_then(|bar| bar.dyn_into::<Element>().ok()) {
            fit(&bar);
        }
    }
}

/// Moves a bar's list so many steps and stands the handle anew; says how many it moved.
fn step(bar: &Element, steps: i32) -> i32 {
    let Some(Measured { bar: rules, list, unit, from_end, .. }) = measured(bar) else { return 0 };
    let top = scrolling::moved(list.scroll_top() + from_end.unwrap_or(0), unit, steps, list.scroll_height() - list.client_height());
    list.set_scroll_top(top - from_end.unwrap_or(0));
    fit(bar);
    measured(bar).map_or(0, |after| after.bar.position - rules.position)
}

/// A press on a bar (`UIScrollBar::OnLBtnDown`): an arrow moves a step, the way above or below the handle a
/// page (the bar's `data-page`, or one less than the list shows), and the handle is taken to be dragged.
fn pressed_on(bar: Element, event: &web_sys::MouseEvent) {
    let Some(Measured { bar: rules, scale, .. }) = measured(&bar) else { return };
    let y = ((f64::from(event.client_y()) - bar.get_bounding_client_rect().top()) / scale) as i32;
    let page = bar.get_attribute("data-page").and_then(|page| page.parse().ok()).unwrap_or_else(|| scrolling::page(rules.visible));
    match rules.part(y) {
        Some(Part::Thumb) => HELD.set(Some(Held { bar, taken: f64::from(event.client_y()), dragged: 0 })),
        Some(part) => {
            step(&bar, Bar::pressed(part, page));
        }
        None => {}
    }
}

/// The pointer moved with a handle held (`UIScrollBar::OnMouseMove`).
fn drag(event: &web_sys::MouseEvent) {
    HELD.with_borrow_mut(|held| {
        let Some(held) = held else { return };
        let Some(Measured { bar: rules, scale, .. }) = measured(&held.bar) else { return };
        let more = rules.dragged(((f64::from(event.client_y()) - held.taken) / scale) as i32, held.dragged);
        if more != 0 {
            held.dragged += step(&held.bar, more);
        }
    });
}

/// The page's events a bar lives by: a press on it, the pointer moved and let go while its handle is held
/// (`wire_gesture`), and its list scrolled by anything (a page tells of that at the list alone, so it is
/// caught on the way down).
pub(super) fn wire(ui: &Element, document: &web_sys::Document) -> Result<(), String> {
    let on_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        let bar = event.target().and_then(|target| target.dyn_into::<Element>().ok()).and_then(|target| target.closest(&format!("[{BAR}]")).ok().flatten());
        if let Some(bar) = bar.filter(|_| event.button() == 0) {
            event.prevent_default();
            pressed_on(bar, &event);
        }
    });
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| drag(&event));
    let on_up = Closure::<dyn FnMut()>::new(|| HELD.set(None));
    let on_scroll = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        let list = event.target().and_then(|target| target.dyn_into::<Element>().ok());
        if let Some(bar) = list.and_then(|list| list.next_element_sibling()).filter(|bar| bar.has_attribute(BAR)) {
            fit(&bar);
        }
    });
    ui.add_event_listener_with_callback_and_bool("scroll", on_scroll.as_ref().unchecked_ref(), true).map_err(js_error)?;
    on_scroll.forget();
    wire_gesture(document, on_down, on_move, on_up)
}
