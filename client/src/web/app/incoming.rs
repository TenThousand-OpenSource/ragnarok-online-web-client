//! What the servers say: each packet is read and the app's state, the windows and the world follow.

use super::*;

mod travel;
mod actors;
mod combat;
mod talk;
mod status;
mod inventory;
mod shops;
mod pets;

impl App {
    pub(super) fn packet(&mut self, id: u16, packet: &[u8]) {
        match (self.server, id) {
            // The account is in use elsewhere. The servers are throwing that session out and refuse this login
            // meanwhile: it is tried again in a moment, so the newer login is the one that gets in.
            (Server::Login | Server::Char, packets::SC_NOTIFY_BAN)
                if packets::ban_reason(packet) == Some(STILL_LOGGED_IN) && self.relogin.is_some() && self.login_tries < LOGIN_RETRIES =>
            {
                self.socket = None;
                self.generation += 1;
                self.server = Server::None;
                self.account = None;
                let generation = self.generation;
                let retry = Closure::once_into_js(move || dispatch(Action::RetryLogin(generation)));
                if let Some(window) = web_sys::window() {
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(retry.unchecked_ref(), LOGIN_RETRY_MS);
                }
            }
            (_, packets::SC_NOTIFY_BAN) => {
                let text = match packets::ban_reason(packet) {
                    Some(1) => self.refusal(Key::ServerClosed, ""),
                    Some(2) => self.refusal(Key::AlreadyLoggedIn, ""),
                    Some(8) => self.refusal(Key::StillRecognized, ""),
                    Some(code) => self.refusal(Key::DisconnectedCode, &code.to_string()),
                    None => self.refusal(Key::Disconnected, ""),
                };
                self.leave(text);
            }

            (Server::Login, packets::AC_ACCEPT_LOGIN) => match packets::login_accepted(packet) {
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
            (Server::Login, packets::AC_REFUSE_LOGIN) => {
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

            (Server::Char, packets::HC_ACCEPT_ENTER2) => self.slots = packets::char_slots(packet).map_or(0, |(usable, _)| usable),
            (Server::Char, packets::HC_ACCEPT_ENTER) => {
                self.characters = packets::char_list(packet);
                let now = epoch();
                self.deletions = self.characters.iter().filter_map(|character| Some((character.char_id, now + f64::from(character.delete_in?)))).collect();
                // The first character is chosen, or the first slot to make one in.
                self.selected = self.characters.iter().map(|character| character.slot).min().unwrap_or(0);
                // In: the password is not needed any more.
                self.relogin = None;
                self.show(Screen::CharSelect);
            }
            (Server::Char, packets::HC_REFUSE_ENTER) => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            (Server::Char, packets::HC_ACCEPT_MAKECHAR) => {
                if let Some(character) = packets::char_made(packet) {
                    self.selected = character.slot;
                    self.characters.push(character);
                }
                self.show(Screen::CharSelect);
            }
            (Server::Char, packets::HC_REFUSE_MAKECHAR) => self.say(match packets::make_char_refused(packet) {
                0x00 => Key::NameExists,
                0x01 => Key::Underaged,
                0x02 => Key::SymbolsForbidden,
                0x03 => Key::SlotNotAllowed,
                _ => Key::CreationDenied,
            }),
            (Server::Char, packets::HC_DELETE_CHAR3_RESERVED) => match packets::delete_reserved(packet) {
                Some((char_id, 1, seconds)) => {
                    self.deletions.insert(char_id, epoch() + f64::from(seconds));
                    self.show(Screen::CharSelect);
                }
                Some((_, 4, _)) => self.say(Key::DeleteGuild),
                Some((_, 5, _)) => self.say(Key::DeleteParty),
                _ => self.say(Key::DeleteFailed),
            },
            (Server::Char, packets::HC_DELETE_CHAR3_CANCEL) => match packets::delete_result(packet) {
                Some((char_id, 1)) => {
                    self.deletions.remove(&char_id);
                    self.show(Screen::CharSelect);
                }
                _ => self.say(Key::DeleteFailed),
            },
            (Server::Char, packets::HC_DELETE_CHAR3) => match packets::delete_result(packet) {
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
            (Server::Char, packets::HC_NOTIFY_ZONESVR) => match packets::zone_server(packet) {
                Some(zone) => {
                    let port = zone.port;
                    self.zone = Some(zone);
                    self.connect(Server::Map, port);
                }
                None => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            },

            // The map server's packets are read by kind, each kind in a file of its own (see `incoming/`).
            (Server::Map, _) => {
                let known = self.travel_packet(id, packet) || self.unit_packet(id, packet) || self.combat_packet(id, packet) || self.talk_packet(id, packet) || self.status_packet(id, packet) || self.item_packet(id, packet) || self.trade_packet(id, packet) || self.pet_packet(id, packet);
                // Whatever is not read yet is at least counted, so that it shows there is more to do.
                if !known {
                    if let Some(game) = &mut self.game {
                        game.packets += 1;
                        if let Some(counter) = element("packets") {
                            counter.set_text_content(Some(&game.packets.to_string()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
