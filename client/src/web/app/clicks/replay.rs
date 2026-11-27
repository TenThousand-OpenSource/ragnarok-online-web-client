//! The replays: what the recorder, the list of recordings and the player do when pressed (the client's
//! `UIMenuIconWnd::SendMsg` with `MENUICON_REC`, `UIReplayRECControlWnd::SendMsg`,
//! `UISelectReplayDataWnd::SendMsg`, `UIReplayControlWnd::SendMsg`, and what its modes do at
//! `MM_START_REPLAY_RECORDING`, `MM_STOP_REPLAY_RECORDING` and the player's messages).

use std::rc::Rc;

use super::*;
use crate::press::replay::Press;
use crate::replay::{File, Header};

/// The field of the recorder a name is typed into.
pub(in crate::web) const NAME_FIELD: &str = "replay-name";

impl App {
    /// A press of this system (`crate::press::replay`).
    pub(super) fn replay_pressed(&mut self, press: Press) {
        if self.replay.open && self.replay.mode == 3 && element(NAME_FIELD).is_some() {
            self.replay.name = field(NAME_FIELD);
        }
        match press {
            // While one is recorded the icon only says so (`MENUICON_REC` with the replay's state not 1: the
            // client's box "REC End please", in its own words); while one plays the icon does nothing.
            Press::Window if self.replay.playing.is_some() => return,
            Press::Window if self.replay.recording.is_some() => self.message = Some("REC End please".to_owned()),
            Press::Window => (self.replay.open, self.replay.mode) = (!self.replay.open, 0),
            // With a name to be asked for, the recorder asks first; else the recording begins.
            Press::Start if self.replay.name_asked => self.replay.mode = 3,
            Press::Start => self.replay_recording_begins(None),
            Press::NameOk => self.replay_named(),
            Press::Stop => self.replay_recording_ends(),
            Press::Options => self.replay.mode = 2,
            Press::OptionsClose => self.replay.mode = 0,
            Press::End => (self.replay.open, self.replay.mode) = (false, 0),
            Press::Option(place) => {
                let on = self.replay.options.get(place).is_some_and(|option| *option == 0);
                self.replay.options = crate::replay::ticked(self.replay.options, place, on);
            }
            Press::NameAsked => self.replay.name_asked ^= true,
            Press::Checked => self.replay.checked ^= true,
            _ => return self.replay_list_pressed(press),
        }
        if matches!(self.screen, Screen::Game) && self.message.is_none() { self.redraw(&[&wnd::REPLAY_RECORDER]) } else { self.render() }
    }

    /// A press of the list of recordings, or of the player.
    fn replay_list_pressed(&mut self, press: Press) {
        let picked = self.replay.picked.and_then(|picked| self.replay.files.get(picked).cloned());
        match press {
            Press::List => {
                self.replay.list = true;
                self.replays_wanted();
            }
            Press::Back => self.replay.list = false,
            Press::Row(row) => self.replay.picked = Some(row),
            // `ID_OK`: the one picked is played, if it is a recording this client can play.
            // (`Replay::MatchesService`: a recording of another kind of service is refused with line 1970.
            // One with neither the character as the client keeps it nor the packets this client once began
            // from has nothing to begin with: "No Replay File.".)
            Press::Play => match picked.as_ref().map(|(name, bytes)| (name, crate::replay::read(bytes))) {
                Some((_, Some(file))) if ![0, 1, 2, 13].contains(&file.header.service_type) => {
                    self.message = Some(Self::words(1970, "Server No Matching"));
                }
                Some((name, Some(file))) if file.begins() => return self.replay_begun(name.clone(), Rc::new(file)),
                Some(_) => self.message = Some(Self::words(1969, "No Replay File.")),
                None => {}
            },
            Press::Save => {
                if let Some((name, bytes)) = &picked {
                    super::replay_files::save(name, bytes);
                }
            }
            Press::Load => wasm_bindgen_futures::spawn_local(async {
                if let Some((name, bytes)) = super::replay_files::pick().await {
                    dispatch(Action::ReplayPicked(name, bytes));
                }
            }),
            Press::Delete => {
                if let (Some((name, _)), Some(place)) = (picked, self.replay.picked) {
                    self.replay.files.remove(place);
                    self.replay.picked = None;
                    wasm_bindgen_futures::spawn_local(async move { super::replay_files::drop_file(&name).await });
                }
            }
            _ => return self.replay_player_pressed(press),
        }
        self.render();
    }

    /// A press of the player (`UIReplayControlWnd::SendMsg`).
    fn replay_player_pressed(&mut self, press: Press) {
        let Some(playing) = &mut self.replay.playing else { return };
        match press {
            // (At its end the button begins nothing: the client's clock stands at the recording's length.)
            Press::Hold => (playing.held, playing.counted) = (!playing.held && !playing.over, crate::web::now()),
            Press::Faster | Press::Slower => playing.speed = crate::replay::stepped(playing.speed, press == Press::Faster),
            // `MM_RESTART_REPLAY`: from the beginning again, at the usual speed.
            Press::Again => {
                let (name, file) = (playing.name.clone(), playing.file.clone());
                self.replay_left();
                return self.replay_begun(name, file);
            }
            // `MM_EXIT_REPLAY`: back before the game, to the list.
            Press::Out => {
                self.replay_left();
                self.replay.list = true;
                return self.render();
            }
            _ => {}
        }
        self.redraw(&[&wnd::REPLAY_PLAYER]);
    }

    /// The game a recording was played in is left: back to the login screen, as one who was never in.
    fn replay_left(&mut self) {
        self.replay.playing = None;
        self.back_to_characters();
        self.leave(None);
    }

    /// The recordings the browser keeps are asked for, the first time the list wants them.
    fn replays_wanted(&mut self) {
        if !self.replay.files_read {
            self.replay.files_read = true;
            wasm_bindgen_futures::spawn_local(async { dispatch(Action::ReplaysRead(super::replay_files::files().await)) });
        }
    }

    /// The recordings the browser keeps were read.
    pub(in crate::web::app) fn replays_read(&mut self, files: Vec<(String, Vec<u8>)>) {
        for (name, bytes) in files {
            if !self.replay.files.iter().any(|(kept, _)| *kept == name) {
                self.replay.files.push((name, Rc::new(bytes)));
            }
        }
        self.replay.files.sort_by(|one, other| one.0.cmp(&other.0));
        if matches!(self.screen, Screen::Login) {
            self.render();
        }
    }

    /// A file of the player's machine was picked: it is kept with the others, if it is a replay at all.
    pub(in crate::web::app) fn replay_taken(&mut self, name: String, bytes: Vec<u8>) {
        if crate::replay::header(&bytes).is_none() {
            self.message = Some(Self::words(1969, "No Replay File."));
        } else {
            self.replay_kept(name, bytes);
        }
        self.render();
    }

    /// A recording is kept: in the list, and in the browser.
    fn replay_kept(&mut self, name: String, bytes: Vec<u8>) {
        self.replay.files.retain(|(kept, _)| *kept != name);
        self.replay.files.push((name.clone(), Rc::new(bytes.clone())));
        self.replay.files.sort_by(|one, other| one.0.cmp(&other.0));
        wasm_bindgen_futures::spawn_local(async move {
            if !super::replay_files::keep(&name, &bytes).await {
                web_sys::console::error_1(&format!("replay: {name} could not be kept in the browser").into());
            }
        });
    }

    /// The name typed is confirmed (`ID_CONFIRM_REPLAY_RECORDING_PATH`): none typed, a box asks for one
    /// (line 1972); one with the mark of an item's link in it is refused (2812); one there is already, with
    /// the box "Repeated File Check" ticked, begins nothing; else the recording begins under it.
    fn replay_named(&mut self) {
        let typed = self.replay.name.trim().to_owned();
        if typed.is_empty() {
            return self.message = Some(Self::words(1972, "Enter File Name"));
        }
        if typed.contains("<ITEM>") || typed.contains("</ITEM>") {
            return self.message = Some(Self::words(2812, "Name with this tag cannot be used."));
        }
        let name = crate::replay::named(&typed);
        // (`ReplayFileList::Open` answering -2: the box of line 1989, and nothing begins.)
        if self.replay.checked && self.replay.files.iter().any(|(kept, _)| kept.eq_ignore_ascii_case(&name)) {
            return self.message = Some(Self::words(1989, "The same file exists already."));
        }
        self.replay.name.clear();
        self.replay_recording_begins(Some(name));
    }

    /// `MM_START_REPLAY_RECORDING`: the recording begins. Its header says who, where and when; what the
    /// session and the map hold now is kept in the file's sections, and the map's kind and the skills stand
    /// at the head of its packets (`replay_snapshot/taken.rs`).
    fn replay_recording_begins(&mut self, name: Option<String>) {
        self.replays_wanted();
        let Some(game) = &self.game else { return };
        let name = name.unwrap_or_else(|| crate::replay::next_name(self.replay.files.iter().map(|(name, _)| name.as_str())));
        let date = js_sys::Date::new_0();
        let when = [date.get_full_year(), date.get_month() + 1, date.get_date(), date.get_hours(), date.get_minutes(), date.get_seconds()];
        let when = when.map(|part| part as u16);
        let super::replay_snapshot::Taken { client, head, direction } = self.replay_gathered();
        let header = Header {
            when,
            service: 0,
            service_type: 0,
            name: game.name.clone(),
            map: game.map.clone(),
            account: self.own_account(),
            x: i32::from(game.x),
            y: i32::from(game.y),
            direction,
            duration: 0,
            options: self.replay.options.to_vec(),
        };
        let quests = self.quests.open;
        self.replay.recording = Some(Recording { started: now(), name, header, packets: head, client, quests });
        self.replay.mode = 1;
        // (`MSI_REPLAY_RECORDSTART`, in the colour 0x62B0FF of the client's blue, green, red.)
        self.chat_line(Self::words(1990, "Record Start"), "#ffb062");
        after(250, Action::ReplayTick);
    }

    /// A press is about to be done: whether it is. While a recording plays, only what the client leaves the
    /// one watching (`crate::replay::playing::press_passes`), and what the recording itself does again;
    /// while one is made, a press of a window the server opened is written into it first.
    pub(in crate::web) fn replay_lets(&mut self, press: &crate::press::Press) -> bool {
        if let Some(options) = self.replay.held_by() {
            return self.replay.told || crate::replay::playing::press_passes(press, &options);
        }
        if self.replay.recording.is_some() {
            let records = crate::replay::told::records_of(press, &self.replay_open(press));
            self.replay_written(records);
        }
        true
    }

    /// Records of the windows go into the recording that runs, at the time it is now.
    fn replay_written(&mut self, records: Vec<Vec<u8>>) {
        if let Some(recording) = &mut self.replay.recording {
            let at = now().wrapping_sub(recording.started);
            recording.packets.extend(records.into_iter().map(|record| (at, record)));
        }
    }

    /// What stands open now, as far as the records of the windows need it (`crate::replay::told`); the field
    /// read is that of the window the press is of.
    pub(in crate::web::app) fn replay_open(&self, press: &crate::press::Press) -> crate::replay::told::Open {
        use crate::replay::told::{List, Open, Shop as Told, Talk};
        let talk = self.dialog.as_ref().map(|dialog| match dialog.asks {
            _ if !dialog.menu.is_empty() => Talk::Menu(dialog.picked),
            Some(true) => Talk::Words,
            Some(false) => Talk::Number,
            None => Talk::Says,
        });
        let list = self.choosing.open.as_ref().map(|choice| match choice {
            Choice::Warp(..) => List::Warp,
            Choice::Arrows(_) | Choice::Mix(..) => List::Arrows,
            Choice::Spells(_) => List::Spells,
            Choice::Compound(..) => List::Compound,
            Choice::Make(_) => List::Make,
            Choice::Forge(..) => List::Forge,
            Choice::Repair(_) => List::Repair,
            Choice::Refine(_) => List::Refine,
        });
        Open {
            talk,
            typed: match press {
                crate::press::Press::Talk(_) => field("dialog-input"),
                crate::press::Press::Exchange(_) => field("exchange-zeny"),
                _ => String::new(),
            },
            shop: self.shops.shop.as_ref().map(|shop| match shop {
                Shop::Choose(_) => Told::Choose,
                Shop::Buy(_) => Told::Buy,
                Shop::Sell(_) => Told::Sell,
            }),
            whole_stacks: self.shops.whole_stacks,
            list: list.map(|list| (list, self.choosing.picked)),
            identify: self.bag.identify_picked,
            egg: self.pets.egg_picked,
            party_invited: matches!(self.party.asks, Some(PartyAsk::Invited(..))),
            guild_invited: matches!(self.guild.asks, Some(GuildAsk::Invited(..))),
            quests_shown: !self.quests.display_off,
        }
    }

    /// A thing was let go over a window the client records the drops of (`UIWindow::RecordDrop`): a row of a
    /// shop's list over the basket or back, a thing carried over the deal or the forging.
    pub(in crate::web::app) fn replay_let_go(&mut self, dragged: &Drag, onto: u32) {
        use crate::replay::told::{from, window, Dropped};
        if self.replay.recording.is_none() {
            return;
        }
        let carried = |index: u16| self.bag.at(index).map_or((0, 0, 0), |item| (item.id, i32::from(item.amount), u32::from(item.kind)));
        let dropped = match (dragged, &self.shops.shop) {
            // (What a shop sells has no place of its own in the client: -1, and one of it.)
            (Drag::Offer(row) | Drag::Basket(row), Some(Shop::Buy(offers))) if onto != window::EXCHANGE => {
                let Some(offer) = offers.get(*row) else { return };
                let from = if onto == window::SHOP_LIST { from::PURCHASE } else { from::SHOP_LIST };
                Dropped { onto, from, id: u32::MAX, item: offer.id, amount: 1, kind: u32::from(offer.kind) }
            }
            (Drag::Offer(row) | Drag::Basket(row), Some(Shop::Sell(bids))) if onto != window::EXCHANGE => {
                let Some(bid) = bids.get(*row) else { return };
                let (item, amount, kind) = carried(bid.index);
                let from = if onto == window::SHOP_LIST { from::SELL } else { from::SHOP_LIST };
                Dropped { onto, from, id: u32::from(bid.index), item, amount, kind }
            }
            (Drag::Item(item), _) => match self.bag.items.iter().find(|carried| carried.id == *item && carried.worn == 0) {
                Some(carried) => {
                    let (id, amount) = (u32::from(carried.index), i32::from(carried.amount));
                    Dropped { onto, from: from::INVENTORY, id, item: *item, amount, kind: u32::from(carried.kind) }
                }
                None => return,
            },
            _ => return,
        };
        self.replay_written(vec![crate::replay::told::drop_record(&dropped)]);
    }

    /// The count box's OK was pressed (`UIItemDropCntWnd`): its field and its number go into the recording.
    pub(in crate::web::app) fn replay_counted(&mut self, asked: u32, typed: &str) {
        if self.replay.recording.is_some() {
            self.replay_written(crate::replay::told::counted(asked, typed));
        }
    }

    /// A key while a recording plays (`UIWindowMgr::ProcessShortcutKey`, `ProcessEnterKey`, and Escape in
    /// `ProcessHotkeyAction`): the player's three keys; a key of the interface the client leaves, by the
    /// recording's options; Escape closes the window in front and never opens the game's options; Enter and
    /// the hotkey bars do nothing. Whether the key was used.
    pub(in crate::web::app) fn replay_key(&mut self, key: &str, chord: Chord) -> bool {
        use crate::replay::playing;
        let Some(options) = self.replay.held_by() else { return false };
        match self.bound_to(chord) {
            Some((crate::hotkeys::table::INTERFACE, place)) if playing::key_acts(place, &options) => match place {
                // (`HK_REPLAY_STARTSTOP`: the button that holds or goes on, and its picture with it.)
                playing::START_STOP => self.pressed(Press::Hold),
                playing::FASTER => self.pressed(Press::Faster),
                playing::SLOWER => self.pressed(Press::Slower),
                _ => {
                    self.replay.told = true;
                    self.interface_key(place);
                    self.replay.told = false;
                }
            },
            _ if key == "Escape" => {
                self.replay.told = true;
                self.close_front();
                self.replay.told = false;
            }
            _ => return matches!(key, "Enter"),
        }
        true
    }

    /// The recording that runs is over without Stop being pressed: the character changed maps, or the
    /// connection ended. Its file is kept as far as it went, and the recorder is put away.
    pub(in crate::web::app) fn replay_recording_cut(&mut self) {
        if self.replay.recording.is_some() {
            self.replay_recording_kept();
            (self.replay.open, self.replay.mode) = (false, 0);
            if matches!(self.screen, Screen::Game) {
                self.redraw(&[&wnd::REPLAY_RECORDER]);
            }
        }
    }

    /// `MM_STOP_REPLAY_RECORDING`: the recording ends, its file is kept, and the chat says where
    /// (the file's path and `MSI_REPLAY_RECORDEND`; the path here is the folder the client keeps them in).
    fn replay_recording_ends(&mut self) {
        self.replay.mode = 0;
        if let Some(name) = self.replay_recording_kept() {
            self.chat_line(format!("\\Replay\\{name}{}", Self::words(1991, " is Saved.")), "#ffb062");
        }
    }

    /// The recording that runs ends and its file is kept: the file's name, or none when none ran.
    fn replay_recording_kept(&mut self) -> Option<String> {
        let mut recording = self.replay.recording.take()?;
        recording.header.duration = now().wrapping_sub(recording.started);
        let name = recording.name.clone();
        let file = File { header: recording.header, packets: recording.packets, client: recording.client, ..File::default() };
        self.replay_kept(recording.name, crate::replay::written(&file));
        Some(name)
    }
}
