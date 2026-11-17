//! The connection to the login, character and map servers in turn, and the way from one to the next.

use super::*;

impl App {
    /// Closes the connection and goes back to the login window, saying why if there is a reason.
    pub(super) fn leave(&mut self, error: Option<String>) {
        self.socket = None;
        self.generation += 1;
        self.server = Server::None;
        self.credentials = None;
        self.relogin = None;
        self.account = None;
        self.characters.clear();
        self.deletions.clear();
        self.ask = None;
        self.zone = None;
        self.game = None;
        world::leave();
        self.screen = Screen::Login;
        self.wait = None;
        self.message = error;
        self.tune();
        self.render();
    }

    /// Enters the server picked in the list: its char server, where the characters are.
    pub(super) fn enter_server(&mut self) {
        let Some(port) = self.account.as_ref().and_then(|account| account.char_servers.get(self.server_picked)).map(|server| server.port) else { return };
        self.connect(Server::Char, port);
    }

    /// Out of the game and back to the characters of the account, on the server it was played on.
    pub(super) fn back_to_characters(&mut self) {
        world::leave();
        self.game = None;
        self.zone = None;
        self.dialog = None;
        self.shop = None;
        self.storage = None;
        self.throwing = None;
        self.item_info = None;
        self.obtained = None;
        self.dead = false;
        self.esc = 0;
        self.close_taming();
        self.sitting = false;
        self.inventory.clear();
        self.basket.clear();
        self.chat.clear();
        self.enter_server();
    }

    /// Opens a connection to another server, closing the current one.
    pub(super) fn connect(&mut self, server: Server, port: u16) {
        self.generation += 1;
        self.framer = Framer::new();
        if server == Server::Char {
            // The char server answers with the bare account ID before its first packet.
            self.framer.expect_raw(4);
        }
        self.server = server;
        let generation = self.generation;
        match Socket::open(port, move |event| later(Action::Socket(generation, event))) {
            Ok(socket) => {
                self.socket = Some(socket);
                self.wait = Some(Key::PleaseWait);
                self.render();
            }
            Err(error) => {
                web_sys::console::error_1(&error.into());
                self.leave(Some(Key::ConnectFailed.text(self.lang).to_owned()));
            }
        }
    }

    /// The socket opened: each server expects the client to speak first.
    pub(super) fn connected(&mut self) {
        let Some(socket) = &self.socket else { return };
        match self.server {
            Server::Login => {
                if let Some((user, password)) = self.credentials.take() {
                    socket.send(&packets::login(&user, &password));
                }
            }
            Server::Char => {
                if let Some(account) = &self.account {
                    socket.send(&packets::char_enter(account));
                }
            }
            Server::Map => {
                if let (Some(account), Some(zone)) = (&self.account, &self.zone) {
                    socket.send(&packets::map_enter(account, zone.char_id, now()));
                }
            }
            Server::None => {}
        }
    }

    pub(super) fn received(&mut self, generation: u32, bytes: &[u8]) {
        self.framer.push(bytes);
        // A packet may move the client to another server: what is left of this stream is then dropped with it.
        while generation == self.generation {
            match self.framer.next() {
                Ok(Some(Frame::Packet(id, packet))) => self.packet(id, &packet),
                Ok(Some(Frame::Raw(_))) => {}
                Ok(None) => {
                    // More held than any packet is long: the length table is wrong about this one.
                    if let Some((id, held)) = self.framer.waiting().filter(|(_, held)| *held > 600) {
                        web_sys::console::error_1(&format!("protocol: waiting on packet {id:#06x} with {held} bytes held").into());
                    }
                    break;
                }
                Err(error) => {
                    web_sys::console::error_1(&format!("protocol: {error}").into());
                    return self.leave(Some(Key::Disconnected.text(self.lang).to_owned()));
                }
            }
        }
    }

    pub(super) fn refusal(&self, key: Key, value: &str) -> Option<String> {
        Some(fill(key.text(self.lang), &[value]))
    }

    /// Enter or a double-click on a slot: into the game with its character, or to making one in it.
    pub(super) fn open(&mut self, slot: u8) {
        if !matches!(self.screen, Screen::CharSelect) || self.wait.is_some() {
            return;
        }
        if let Some(character) = self.character_in(slot) {
            // A character waiting to be deleted stays out of the game.
            if self.deletions.contains_key(&character.char_id) {
                return;
            }
            if let Some(socket) = &self.socket {
                socket.send(&packets::select_char(slot));
                self.wait = Some(Key::PleaseWait);
                self.render();
            }
        } else if slot < self.slots {
            self.make = Make::default();
            self.show(Screen::CharCreate(slot));
        }
    }

    pub(super) fn create_character(&mut self) {
        let Screen::CharCreate(slot) = self.screen else { return };
        let name = field("char-name");
        let name = name.trim();
        if !NAME_LENGTH.contains(&name.len()) || !name.chars().all(|character| character.is_ascii_alphanumeric()) {
            return self.say(Key::BadName);
        }
        if let Some(socket) = &self.socket {
            // A human starts as a Novice (job 0), a Doram as a Summoner.
            let job = if self.make.doram { u32::from(data::SUMMONER) } else { 0 };
            socket.send(&packets::make_char(name, slot, self.make.hair_colour, self.make.hair_style, job, self.make.sex));
            self.wait = Some(Key::PleaseWait);
            self.render();
        }
    }
}
