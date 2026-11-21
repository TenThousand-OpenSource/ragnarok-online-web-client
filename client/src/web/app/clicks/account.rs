//! Before the game: the login, the servers, the characters and the making of one; and leaving.

use super::*;
use crate::press::account::{Press, Race, Sex, Turn};

impl App {
    /// A press of this system (`crate::press::account`).
    pub(super) fn account_pressed(&mut self, press: Press) {
        match press {
            Press::Yes => self.answer(true),
            Press::No => self.answer(false),
            Press::CancelWait => {
                // Whatever was being waited for is given up; a form stays as it was filled in.
                match self.screen {
                    Screen::Register | Screen::Recover | Screen::Reset(_) => {
                        self.wait = None;
                        self.render();
                    }
                    _ => self.leave(None),
                }
            }
            Press::SaveId => {
                self.save_id = !self.save_id;
                self.render();
            }
            Press::ToRegister => self.show(Screen::Register),
            Press::ToRecover => self.show(Screen::Recover),
            Press::ToLogin => self.show(Screen::Login),
            Press::Server(picked) => {
                // A press on the one already picked leaves the page as it is, so a second makes a double click.
                if picked == self.server_picked {
                    return;
                }
                self.server_picked = picked;
                self.render();
            }
            Press::ServerOk => {
                self.chime();
                self.enter_server()
            }
            Press::ServerCancel | Press::Exit => self.leave(None),
            // In the game, leaving first goes back to the characters: another one may be wanted. The server
            // says when the character is out of the game; from the characters, leaving goes out altogether.
            Press::Logout if matches!(self.screen, Screen::Game) => {
                self.esc = 0;
                self.esc_redrawn();
                self.send(&crate::net::packets::restart(true));
            }
            Press::Logout => self.leave(None),
            Press::Delete => self.ask_deletion(false),
            Press::DeleteNow => self.ask_deletion(true),
            Press::CancelDelete => {
                if let (Some(character), Some(socket)) = (self.character_in(self.selected), &self.socket) {
                    socket.send(&packets::delete_cancel(character.char_id));
                    self.wait = Some(Key::PleaseWait);
                    self.render();
                }
            }
            Press::Slot(slot) => self.choose_slot(slot),
            Press::Start => {
                self.chime();
                self.open(self.selected)
            }
            Press::Race(race) => {
                // No Doram where the server makes none (see the window).
                self.make.doram = race == Race::Doram && data::renewal();
                self.make.hair_style = self.make.hair_style.min(hair_styles(self.make.doram));
                self.render();
            }
            Press::Sex(sex) => {
                self.make.sex = u8::from(sex == Sex::Male);
                self.render();
            }
            Press::Hair(style) => {
                if (1..=hair_styles(self.make.doram)).contains(&style) {
                    self.make.hair_style = style;
                    self.render();
                }
            }
            Press::Colour(colour) => {
                if colour < HAIR_COLOURS {
                    self.make.hair_colour = colour;
                    self.render();
                }
            }
            Press::Turn(turn) => {
                let step = if turn == Turn::Left { 7 } else { 1 };
                self.make.direction = (self.make.direction + step) % 8;
                self.render();
            }
            Press::Back => self.show(Screen::CharSelect),
        }
    }

    /// Asks whether the chosen character is to be deleted: put on the wait for it, or, with the wait over,
    /// deleted `now`.
    fn ask_deletion(&mut self, now: bool) {
        if let Some(character) = self.character_in(self.selected) {
            self.ask = Some(if now { Ask::Delete(character.char_id) } else { Ask::Reserve(character.char_id) });
            self.render();
        }
    }

    /// A slot of the page of characters is pressed: it is the chosen one now.
    fn choose_slot(&mut self, slot: u8) {
        // A press on the one already chosen changes nothing: the page stays as it is, and a second press
        // makes a double click of it.
        if self.selected == slot && self.character_in(slot).is_some() {
            return;
        }
        self.selected = slot;
        // An empty slot opens the making of a character at once, as in the original client.
        if self.character_in(slot).is_none() {
            self.open(slot);
            return;
        }
        self.render();
    }
}
