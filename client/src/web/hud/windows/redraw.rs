//! The list of windows on the page: the screen written from it, and windows drawn anew where they are.

use super::*;
use wasm_bindgen::JsCast;

/// The number in a box that asks how many is ready to be typed over: its field has the keyboard, and what is
/// in it is marked.
pub(super) fn typed_over(window: &web_sys::Element) {
    if let Some(count) = window.query_selector("input").ok().flatten().and_then(|count| count.dyn_into::<web_sys::HtmlInputElement>().ok()) {
        let _ = count.focus();
        count.select();
    }
}

/// The field of a window that is typed into has the keyboard, with nothing marked (the bank's sum).
pub(super) fn typed_into(window: &web_sys::Element) {
    if let Some(field) = window.query_selector("input").ok().flatten().and_then(|field| field.dyn_into::<web_sys::HtmlInputElement>().ok()) {
        let _ = field.focus();
    }
}

impl Window {
    /// What is in it now: nothing while it is closed.
    fn inside(&self, app: &App) -> String {
        if (self.open)(app) { (self.draw)(app) } else { String::new() }
    }
}

impl App {
    /// Every window of the game, each in its own element. `view` is the stage's size.
    pub(in crate::web) fn hud(&self, view: (f64, f64)) -> String {
        ALL.iter()
            .map(|window| {
                let hidden = if (window.open)(self) { "" } else { " hidden" };
                format!(r#"<div id="{}" {}{}{hidden}>{}</div>"#, window.id, (window.frame)(self, view), window.layer.written(), window.inside(self))
            })
            .collect()
    }

    /// Draws windows anew where they are on the page: each open one with what is in it now, each closed one
    /// emptied and hidden. A list keeps its place and a field being typed into keeps its text and the keyboard.
    pub(in crate::web) fn redraw(&self, windows: &[&Window]) {
        let mut figures = false;
        for window in windows {
            let Some(element) = crate::web::by_id(window.id) else { continue };
            let inside = window.inside(self);
            figures |= inside.contains("<canvas");
            let scrolled = element.query_selector(".scroll, .viewer").ok().flatten().map(|list| list.scroll_top());
            let typed = crate::web::document()
                .and_then(|document| document.active_element())
                .filter(|active| !active.id().is_empty() && element.contains(Some(active)))
                .and_then(|active| active.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| (input.id(), input.value()));
            element.set_inner_html(&inside);
            let _ = if inside.is_empty() && !(window.open)(self) { element.set_attribute("hidden", "") } else { element.remove_attribute("hidden") };
            if let (Some(scrolled), Ok(Some(list))) = (scrolled, element.query_selector(".scroll, .viewer")) {
                list.set_scroll_top(scrolled);
            }
            crate::web::app::fit_bars(&element);
            if let Some(drawn) = window.drawn {
                drawn(&element);
            }
            if let Some((id, text)) = typed {
                if let Some(input) = crate::web::by_id(&id).and_then(|input| input.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                    input.set_value(&text);
                    let _ = input.focus();
                }
            }
        }
        // A window written anew has its canvases anew: the character is drawn on them again.
        if figures {
            self.paint_figures();
        }
        // One opened this way goes over the other windows.
        self.order_windows();
    }

    /// Redraws the windows that show the character's values.
    pub(in crate::web) fn refresh_values(&self) {
        crate::web::world::own_life(self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        crate::web::world::own_level(self.value(parameter::BASE_LEVEL) as u16);
        self.redraw(&[&BASIC, &STATUS]);
        // The player's own life shows in the party's windows too.
        self.party_shown();
    }

    /// Redraws the windows that show items and skills.
    pub(in crate::web) fn refresh_items(&self) {
        self.redraw(&[&INVENTORY, &SKILLS, &HOTKEYS, &ITEM_INFO, &SKILL_INFO, &EQUIPMENT, &SHOP_COMPARE]);
        self.redraw(&STORAGE_WINDOWS);
    }
}
