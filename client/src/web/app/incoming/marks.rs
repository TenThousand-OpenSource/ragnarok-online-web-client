//! Marks on the minimap that the server tells of: the members of the guild on this map, and the boss monster
//! (the client's `MM_ADDGUILDPOS` and `CGameMode::ReceiveBossMonsterInfo`).

use super::*;
use crate::net::marks;
use crate::net::notices::filled;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn mark_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // Where a member of the guild is; the player's own place is not kept (`case
            // ID_ZC_NOTIFY_POSITION_TO_GUILDM`). A place of -1, -1 is one the client keeps and never draws:
            // here it is forgotten.
            marks::ZC_NOTIFY_POSITION_TO_GUILDM => {
                if let Some((account, cell)) = marks::guild_place(packet).filter(|(account, _)| *account != self.own_account()) {
                    let kept = self.marks_here();
                    match cell {
                        Some(cell) => kept.guild.insert(account, cell),
                        None => kept.guild.remove(&account),
                    };
                    self.party_marks();
                }
            }
            // What is known of the boss monster: its place on the minimap, and a line in the chat, in white.
            marks::ZC_BOSS_INFO => {
                if let Some(boss) = marks::boss(packet) {
                    self.marks_here().boss = boss.marked();
                    if let Some((words, with)) = boss.says().and_then(|(line, with)| Some((data::message(line)?, with))) {
                        let with: Vec<&str> = with.iter().map(String::as_str).collect();
                        self.chat_line(filled(crate::i18n::tr(&words), &with), WHITE);
                    }
                    self.party_marks();
                }
            }
            _ => return false,
        }
        true
    }

    /// The marks of the map the character is on now: those kept of another map are forgotten first.
    fn marks_here(&mut self) -> &mut Marks {
        let visit = world::visit();
        if self.marks.visit != visit {
            self.marks = Marks { visit, ..Marks::default() };
        }
        &mut self.marks
    }
}
