//! The waits before skills may be used again, as the server tells of them.

use super::*;
use crate::cooldowns::Wait;
use crate::net::cooldowns::{self, Told};

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn cooldown_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let told = match id {
            // A skill was used and cannot be again for a time (the client's `ApplyCooldownStart`).
            cooldowns::ZC_SKILL_POSTDELAY => cooldowns::started(packet).into_iter().collect(),
            // The waits still running as the character comes in (`LoadCooldownRecords`).
            cooldowns::ZC_SKILL_POSTDELAY_LIST => cooldowns::running(packet),
            // The same with their lengths: it replaces all that was known (`LoadCooldownSnapshot`).
            cooldowns::ZC_SKILL_POSTDELAY_LIST2 => {
                self.cooldowns.waits.0.clear();
                cooldowns::snapshot(packet)
            }
            _ => return false,
        };
        let now = now();
        self.cooldowns.waits.prune(now);
        for Told { skill, duration, remaining } in told {
            self.cooldowns.waits.begin(Wait { skill, end: now.saturating_add(remaining), duration });
        }
        self.redraw(&[&wnd::HOTKEYS]);
        true
    }

    /// A skill that still waits was asked for: the chat says so as the client does (`MSI_SKILLINTERVAL2`, line
    /// 1460 of its message table, with the skill's name, in red among the lines of skills that failed;
    /// `CSession::IsRequestReady` is what holds it back). The client keeps a wait only for a skill that is on
    /// the hotkey bar (`ApplyCooldownStart`); here every wait the server tells of is kept.
    pub(in crate::web::app) fn skill_waits(&mut self, skill: u16) {
        let name = data::skill(skill).map_or_else(|| skill.to_string(), |(_, name, _)| name);
        if let Some(line) = data::message(1460) {
            self.chat_line_of(said::SKILL_FAILED, line.replacen("%s", &name, 1), RED);
        }
    }
}
