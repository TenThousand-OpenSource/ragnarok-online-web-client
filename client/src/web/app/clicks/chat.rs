//! The chat window, its tabs' own windows, and the window that sets what a tab lists.

use super::*;
use crate::chat_tabs::{self, Tab};

/// Where the browser keeps the chat's tabs, as the client writes them to `SaveData\ChatWndInfo_U.lua`.
const KEPT: &str = "roweb.chat";

impl App {
    /// Reads the tabs the browser keeps.
    pub(in crate::web::app) fn load_chat(&mut self) {
        if let Some((tabs, locked)) = stored(KEPT).and_then(|text| chat_tabs::read(&text)) {
            (self.chat.tabs, self.chat.locked) = (tabs, locked);
        }
        // (A tab's own window kept at a place that is out of sight is shown on the screen all the same:
        // `chat_tabs::own_shown`, asked as the window is placed, `hud/windows/chat.rs`.)
    }

    /// The tabs made the player's own (until one is changed they are the client's, read from its table), to be
    /// changed; `kept` keeps them afterwards.
    fn own_tabs(&mut self) -> &mut Vec<Tab> {
        if self.chat.tabs.is_empty() {
            self.chat.tabs = data::chat_tabs_start();
        }
        &mut self.chat.tabs
    }

    /// Keeps the tabs in the browser and writes the screen again.
    fn tabs_changed(&mut self) {
        store(KEPT, Some(&chat_tabs::written(&self.chat.tabs, self.chat.locked)));
        self.render();
    }

    /// The items a line of chat links to (`<ITEML>`) are asked about, so that the line can be written with
    /// their names.
    pub(in crate::web::app) fn chat_links_wanted(&mut self, words: &str) {
        let items: Vec<u32> = crate::links::items::pieces(words)
            .into_iter()
            .filter_map(|piece| if let crate::links::items::Piece::Item(link) = piece { Some(link.item) } else { None })
            .collect();
        if !items.is_empty() {
            self.chat.linked = true;
            data::want_items(items.into_iter(), || dispatch(Action::ItemsKnown));
        }
    }

    /// Items became known: the lines that link to items are written again with their names.
    pub(in crate::web::app) fn chat_links_named(&mut self) {
        if std::mem::take(&mut self.chat.linked) {
            self.refresh_chat();
        }
    }

    /// Says words aloud, to everyone in sight; `false` when there is nothing to say or no one to say it to.
    pub(in crate::web::app) fn say_aloud(&mut self, words: &str) -> bool {
        let (false, Some(socket), Some(game)) = (words.is_empty(), &self.socket, &self.game) else { return false };
        socket.send(&packets::say(&game.name, words));
        true
    }

    /// With the battleground's own chat switched on (`/battlechat`), a line that is no command goes to it,
    /// whomever its mark was for (`CGameMode`'s `TT_NORMAL`: `MM_CHAT_BATTLEFIELD` before the party, the
    /// guild and saying aloud). `false` when the chat is off or the line is a command.
    fn battle_chat_said(&mut self, words: &str) -> bool {
        if !self.option("TT_BATTLECHAT_ON_OFF") || words.starts_with('/') {
            return false;
        }
        if let (Some(words), Some(name)) = (crate::battleground::chat_words(words), self.game.as_ref().map(|game| game.name.clone())) {
            self.send_on_map(&crate::net::battleground::chat(&name, words));
        }
        true
    }

    /// A press of this system (`crate::press::chat`).
    pub(super) fn chat_pressed(&mut self, press: crate::press::chat::Press) {
        use crate::press::chat::Press;
        match press {
            Press::Tab(tab) => {
                (self.chat.tab, self.chat.renaming) = (tab, None);
                // The window that sets what a tab lists follows the tab that is open.
                if self.chat.options.is_some() {
                    self.chat.options = Some(tab);
                }
                self.render();
            }
            Press::Lock => {
                self.chat.locked = !self.chat.locked;
                self.own_tabs();
                self.tabs_changed();
            }
            Press::Size => {
                self.chat_resize();
                self.render();
            }
            // Hides the log, leaving the input line, or brings it back as tall as it was.
            Press::Mini => {
                if self.chat.rows > 0 {
                    self.chat.rows_kept = self.chat.rows;
                    self.chat.rows = 0;
                } else {
                    self.chat.rows = self.chat.rows_kept.max(1);
                }
                self.render();
            }
            Press::Add => self.chat_tab_added(),
            Press::Remove => {
                let tabs = self.chat_tabs();
                self.chat_tab_removed(self.chat_tab_open(&tabs));
            }
            Press::OwnRemove(tab) => self.chat_tab_removed(tab),
            Press::Detach => self.chat_tab_detached(),
            Press::Attach(tab) => self.chat_tab_attached(tab),
            Press::Options | Press::OwnOptions(_) => {
                let tabs = self.chat_tabs();
                let tab = if let Press::OwnOptions(tab) = press { tab } else { self.chat_tab_open(&tabs) };
                // Pressed again for the same tab it closes the window (`ID_OPTION`).
                self.chat.options = (self.chat.options != Some(tab)).then_some(tab);
                self.redraw(&[&wnd::CHAT_OPTIONS]);
            }
            Press::Lists(kind) => self.chat_lists(Some(kind)),
            Press::ListsAll => self.chat_lists(None),
            Press::OptionsClose => {
                self.chat.options = None;
                self.redraw(&[&wnd::CHAT_OPTIONS]);
            }
            // What is known of the item of a link (`UM_ITEMEXPLANATION`), as the right button on one shows it.
            Press::Link(item) => self.handle(Action::Info(item, true)),
        }
    }

    /// Keeps the tabs after one of them changed its place among them: the windows that follow a tab by its
    /// place (what it lists, its name being typed) are put away, and no own window is where an earlier one
    /// of its place was dragged to.
    fn tabs_moved(&mut self) {
        (self.chat.options, self.chat.renaming) = (None, None);
        for window in wnd::CHAT_OWN {
            forget_moved(window.id);
        }
        self.tabs_changed();
    }

    /// The button that adds a tab (`ID_ADD_CHAT_TAB`): with ten already the client says so in a box (line 1578
    /// of its message table); else the tab is added, listing nothing yet, and opened. One that no longer fits
    /// among the tabs (the tabs may take the window's width less 90) opens in a window of its own at once.
    /// (The client asks whether one more tab as wide as those there are fits; here the new tab's own name
    /// counts, as it does once the tab is there.)
    fn chat_tab_added(&mut self) {
        let total = self.own_tabs().len();
        if total >= chat_tabs::MOST {
            self.message = Some(Self::words(1578, "Cannot add more."));
            return self.render();
        }
        let mut tab = Tab::added(total);
        let inside = chat_tabs::inside(&self.chat.tabs);
        // (The tabs are as wide as the widest name of them, the new one's too: `UITabControl::AddTab`.)
        let mut with = self.chat.tabs.clone();
        with.insert(inside, tab.clone());
        if chat_tabs::inside(&with) as i32 * Self::chat_tab_wide(&with) <= chat_tabs::WINDOW - 90 {
            self.chat.tabs.insert(inside, tab);
            self.chat.tab = inside;
        } else {
            tab.own = Some(chat_tabs::own_place(total - inside));
            self.chat.tabs.push(tab);
        }
        self.tabs_moved();
    }

    /// A tab is taken away with what it was set to list (`ID_MINI`): of those in the window the last one
    /// stays, and the tab before the one taken is the open one then.
    fn chat_tab_removed(&mut self, tab: usize) {
        let tabs = self.own_tabs();
        let Some(removed) = tabs.get(tab) else { return };
        if removed.own.is_none() && chat_tabs::inside(tabs) <= 1 {
            return;
        }
        if tabs.remove(tab).own.is_none() {
            self.chat.tab = tab.saturating_sub(1);
        }
        self.tabs_moved();
    }

    /// The open tab goes out into a window of its own (`ID_STICK_OFF`, `UM_DETACH_CHAT_TAB`): not while the
    /// window is locked, and not the only tab. The window opens where the client opens the next one
    /// (`chat_tabs::own_place`).
    fn chat_tab_detached(&mut self) {
        let open = self.chat_tab_open(&self.chat_tabs());
        let place = chat_tabs::own_place(self.own_tabs().len() - chat_tabs::inside(&self.chat.tabs));
        if self.chat.locked {
            return;
        }
        if let Some(open) = chat_tabs::detached(&mut self.chat.tabs, open, place) {
            self.chat.tab = open;
            self.tabs_moved();
        }
    }

    /// A tab held by the pointer was pulled out of the window (`UITabControl::OnMouseMove`, then
    /// `UM_DETACH_CHAT_TAB` from the tabs themselves): it goes into a window of its own, which stands where
    /// the tab was, at the top of the chat window, and is held by the pointer from there on. Let go while it
    /// is still over the chat, it goes back (`chat_own_dropped`).
    pub(in crate::web::app) fn chat_tab_torn(&mut self, tab: usize, pointer: (f64, f64)) {
        let Some((x, y, ..)) = Self::window_rect(wnd::CHAT.id) else { return };
        if self.chat.locked {
            return;
        }
        let before = self.own_tabs().iter().take(tab).filter(|tab| tab.own.is_none()).count() as i32;
        let place = (x + Self::chat_tab_wide(&self.chat.tabs) * before, y, chat_tabs::OWN_SIZE.0, chat_tabs::OWN_SIZE.1);
        if let Some(open) = chat_tabs::detached(&mut self.chat.tabs, tab, place) {
            self.chat.tab = open;
            self.tabs_moved();
            drag_begun(wnd::CHAT_OWN[(self.chat.tabs.len() - 1).min(chat_tabs::MOST - 1)].id, pointer);
        }
    }

    /// A tab's own window was let go where it was moved to (`UISubChatHisWnd::OnLBtnUp`): it is kept there,
    /// and with its left side within the chat window's width and its bottom within the chat window's height
    /// it goes back among the tabs.
    pub(in crate::web::app) fn chat_own_dropped(&mut self, tab: usize) {
        let (Some(window), Some(own)) = (wnd::CHAT_OWN.get(tab), Self::window_rect(wnd::CHAT_OWN[tab.min(chat_tabs::MOST - 1)].id)) else { return };
        let Some(kept) = self.chat.tabs.get_mut(tab).and_then(|tab| tab.own.as_mut()) else { return };
        (kept.0, kept.1) = (own.0, own.1);
        forget_moved(window.id);
        if Self::window_rect(wnd::CHAT.id).is_some_and(|chat| !self.chat.hidden && chat_tabs::own_dropped_in(own, chat)) {
            return self.chat_tab_attached(tab);
        }
        self.tabs_changed();
    }

    /// The corner of a tab's own window was dragged (`UISubChatHisWnd::SendMsg`, `UM_RESIZE`): the window
    /// takes the size, unless the chat is locked.
    pub(in crate::web::app) fn chat_own_sized(&mut self, tab: usize, width: i32, height: i32) {
        if self.chat.locked {
            return;
        }
        if let Some(own) = self.chat.tabs.get_mut(tab).and_then(|tab| tab.own.as_mut()) {
            (own.2, own.3) = chat_tabs::own_resized(width, height);
            self.tabs_changed();
        }
    }

    /// A tab's own window goes back among the tabs (`ID_STICK_ON`, `UM_ATTACH_CHAT_TAB`), as the last of them
    /// and the open one: not while the chat is locked, nor when the tabs there are leave no room for one
    /// more; and when its own name, as wide as it is, times the tabs there would be, is more than the room,
    /// the chat says that there is none (line 1776, in red).
    fn chat_tab_attached(&mut self, tab: usize) {
        let wide = Self::chat_tab_wide(self.own_tabs());
        let Some(name) = self.chat.tabs.get(tab).map(|tab| crate::web::text_widths()(crate::i18n::tr(&tab.name)).ceil() as i32) else { return };
        if self.chat.locked || !chat_tabs::room(&self.chat.tabs, wide) {
            return self.tabs_changed();
        }
        if !chat_tabs::room(&self.chat.tabs, name) {
            self.tabs_changed();
            return self.chat_line(Self::words(1776, "Failed to add because you have reached the limit."), RED);
        }
        if let Some(back) = chat_tabs::attached(&mut self.chat.tabs, tab) {
            self.chat.tab = back;
            self.tabs_moved();
        }
    }

    /// A double click on a tab (`UITabControl::OnLBtnDblClk`), or on the name of a tab's own window
    /// (`UISubChatHisWnd::OnLBtnDblClk`: not while the chat is locked): its name is typed anew in its place.
    pub(in crate::web::app) fn chat_tab_named(&mut self, tab: usize) {
        let Some(own) = self.own_tabs().get(tab).map(|tab| tab.own.is_some()) else { return };
        if own && self.chat.locked {
            return;
        }
        (self.chat.tab, self.chat.renaming) = (if own { self.chat.tab } else { tab }, Some(tab));
        self.render();
        if let Some(name) = element(crate::web::hud::CHAT_TAB_NAME).and_then(|name| name.dyn_into::<HtmlInputElement>().ok()) {
            let _ = name.focus();
            name.select();
        }
    }

    /// Enter in the name of a tab being typed anew (`UITabControl::SendMsg` and `UISubChatHisWnd::SendMsg`,
    /// `UM_DEFPUSH`): nothing typed leaves the name; a name too wide is refused in a box (line 1577): in the
    /// window, one that with its 10 of room, times the tabs there, would not leave 90 of the window's width;
    /// in a window of its own, one that with those 10 would not leave 50 of that window's.
    pub(in crate::web::app) fn chat_tab_name_typed(&mut self) {
        let name = field(crate::web::hud::CHAT_TAB_NAME).trim().replace("]]", "");
        let Some(tab) = self.chat.renaming.take() else { return };
        let wide = crate::web::text_widths()(&name).ceil() as i32;
        let inside = chat_tabs::inside(&self.chat.tabs);
        match self.chat.tabs.get_mut(tab) {
            Some(_) if name.is_empty() => {}
            Some(tab) if chat_tabs::name_fits(wide, inside, tab.own.map(|own| own.2)) => tab.name = name,
            Some(_) => self.message = Some(Self::words(1577, "Not very long to change the name of the specified tab.")),
            None => {}
        }
        self.tabs_changed();
    }

    /// A row of the window that sets what a tab lists switches that kind of line; its box "all on" has the tab
    /// list every kind (`ID_CHECK`).
    fn chat_lists(&mut self, kind: Option<usize>) {
        let Some(open) = self.chat.options else { return };
        let Some(tab) = self.own_tabs().get_mut(open) else { return };
        match kind {
            Some(kind) if kind < chat_tabs::KINDS => tab.shows[kind] = !tab.shows[kind],
            Some(_) => return,
            None => tab.shows = [true; chat_tabs::KINDS],
        }
        store(KEPT, Some(&chat_tabs::written(&self.chat.tabs, self.chat.locked)));
        self.redraw(&[&wnd::CHAT_OPTIONS]);
        self.refresh_chat();
    }

    /// The chat's line was sent: a command is done, words with a name beside them are whispered, anything
    /// else is said aloud.
    ///
    /// Whatever was typed is sent with stars for a word that is not let be written (`CGameMode::SendMsg`,
    /// `MM_CHATMSG`, `crate::manner`): the client does that first of all, to a command too.
    pub(in crate::web::app) fn chat_typed(&mut self) {
        let words = data::starred(field("chat-input").trim());
        let words = words.as_str();
        // The client's own commands come first; everything else is said (the server reads its
        // @commands).
        if !words.is_empty() && (self.battle_chat_said(words) || self.typed_command(words)) {
            set_field("chat-input", "");
            return;
        }
        // A name in the small box at the left sends the words to that one alone: the line shows once
        // the server has answered (`clicks/whispers.rs`).
        let to = field("chat-to");
        let to = to.trim();
        if !words.is_empty() && !to.is_empty() && self.game.is_some() {
            self.whisper_send(to, words);
            return set_field("chat-input", "");
        }
        if self.say_aloud(words) {
            set_field("chat-input", "");
        }
    }
}
