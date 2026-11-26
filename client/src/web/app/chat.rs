//! The chat window on the app's side: a line added to it, its next size, and what the pointer does to its
//! tabs that is no press: a tab pulled out of the window, the corner of a tab's own window dragged.

use super::*;
use crate::press::Named;

/// What the pointer holds of the chat's tabs.
#[derive(Clone, Copy)]
enum Held {
    /// A tab in the window, pressed there: the tab, and where the pointer was.
    Tab(usize, (f64, f64)),
    /// The corner of a tab's own window: the tab, where the pointer was, and how large the window was.
    Corner(usize, (f64, f64), (i32, i32), (i32, i32)),
}

thread_local! {
    static HELD: std::cell::Cell<Option<Held>> = const { std::cell::Cell::new(None) };
}

/// What was pressed of the chat's tabs, if anything: a tab in the chat window, or the corner of a tab's own
/// window (`data-resize`, written by `hud/chat.rs`).
fn held(event: &web_sys::MouseEvent) -> Option<Held> {
    let target = event.target().and_then(|target| target.dyn_into::<Element>().ok())?;
    let pointer = (f64::from(event.client_x()), f64::from(event.client_y()));
    if let Some(corner) = target.closest("[data-resize]").ok().flatten() {
        let tab: usize = corner.get_attribute("data-resize")?.parse().ok()?;
        let (_, _, width, height) = App::window_rect(wnd::CHAT_OWN.get(tab)?.id)?;
        return Some(Held::Corner(tab, pointer, (width, height), (width, height)));
    }
    let tab = target.closest("[data-click]").ok().flatten()?;
    match crate::press::chat::Press::parse(&tab.get_attribute("data-click")?, &tab.get_attribute("data-arg")?) {
        Some(crate::press::chat::Press::Tab(tab)) => Some(Held::Tab(tab, pointer)),
        _ => None,
    }
}

/// The pointer on the chat's tabs: a tab held and moved more than a little is pulled out of the window
/// (`chat_tabs::torn`, the client's `UITabControl::OnMouseMove`), and the corner of a tab's own window held
/// and moved makes that window larger and smaller (`UIResizer`, then `UISubChatHisWnd`'s `UM_RESIZE`).
pub(super) fn drag_chat_tabs(document: &web_sys::Document) -> Result<(), String> {
    let on_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        HELD.set(if event.button() == 0 { held(&event) } else { None });
    });
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        let Some(held) = HELD.get() else { return };
        if event.buttons() & 1 == 0 {
            return HELD.set(None);
        }
        let scale = crate::web::stage_scale();
        let to = (f64::from(event.client_x()), f64::from(event.client_y()));
        match held {
            Held::Tab(tab, from) => {
                if crate::chat_tabs::torn((from.0 / scale, from.1 / scale), (to.0 / scale, to.1 / scale)) {
                    HELD.set(None);
                    dispatch(Action::ChatTabTorn(tab, to));
                }
            }
            Held::Corner(tab, from, was, last) => {
                let size = crate::chat_tabs::own_resized(was.0 + ((to.0 - from.0) / scale) as i32, was.1 + ((to.1 - from.1) / scale) as i32);
                if size != last {
                    HELD.set(Some(Held::Corner(tab, from, was, size)));
                    dispatch(Action::ChatOwnSized(tab, size.0, size.1));
                }
            }
        }
    });
    let on_up = Closure::<dyn FnMut()>::new(|| HELD.set(None));
    document.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref()).map_err(js_error)?;
    on_down.forget();
    on_move.forget();
    on_up.forget();
    Ok(())
}

impl App {
    /// The chat takes its next size (`crate::chat::rows_stepped`).
    pub(super) fn chat_resize(&mut self) {
        self.chat.rows = crate::chat::rows_stepped(self.chat.rows);
    }

    /// Adds a line to the chat.
    pub(super) fn chat_line(&mut self, words: String, colour: &str) {
        self.chat_line_of(said::GENERAL, words, colour);
    }

    /// A line for the chat window, of one of the client's kinds (`said`): the window's tabs list it or not by that.
    pub(super) fn chat_line_of(&mut self, kind: u8, words: String, colour: &str) {
        // The items a line links to are asked about, so that the line can name them.
        self.chat_links_wanted(&words);
        self.chat.lines.push(Line { words, red: said::colour::pure_red(colour), colour: colour.to_owned(), kind });
        if self.chat.lines.len() > super::super::hud::CHAT_KEPT {
            self.chat.lines.remove(0);
        }
        self.refresh_chat();
    }

    /// A line of the client's message table in the chat, with what it asks for written into it
    /// (`numbers::filled`); nothing while the table does not have the line.
    pub(super) fn say_filled(&mut self, kind: u8, line: usize, with: &[&str], colour: &str) {
        if let Some(words) = data::message(line).filter(|words| !words.is_empty()) {
            self.chat_line_of(kind, crate::numbers::filled(crate::i18n::tr(&words), with), colour);
        }
    }
}
