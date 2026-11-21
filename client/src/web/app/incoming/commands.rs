//! What the server answers to the chat commands: how many play, who may whisper, the rankings and the points
//! of fame, and what a game master's commands hear back.

use super::*;
use crate::net::commands;

/// Where the client's message table has what these answers say: the count of players (`MSI_NUMPEOPLE`, with
/// the number); after a character's name, that its whispers could not be blocked, or not for the list being
/// full, that they are let in again, and that this could not be done, or not for the list being full
/// (`ReceiveWhisperFilterResult`); that all whispers are blocked, that this failed, that all are let in, that
/// this failed (ZC_SETTING_WHISPER_STATE); and the heading of the list of the blocked, or that there is none
/// (ZC_WHISPER_LIST).
const PLAYERS: usize = 178;
const NOT_BLOCKED: [usize; 2] = [194, 195];
const LET_IN: [usize; 3] = [196, 197, 198];
const ALL_BLOCKED: [usize; 2] = [3427, 3428];
const ALL_LET_IN: [usize; 2] = [3429, 3430];
const BLOCKED_LIST: [usize; 2] = [3396, 3395];

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other, and for the count of players,
    /// which the chat's tabs show too (`incoming/status.rs`).
    pub(super) fn commands_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The count that `/who` asked for is told in the chat, in yellow (`0xFFFF`).
            commands::ZC_USER_COUNT => {
                if let (true, Some(count)) = (std::mem::take(&mut self.commands.who_asked), commands::users(packet)) {
                    self.say_line(PLAYERS, &count.to_string(), "");
                }
                return false;
            }
            // A character's whispers were blocked or let in again: the chat tells how it went, the name first.
            // That they were blocked is not told (the client has no line for it).
            commands::ZC_SETTING_WHISPER_PC => {
                let name = self.commands.whisper_name.clone();
                match commands::whisper_set(packet) {
                    Some((false, commands::FAILED)) => self.say_line(NOT_BLOCKED[0], "", &name),
                    Some((false, commands::FULL)) => self.say_line(NOT_BLOCKED[1], "", &name),
                    Some((true, result @ (commands::DONE | commands::FAILED | commands::FULL))) => self.say_line(LET_IN[usize::from(result)], "", &name),
                    _ => {}
                }
            }
            commands::ZC_SETTING_WHISPER_STATE => match commands::whisper_set(packet) {
                Some((allow, result @ (commands::DONE | commands::FAILED))) =>
                    self.say_line(if allow { ALL_LET_IN } else { ALL_BLOCKED }[usize::from(result)], "", ""),
                _ => {}
            },
            // The characters whose whispers are blocked: a heading, then each name in its own line (`0xFFFF00`).
            commands::ZC_WHISPER_LIST => {
                let names = commands::whisper_blocked(packet);
                self.say_line(BLOCKED_LIST[usize::from(names.is_empty())], "", "");
                for name in names {
                    self.chat_line(name, CYAN);
                }
            }
            // A ranking asked for with `/blacksmith`, `/alchemist` or `/taekwon`: its lines in the chat
            // (`CGameMode::ReceiveRankingList`).
            commands::ZC_ACK_RANKING => {
                if let Some(commands::Ranking { kind, ranked, own }) = commands::ranking(packet) {
                    self.ranking_said(kind, &ranked, own, true);
                }
            }
            // The same told by the characters' numbers: it waits for their names. While one waits, another
            // is not taken (`CSession::SetShowRankingSchedule`).
            commands::ZC_ACK_RANKING2 => {
                if let (None, Some(ranking)) = (&self.commands.ranking, commands::ranking_by_number(packet)) {
                    self.commands.ranking = Some((ranking, crate::web::now()));
                    self.ranking_looked();
                }
            }
            // Points of fame gained: a line of the message table with the points and their sum, in the
            // colour of its ranking.
            commands::ZC_UPDATE_RANKING_POINT => {
                let gained = commands::fame_gained(packet).and_then(|(kind, points, sum)| Some((crate::commands::fame_line(kind)?, points, sum)));
                if let Some(((line, colour), points, sum)) = gained {
                    self.say_filled(said::GENERAL, line, &[&points.to_string(), &sum.to_string()], colour);
                }
            }
            // What a game master's kick came to (`case ID_ZC_ACK_DISCONNECT_CHARACTER`: `MSI_FAIL_DISCONNECT_CHARACTER`
            // or `MSI_SUCCESS_DISCONNECT_CHARACTER`, in its 0x32FF).
            commands::ZC_ACK_DISCONNECT_CHARACTER => {
                if let Some(done) = commands::kicked(packet) {
                    self.say_filled(said::GENERAL, if done { 151 } else { 152 }, &[], "#ff3200");
                }
            }
            // A player's stats told to a game master: five lines (`CGameMode::ReceiveStatDump`).
            commands::ZC_ACK_STATUS_GM => {
                if let Some((stats, more)) = commands::gm_stats(packet) {
                    for (line, colour) in crate::commands::stat_dump(stats, more) {
                        self.chat_line(line, colour);
                    }
                }
            }
            // The client's handler has an arm for it that does nothing (`CGameMode`'s packet handler: with
            // `ID_PING` and the others it only takes).
            commands::ZC_ACK_ACCOUNTNAME => {}
            _ => return false,
        }
        true
    }

    /// The lines of a ranking in the chat, each in its colour.
    fn ranking_said(&mut self, kind: u16, ranked: &[(String, u32)], own: u32, padded: bool) {
        for (line, colour) in crate::commands::ranking_lines(kind, ranked, own, padded) {
            self.chat_line(line, colour);
        }
    }

    /// Looks whether the ranking that waits for names can be written (`CShowRankingScheduler::Process`, run as
    /// the ranking arrives and every second after): it is once every name is known or three seconds have
    /// passed, with "Nameless" (`MSI_NAMELESS_USER`) for a name still unknown; until then each look asks the
    /// server for the names and says "Fetching rank..." (`MSI_RANK_LOADING`, in the client's 0xFFFF).
    pub(in crate::web::app) fn ranking_looked(&mut self) {
        use crate::commands::{RANKING_LOADING, RANKING_LOOKS_EVERY, RANKING_WAITS, ranking_named};
        let Some((ranking, since)) = self.commands.ranking.take() else { return };
        let (named, known) = ranking_named(&ranking.ranked, data::character_name, &Self::words(581, "Nameless"));
        if known || crate::web::now() - since >= RANKING_WAITS {
            return self.ranking_said(ranking.kind, &named, ranking.own, false);
        }
        self.ask_names_wanted();
        self.say_filled(said::GENERAL, RANKING_LOADING, &[], YELLOW);
        self.commands.ranking = Some((ranking, since));
        after(RANKING_LOOKS_EVERY, Action::RankingLooked);
    }

    /// A line of the client's message table in the chat, in yellow: after `before`, and with `number` written
    /// into it where it asks for one.
    fn say_line(&mut self, line: usize, number: &str, before: &str) {
        if let Some(said) = data::message(line) {
            self.chat_line(format!("{before}{}", said.replacen("%d", number, 1)), YELLOW);
        }
    }
}
