//! What the server answers to the chat commands: how many play, and who may whisper.

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
                Some((allow, result @ (commands::DONE | commands::FAILED))) => self.say_line(if allow { ALL_LET_IN } else { ALL_BLOCKED }[usize::from(result)], "", ""),
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
            // A ranking asked for with `/blacksmith`, `/alchemist` or `/taekwon`: its lines in the chat.
            commands::ZC_ACK_RANKING => {
                if let Some((kind, ranked, own)) = commands::ranking(packet) {
                    for line in commands::ranking_lines(kind, &ranked, own) {
                        self.chat_line(line, YELLOW);
                    }
                }
            }
            _ => return false,
        }
        true
    }

    /// A line of the client's message table in the chat, in yellow: after `before`, and with `number` written
    /// into it where it asks for one.
    fn say_line(&mut self, line: usize, number: &str, before: &str) {
        if let Some(said) = data::message(line) {
            self.chat_line(format!("{before}{}", said.replacen("%d", number, 1)), YELLOW);
        }
    }
}
