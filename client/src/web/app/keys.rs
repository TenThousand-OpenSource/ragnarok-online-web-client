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
            return match key {
                "Enter" => {
                    self.answer(true);
                    true
                }
                "Escape" => {
                    self.answer(false);
                    true
                }
                _ => false,
            };
        }
        if matches!(self.screen, Screen::Game) {
            return self.game_key(key, code, with);
        }
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
                self.server_picked = if key == "ArrowUp" { self.server_picked.saturating_sub(1) } else { (self.server_picked + 1).min(servers.saturating_sub(1)) };
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
        let writing = chat.as_ref().is_some_and(|input| web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active == **input));
        let chord = Chord { key: hotkeys::key_of(code).unwrap_or(0), with };
        // A row of the Shortcut Settings window is picked: the key is its new key, whatever it did before.
        if self.keys.open && self.keys.row.is_some() {
            self.assign_key(chord);
            return true;
        }
        // Letters and digits typed into a line of text are text, not keys of the game.
        let typing = web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active.tag_name() == "INPUT");
        let bound = self.bound_to(chord).filter(|_| !(typing && with == 0 && !(112..=123).contains(&chord.key)));
        match (bound, key) {
            (Some((hotkeys::table::INTERFACE, place)), _) => match place {
                hotkeys::interface::BASIC_INFO => self.hero.basic_small = !self.hero.basic_small,
                hotkeys::interface::STATUS => self.hero.status_open = !self.hero.status_open,
                hotkeys::interface::INVENTORY => self.bag.open = !self.bag.open,
                hotkeys::interface::EQUIPMENT => self.bag.equipment_open = !self.bag.equipment_open,
                hotkeys::interface::SKILLS => self.skills.open = !self.skills.open,
                // The pet window opens only for one who has a pet out (the client's `WID_PETINFOWND`).
                hotkeys::interface::PET => self.pets.open = !self.pets.open && self.pets.pet.is_some(),
                hotkeys::interface::SIT => {
                    // One who walks does not sit down: the key does nothing until the character stands. (The
                    // server would stop the walk and seat it; the rule is this side's.)
                    if world::walking() && !self.hero.sitting {
                        return true;
                    }
                    if let Some(socket) = &self.socket {
                        socket.send(&units::sit(!self.hero.sitting));
                    }
                    return true;
                }
                // The bar runs through its sizes, as the client's does: a row more each time, then away, then
                // one row again.
                hotkeys::interface::HOTKEY_BAR => {
                    if self.skills.hotkeys_hidden {
                        self.skills.hotkeys_hidden = false;
                    } else if self.skills.hotkey_rows >= 3 {
                        (self.skills.hotkey_rows, self.skills.hotkeys_hidden) = (0, true);
                    } else {
                        self.skills.hotkey_rows += 1;
                    }
                }
                hotkeys::interface::CHAT_SIZE => self.chat_resize(),
                // An entry of the client this client does nothing for yet.
                _ => return false,
            },
            // A slot of the hotkey bar: the first table's places are the bar's slots, row after row.
            (Some((hotkeys::table::BAR, slot)), _) => {
                self.use_it(&format!("slot:{slot}"));
                return true;
            }
            (_, "Escape") if self.bag.throwing.is_some() => {
                self.bag.throwing = None;
                self.refresh_throw();
                return true;
            }
            // Enter goes to the chat's line; from there it sends (the form does that).
            // In a field of some window Enter belongs to that window.
            (_, "Enter") if web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active.tag_name() == "INPUT" && active.id() != "chat-input") => {
                return false;
            }
            (_, "Enter") if !writing => {
                if let Some(input) = chat {
                    let _ = input.focus();
                }
                return true;
            }
            // Escape lets go of a skill being aimed.
            (_, "Escape") if world::aiming() => {
                world::aim(None, 0);
                self.skills.aimed = None;
                return true;
            }
            // Escape opens the window of game options and closes it, unless an NPC is being talked to.
            (_, "Escape") if !writing && self.dialog.is_none() => {
                self.esc = u8::from(self.esc == 0);
                self.refresh_dialog();
                return true;
            }
            (_, "Escape") if writing => {
                if let Some(input) = chat {
                    let _ = input.blur();
                }
                return true;
            }
            _ => return false,
        }
        self.render();
        true
    }
}
