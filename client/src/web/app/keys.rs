//! The keyboard: on the windows before the game, and in the game.

use super::*;

impl App {
    /// A key was pressed. Returns whether it was used, so the browser does nothing else with it.
    pub(super) fn key(&mut self, key: &str, code: &str, alt: bool) -> bool {
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
            return self.game_key(key, code, alt);
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

    /// The game's own keys, as the original client has them: Alt+V and Alt+A for the Basic Information and
    /// Status windows, F12 for the hotkey bar, F10 for the chat's size, Enter to write.
    pub(super) fn game_key(&mut self, key: &str, code: &str, alt: bool) -> bool {
        let chat = element("chat-input").and_then(|input| input.dyn_into::<web_sys::HtmlElement>().ok());
        let writing = chat.as_ref().is_some_and(|input| web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active == **input));
        match (key, code, alt) {
            (_, "KeyV", true) => self.basic_small = !self.basic_small,
            (_, "KeyA", true) => self.status_open = !self.status_open,
            (_, "KeyE", true) => self.inventory_open = !self.inventory_open,
            (_, "KeyQ", true) => self.equipment_open = !self.equipment_open,
            (_, "KeyS", true) => self.skills_open = !self.skills_open,
            // The pet window opens only for one who has a pet out (the client's `WID_PETINFOWND`).
            (_, "KeyJ", true) => self.pet_open = !self.pet_open && self.pet.is_some(),
            // F1 to F9 use the first row of the hotkey bar.
            (_, "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9", false) => {
                let slot = code[1..].parse::<usize>().unwrap_or(1) - 1;
                self.use_it(&format!("slot:{slot}"));
                return true;
            }
            ("Escape", ..) if self.throwing.is_some() => {
                self.throwing = None;
                self.refresh_throw();
                return true;
            }
            // Insert sits down and stands up.
            ("Insert", ..) => {
                if let Some(socket) = &self.socket {
                    socket.send(&units::sit(!self.sitting));
                }
                return true;
            }
            // F12 runs the bar through its sizes, as the client does: a row more each time, then away, then one
            // row again.
            ("F12", ..) => {
                if self.hotkeys_hidden {
                    self.hotkeys_hidden = false;
                } else if self.hotkey_rows >= 3 {
                    (self.hotkey_rows, self.hotkeys_hidden) = (0, true);
                } else {
                    self.hotkey_rows += 1;
                }
            }
            ("F10", ..) => self.chat_resize(),
            // Enter goes to the chat's line; from there it sends (the form does that).
            // In a field of some window Enter belongs to that window.
            ("Enter", ..) if web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active.tag_name() == "INPUT" && active.id() != "chat-input") => {
                return false;
            }
            ("Enter", ..) if !writing => {
                if let Some(input) = chat {
                    let _ = input.focus();
                }
                return true;
            }
            // Escape lets go of a skill being aimed.
            ("Escape", ..) if world::aiming() => {
                world::aim(None, 0);
                self.aimed = None;
                return true;
            }
            // Escape opens the window of game options and closes it, unless an NPC is being talked to.
            ("Escape", ..) if !writing && self.dialog.is_none() => {
                self.esc = u8::from(self.esc == 0);
                self.refresh_dialog();
                return true;
            }
            ("Escape", ..) if writing => {
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
