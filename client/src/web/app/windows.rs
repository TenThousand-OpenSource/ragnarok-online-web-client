//! The order of the game's windows on the page (`crate::stack`): one opened or pressed lies over the others, and
//! Escape closes the one in front. Which windows have a part in it, and what closes each, is in the list of
//! windows (`hud/windows.rs`).

use super::*;
use crate::stack::Stack;

/// The windows that lie in the order now, by their elements' names.
fn ordered(app: &App) -> impl Iterator<Item = &'static wnd::Window> {
    let playing = matches!(app.screen, Screen::Game);
    wnd::ALL.iter().copied().filter(move |window| playing && window.order.as_ref().is_some_and(|order| (order.when)(app)))
}

/// Lays the windows on the page in their order: each a step over the one behind it. One that is not in the order
/// lies as the page has it: the windows that ask something (an NPC's, a deal, how many to drop) are over them
/// all, by the page's own rules.
fn laid(order: &Stack) {
    for window in wnd::ALL.iter().filter(|window| window.order.is_some()) {
        let Some(element) = element(window.id).and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok()) else { continue };
        let _ = match order.windows().iter().position(|open| *open == window.id) {
            Some(rank) => element.style().set_property("z-index", &(11 + rank).to_string()),
            None => element.style().remove_property("z-index").map(|_| ()),
        };
    }
}

impl App {
    /// A window was pressed: it comes to the front (the client's `RaiseWindow`).
    pub(super) fn window_pressed(&self, id: &str) {
        if self.windows.borrow_mut().raise(id) {
            laid(&self.windows.borrow());
        }
    }

    /// After the windows were drawn: one newly open goes to the front, and all lie in their order again.
    pub(in crate::web) fn order_windows(&self) {
        let open: Vec<&'static str> = ordered(self).map(|window| window.id).collect();
        self.windows.borrow_mut().settle(&open);
        laid(&self.windows.borrow());
    }

    /// Escape: closes the window in front, each as its own close button does. Gives back whether there was one.
    pub(super) fn close_front(&mut self) -> bool {
        self.order_windows();
        let Some(front) = self.windows.borrow().front() else { return false };
        let Some(close) = wnd::ALL.iter().find(|window| window.id == front).and_then(|window| window.order.as_ref()).map(|order| order.close) else { return false };
        close(self);
        self.order_windows();
        true
    }
}
