//! The chat window: its tabs, its lines and the line to type in.

use super::*;

impl App {
    /// Redraws the chat's lines, the newest at the bottom.
    pub(in crate::web) fn refresh_chat(&self) {
        if let Some(log) = crate::web::by_id("chat-lines") {
            log.set_inner_html(&self.chat_lines(self.chat.rows));
        }
    }

    pub(in crate::web) fn chat_lines(&self, lines: usize) -> String {
        // Every line kept is there: the window shows the last, and scrolls back through the others. Each tab
        // lists the kinds of line the client's own table gives it (`data::chat_tab_shows`): as the client starts,
        // the first nearly everything, the second only what is said in red when something went wrong.
        let _ = lines;
        self.chat.lines
            .iter()
            .filter(|(_, colour, kind)| data::chat_tab_shows(self.chat.tab, *kind, matches!(colour.as_str(), "#ff0000" | "#f00")))
            .map(|(words, colour, _)| format!(r#"<div style="color:{colour}">{}</div>"#, escape(words)))
            .collect()
    }

    /// The chat, as the client's `UINewChatWnd` lays it out, 600 wide: the tabs at the top (71 wide, 16 high,
    /// the open one lighter), the small buttons at the right of them and the lock that lets the window be
    /// dragged; the see-through list between its white and grey lines; and the input line with the box for
    /// whom to whisper to, the two edits and the round buttons. With no rows only the input line is left.
    pub(in crate::web) fn chat_window(&self, lines: usize, height: i32) -> String {
        const TABS: [&str; 2] = ["Public Log", "Battle Log"];
        const TAB: i32 = 71;
        let base = height - 24;
        let mut window = String::new();
        if lines > 0 {
            let tabs = TABS.len() as i32 * TAB;
            for (index, name) in TABS.iter().enumerate() {
                let open = index == self.chat.tab;
                window.push_str(&format!(
                    r#"<button type="button" class="a chat-tab" style="{};background:rgba(0,0,0,{})" data-click="chat-tab" data-arg="{index}">{}</button>"#,
                    at(4 + TAB * index as i32, if open { 1 } else { 2 }, TAB - 1, if open { 16 } else { 15 }),
                    if open { ".4" } else { ".6" },
                    tr(name),
                ));
            }
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
            window.push_str(&format!(r#"<div class="a" style="{};background:rgba(0,0,0,.4)"></div>"#, at(4, 17, 600 - 8, height - 16 - 24 - 1)));
            window.push_str(&format!(r#"<div class="a" style="{};background:#fff"></div>"#, at(3, 16, 1, height - 16)));
            window.push_str(&format!(r#"<div class="a" style="{};background:#fff"></div>"#, at(tabs + 3, 16, 600 - tabs - 10, 1)));
            window.push_str(&format!(r#"<div class="a" style="{};background:#8e8e8e"></div>"#, at(600 - 4, 18, 1, height - 18)));
            window.push_str(&format!(
                r#"<div id="chat-log" class="a" style="{}"><div id="chat-lines">{}</div></div>"#,
                at(5, 18, 600 - 8, height - 16 - 24 - 4),
                self.chat_lines(lines)
            ));
            // From the right: the lock, then the small buttons, each 2 from the next (`ApplyHeaderLayout`).
            let lock = if self.chat.locked { ("lock_dragwnd", 9) } else { ("unlock_dragwnd", 15) };
            window.push_str(&format!(
                r#"<button type="button" class="a b" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="chat-lock" title="Lock the window in place"></button>"#,
                at(600 - 20, 4, lock.1, 9),
                picture(&format!("basic_interface/{}.bmp", lock.0)),
                picture(&format!("basic_interface/{}.bmp", lock.0)),
                picture(&format!("basic_interface/{}.bmp", lock.0)),
            ));
            let mut right = 600 - 20;
            for (name, width, tall, click, title) in [("wnd_mini", 8, 8, "chat-mini", "Hide the log"), ("battle_option", 8, 8, "", "Add a tab"), ("stickoff", 14, 9, "", "Detach"), ("battle_option2", 9, 9, "", "Options")] {
                right -= width + 2;
                window.push_str(&format!(
                    r#"<button type="button" class="a b{}" style="{};--n:url('{}');--o:url('{}');--p:url('{}')" data-click="{click}" title="{title}"></button>"#,
                    if click.is_empty() { " dim" } else { "" },
                    at(right, 4, width, tall),
                    picture(&format!("basic_interface/{name}_a.bmp")),
                    picture(&format!("basic_interface/{name}_b.bmp")),
                    picture(&format!("basic_interface/{name}_b.bmp")),
                    title = tr(title),
                ));
            }
        }
        window.push_str(&format!(r#"<div class="a" style="{};background:#8e8e8e"></div>"#, at(4, base, 600 - 8, 1)));
        window.push_str(&format!(
            r#"<form data-submit="chat" class="a" style="{};background:url('{}') 0 0 no-repeat">
                <input class="a in" id="chat-to" maxlength="23" autocomplete="off" style="{};background:#f0f0f0" title="Whisper to">
                <div class="a" style="{};background:url('{}') 0 0 no-repeat"></div>
                <input class="a in" id="chat-input" maxlength="70" autocomplete="off" style="{}">
                <button type="button" class="a b" style="{};--n:url('{off}');--o:url('{on}');--p:url('{on}')" data-click="chat-size" title="Change the size (F10)"></button>
                <button type="button" class="a b" style="{};--n:url('{off}');--o:url('{on}');--p:url('{on}')" data-click="chat-mini" title="Hide or show the log"></button>
                <button type="submit" hidden></button>
            </form>"#,
            at(0, base, 600, 24),
            picture("basic_interface/dialog_bg.bmp"),
            at(6, 4, 90, 16),
            at(97, 3, 8, 18),
            picture("basic_interface/dialog_btn0.bmp"),
            at(110, 4, 460, 16),
            at(572, 7, 11, 11),
            at(586, 7, 11, 11),
            off = picture("basic_interface/sys_base_off.bmp"),
            on = picture("basic_interface/sys_base_on.bmp"),
        ));
        window
    }
}
