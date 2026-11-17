//! Before the game: the login, the servers, the characters and the making of one; and leaving.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn account_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "yes" => self.answer(true),
            "no" => self.answer(false),
            "delete" | "delete-now" => {
                if let Some(character) = self.character_in(self.selected) {
                    self.ask = Some(if what == "delete" { Ask::Reserve(character.char_id) } else { Ask::Delete(character.char_id) });
                    self.render();
                }
            }
            "cancel-delete" => {
                if let (Some(character), Some(socket)) = (self.character_in(self.selected), &self.socket) {
                    socket.send(&packets::delete_cancel(character.char_id));
                    self.wait = Some(Key::PleaseWait);
                    self.render();
                }
            }
            "race" => {
                self.make.doram = arg == "doram";
                self.make.hair_style = self.make.hair_style.min(hair_styles(self.make.doram));
                self.render();
            }
            "cancel-wait" => {
                // Whatever was being waited for is given up; a form stays as it was filled in.
                match self.screen {
                    Screen::Register | Screen::Recover | Screen::Reset(_) => {
                        self.wait = None;
                        self.render();
                    }
                    _ => self.leave(None),
                }
            }
            "save-id" => {
                self.save_id = !self.save_id;
                self.render();
            }
            "to-register" => self.show(Screen::Register),
            "to-recover" => self.show(Screen::Recover),
            "to-login" => self.show(Screen::Login),
            "slot" => {
                let Ok(slot) = arg.parse::<u8>() else { return true };
                // A press on the one already chosen changes nothing: the page stays as it is, and a second press
                // makes a double click of it.
                if self.selected == slot && self.character_in(slot).is_some() {
                    return true;
                }
                self.selected = slot;
                // An empty slot opens the making of a character at once, as in the original client.
                if self.character_in(slot).is_none() {
                    self.open(slot);
                    return true;
                }
                self.render();
            }
            "start" => {
                self.chime();
                self.open(self.selected)
            }
            "sex" => {
                self.make.sex = u8::from(arg == "1");
                self.render();
            }
            "hair" => {
                if let Some(style) = arg.parse().ok().filter(|style| (1..=hair_styles(self.make.doram)).contains(style)) {
                    self.make.hair_style = style;
                    self.render();
                }
            }
            "colour" => {
                if let Some(colour) = arg.parse().ok().filter(|colour| *colour < HAIR_COLOURS) {
                    self.make.hair_colour = colour;
                    self.render();
                }
            }
            "turn" => {
                let step = if arg == "-1" { 7 } else { 1 };
                self.make.direction = (self.make.direction + step) % 8;
                self.render();
            }
            "back" => self.show(Screen::CharSelect),
            "server" => {
                let picked = arg.parse().unwrap_or(0);
                // A press on the one already picked leaves the page as it is, so a second makes a double click.
                if picked == self.server_picked {
                    return true;
                }
                self.server_picked = picked;
                self.render();
            }
            "server-ok" => {
                self.chime();
                self.enter_server()
            }
            "server-cancel" | "exit" => self.leave(None),
            // In the game, leaving first goes back to the characters: another one may be wanted. The server
            // says when the character is out of the game; from the characters, leaving goes out altogether.
            "logout" if matches!(self.screen, Screen::Game) => {
                self.esc = 0;
                self.refresh_dialog();
                if let Some(socket) = &self.socket {
                    socket.send(&npc::restart(true));
                }
            }
            "logout" => self.leave(None),
            _ => return false,
        }
        true
    }
}
