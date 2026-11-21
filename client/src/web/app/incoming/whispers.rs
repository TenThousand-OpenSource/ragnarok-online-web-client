//! Whispers: one heard, and the server's answer to one sent. Each goes to the window of the talk with that
//! player if there is one or one opens for it, and to the chat otherwise (the client's
//! `CGameMode::ProcessWhisper` and `ReceiveWhisperResult`).

use super::*;
use crate::whispers;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn whispers_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            crate::net::whispers::ZC_WHISPER => self.whisper_heard(packet),
            crate::net::whispers::ZC_ACK_WHISPER => self.whisper_answered(packet),
            _ => return false,
        }
        true
    }

    /// ZC_WHISPER: someone whispers. In the chat a friend's and a guild member's are marked as such.
    fn whisper_heard(&mut self, packet: &[u8]) {
        let Some((name, words)) = crate::net::whispers::whispered(packet) else { return };
        // While the away message is on the one who whispers is answered with it, whether a window of the
        // talk takes the whisper or the chat does (`ProcessWhisper` has the answer in both of its ways).
        self.away_answers(&name, &words);
        if self.whisper_shown(&name, whispers::line(&name, &words), whispers::THEIRS) {
            return;
        }
        if self.game.as_ref().is_some_and(|game| game.name == name) {
            return;
        }
        let guild = self.guild.members.iter().any(|member| member.name == name);
        self.chat_line_of(said::WHISPER, whispers::heard(&name, self.friends.has(&name), guild, crate::i18n::tr("From"), &words), YELLOW);
    }

    /// ZC_ACK_WHISPER: how the last whisper went. One that arrived is shown now, as the player's own line of
    /// the talk; one that did not, the chat says why after the other's name.
    fn whisper_answered(&mut self, packet: &[u8]) {
        let (to, words) = self.whispers.last.clone();
        match crate::net::whispers::whisper_answered(packet) {
            Some(whispers::answer::ARRIVED) => {
                let own = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
                if !self.whisper_shown(&to, whispers::line(&own, &words), whispers::OWN) {
                    let close = self.friends.has(&to) || self.guild.members.iter().any(|member| member.name == to);
                    self.chat_line_of(said::WHISPER, whispers::sent(&to, close, &words), YELLOW);
                }
            }
            Some(result) => {
                let why = Self::words(whispers::failure(result), "There is no such character name or the user is offline.");
                self.chat_line_of(said::WHISPER, format!("({to}){why}"), YELLOW);
            }
            None => {}
        }
    }
}
