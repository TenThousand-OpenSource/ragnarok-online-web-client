//! What the replays take of what the map server says: every packet, for what a recording begins from and for
//! the recording itself (the client's `Replay::Record` under its connection), and the packets of a recording
//! being played, handed to the game in the server's place (`Replay::ReadRecord`).

use super::*;

impl App {
    /// A packet of the map server came: it is kept for what a recording begins from (anew from each entering
    /// of the game), and written into the recording that runs, unless its options leave it out. Nothing is
    /// kept of a recording being played.
    pub(super) fn replay_heard(&mut self, id: u16, packet: &[u8]) {
        let replay = &mut self.replay;
        if replay.playing.is_some() {
            return;
        }
        if let Some(recording) = &mut replay.recording {
            if crate::replay::recorded(id, &replay.options) {
                recording.packets.push((now().wrapping_sub(recording.started), packet.to_vec()));
            }
            return;
        }
        if id == packets::ZC_ACCEPT_ENTER {
            (replay.before, replay.before_bytes) = (Vec::new(), 0);
        }
        if replay.before_bytes + packet.len() <= BEFORE_MOST {
            replay.before_bytes += packet.len();
            replay.before.push(packet.to_vec());
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
        self.packet(id, packet);
    }

    /// Begins playing a recording (`LMM_START_REPLAY`): the game is entered as the character of the
    /// recording with no server behind it, everything that came before the recording began is caught up
    /// with at once, and the player's window opens.
    pub(in crate::web::app) fn replay_begun(&mut self, name: String, file: std::rc::Rc<crate::replay::File>) {
        let (header, who) = (&file.header, &file.who);
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
        let counted = crate::web::now();
        self.replay.playing = Some(Playing { name, file: file.clone(), next: 0, elapsed: 0.0, counted, speed: 1.0, held: false, over: false });
        for packet in &file.before {
            self.replay_packet(packet, true);
        }
        if !matches!(self.screen, Screen::Game) {
            // The recording did not begin with entering the game: there is nothing to stand in.
            self.replay.playing = None;
            return self.leave(Some(Self::words(1969, "No Replay File.")));
        }
        after(REPLAY_TICK, Action::ReplayTick);
    }

    /// The player's clock goes on (`Replay::AdvanceFrame`): by the time passed, as fast as the speed says,
    /// unless it is held; every packet of the recording whose time has come is handed to the game; at the
    /// recording's end the clock stops there.
    pub(in crate::web::app) fn replay_tick(&mut self) {
        // (The same tick turns the recorder's picture and writes its time while a recording runs.)
        if self.replay.recording.is_some() {
            if self.replay.open && self.replay.mode == 1 {
                self.redraw(&[&wnd::REPLAY_RECORDER]);
            }
            return after(250, Action::ReplayTick);
        }
        let Some(playing) = &mut self.replay.playing else { return };
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

/// How often the player's clock is looked at, in milliseconds.
const REPLAY_TICK: i32 = 50;
