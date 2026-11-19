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
        let items: Vec<u32> = crate::links::items::pieces(words).into_iter().filter_map(|piece| if let crate::links::items::Piece::Item(link) = piece { Some(link.item) } else { None }).collect();
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

    /// A press of this system (`crate::press::chat`).
    pub(super) fn chat_pressed(&mut self, press: crate::press::chat::Press) {
        use crate::press::chat::Press;
        match press {
            Press::Tab(tab) => {
                self.chat.tab = tab;
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

    /// The button that adds a tab (`ID_ADD_CHAT_TAB`): with ten already the client says so in a box (line 1578
    /// of its message table); else the tab is added, listing nothing yet, and opened. One that no longer fits
    /// among the tabs (the tabs may take the window's width less 90) opens in a window of its own at once.
    fn chat_tab_added(&mut self) {
        let total = self.own_tabs().len();
        if total >= chat_tabs::MOST {
            self.message = Some(Self::words(1578, "Cannot add more."));
            return self.render();
        }
        let mut tab = Tab::added(total);
        let inside = self.chat.tabs.iter().filter(|tab| tab.own.is_none()).count() as i32;
        if (inside + 1) * 71 > 600 - 90 {
            tab.own = Some(chat_tabs::own_place(total));
        } else {
            self.chat.tab = total;
        }
        self.chat.tabs.push(tab);
        self.tabs_changed();
    }

    /// A tab is taken away with what it was set to list (`ID_MINI`): of those in the window the last one
    /// stays.
    fn chat_tab_removed(&mut self, tab: usize) {
        let tabs = self.own_tabs();
        let Some(removed) = tabs.get(tab) else { return };
        if removed.own.is_none() && tabs.iter().filter(|tab| tab.own.is_none()).count() <= 1 {
            return;
        }
        tabs.remove(tab);
        self.chat.options = None;
        self.chat.tab = self.chat.tab.saturating_sub(usize::from(self.chat.tab >= tab && self.chat.tab > 0));
        self.tabs_changed();
    }

    /// The open tab goes out into a window of its own (`UM_DETACH_CHAT_TAB`): not while the window is locked,
    /// and not the only tab.
    fn chat_tab_detached(&mut self) {
        let locked = self.chat.locked;
        let tabs = self.chat_tabs();
        let open = self.chat_tab_open(&tabs);
        if locked || tabs.iter().filter(|tab| tab.own.is_none()).count() <= 1 {
            return;
        }
        self.own_tabs()[open].own = Some(chat_tabs::own_place(open));
        forget_moved(wnd::CHAT_OWN[open.min(chat_tabs::MOST - 1)].id);
        self.tabs_changed();
    }

    /// A tab's own window goes back among the tabs (`UM_ATTACH_CHAT_TAB`), if there is room for one more;
    /// else the client says in the chat that there is not (line 1776, in red).
    fn chat_tab_attached(&mut self, tab: usize) {
        let inside = self.own_tabs().iter().filter(|tab| tab.own.is_none()).count() as i32;
        if (inside + 1) * 71 > 600 - 90 {
            return self.chat_line(Self::words(1776, "Failed to add because you have reached the limit."), RED);
        }
        if let Some(tab) = self.chat.tabs.get_mut(tab) {
            tab.own = None;
        }
        self.chat.tab = tab;
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
    pub(in crate::web::app) fn chat_typed(&mut self) {
        let words = field("chat-input");
        let words = words.trim();
        // The client's own commands come first; everything else is said (the server reads its
        // @commands).
        if !words.is_empty() && self.typed_command(words) {
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
