//! The chat window: its tabs, its lines and the line to type in; a tab's own window; and the window that sets
//! what a tab lists.

use super::*;
use crate::chat_tabs::{self, Tab};
use crate::links::items::Piece;
use crate::press::chat::Press;

/// The grey of the lines at the right and the bottom of the chat's windows (`0x8E8E8E`).
const GREY: &str = "#8e8e8e";
/// The least width of the line a name is typed in, in a tab's own window.
const TAB_FIELD: i32 = 71;

/// The colour of a link to an item in a line (`UIChatHisBox::RenderItemLinks`: `0x82F2E1` as the client
/// writes colours, blue first).
const LINK: &str = "#e1f282";

/// How wide the list of the chat window is: the window's 600 less the 4 at each side. Its rows are cut by it
/// (`App::chat_lines`), wherever the list is written.
const LIST_WIDE: i32 = 600 - 8;

impl App {
    /// The chat's tabs as they are now: the player's, or the client's own until the player changes one.
    pub(in crate::web) fn chat_tabs(&self) -> Vec<Tab> {
        if self.chat.tabs.is_empty() { data::chat_tabs_start() } else { self.chat.tabs.clone() }
    }

    /// How wide each tab is in the window's line of tabs (`chat_tabs::tab_wide`), by how wide the page
    /// writes the names.
    pub(in crate::web) fn chat_tab_wide(tabs: &[Tab]) -> i32 {
        let mut wide = crate::web::text_widths();
        chat_tabs::tab_wide(tabs.iter().filter(|tab| tab.own.is_none()).map(|tab| wide(tr(&tab.name)).ceil() as i32))
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
            log.set_inner_html(&self.chat_lines(tabs.get(self.chat_tab_open(&tabs)), LIST_WIDE));
        }
        if tabs.iter().any(|tab| tab.own.is_some()) {
            self.redraw(&windows::CHAT_OWN);
        }
    }

    /// Every line kept that a tab lists (`Tab::lists`): the window shows the last, and scrolls back through
    /// the others. A line is cut into rows as the client's list has them (`crate::chat::rows`), and a row
    /// wider than the list, which is `wide`, is cut again by its width at the room of the scroll bar
    /// (`UIChatHisBox::AddTextWrapped`; the client asks whether a row is wider than the box less 20, and
    /// then cuts it at the box less 16: here it is cut at the box less 16 in both). A link to an item in a
    /// line is the item's name between `<` and `>`, in the links' colour, and a press on it shows what is
    /// known of the item (`UIChatHisBox::RenderItemLinks`, `UM_TEXT_LBTN_UP`).
    pub(in crate::web) fn chat_lines(&self, tab: Option<&Tab>, wide: i32) -> String {
        let Some(tab) = tab else { return String::new() };
        let rows: String = self
            .chat
            .listed(tab)
            .flat_map(|said| crate::chat::rows(&said.words).into_iter().map(|row| log_line(&said.colour, &Self::chat_row(&row))))
            .collect();
        // (Two of the box's width are the room at the left of its letters.)
        format!(r#"<div style="width:{}px">{rows}</div>"#, wide - crate::chat::BAR - 2)
    }

    /// A row of the chat's list as the page writes it: its words, what stands in brackets kept together
    /// (`crate::chat::held`), and its links to items.
    fn chat_row(row: &str) -> String {
        crate::links::items::pieces(row)
            .into_iter()
            .map(|piece| match piece {
                Piece::Words(words) => crate::navigation::texts::chat_parts(words).into_iter().map(Self::chat_words).collect(),
                Piece::Item(link) => {
                    let name = data::item_as(link.item, true).map_or_else(|| format!("#{}", link.item), |(name, _)| name);
                    let refined = if link.refine > 0 { format!("+{} ", link.refine) } else { String::new() };
                    format!(
                        r#"<span style="color:{LINK};cursor:pointer" {}>&lt;{refined}{}&gt;</span>"#,
                        pressed(Press::Link(link.item)),
                        escape(&name)
                    )
                }
            })
            .collect()
    }

    /// The words of a row between its links to items: what stands in brackets kept together, and a link to
    /// a place (`<NAVIL>…</NAVIL>`) as the client shows one, the map and the cell between `<` and `>`
    /// (`NavigationLinkParser::DisplayNavigationLink`), pressed to look at that place in the navigation's
    /// window (`UINaviLinkButton::OnLBtnDown`).
    fn chat_words(part: Result<&str, &str>) -> String {
        use crate::navigation::texts;
        match part {
            Ok(words) => crate::chat::held(words)
                .into_iter()
                .map(|(together, words)| if together { format!(r#"<span class="kept">{}</span>"#, escape(words)) } else { escape(words) })
                .collect(),
            Err(link) => match texts::chat_linked(link) {
                Some((map, x, y)) => format!(
                    r#"<span style="color:{LINK};cursor:pointer" {}>{}</span>"#,
                    pressed(crate::press::navigation::Press::Chat(link.to_owned())),
                    escape(&texts::chat_link_words(map, x, y))
                ),
                None => escape(link),
            },
        }
    }

    /// One of the small buttons of the tabs' line, or of a tab's own window: its two pictures
    /// (`basic_interface\<name>_a.bmp` as it stands, `_b.bmp` under the pointer and pressed).
    fn chat_button(x: i32, name: &str, (width, height): (i32, i32), press: Press, title: &str) -> String {
        pictured_by(Convention::AThenB, x, 4, (width, height), &format!("basic_interface/{name}"), press, &format!(r#" title="{}""#, escape(tr(title))))
    }

    /// The tabs' line of the chat (`UINewChatWnd`): the tabs that are in the window, each as wide as the
    /// widest of their names and 10 more (`chat_tab_wide`) and 16 high, the open one lighter; at the right the lock, and from it leftwards, each 2 from the next
    /// (`ApplyHeaderLayout`), the buttons that take the open tab away, add a tab, take the open tab out into
    /// a window of its own and set what it lists. Locked, only the last of them is there.
    fn chat_header(&self, tabs: &[Tab], open: usize) -> String {
        let wide = Self::chat_tab_wide(tabs);
        let mut header = String::new();
        for (place, (index, tab)) in tabs.iter().enumerate().filter(|(_, tab)| tab.own.is_none()).enumerate() {
            let lit = index == open;
            // A press on a tab opens it, a double click has its name typed anew, and the pointer pulls it out
            // of the window (`app/chat.rs`).
            header.push_str(&format!(
                r#"<button type="button" class="a chat-tab" style="{};background:rgba(0,0,0,{})" {} {}>{}</button>"#,
                at(4 + wide * place as i32, if lit { 1 } else { 2 }, wide - 1, if lit { 16 } else { 15 }),
                if lit { ".4" } else { ".6" },
                pressed(Press::Tab(index)),
                used(Use::ChatTab(index)),
                escape(tr(&tab.name)),
            ));
            if self.chat.renaming == Some(index) {
                header.push_str(&Self::chat_name_field(4 + wide * place as i32 + 2, 2, wide - 4, &tab.name));
            }
        }
        let lock = if self.chat.locked { ("lock_dragwnd", 9) } else { ("unlock_dragwnd", 15) };
        header.push_str(&pictured(
            600 - 20,
            4,
            (lock.1, 9),
            [
                &format!("basic_interface/{}.bmp", lock.0),
                &format!("basic_interface/{}.bmp", lock.0),
                &format!("basic_interface/{}.bmp", lock.0)
            ],
            Press::Lock,
            &format!(r#" title="{}""#, escape(tr(if self.chat.locked { "Unlock the chat windows" } else { "Lock the chat windows in place" })))
        ));
        let buttons = [
            ("wnd_mini", (8, 8), Press::Remove, "Remove the tab"),
            ("battle_option", (8, 8), Press::Add, "Add a tab"),
            ("stickoff", (14, 9), Press::Detach, "Take the tab out into a window of its own"),
            ("battle_option2", (9, 9), Press::Options, "Options")
        ];
        let mut right = 600 - 20;
        for (name, size, press, title) in buttons.into_iter().filter(|(_, _, press, _)| !self.chat.locked || *press == Press::Options) {
            right -= size.0 + 2;
            header.push_str(&Self::chat_button(right, name, size, press, title));
        }
        header
    }

    /// The line a tab's name is typed anew in, over the tab (`UITabControl::OnLBtnDblClk`: the tab's width
    /// less 4, at 2, 2 of it): Enter sends it.
    fn chat_name_field(x: i32, y: i32, width: i32, name: &str) -> String {
        format!(
            r#"<form {} class="a" style="{}"><input class="a in" id="{}" maxlength="23" autocomplete="off" value="{}" style="{}"></form>"#,
            sent(Form::ChatTabName),
            at(x, y, width, 14),
            CHAT_TAB_NAME,
            escape(name),
            at(0, 0, width, 14)
        )
    }

    /// The chat's height: the client's own sum (`UINewChatWnd::ResizeRows`), the input line, the rows and three
    /// more, the tabs.
    pub(in crate::web) fn chat_height(&self) -> i32 {
        crate::chat::height(self.chat.rows)
    }

    /// The chat, as the client's `UINewChatWnd` lays it out, 600 wide: the tabs' line at the top; the
    /// see-through list between its white and grey lines; and the input line with the box for whom to whisper
    /// to, the two edits and the round buttons. With no rows only the input line is left.
    pub(in crate::web) fn chat_window(&self, lines: usize, height: i32) -> String {
        let base = height - 24;
        let mut window = String::new();
        if lines > 0 {
            let all = self.chat_tabs();
            let open = self.chat_tab_open(&all);
            let tabs = chat_tabs::inside(&all) as i32 * Self::chat_tab_wide(&all);
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
                at(5, 18, LIST_WIDE, height - 16 - 24 - 4),
                self.chat_lines(all.get(open), LIST_WIDE)
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

    /// A tab's own window (the client's `UISubChatHisWnd`, `Paint` and `OnCreate`): the tab's name in white
    /// at (4, 3) in a tab 16 high and as wide as the name and 10 more (25 at least), with a white line over
    /// it and at its left and a grey one at its right; from the tab's foot a white line to the right side,
    /// and grey lines down that side and along the bottom; at the right, from 2 inside the edge leftwards
    /// (`ApplyButtonLayout`), the buttons that take the tab away, put it back among the tabs (`stickon`) and
    /// set what it lists, of which only the last is there while the chat is locked; the see-through back
    /// from 16 down and the lines in a box at (5, 20), 8 narrower and 35 lower than the window; and in the
    /// corner the handle that makes the window larger and smaller (`btn_resize.bmp`, 14 by 14, 15 in from
    /// the right and the bottom). A double click on the name has it typed anew (not while locked).
    pub(in crate::web) fn chat_own_window(&self, index: usize) -> String {
        let (Some(tab), Some(own)) = (self.chat.tabs.get(index), self.chat_own(index)) else { return String::new() };
        // (As large as it is shown: a size that cannot be is the client's own, `chat_tabs::own_shown`.)
        let (_, _, width, height) = chat_tabs::own_shown(own, (i32::MAX, i32::MAX));
        let name = (crate::web::text_widths()(tr(&tab.name)) as i32 + 10).max(25);
        let mut window = [
            format!(r#"<div class="a" style="{};background:rgba(0,0,0,.4)"></div>"#, at(0, 16, width, height - 16)),
            format!(
                r#"<div class="a chat-tab" style="{};background:rgba(0,0,0,.4);text-align:left;text-indent:4px" {}>{}</div>"#,
                at(0, 0, name, 16),
                used(Use::ChatTab(index)),
                escape(tr(&tab.name))
            ),
            fill(2, 0, name - 5, 1, WHITE),
            fill(1, 1, 1, 1, WHITE),
            fill(0, 2, 1, height - 4, WHITE),
            fill(name, 16, width - name - 3, 1, WHITE),
            fill(name - 3, 0, 1, 1, GREY),
            fill(name - 2, 1, 1, 1, GREY),
            fill(name - 1, 2, 1, 15, GREY),
            fill(width - 3, 16, 1, 1, GREY),
            fill(width - 2, 17, 1, 1, GREY),
            fill(width - 1, 18, 1, height - 20, GREY),
            fill(1, height - 2, 1, 1, GREY),
            fill(width - 2, height - 2, 1, 1, GREY),
            fill(2, height - 1, width - 4, 1, GREY),
        ].concat();
        if self.chat.renaming == Some(index) {
            window.push_str(&Self::chat_name_field(5, 2, name.max(TAB_FIELD) - 10, ""));
        }
        let buttons = [
            ("wnd_mini", (8, 8), Press::OwnRemove(index), "Remove the tab"),
            ("stickon", (14, 9), Press::Attach(index), "Put the tab back into the chat window"),
            ("battle_option2", (9, 9), Press::OwnOptions(index), "Options")
        ];
        let mut right = width;
        for (name, size, press, title) in buttons.into_iter().filter(|(name, ..)| !self.chat.locked || *name == "battle_option2") {
            right -= size.0 + 2;
            window.push_str(&Self::chat_button(right, name, size, press, title));
        }
        window.push_str(&format!(
            r#"<div class="a chat-log" style="{}"><div>{}</div></div>"#,
            at(5, 20, width - 8, height - 35),
            self.chat_lines(Some(tab), width - 8)
        ));
        window.push_str(&format!(
            r#"<button type="button" class="a" data-resize="{index}" style="{};padding:0;border:0;background:url('{}') no-repeat;cursor:nwse-resize"></button>"#,
            at(width - 15, height - 15, 14, 14),
            picture("btn_resize.bmp")
        ));
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
                [
                    format!(r#"<div style="position:relative;height:20px;cursor:default" {}>"#, pressed(Press::Lists(*kind))),
                    format!(
                        r#"<div class="a" style="{};background:url('{}') no-repeat"></div>"#,
                        at(27, 3, 14, 14),
                        picture(if tab.shows[*kind] { "basic_interface/grp_online.bmp" } else { "basic_interface/grp_offline.bmp" })
                    ),
                    format!(r#"<div class="a t" style="left:50px;top:3px">{}</div></div>"#, escape(tr(&name)))
                ].concat()
            })
            .collect();
        window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-y:auto">{rows}</div>"#, at(0, 19, 279, 160)));
        window.push_str(&check_words(20, 184, tab.shows.iter().all(|shown| *shown), " all on", Press::ListsAll));
        window
    }
}
