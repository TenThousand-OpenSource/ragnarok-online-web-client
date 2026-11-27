//! The keyboard: on the windows before the game, and in the game.

use super::*;

impl App {
    /// A key was pressed. Returns whether it was used, so the browser does nothing else with it.
    pub(super) fn key(&mut self, key: &str, code: &str, with: u16) -> bool {
        if self.message.is_some() {
            if matches!(key, "Enter" | "Escape" | " ") {
                self.message = None;
                self.render();
                return true;
            }
            return false;
        }
        if self.wait.is_some() {
            return false;
        }
        if self.ask.is_some() {
            return self.ask_key(key);
        }
        if matches!(self.screen, Screen::Game) {
            return self.game_key(key, code, with);
        }
        self.way_in_key(key)
    }

    /// A key while a question waits over the window: Enter is its OK, Escape its Cancel.
    fn ask_key(&mut self, key: &str) -> bool {
        match key {
            "Enter" => {
                self.answer(true);
                true
            }
            "Escape" => {
                self.answer(false);
                true
            }
            _ => false,
        }
    }

    /// A key on the windows before the game: the arrows pick a server or a character, Enter enters, Escape
    /// goes back.
    fn way_in_key(&mut self, key: &str) -> bool {
        match (&self.screen, key) {
            (Screen::CharSelect, "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown") => {
                let step: i16 = match key {
                    "ArrowLeft" => -1,
                    "ArrowRight" => 1,
                    "ArrowUp" => -5,
                    _ => 5,
                };
                let moved = i16::from(self.selected) + step;
                if (0..i16::from(SLOTS_PER_PAGE)).contains(&moved) {
                    self.selected = moved as u8;
                    self.render();
                }
                true
            }
            (Screen::ServerSelect, "Enter") => {
                self.enter_server();
                true
            }
            (Screen::ServerSelect, "Escape") => {
                self.leave(None);
                true
            }
            (Screen::ServerSelect, "ArrowUp" | "ArrowDown") => {
                let servers = self.account.as_ref().map_or(0, |account| account.char_servers.len());
                self.server_picked = if key == "ArrowUp" { self.server_picked.saturating_sub(1) } else { (self.server_picked + 1).min(
                    servers.saturating_sub(1)
                ) };
                self.render();
                true
            }
            (Screen::CharSelect, "Enter") => {
                self.open(self.selected);
                true
            }
            (Screen::CharSelect, "Escape") => {
                self.leave(None);
                true
            }
            (Screen::CharCreate(_), "Escape") => {
                self.show(Screen::CharSelect);
                true
            }
            (Screen::Register | Screen::Recover, "Escape") => {
                self.show(Screen::Login);
                true
            }
            _ => false,
        }
    }

    /// The game's keys. What the client's tables bind (its `hotkey.lub`: Alt+V for Basic Information, Alt+E
    /// for the inventory, F1 to F9 for the hotkey bar, Insert to sit and the rest) goes by what each entry is
    /// bound to for this player (`bound_to`), so a key set in the Shortcut Settings window, or this client's
    /// key for one a Mac lacks, does the same; Enter and Escape are the client's own, bound nowhere.
    pub(super) fn game_key(&mut self, key: &str, code: &str, with: u16) -> bool {
        let chat = element("chat-input").and_then(|input| input.dyn_into::<web_sys::HtmlElement>().ok());
        let writing = chat.as_ref().is_some_and(
            |input| crate::web::document().and_then(|document| document.active_element()).is_some_and(|active| active == **input)
        );
        let chord = Chord { key: crate::hotkeys::key_of(code).unwrap_or(0), with };
        // While a recording plays the keys are the player's, and what the client leaves of the others.
        if self.replay.playing.is_some() {
            return self.replay_key(key, chord);
        }
        // Escape while the box of the away message is open switches the message off, and then does what it
        // does otherwise (`UIWindowMgr::ProcessKeyDown`, `VK_ESCAPE`: `m_autoReplyEnabled = false`). The box
        // itself has no place in the order of windows, and stays.
        if key == "Escape" && self.whispers.away_asked {
            self.whispers.away = false;
        }
        // A row of the Shortcut Settings window is picked: the key is its new key, whatever it did before.
        if self.keys.open && self.keys.row.is_some() {
            self.assign_key(chord);
            return true;
        }
        // An NPC is talked to: Enter and the arrows up and down are the talk's, wherever the keyboard is.
        if self.talk_key(key) {
            return true;
        }
        // Letters and digits typed into a line of text (or into the text of a letter) are text, not keys of the game.
        let typing = crate::web::document().and_then(|document| document.active_element()).is_some_and(
            |active| matches!(active.tag_name().as_str(), "INPUT" | "TEXTAREA")
        );
        let bound = self.bound_to(chord).filter(|_| !(typing && with == 0 && !(112..=123).contains(&chord.key)));
        match (bound, key) {
            // Tab ticks the World Map's box while the map is open.
            _ if self.worldmap_key(key, with) => true,
            // The arrows turn the pages of a book that is open, unless a line of text is being typed.
            _ if !typing && self.book_key(key) => true,
            (Some((crate::hotkeys::table::INTERFACE, place)), _) => self.interface_key(place),
            // A line of the Shortcut List (Alt+1 to Alt+0: the table's first ten entries). The ten after them
            // are the flags, emotions bought with cash, which this client does not have.
            (Some((crate::hotkeys::table::MACROS, place)), _) => {
                if place < 10 {
                    self.shortcut_pressed(place);
                }
                place < 10
            }
            // Scroll Lock makes a picture of the screen as Print Screen does (`MM_SCREENSHOT`).
            _ if chord == (Chord { key: 145, with: 0 }) && !typing => {
                self.screenshot("");
                true
            }
            // A slot of the hotkey bar: the first table's places are the bar's slots, row after row.
            (Some((crate::hotkeys::table::BAR, slot)), _) => {
                self.used(Use::Slot(slot));
                true
            }
            (_, "Escape") => self.escape_key(chat, writing),
            // Enter goes to the chat's line; from there it sends (the form does that).
            // In a field of some window Enter belongs to that window.
            (_, "Enter") if crate::web::document().and_then(|document| document.active_element()).is_some_and(
                |active| matches!(active.tag_name().as_str(), "INPUT" | "TEXTAREA") && active.id() != "chat-input"
            ) => {
                false
            }
            (_, "Enter") if !writing => {
                if let Some(input) = chat {
                    let _ = input.focus();
                }
                true
            }
            _ => false,
        }
    }

    /// Escape, when no key bound for this player took it: the first of these that there is. Whether the key
    /// was used. `chat` is the chat's line, and `writing` whether it has the keyboard.
    fn escape_key(&mut self, chat: Option<web_sys::HtmlElement>, writing: bool) -> bool {
        // The box that asks how many to drop goes away.
        if self.bag.throwing.is_some() {
            self.bag.throwing = None;
            self.redraw(&[&wnd::THROW]);
            return true;
        }
        // Escape lets go of a skill being aimed.
        if world::aiming() {
            world::aim(None, 0);
            self.skills.aimed = None;
            return true;
        }
        // Escape closes the window in front, and with none open it opens the window of game options (or
        // closes that), unless an NPC is being talked to: the client's `UIWindowMgr::ProcessHotkeyAction`,
        // at `VK_ESCAPE` (`CloseWindow` of the first of `windowHistory`, else `WID_ESCOPTIONWND`). The
        // client does so whether or not the chat's line has the cursor (`ProcessKeyDown` hands it every
        // Escape, and its line of chat does nothing with one), so it is the same here; the line keeps
        // the keyboard and what was typed.
        if self.dialog.is_none() {
            if !self.close_front() {
                self.esc = u8::from(self.esc == 0);
                self.esc_redrawn();
            }
            return true;
        }
        // (While an NPC is talked to, Escape only lets go of the chat's line.)
        if let Some(input) = chat.filter(|_| writing) {
            let _ = input.blur();
        }
        writing
    }

    /// A key that does what a press on the page does. The key is used.
    fn pressed_key(&mut self, press: impl Into<crate::press::Press>) -> bool {
        self.pressed(press);
        true
    }

    /// A key bound to an entry of the client's table of the interface's keys (`crate::hotkeys::interface`): a window
    /// opened or closed, the character seated, the hotkey bar and the chat run through their sizes. Returns
    /// whether the key was used.
    pub(super) fn interface_key(&mut self, place: usize) -> bool {
        match place {
            crate::hotkeys::interface::BASIC_INFO => return self.pressed_key(crate::press::status::Press::BasicSize),
            crate::hotkeys::interface::STATUS => return self.pressed_key(crate::press::status::Press::Window),
            crate::hotkeys::interface::INVENTORY => return self.pressed_key(crate::press::inventory::Press::Window),
            crate::hotkeys::interface::EQUIPMENT => return self.pressed_key(crate::press::inventory::Press::Equipment),
            crate::hotkeys::interface::SKILLS => return self.pressed_key(crate::press::skills::Press::Window),
            crate::hotkeys::interface::GUILD => return self.pressed_key(crate::press::guild::Press::Window),
            // The pet window opens only for one who has a pet out (the client's `WID_PETINFOWND`).
            crate::hotkeys::interface::PET => self.pets.open = !self.pets.open && self.pets.pet.is_some(),
            // The windows of the homunculus and of the mercenary open only while that one is out, and the
            // key that has one stand by does nothing else (`UIWindowMgr`, at `HK_HOMUNWND_ONOFF` and on).
            crate::hotkeys::interface::HOMUNCULUS => self.homunculus.open = !self.homunculus.open && self.homunculus.shown(),
            crate::hotkeys::interface::MERCENARY => self.mercenary.open = !self.mercenary.open && self.mercenary.shown(),
            crate::hotkeys::interface::HOMUNCULUS_STAND_BY | crate::hotkeys::interface::MERCENARY_STAND_BY => {
                self.companion_stands_by(if place == crate::hotkeys::interface::MERCENARY_STAND_BY {
                    crate::ai::Kind::Mercenary
                } else {
                    crate::ai::Kind::Homunculus
                });
                return true;
            }
            crate::hotkeys::interface::SIT => {
                // One who walks does not sit down: the key does nothing until the character stands. (The
                // server would stop the walk and seat it; the rule is this side's.)
                if world::walking() && !self.hero.sitting {
                    return true;
                }
                self.send(&net::actors::sit(!self.hero.sitting));
                return true;
            }
            // The bar runs through its sizes, as the client's does: a row more each time, then away, then
            // one row again.
            crate::hotkeys::interface::HOTKEY_BAR => self.hotkey_bar_stepped(),
            crate::hotkeys::interface::CHAT_SIZE => self.chat_resize(),
            crate::hotkeys::interface::QUESTS => return self.quest_key(),
            crate::hotkeys::interface::CHAT_ROOM => {
                self.room_make_asked();
                return true;
            }
            crate::hotkeys::interface::MAIL => return self.mail_key(),
            crate::hotkeys::interface::BANK => return self.bank_key(),
            crate::hotkeys::interface::INSTANCE => return self.pressed_key(crate::press::instances::Press::Window),
            crate::hotkeys::interface::CASH_SHOP => return self.cashshop_key(),
            crate::hotkeys::interface::CART => return self.cart_key(),
            crate::hotkeys::interface::FRIENDS | crate::hotkeys::interface::PARTY => return self.messenger_key(place == crate::hotkeys::interface::FRIENDS),
            crate::hotkeys::interface::WORLD_MAP => return self.pressed_key(crate::press::worldmap::Press::Window),
            crate::hotkeys::interface::EMOTIONS => return self.pressed_key(crate::press::emotions::Press::Window),
            crate::hotkeys::interface::SHORTCUTS => return self.pressed_key(crate::press::emotions::Press::Shortcuts),
            crate::hotkeys::interface::TIPS => return self.pressed_key(crate::press::tips::Press::Window),
            crate::hotkeys::interface::NAVIGATION => return self.pressed_key(crate::press::navigation::Press::Window),
            crate::hotkeys::interface::ACHIEVEMENTS => return self.pressed_key(crate::press::achievements::Press::Window),
            crate::hotkeys::interface::REPUTATION => return self.pressed_key(crate::press::reputation::Press::Window),
            crate::hotkeys::interface::MENU => return self.pressed_key(crate::press::general::Press::Menu),
            // The recorder, as its icon of the menu (`HK_REPLAY_ONOFF`).
            crate::replay::playing::RECORDER => return self.pressed_key(crate::press::replay::Press::Window),
            crate::hotkeys::interface::FRIEND_SETUP => return self.pressed_key(crate::press::friends::Press::Setup),
            // The party's setup opens only for one who is in a party (`WID_PARTYSETTINGWND_PARTYCOUNT`).
            crate::hotkeys::interface::PARTY_SETUP => return self.party.members.is_empty() || self.pressed_key(crate::press::party::Press::Setup),
            // The minimap runs through shown, shown fainter and away; not while the World Map is open.
            crate::hotkeys::interface::MINIMAP => {
                if self.worldmap.open {
                    return true;
                }
                self.worldmap.minimap = (self.worldmap.minimap + 1) % 3;
            }
            // The Game Settings open on their page of commands, or close if they were open on it
            // (`ToggleGameSettingsTab4`).
            crate::hotkeys::interface::COMMANDS => (self.settings.open, self.settings.tab) = (!(self.settings.open && self.settings.tab == 4), 4),
            crate::hotkeys::interface::CHAT => self.chat.hidden = !self.chat.hidden,
            crate::hotkeys::interface::SCREENSHOT => {
                self.screenshot("");
                return true;
            }
            // Other players are drawn faintly, or as they are (`ToggleOption` of `TT_HIDE_PLAYER_ON_OFF`).
            crate::hotkeys::interface::HIDE_PLAYERS => {
                self.switch_option("TT_HIDE_PLAYER_ON_OFF");
                self.switched(data::option_says("TT_HIDE_PLAYER_ON_OFF", self.option("TT_HIDE_PLAYER_ON_OFF")));
                return true;
            }
            // An entry of the client this client does nothing for yet.
            _ => return false,
        }
        self.render();
        true
    }
}
