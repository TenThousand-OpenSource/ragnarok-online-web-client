//! The window of a talk with an NPC.

use super::*;

impl App {
    /// A press of this system (`crate::press::talk`).
    pub(super) fn talk_pressed(&mut self, press: crate::press::talk::Press) {
        use crate::press::talk::Press;
        // A link to an item (`CCreateLineInfo::OnItemLink`: `UM_ITEMEXPLANATION`, as one that is identified).
        if let Press::Item(item) = press {
            return self.handle(Action::Info(item, true));
        }
        // (A press a recording was made with is done again as it plays: there is no server then, and nothing
        // is sent.)
        let told = self.replay.told;
        let (Some(dialog), socket) = (&mut self.dialog, &self.socket) else { return };
        if socket.is_none() && !told {
            return;
        }
        let send = |packet: &[u8]| {
            if let Some(socket) = socket {
                socket.send(packet);
            }
        };
        let npc = dialog.npc;
        // "Next" and a menu's OK are not taken twice within a moment (`UISayDialogWnd::SendMsg` and
        // `UIChooseWnd::SendMsg` share `m_itemSendTick`), by the mouse or by Enter.
        if press == Press::Next || (press == Press::Ok && !dialog.menu.is_empty()) {
            if !told && crate::talk::too_soon(now(), dialog.answered) {
                return;
            }
            dialog.answered = now();
        }
        match press {
            Press::Next => {
                dialog.button = None;
                dialog.stale = true;
                send(&net::talk::next(npc));
            }
            Press::Close => {
                send(&net::talk::close(npc));
                self.dialog = None;
            }
            // (A row there is none of is not picked: a recording may name any.)
            Press::Pick(choice) => dialog.picked = choice.min(dialog.menu.len().saturating_sub(1)),
            Press::Item(_) => {}
            Press::Ok if !dialog.menu.is_empty() => {
                // The choices are counted from 1.
                send(&net::talk::choose(npc, dialog.picked as u8 + 1));
                dialog.menu.clear();
                dialog.stale = true;
            }
            Press::Ok => {
                let typed = field("dialog-input");
                // An empty line is not sent (`UINpcEditDialogWnd::SendMsg`: OK only with `edit->Length()`).
                if typed.is_empty() {
                    return;
                }
                // A text with a word that is not let be written is not sent, and the chat says the client's
                // line 14 in red (`UINpcTextEditDialogWnd::SendMsg`, `crate::manner`).
                if dialog.asks == Some(true) && data::foul(&typed) {
                    return self.chat_line(Self::words(14, "Foul Language Detected."), RED);
                }
                send(&if dialog.asks == Some(true) { net::talk::words(npc, &typed) } else { net::talk::number(npc, typed.trim().parse().unwrap_or(0)) });
                dialog.asks = None;
                dialog.stale = true;
            }
            Press::Cancel => {
                // Turning a menu down ends the talk.
                send(&net::talk::choose(npc, 255));
                self.dialog = None;
            }
        }
        self.redraw(&[&wnd::DIALOG]);
    }

    /// A key while an NPC is talked to; whether it was this talk's key. Enter presses the button of one of the
    /// talk's windows (`crate::talk::entered`) and goes nowhere else, wherever the keyboard is: the chat's
    /// line does not open or send. The arrows up and down move the menu's picked line, or with no menu what
    /// the NPC said (`crate::talk::stepped`, `crate::scrolling::TEXT_STEP`).
    pub(in crate::web::app) fn talk_key(&mut self, key: &str) -> bool {
        // The game's menu asking about the save point is a box over everything, and has the keys.
        let Some(dialog) = self.dialog.as_ref().filter(|_| self.esc != 2) else { return false };
        let (rows, picked) = (dialog.menu.len(), dialog.picked);
        match key {
            "Enter" => {
                if let Some(press) = crate::talk::entered(dialog.asks, rows > 0, dialog.button) {
                    self.talk_pressed(press);
                }
                true
            }
            "ArrowUp" | "ArrowDown" if rows > 0 => {
                let picked = crate::talk::stepped(picked, rows, key == "ArrowDown");
                let scrolled = Self::menu_list().map_or(0, |list| list.scroll_top());
                self.talk_pressed(crate::press::talk::Press::Pick(picked));
                Self::menu_shown(picked, scrolled);
                true
            }
            "ArrowUp" | "ArrowDown" => {
                if let Some(said) = element(wnd::DIALOG.id).and_then(|window| window.query_selector(".said").ok().flatten()) {
                    // The same step as the wheel's, and as little past the text's ends (`UM_UPPUSH`, `UM_DOWNPUSH`).
                    step_list(&said, if key == "ArrowDown" { 1 } else { -1 });
                }
                true
            }
            _ => false,
        }
    }

    /// The list of the menu on the page: the last of the talk's boxes that scroll.
    fn menu_list() -> Option<web_sys::HtmlElement> {
        let lists = element(wnd::DIALOG.id)?.query_selector_all(".scroll").ok()?;
        lists.item(lists.length().checked_sub(1)?)?.dyn_into().ok()
    }

    /// The menu's list, drawn anew after an arrow, is put back where it was scrolled to and moved no farther
    /// than brings the picked line into it (`UIListBox::SendMsg` scrolls a line when the picked one leaves
    /// what shows).
    fn menu_shown(picked: usize, scrolled: i32) {
        let Some(list) = Self::menu_list() else { return };
        list.set_scroll_top(scrolled);
        let row = list.query_selector_all(":scope > div").ok().and_then(|rows| rows.item(picked as u32));
        let Some(row) = row.and_then(|row| row.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
        // The list is placed by itself, so a line's place is counted from the list's top.
        let (top, bottom) = (row.offset_top(), row.offset_top() + row.offset_height());
        if top < list.scroll_top() {
            list.set_scroll_top(top);
        } else if bottom > list.scroll_top() + list.client_height() {
            list.set_scroll_top(bottom - list.client_height());
        }
    }
}
