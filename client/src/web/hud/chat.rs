//! The chat window: its tabs, its lines and the line to type in; a tab's own window; and the window that sets
//! what a tab lists.

use super::*;
use crate::chat_tabs::{self, Tab};
use crate::links::items::Piece;
use crate::press::chat::Press;

/// The colour of a link to an item in a line (`UIChatHisBox::RenderItemLinks`: `0x82F2E1` as the client
/// writes colours, blue first).
const LINK: &str = "#e1f282";

impl App {
    /// The chat's tabs as they are now: the player's, or the client's own until the player changes one.
    pub(in crate::web) fn chat_tabs(&self) -> Vec<Tab> {
        if self.chat.tabs.is_empty() { data::chat_tabs_start() } else { self.chat.tabs.clone() }
    }

    /// The tab that is open among those in the window: the one chosen, or the first of them if that one was
    /// taken out or away.
    pub(in crate::web) fn chat_tab_open(&self, tabs: &[Tab]) -> usize {
        match tabs.get(self.chat.tab) {
            Some(tab) if tab.own.is_none() => self.chat.tab,
            _ => tabs.iter().position(|tab| tab.own.is_none()).unwrap_or(0),
        }
    }

    /// Redraws the chat's lines, the newest at the bottom, and the windows of the tabs that have their own.
    pub(in crate::web) fn refresh_chat(&self) {
        let tabs = self.chat_tabs();
        if let Some(log) = crate::web::by_id("chat-lines") {
            log.set_inner_html(&self.chat_lines(tabs.get(self.chat_tab_open(&tabs))));
        }
        if tabs.iter().any(|tab| tab.own.is_some()) {
            self.redraw(&windows::CHAT_OWN);
        }
    }

    /// Every line kept that a tab lists (`Tab::lists`): the window shows the last, and scrolls back through
    /// the others. A link to an item in a line is the item's name between `<` and `>`, in the links' colour,
    /// and a press on it shows what is known of the item (`UIChatHisBox::RenderItemLinks`, `UM_TEXT_LBTN_UP`).
    pub(in crate::web) fn chat_lines(&self, tab: Option<&Tab>) -> String {
        let Some(tab) = tab else { return String::new() };
        self.chat
            .lines
            .iter()
            .filter(|(_, colour, kind)| tab.lists(*kind, matches!(colour.as_str(), "#ff0000" | "#f00")))
            .map(|(words, colour, _)| {
                let line: String = crate::links::items::pieces(words)
                    .into_iter()
                    .map(|piece| match piece {
                        Piece::Words(words) => escape(words),
                        Piece::Item(link) => {
                            let name = data::item_as(link.item, true).map_or_else(|| format!("#{}", link.item), |(name, _)| name);
                            let refined = if link.refine > 0 { format!("+{} ", link.refine) } else { String::new() };
                            format!(r#"<span style="color:{LINK};cursor:pointer" {}>&lt;{refined}{}&gt;</span>"#, pressed(Press::Link(link.item)), escape(&name))
                        }
                    })
                    .collect();
                log_line(colour, &line)
            })
            .collect()
    }

    /// One of the small buttons of the tabs' line, or of a tab's own window: its two pictures
    /// (`basic_interface\<name>_a.bmp` as it stands, `_b.bmp` under the pointer and pressed).
    fn chat_button(x: i32, name: &str, (width, height): (i32, i32), press: Press, title: &str) -> String {
        pictured_by(Convention::AThenB, x, 4, (width, height), &format!("basic_interface/{name}"), press, &format!(r#" title="{}""#, escape(tr(title))))
    }

    /// The tabs' line of the chat (`UINewChatWnd`): the tabs that are in the window, 71 wide and 16 high, the
    /// open one lighter; at the right the lock, and from it leftwards, each 2 from the next
    /// (`ApplyHeaderLayout`), the buttons that take the open tab away, add a tab, take the open tab out into
    /// a window of its own and set what it lists. Locked, only the last of them is there.
    fn chat_header(&self, tabs: &[Tab], open: usize) -> String {
        const TAB: i32 = 71;
        let mut header = String::new();
        for (place, (index, tab)) in tabs.iter().enumerate().filter(|(_, tab)| tab.own.is_none()).enumerate() {
            let lit = index == open;
            header.push_str(&format!(
                r#"<button type="button" class="a chat-tab" style="{};background:rgba(0,0,0,{})" {}>{}</button>"#,
                at(4 + TAB * place as i32, if lit { 1 } else { 2 }, TAB - 1, if lit { 16 } else { 15 }),
                if lit { ".4" } else { ".6" },
                pressed(Press::Tab(index)),
                escape(tr(&tab.name)),
            ));
        }
        let lock = if self.chat.locked { ("lock_dragwnd", 9) } else { ("unlock_dragwnd", 15) };
        header.push_str(&pictured(600 - 20, 4, (lock.1, 9), [&format!("basic_interface/{}.bmp", lock.0), &format!("basic_interface/{}.bmp", lock.0), &format!("basic_interface/{}.bmp", lock.0)], Press::Lock, r#" title="Lock the window in place""#));
        let buttons = [("wnd_mini", (8, 8), Press::Remove, "Remove the tab"), ("battle_option", (8, 8), Press::Add, "Add a tab"), ("stickoff", (14, 9), Press::Detach, "Detach"), ("battle_option2", (9, 9), Press::Options, "Options")];
        let mut right = 600 - 20;
        for (name, size, press, title) in buttons.into_iter().filter(|(_, _, press, _)| !self.chat.locked || *press == Press::Options) {
            right -= size.0 + 2;
            header.push_str(&Self::chat_button(right, name, size, press, title));
        }
        header
    }

    /// The chat's height: the client's own sum (`UINewChatWnd::ResizeRows`), the input line, the rows and three
    /// more, the tabs.
    pub(in crate::web) fn chat_height(&self) -> i32 {
        if self.chat.rows == 0 { 24 } else { 24 + (self.chat.rows as i32 + 3) * 14 + 16 }
    }

    /// The chat, as the client's `UINewChatWnd` lays it out, 600 wide: the tabs' line at the top; the
    /// see-through list between its white and grey lines; and the input line with the box for whom to whisper
    /// to, the two edits and the round buttons. With no rows only the input line is left.
    pub(in crate::web) fn chat_window(&self, lines: usize, height: i32) -> String {
        const TAB: i32 = 71;
        let base = height - 24;
        let mut window = String::new();
        if lines > 0 {
            let all = self.chat_tabs();
            let open = self.chat_tab_open(&all);
            let tabs = all.iter().filter(|tab| tab.own.is_none()).count() as i32 * TAB;
            window.push_str(&self.chat_header(&all, open));
            // How many play on the server, at the right of the tabs' line, before its small buttons. The client
            // has no such thing (it answers `/who` in the chat): this is this client's own.
            if let Some(count) = self.online {
                window.push_str(&format!(
                    r#"<div class="a online" style="right:{}px;top:0;height:15px" title="{}"><i></i><b id="online-count">{count}</b><span>{}</span></div>"#,
                    82,
                    escape(tr("Players online")),
                    escape(tr("online"))
                ));
            }
            // The list's see-through back, and the lines around it.
            window.push_str(&fill(4, 17, 600 - 8, height - 16 - 24 - 1, "rgba(0,0,0,.4)"));
            window.push_str(&fill(3, 16, 1, height - 16, WHITE));
            window.push_str(&fill(tabs + 3, 16, (600 - tabs - 10).max(0), 1, WHITE));
            window.push_str(&fill(600 - 4, 18, 1, height - 18, "#8e8e8e"));
            window.push_str(&format!(
                r#"<div id="chat-log" class="a chat-log" style="{}"><div id="chat-lines">{}</div></div>"#,
                at(5, 18, 600 - 8, height - 16 - 24 - 4),
                self.chat_lines(all.get(open))
            ));
        }
        window.push_str(&fill(4, base, 600 - 8, 1, "#8e8e8e"));
        let chat = sent(Form::Chat);
        // The round buttons of the input line: as they stand, and lit under the pointer and pressed.
        const ROUND: [&str; 3] = ["basic_interface/sys_base_off.bmp", "basic_interface/sys_base_on.bmp", "basic_interface/sys_base_on.bmp"];
        window.push_str(&format!(
            r#"<form {chat} class="a" style="{};background:url('{}') 0 0 no-repeat">
                <input class="a in" id="chat-to" maxlength="23" autocomplete="off" style="{};background:#f0f0f0" title="Whisper to">
                <div class="a" style="{};background:url('{}') 0 0 no-repeat"></div>
                <input class="a in" id="chat-input" maxlength="70" autocomplete="off" style="{}">
                {}
                {}
                <button type="submit" hidden></button>
            </form>"#,
            at(0, base, 600, 24),
            picture("basic_interface/dialog_bg.bmp"),
            at(6, 4, 90, 16),
            at(97, 3, 8, 18),
            picture("basic_interface/dialog_btn0.bmp"),
            at(110, 4, 460, 16),
            pictured(572, 7, (11, 11), ROUND, Press::Size, r#" title="Change the size (F10)""#),
            pictured(586, 7, (11, 11), ROUND, Press::Mini, r#" title="Hide or show the log""#),
        ));
        window
    }

    /// Where the own window of a tab stands and how large it is, if the tab has one.
    pub(in crate::web) fn chat_own(&self, index: usize) -> Option<(i32, i32, i32, i32)> {
        self.chat.tabs.get(index).and_then(|tab| tab.own)
    }

    /// A tab's own window (the client's `UISubChatHisWnd`): the tab's name in white at (4, 3) in a tab 16
    /// high and as wide as the name and 10 more; at the right, from 2 inside the edge leftwards, the buttons
    /// that take the tab away, put it back among the tabs (`stickon`) and set what it lists; the see-through
    /// back from 16 down and the lines in a box at (5, 20), 8 narrower and 35 lower than the window. (The
    /// client's window is made larger and smaller by its corner; this one keeps the size it opens with.)
    pub(in crate::web) fn chat_own_window(&self, index: usize) -> String {
        let (Some(tab), Some((_, _, width, height))) = (self.chat.tabs.get(index), self.chat_own(index)) else { return String::new() };
        let mut window = format!(
            r#"<div class="a" style="{};background:rgba(0,0,0,.4)"></div><div class="a chat-tab" style="left:0;top:0;height:16px;padding:0 5px;background:rgba(0,0,0,.4);border:1px solid #fff;border-bottom:0;box-sizing:border-box">{}</div>"#,
            at(0, 16, width, height - 16),
            escape(tr(&tab.name))
        );
        let buttons = [("wnd_mini", (8, 8), Press::OwnRemove(index), "Remove the tab"), ("stickon", (14, 9), Press::Attach(index), "Attach"), ("battle_option2", (9, 9), Press::OwnOptions(index), "Options")];
        let mut right = width - 2;
        for (name, size, press, title) in buttons.into_iter().filter(|(name, ..)| !self.chat.locked || *name == "battle_option2") {
            right -= size.0 + 2;
            window.push_str(&Self::chat_button(right, name, size, press, title));
        }
        window.push_str(&format!(r#"<div class="a chat-log" style="{}"><div>{}</div></div>"#, at(5, 20, width - 8, height - 35), self.chat_lines(Some(tab))));
        window
    }

    /// The window that sets what a tab lists (the client's `UIBattleMsgOptionWnd`, 280 by 200): its title the
    /// tab's name and the client's words after it (line 1777); a row 20 high to a kind of line, the first 22
    /// down, eight in sight and the rest scrolled to: a mark at 27 (`grp_online.bmp` listed, `grp_offline.bmp`
    /// not) and the kind's name at 50; a press on a row switches it. At the bottom left the box that has the
    /// tab list everything. A kind whose name the English message table lacks is left out.
    pub(in crate::web) fn chat_options_window(&self) -> String {
        let tabs = self.chat_tabs();
        let Some(tab) = self.chat.options.and_then(|index| tabs.get(index)) else { return String::new() };
        let mut window = title_bar(280, &format!("{}{}", tr(&tab.name), Self::words(1777, " message log settings")));
        window.push_str(&cross(280, Press::OptionsClose));
        window.push_str(&panel_ground(0, 17, 280, 183));
        window.push_str(&fill(18, 17, 1, 162, "#e0e0e0"));
        let rows: String = chat_tabs::ROWS
            .iter()
            .filter_map(|(kind, line)| Some((kind, data::message(*line).filter(|name| !name.is_empty())?)))
            .map(|(kind, name)| {
                format!(
                    r#"<div style="position:relative;height:20px;cursor:default" {}><div class="a" style="{};background:url('{}') no-repeat"></div><div class="a t" style="left:50px;top:3px">{}</div></div>"#,
                    pressed(Press::Lists(*kind)),
                    at(27, 3, 14, 14),
                    picture(if tab.shows[*kind] { "basic_interface/grp_online.bmp" } else { "basic_interface/grp_offline.bmp" }),
                    escape(tr(&name))
                )
            })
            .collect();
        window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-y:auto">{rows}</div>"#, at(0, 19, 279, 160)));
        window.push_str(&check_words(20, 184, tab.shows.iter().all(|shown| *shown), " all on", Press::ListsAll));
        window
    }
}
