//! The menu the right button opens on another player (the client's `CGameMode::OpenActorMenu`, the part for
//! a player who is not a game master, looked at by one who is not): its rows, in the client's order and with
//! the words of its message table.
//!
//! A system that has something to do with another player gives the menu a row here (and its `Entry` in
//! `crate::press::player_menu`, and what it does in `app/clicks/player_menu.rs`).

use super::*;
use crate::press::player_menu::{Entry, Press};

impl App {
    /// The rows: looking at what the player wears, asking for a deal, inviting into the party (for a leader,
    /// when the player is not in it already) and into the guild, and for a guild's master the alliance and the
    /// opposition (each only while it does not stand); then the talk between two, making a friend of the player and a
    /// letter; then turning the player's whispers away.
    ///
    /// Not here, with the reason: the first two rows are left out by the client in two states of the player's
    /// own character that were not made out (`StateFlag01`, `StateFlag02`), and always show here; adopting
    /// (`MSI_REQ_JOIN_BABY`) needs a family, which this client has not; the row that reports a player (line
    /// 4067) is past the end of this client's message table, so its words are not known; the last row,
    /// `[C-Code]`, shows a code of the character whose making was not found. A game master's rows are left out.
    pub(in crate::web) fn player_menu(&self) -> String {
        let Some(menu) = &self.player_menu else { return String::new() };
        let named = |line: usize, otherwise: &str| Self::words(line, otherwise).replace("%s", &menu.name);
        let mut rows = menu_row(&named(1360, "Check %s's Equipment Info"), Press::Do(Entry::Equipment));
        rows.push_str(&menu_row(&named(391, "Request a deal with (%s)"), Press::Do(Entry::Deal)));
        if self.leads_party() && !self.party.members.iter().any(|member| member.name == menu.name) {
            rows.push_str(&menu_row(&named(392, "Ask (%s) to join your party"), Press::Do(Entry::Invite)));
        }
        // Into the guild: for one whose position invites, when the player is of no guild.
        if self.guild.id != 0 && self.guild.invites && menu.guild == 0 {
            rows.push_str(&menu_row(&named(382, "Send (%s) a Guild Invitation"), Press::Do(Entry::Guild)));
        }
        // The master's rows, under a line of their own, on a player of another guild: an alliance asked of that
        // player's guild, and that guild set as one opposed; neither for a relation that stands already
        // (`crate::guild::relation_rows`: the owner's decision, against the client, which offers both always).
        let related = self.guild.related.iter().map(|(guild, _, opposed)| (*guild, *opposed));
        let (ally, oppose) = crate::guild::relation_rows(self.guild.id, self.guild.master, menu.guild, related);
        if ally || oppose {
            rows.push_str(MENU_BOUNDARY);
        }
        if ally {
            rows.push_str(&menu_row(&Self::words(399, "Send an Guild Alliance Request"), Press::Do(Entry::Ally)));
        }
        if oppose {
            rows.push_str(&menu_row(&Self::words(403, "Set this guild as an Antagonist"), Press::Do(Entry::Oppose)));
        }
        rows.push_str(MENU_BOUNDARY);
        rows.push_str(&menu_row(&Self::words(360, "Open 1:1 Chat"), Press::Do(Entry::Chat)));
        // Not for one who is a friend already.
        if !self.friends.has(&menu.name) {
            rows.push_str(&menu_row(&Self::words(358, "Register as a Friend"), Press::Do(Entry::Friend)));
        }
        rows.push_str(&menu_row(&Self::words(2808, "Send a mail"), Press::Do(Entry::Mail)));
        rows.push_str(MENU_BOUNDARY);
        // For one whose whispers the player has turned away the row lets them in again (the client's
        // `g_whisperBlockMgr`).
        let block = if self.commands.blocked.contains(&menu.name) { Self::words(3819, "Unblock Chat (Toggle)") } else { Self::words(
            3820,
            "Block Chat (Toggle)"
        ) };
        rows.push_str(&menu_row(&block, Press::Do(Entry::Block)));
        rows
    }
}
