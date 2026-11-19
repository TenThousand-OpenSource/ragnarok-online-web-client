//! What the login server and the character server say on the way in: the login accepted or refused, the
//! characters listed, made and deleted, the map server to go to; and a session thrown out by any server.

use super::*;

impl App {
    /// SC_NOTIFY_BAN: the server ends the session, and says why.
    pub(super) fn banned(&mut self, packet: &[u8]) {
        // The account is in use elsewhere. The servers are throwing that session out and refuse this login
        // meanwhile: it is tried again in a moment, so the newer login is the one that gets in.
        if matches!(self.server, Server::Login | Server::Char) && packets::ban_reason(packet) == Some(STILL_LOGGED_IN) && self.relogin.is_some() && self.login_tries < LOGIN_RETRIES {
            self.socket = None;
            self.generation += 1;
            self.server = Server::None;
            self.account = None;
            let generation = self.generation;
            let retry = Closure::once_into_js(move || dispatch(Action::RetryLogin(generation)));
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(retry.unchecked_ref(), LOGIN_RETRY_MS);
            }
            return;
        }
        // The reason, in the client's words for it (`incoming/link.rs`).
        let text = match packets::ban_reason(packet) {
            Some(reason) => Some(self.ban_words(reason)),
            None => self.refusal(Key::Disconnected, ""),
        };
        self.leave(text);
    }

    /// A packet of the login server.
    pub(super) fn login_packet(&mut self, id: u16, packet: &[u8]) {
        match id {
            packets::AC_ACCEPT_LOGIN => match packets::login_accepted(packet) {
                // With one char server there is nothing to choose. A server list comes with the second server.
                // The servers are shown with how many play on each, as the original does, and the player enters
                // the one picked.
                Some(account) if !account.char_servers.is_empty() => {
                    store("roweb.id", (!self.saved_id.is_empty()).then_some(self.saved_id.as_str()));
                    self.account = Some(account);
                    self.server_picked = 0;
                    self.relogin = None;
                    // The login server has said all it had to.
                    self.socket = None;
                    self.generation += 1;
                    self.server = Server::None;
                    self.show(Screen::ServerSelect);
                }
                _ => self.leave(Some(Key::NoCharServer.text(self.lang).to_owned())),
            },
            packets::AC_REFUSE_LOGIN => {
                let text = match packets::login_refused(packet) {
                    Some((0, _)) => self.refusal(Key::UnregisteredId, ""),
                    Some((1, _)) => self.refusal(Key::WrongPassword, ""),
                    Some((2, _)) => self.refusal(Key::IdExpired, ""),
                    Some((4, _)) => self.refusal(Key::Blocked, ""),
                    Some((6, until)) => self.refusal(Key::BlockedUntil, &until),
                    Some((7, _)) => self.refusal(Key::Overpopulated, ""),
                    Some((3, _)) | None => self.refusal(Key::Rejected, ""),
                    Some((code, _)) => self.refusal(Key::RejectedCode, &code.to_string()),
                };
                self.leave(text);
            }
            _ => {}
        }
    }

    /// A packet of the character server.
    pub(super) fn char_packet(&mut self, id: u16, packet: &[u8]) {
        match id {
            packets::HC_ACCEPT_ENTER2 => self.slots = packets::char_slots(packet).map_or(0, |(usable, _)| usable),
            packets::HC_ACCEPT_ENTER => {
                self.characters = packets::char_list(packet);
                let now = epoch();
                self.deletions = self.characters.iter().filter_map(|character| Some((character.char_id, now + f64::from(character.delete_in?)))).collect();
                // The first character is chosen, or the first slot to make one in.
                self.selected = self.characters.iter().map(|character| character.slot).min().unwrap_or(0);
                // In: the password is not needed any more.
                self.relogin = None;
                self.show(Screen::CharSelect);
            }
            packets::HC_REFUSE_ENTER => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            packets::HC_ACCEPT_MAKECHAR => {
                if let Some(character) = packets::char_made(packet) {
                    self.selected = character.slot;
                    self.characters.push(character);
                }
                self.show(Screen::CharSelect);
            }
            packets::HC_REFUSE_MAKECHAR => self.say(match packets::make_char_refused(packet) {
                0x00 => Key::NameExists,
                0x01 => Key::Underaged,
                0x02 => Key::SymbolsForbidden,
                0x03 => Key::SlotNotAllowed,
                _ => Key::CreationDenied,
            }),
            packets::HC_DELETE_CHAR3_RESERVED => match packets::delete_reserved(packet) {
                Some((char_id, 1, seconds)) => {
                    self.deletions.insert(char_id, epoch() + f64::from(seconds));
                    self.show(Screen::CharSelect);
                }
                Some((_, 4, _)) => self.say(Key::DeleteGuild),
                Some((_, 5, _)) => self.say(Key::DeleteParty),
                _ => self.say(Key::DeleteFailed),
            },
            packets::HC_DELETE_CHAR3_CANCEL => match packets::delete_result(packet) {
                Some((char_id, 1)) => {
                    self.deletions.remove(&char_id);
                    self.show(Screen::CharSelect);
                }
                _ => self.say(Key::DeleteFailed),
            },
            packets::HC_DELETE_CHAR3 => match packets::delete_result(packet) {
                Some((char_id, 1)) => {
                    self.characters.retain(|character| character.char_id != char_id);
                    self.deletions.remove(&char_id);
                    self.say(Key::Deleted);
                }
                Some((_, 4)) => self.say(Key::DeleteNotYet),
                // A date of birth is wanted: asked for, unless the one just given was the wrong one.
                Some((char_id, 5)) if !self.sent_birthdate => {
                    self.wait = None;
                    self.ask = Some(Ask::Birthdate(char_id));
                    self.render();
                }
                Some((_, 5)) => self.say(Key::WrongBirthdate),
                _ => self.say(Key::DeleteFailed),
            },
            packets::HC_NOTIFY_ZONESVR => match packets::zone_server(packet) {
                Some(zone) => {
                    let port = zone.port;
                    self.zone = Some(zone);
                    self.connect(Server::Map, port);
                }
                None => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            },
            _ => {}
        }
    }
}
