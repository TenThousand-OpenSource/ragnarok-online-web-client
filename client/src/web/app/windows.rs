//! The order of the game's windows on the page (`crate::stack`): one opened or pressed lies over the others, and
//! Escape closes the one in front. Which windows have a part in it, and what closes each, is in the list of
//! windows (`hud/windows.rs`). And what the menus and the boxes of every system share on the app's side: where
//! on the stage the pointer is, a window opened there, every menu put away, the number typed into a box.

use super::*;
use crate::stack::Stack;

/// The windows that lie in the order now, by their elements' names.
fn ordered(app: &App) -> impl Iterator<Item = &'static wnd::Window> {
    let playing = matches!(app.screen, Screen::Game);
    wnd::ALL.iter().copied().filter(move |window| playing && window.order.as_ref().is_some_and(|order| (order.when)(app)))
}

/// Lays the windows on the page in their order: each a step over the one behind it. One that is not in the order
/// lies as its layer has it (`Layer` of `hud/windows.rs`): the windows that ask something (an NPC's, a deal, how
/// many to drop) are over them all.
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

    /// A place on the page in the stage's own measure, a pixel right and down of it, where the client puts a
    /// menu (`menu->Move(g_mouse.x + 1, g_mouse.y + 1)`): the stage may be drawn larger or smaller than it is.
    pub(super) fn on_stage(at: (f32, f32)) -> (i32, i32) {
        let stage = crate::web::document().and_then(|document| document.query_selector(".stage").ok().flatten());
        let scale = stage.as_ref().and_then(|stage| Some(stage.get_bounding_client_rect().width() / f64::from(stage.clone().dyn_into::<web_sys::HtmlElement>().ok()?.offset_width().max(1)))).filter(|scale| *scale > 0.0).unwrap_or(1.0);
        ((f64::from(at.0) / scale) as i32 + 1, (f64::from(at.1) / scale) as i32 + 1)
    }

    /// What was typed into a box that asks how many of a thing there are `most` of (`hud::App::count_box`
    /// of that window): the number, when it is one that can be (`items::count_typed`).
    pub(super) fn counted(window: &wnd::Window, most: u16) -> Option<u16> {
        items::count_typed(&field(&window.field()), most)
    }

    /// A press elsewhere puts away every menu the right button opened.
    pub(super) fn close_menus(&mut self) {
        if self.pets.menu_at.take().is_some() {
            self.render();
        }
        self.close_companion_menus();
        self.close_player_menu();
        self.close_party_menu();
        self.close_guild_menu();
        self.close_friend_menu();
        self.close_whisper_menu();
        self.close_room_menu();
    }

    /// Opens a window at a place decided as it opens (where the pointer is, a step from the last of its
    /// kind): its element is given the place and drawn anew, without the whole screen being written again,
    /// so what is being typed elsewhere stays. `style` is what the window's own entry writes for it then.
    pub(super) fn open_at(&self, window: &'static wnd::Window, style: &str) {
        forget_moved(window.id);
        if let Some(element) = element(window.id) {
            let _ = element.set_attribute("style", style);
        }
        self.redraw(&[window]);
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
