//! The replays: what the recorder, the list of recordings and the player do when pressed (the client's
//! `UIMenuIconWnd::SendMsg` with `MENUICON_REC`, `UIReplayRECControlWnd::SendMsg`,
//! `UISelectReplayDataWnd::SendMsg`, `UIReplayControlWnd::SendMsg`, and what its modes do at
//! `MM_START_REPLAY_RECORDING`, `MM_STOP_REPLAY_RECORDING` and the player's messages).

use std::rc::Rc;

use super::*;
use crate::press::replay::Press;
use crate::replay::{File, Header, Who};

/// The field of the recorder a name is typed into.
pub(in crate::web) const NAME_FIELD: &str = "replay-name";

impl App {
    /// A press of this system (`crate::press::replay`).
    pub(super) fn replay_pressed(&mut self, press: Press) {
        if self.replay.open && self.replay.mode == 3 && element(NAME_FIELD).is_some() {
            self.replay.name = field(NAME_FIELD);
        }
        match press {
            // (While one is being played nothing is recorded: the client says "REC End please" in a box.)
            Press::Window if self.replay.playing.is_some() => self.message = Some("REC End please".to_owned()),
            Press::Window => (self.replay.open, self.replay.mode) = (!self.replay.open, if self.replay.recording.is_some() { 1 } else { 0 }),
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
            Press::Play => match picked.as_ref().map(|(name, bytes)| (name, crate::replay::read(bytes))) {
                Some((name, Some(file))) => return self.replay_begun(name.clone(), Rc::new(file)),
                Some((_, None)) => self.message = Some(Self::words(1969, "No Replay File.")),
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
            Press::Hold => (playing.held, playing.counted) = (!playing.held, crate::web::now()),
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
        if self.replay.checked && self.replay.files.iter().any(|(kept, _)| kept.eq_ignore_ascii_case(&name)) {
            return;
        }
        self.replay.name.clear();
        self.replay_recording_begins(Some(name));
    }

    /// `MM_START_REPLAY_RECORDING`: the recording begins, from what the map server has said since the game
    /// was entered. Its header says who, where and when.
    fn replay_recording_begins(&mut self, name: Option<String>) {
        self.replays_wanted();
        let Some(game) = &self.game else { return };
        let name = name.unwrap_or_else(|| crate::replay::next_name(self.replay.files.iter().map(|(name, _)| name.as_str())));
        let date = js_sys::Date::new_0();
        let when = [date.get_full_year(), date.get_month() + 1, date.get_date(), date.get_hours(), date.get_minutes(), date.get_seconds()];
        let when = when.map(|part| part as u16);
        let header = Header {
            when,
            service: 0,
            service_type: 0,
            name: game.name.clone(),
            map: game.map.clone(),
            account: self.own_account(),
            x: i32::from(game.x),
            y: i32::from(game.y),
            direction: 0,
            duration: 0,
            options: self.replay.options.to_vec(),
        };
        let own = self.characters.iter().find(|character| character.char_id == self.own_character());
        let who = Who {
            character: self.own_character(),
            job: game.job,
            level: self.agency_level() as u16,
            job_level: own.map_or(0, |own| own.job_level),
            zeny: own.map_or(0, |own| own.zeny),
            look: own.map_or([0; 7], |own| {
                [own.hair_style, own.hair_color.into(), own.body_palette, own.headgear[0], own.headgear[1], own.headgear[2], own.sex.into()]
            }),
            stats: own.map_or([0; 6], |own| own.stats),
        };
        let before = self.replay.before.clone();
        self.replay.recording = Some(Recording { started: now(), name, header, who, before, packets: Vec::new() });
        self.replay.mode = 1;
        after(250, Action::ReplayTick);
    }

    /// `MM_STOP_REPLAY_RECORDING`: the recording ends, and its file is kept.
    fn replay_recording_ends(&mut self) {
        self.replay.mode = 0;
        let Some(mut recording) = self.replay.recording.take() else { return };
        recording.header.duration = now().wrapping_sub(recording.started);
        // What came while it ran is of what the next one begins from.
        for (_, packet) in &recording.packets {
            if self.replay.before_bytes + packet.len() <= BEFORE_MOST {
                self.replay.before_bytes += packet.len();
                self.replay.before.push(packet.clone());
            }
        }
        let file = File { header: recording.header, who: recording.who, before: recording.before, packets: recording.packets };
        self.replay_kept(recording.name, crate::replay::written(&file));
    }
}
