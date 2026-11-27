//! What the server says of the battlegrounds (`crate::net::battleground`): the answers of the queue, the
//! question whether to enter, and on a battle's map the teams, the score, where the team is and its life.
//! The client's `CEntryQueueMgr` (`OnApplyResult`, `OnStatus`, `OnCancelResult`, `OnReady`, `OnResultNotice`,
//! `OnNotify`, `ResetCurrent`) and the `ID_ZC_BATTLEFIELD_NOTIFY_*` arms of its packet handler.

use super::*;
use crate::battleground::Applied;
use crate::net::battleground;

/// The colour of the chat's lines about a queue joined and a battle ready (the client's `0x4763FF`, red first).
const TOLD: &str = "#ff6347";

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn battleground_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Overheard: the score and the marks of the team are of the map that is left.
            packets::ZC_ACCEPT_ENTER | packets::ZC_NPCACK_MAPMOVE => {
                self.battleground_map_left();
                return false;
            }
            battleground::ZC_ACK_ENTRY_QUEUE_APPLY => {
                if let Some((result, name)) = battleground::answered(packet) {
                    self.battleground_applied(result, name);
                }
            }
            battleground::ZC_NOTIFY_ENTRY_QUEUE_APPLY => {
                if let Some((name, place)) = battleground::queued(packet) {
                    self.battleground_placed(name, place);
                }
            }
            battleground::ZC_ACK_ENTRY_QUEUE_CANCEL => {
                if let Some(result) = battleground::result(packet) {
                    self.battleground_left(result);
                }
            }
            // `IgnorePacket`.
            battleground::ZC_NOTIFY_ENTRY_QUEUE_ADMISSION => {}
            battleground::ZC_REPLY_ACK_ENTRY_QUEUE_ADMISSION => {
                if let Some(line) = battleground::result(packet).and_then(crate::battleground::admission_refused) {
                    self.message = Some(Self::words(line, ""));
                    self.render();
                }
            }
            battleground::ZC_NOTIFY_LOBBY_ADMISSION => {
                if let Some((name, lobby)) = battleground::ready(packet) {
                    self.battleground_ready(name, lobby);
                }
            }
            // `OnNotify`: someone of the queue agreed to enter.
            battleground::ZC_REPLY_ACK_LOBBY_ADMISSION => {
                if battleground::result(packet) == Some(1) {
                    sound::effect("se_btg_forward.wav", 1.0);
                }
            }
            // `ResetCurrent`: nothing of a queue is remembered.
            battleground::ZC_ENTRY_QUEUE_INIT => {
                (self.battleground.current, self.battleground.ready) = (None, false);
            }
            battleground::ZC_BATTLEFIELD_CHAT => {
                if let Some((speaker, words)) = battleground::chat_heard(packet) {
                    self.battle_chat_heard(speaker, &words);
                }
            }
            _ => return self.battle_packet(id, packet),
        }
        true
    }

    /// What a battle's map tells; `false` for any other packet.
    fn battle_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            battleground::ZC_BATTLEFIELD_NOTIFY_CAMPINFO => {
                if let Some((account, team)) = battleground::camp(packet) {
                    self.battle_team(account, team);
                }
            }
            battleground::ZC_BATTLEFIELD_NOTIFY_POINT => {
                self.battleground.score = battleground::points(packet);
                self.rank_shown();
            }
            // Where one of the team is: a mark on the minimap, as a party member's is (`MM_ADDPARTYPOS`).
            battleground::ZC_BATTLEFIELD_NOTIFY_POSITION => {
                if let Some((account, cell)) = battleground::place(packet).filter(|(account, _)| *account != self.own_account()) {
                    match cell {
                        Some(cell) => self.battleground.places.insert(account, cell),
                        None => self.battleground.places.remove(&account),
                    };
                    self.party_marks();
                }
            }
            // The life of one of the team: the bar under them (`AM_UPDATEGAGE`).
            battleground::ZC_BATTLEFIELD_NOTIFY_HP | battleground::ZC_BATTLEFIELD_NOTIFY_HP2 => {
                if let Some((account, left, most)) = battleground::life(id, packet) {
                    world::unit_life(account, left, most);
                }
            }
            _ => return false,
        }
        true
    }

    /// A line of the battleground's own chat (`CGameMode::ReceiveActorBroadcastText`): in the chat in the
    /// client's `0x9B00FF`, unless what others say is kept out of it, and over the head of whoever said it.
    /// (The colour the client gives the line of a speaker that is a monster by its number, `MonsterJobColor`,
    /// is not made: a player speaks here.)
    fn battle_chat_heard(&mut self, speaker: u32, words: &str) {
        let words = data::starred(words);
        if !self.option("TT_NOTALKMSG_ON_OFF") && !self.option("TT_NOTALKMSG2_ON_OFF") {
            self.chat_line_of(said::CHAT, words.clone(), "#ff009b");
        }
        world::said(speaker, self.own_account(), &words);
    }

    /// The battleground picked, as the list has it.
    pub(in crate::web::app) fn battleground_current(&self) -> Option<crate::battleground::Entry> {
        self.battleground.current.and_then(data::battleground)
    }

    /// A line of the chat about the battleground picked: its name in brackets, then a line of the table.
    fn battleground_told(&mut self, line: usize, otherwise: &str, colour: &str) {
        let shown = self.battleground_current().map(|entry| entry.shown).unwrap_or_default();
        self.chat_line(format!("[{shown}]{}", Self::words(line, otherwise)), colour);
    }

    /// `OnApplyResult`: one is in the queue (its window opens, or closes if it was open; a sound and a line of
    /// the chat), or a box says why not.
    fn battleground_applied(&mut self, result: u8, name: String) {
        match crate::battleground::applied(result) {
            Applied::Queued => {
                self.battleground.standby = !self.battleground.standby;
                self.battleground.waiting = None;
                self.battleground.queue = name;
                sound::effect("se_btg_request.wav", 1.0);
                self.battleground_told(2099, "You are now in the battlefield queue.", TOLD);
                self.battleground.ready = true;
                self.redraw(&[&wnd::BATTLEGROUND_STANDBY]);
            }
            Applied::Refused(line) => {
                self.message = Some(Self::words(line, ""));
                self.render();
            }
            Applied::Nothing => {}
        }
    }

    /// `OnStatus`: one's place in the queue. Its window opens with it and the list closes; the battleground is
    /// looked up by its name when none is picked.
    fn battleground_placed(&mut self, name: String, place: i32) {
        self.battleground.ready = true;
        self.battleground.standby = true;
        if self.battleground.current.is_none() {
            self.battleground.current = data::battleground_named(&name).map(|entry| entry.id);
        }
        if let Some(entry) = self.battleground_current() {
            self.battleground.waiting = Some((name, entry.needed(), place));
        }
        self.battleground.open = false;
        self.redraw(&[&wnd::BATTLEGROUND_STANDBY, &wnd::BATTLEGROUND]);
    }

    /// `OnCancelResult`: a box says how the queue was left, its window closes for two of the reasons, the chat
    /// is told in red, and no battleground is picked any more.
    fn battleground_left(&mut self, result: u8) {
        if let Some((line, closes)) = crate::battleground::cancelled(result) {
            self.message = Some(Self::words(line, ""));
            if closes {
                (self.battleground.standby, self.battleground.ready) = (false, false);
            }
        }
        self.battleground_told(2114, "You have left the queue", RED);
        self.battleground.current = None;
        self.render();
    }

    /// `OnReady`: the battle is ready. The window that asks whether to enter opens (or closes, if it was open),
    /// with a sound and a line of the chat, and counts its twenty seconds.
    fn battleground_ready(&mut self, name: String, lobby: String) {
        (self.battleground.queue, self.battleground.lobby) = (name, lobby);
        if self.battleground.entry.take().is_none() {
            self.battleground.entry = Some(now());
            sound::effect("se_btg_ready.wav", 1.0);
            self.battleground_told(2115, "Are you sure you want to join a battleground?", TOLD);
            after(1000, Action::BattlegroundTick);
        }
        self.redraw(&[&wnd::BATTLEGROUND_ENTRY]);
    }

    /// A second of the question whether to enter has passed: its count is written anew, and at nought it is
    /// answered with no (`UIEntryQueueRequestWnd::OnProcess` and `Paint`).
    pub(in crate::web::app) fn battleground_tick(&mut self) {
        let Some(since) = self.battleground.entry else { return };
        if crate::battleground::entry_seconds(now().wrapping_sub(since)) <= 0 {
            self.pressed(crate::press::battleground::Press::Decline);
        } else {
            self.redraw(&[&wnd::BATTLEGROUND_ENTRY]);
            after(1000, Action::BattlegroundTick);
        }
    }

    /// ZC_BATTLEFIELD_NOTIFY_CAMPINFO: the player's own team, or another's: the world goes by them for whom a
    /// press fights, and puts the client's mark over one of another team (`OpenWordDisplay`).
    fn battle_team(&mut self, account: u32, team: u8) {
        let own = self.own_account();
        if account == own {
            self.battleground.team = team;
            world::own_team(team);
        } else {
            world::team(account, team);
            let foe = crate::battleground::foe(self.battleground.team, team);
            world::sign(account, own, world::Board::Foe, foe.then(Self::foe_mark));
        }
    }

    /// Another map: its score and where the team was are of the map before (the client forgets the score as it
    /// loads a map; a mark of the minimap goes with the map too).
    pub(in crate::web::app) fn battleground_map_left(&mut self) {
        self.battleground.score = None;
        self.battleground.places.clear();
        world::teams_left();
    }
}
