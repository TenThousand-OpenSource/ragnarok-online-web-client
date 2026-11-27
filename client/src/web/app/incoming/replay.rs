//! What the replays take of what the map server says: the packets of the map's kind, which a recording begins
//! with, and every packet for the recording that runs (the client's `Replay::CachePacket` and `Replay::Record`
//! under its connection); and the packets of a recording being played, handed to the game in the server's
//! place (`Replay::ReadRecord`).

use super::*;

impl App {
    /// A packet of the map server came: one of the two that tell the map's kind is kept, for the head of
    /// the next recording; and it is written into the recording that runs, unless its options leave it out.
    /// Nothing is kept of a recording being played.
    pub(super) fn replay_heard(&mut self, id: u16, packet: &[u8]) {
        use crate::replay::snapshot::skills::{MAP_KIND, MAP_PROPERTY};
        let replay = &mut self.replay;
        if replay.playing.is_some() {
            return;
        }
        if let Some(place) = [MAP_PROPERTY, MAP_KIND].iter().position(|kept| *kept == id) {
            replay.map_told[place] = Some(packet.to_vec());
        }
        if let Some(recording) = &mut replay.recording {
            if crate::replay::recorded(id, &replay.options) {
                recording.packets.push((now().wrapping_sub(recording.started), packet.to_vec()));
            }
            // The character is moved to another map, or enters the game anew: the recording is over, and
            // the recorder is put away (`CGameMode::ReceiveMapMove`, `ReceiveAcceptEnter`: the state 3,
            // in which nothing more is written).
            if matches!(id, packets::ZC_NPCACK_MAPMOVE | packets::ZC_ACCEPT_ENTER) {
                self.replay_recording_cut();
            }
        }
    }

    /// A packet of a recording is handed to the game as if the map server had sent it. One that moves the
    /// character to another map while the recording's beginning is being caught up with is acted on at once,
    /// not after the screen went dark: the packets behind it are of the map it leads to.
    fn replay_packet(&mut self, packet: &[u8], at_once: bool) {
        let Some(id) = crate::replay::packet_id(packet) else { return };
        if at_once && id == packets::ZC_NPCACK_MAPMOVE {
            if let Some((map, x, y)) = packets::map_move(packet) {
                return self.arrived(map, x, y);
            }
        }
        if id == crate::replay::sections::CC_REPLAYPACKET {
            return self.replay_told(packet);
        }
        self.packet(id, packet);
    }

    /// A record of the windows is played (`CGameMode::ProcessUiMessage`): the quest window opened or closed
    /// as it was, a press of this client's page done again, and the client's own records of what its windows were
    /// told (`crate::replay::told`). The kinds nothing here knows are passed by.
    fn replay_told(&mut self, packet: &[u8]) {
        use crate::replay::sections::{ui_record, window, Told};
        match ui_record(packet) {
            Some(Told::Window { opened, window: window::QUESTS }) if opened != self.quests.open => {
                self.replay.told = true;
                self.quest_key();
                self.replay.told = false;
            }
            Some(Told::Press(what, with)) => {
                self.replay.told = true;
                self.click(what, with);
                self.replay.told = false;
            }
            // One of the client's own kinds (900 to 902, 913): what it says was done is done again.
            Some(Told::Other(_)) => {
                // (No press is read from a record while nothing would say which window it was of.)
                let open = self.replay_open(&crate::press::general::Press::Menu.into());
                if let Some(deed) = crate::replay::told::deed(packet, &open) {
                    self.replay.told = true;
                    self.replay_done(deed);
                    self.replay.told = false;
                }
            }
            _ => {}
        }
    }

    /// What a record of the windows says was done is done again (`CGameMode::ProcessUiMessage`): the presses
    /// pressed, the fields filled (`UIWindow::RestoreEditTexts`), the thing let go where it was
    /// (`ReceiveUiDragMessage`), the count box answered. Nothing is sent: there is no server.
    fn replay_done(&mut self, deed: crate::replay::told::Deed) {
        use crate::replay::told::{asked, window, Deed};
        match deed {
            Deed::Presses(presses) => presses.into_iter().for_each(|press| self.pressed(press)),
            Deed::Typed { window, texts } => {
                let field = match window {
                    window::NPC_NUMBER | window::NPC_WORDS => "dialog-input".to_owned(),
                    window::EXCHANGE => "exchange-zeny".to_owned(),
                    _ => return,
                };
                if let Some(text) = texts.first() {
                    set_field(&field, text);
                }
            }
            Deed::Counted { asked: asked::DEALT, amount } => {
                set_field(&wnd::EXCHANGE_COUNT.field(), &amount.to_string());
                self.exchange_counted();
            }
            Deed::Counted { asked: asked::BOUGHT..=asked::NOT_SOLD, amount } => {
                set_field(&wnd::SHOP_COUNT.field(), &amount.to_string());
                self.shop_counted();
            }
            Deed::Counted { .. } => {}
            Deed::Dropped(dropped) => self.replay_dropped(dropped),
        }
    }

    /// A thing is let go again where the recording says it was: the row of the shop's list it was is found by
    /// the item's place where it is carried (what is sold) or by its number (what is bought).
    fn replay_dropped(&mut self, dropped: crate::replay::told::Dropped) {
        use crate::replay::told::window;
        let row = match &self.shops.shop {
            Some(Shop::Buy(offers)) => offers.iter().position(|offer| offer.id == dropped.item),
            Some(Shop::Sell(bids)) => bids.iter().position(|bid| u32::from(bid.index) == dropped.id),
            _ => None,
        };
        match (dropped.onto, row) {
            (window::PURCHASE | window::SELL, Some(row)) => self.shop_let_go(Drag::Offer(row), Drop::Basket),
            (window::SHOP_LIST, Some(row)) => self.shop_let_go(Drag::Basket(row), Drop::Shop),
            (window::EXCHANGE, _) => self.exchange_offered(Drag::Item(dropped.item)),
            (window::MAKE_PROCESS, _) => self.forge_dropped(Drag::Item(dropped.item)),
            _ => {}
        }
    }

    /// Begins playing a recording (`LMM_START_REPLAY`): the game is entered as the character of the
    /// recording with no server behind it, from what the file keeps of the session as the Windows client
    /// does (`replay_snapshot/given.rs`), and the player's window opens. A file this client wrote with its
    /// own section begins from the packets in that instead: everything that came before the recording
    /// began is caught up with at once.
    pub(in crate::web::app) fn replay_begun(&mut self, name: String, file: std::rc::Rc<crate::replay::File>) {
        let character = Some(&file).filter(|file| !file.begins_from_packets()).and_then(|file| crate::replay::snapshot::session::character(file));
        let (header, who) = (&file.header, character.as_ref().map_or(&file.who, |character| &character.who));
        self.socket = None;
        self.generation += 1;
        self.server = Server::Map;
        self.account = Some(LoginAccepted {
            login_id1: 0,
            account_id: header.account,
            login_id2: 0,
            sex: 0,
            token: String::new(),
            char_servers: Vec::new(),
        });
        self.characters = vec![Character {
            char_id: who.character,
            base_exp: 0,
            zeny: who.zeny,
            job_level: who.job_level,
            hp: 0,
            max_hp: 0,
            sp: 0,
            max_sp: 0,
            job: who.job,
            hair_style: who.look[0],
            base_level: who.level,
            name: header.name.clone(),
            stats: who.stats,
            slot: 0,
            hair_color: who.look[1] as u8,
            headgear: [who.look[3], who.look[4], who.look[5]],
            body_palette: who.look[2],
            delete_in: None,
            map: crate::maps::bare(&header.map).to_owned(),
            sex: who.look[6] as u8,
        }];
        self.zone = Some(ZoneServer { char_id: who.character, map: header.map.clone(), port: 0 });
        (self.replay.list, self.message, self.wait) = (false, None, None);
        let (counted, waits) = (crate::web::now(), character.is_some());
        self.replay.playing = Some(Playing { name, file: file.clone(), next: 0, elapsed: 0.0, counted, speed: 1.0, held: false, over: false, waits });
        // What the recording was made without is not shown (`UIWindowMgr::ApplyReplayOptions`, and
        // `ShowWindowById(WID_CHATWND, …)` as the game begins): the chat window, the hotkey bar.
        if let Some(options) = self.replay.held_by() {
            self.chat.hidden = !crate::replay::playing::chat_shows(&options);
            self.skills.hotkeys_hidden = !crate::replay::playing::bar_shows(&options);
        }
        match &character {
            Some(character) => {
                self.replay_entered(&file, character);
            }
            None => file.before.iter().for_each(|packet| self.replay_packet(packet, true)),
        }
        if !matches!(self.screen, Screen::Game) {
            // The file has nothing to enter the game by: there is nothing to stand in.
            self.replay.playing = None;
            return self.leave(Some(Self::words(1969, "No Replay File.")));
        }
        // A replay begins with no window open (`UIWindowMgr::ApplyReplayOptions`): what the server had
        // opened before the recording began, and the player had closed since, is closed.
        self.replay.told = true;
        for _ in 0..64 {
            if !self.close_front() {
                break;
            }
        }
        self.replay.told = false;
        // (One that begins from its file's sections waits for its map: `replay_map_shown`.)
        if !waits {
            after(REPLAY_TICK, Action::ReplayTick);
        }
    }

    /// The player's clock goes on (`Replay::AdvanceFrame`): by the time passed, as fast as the speed says,
    /// unless it is held; every packet of the recording whose time has come is handed to the game; at the
    /// recording's end the clock stops there.
    pub(in crate::web::app) fn replay_tick(&mut self) {
        // (The same tick turns the recorder's picture and writes its time while a recording runs.)
        let quests = self.quests.open;
        if let Some(recording) = &mut self.replay.recording {
            // The quest window was opened or closed since: the client writes that into the recording
            // (`UIWindowMgr::RecordWindowPacket` as the window begins, `CloseWindow` as it goes), with the
            // options "Basicinfo", "Item" and "Status" all on.
            let options = &self.replay.options;
            let all = [crate::replay::option::BASIC, crate::replay::option::ITEM, crate::replay::option::STATUS];
            if recording.quests != quests && all.iter().all(|option| options[*option] != 0) {
                let record = crate::replay::sections::window_record(quests, crate::replay::sections::window::QUESTS);
                recording.packets.push((now().wrapping_sub(recording.started), record));
            }
            recording.quests = quests;
            if self.replay.open && self.replay.mode == 1 {
                self.redraw(&[&wnd::REPLAY_RECORDER]);
            }
            return after(250, Action::ReplayTick);
        }
        let Some(playing) = self.replay.playing.as_mut().filter(|playing| !playing.waits) else { return };
        let now = crate::web::now();
        let duration = f64::from(playing.file.header.duration);
        if !playing.held && !playing.over {
            playing.elapsed += (now - playing.counted) * f64::from(playing.speed);
            if playing.elapsed >= duration {
                (playing.elapsed, playing.over) = (duration, true);
            }
        }
        playing.counted = now;
        let (file, elapsed) = (playing.file.clone(), playing.elapsed);
        let next = |app: &Self| app.replay.playing.as_ref().map_or(usize::MAX, |playing| playing.next);
        while let Some((_, packet)) = file.packets.get(next(self)).filter(|(tick, _)| f64::from(*tick) <= elapsed) {
            if let Some(playing) = &mut self.replay.playing {
                playing.next += 1;
            }
            self.replay_packet(packet, false);
        }
        if self.replay.playing.is_some() {
            self.redraw(&[&wnd::REPLAY_PLAYER]);
            after(REPLAY_TICK, Action::ReplayTick);
        }
    }
}
