//! The presses of the menu the right button opens on another player (the client's `CGameMode::OpenActorMenu`).

/// A row of the menu. A system that gives the menu a row adds it here, and `web/hud/player_menu.rs` says when
/// it shows and what it reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entry {
    /// What the player wears is looked at.
    Equipment,
    /// A deal is asked for.
    Deal,
    /// The player is invited into the party.
    Invite,
    /// The player is invited into the guild.
    Guild,
    /// The player's guild is asked for an alliance, by the master of the player's own.
    Ally,
    /// The player's guild is set as one opposed.
    Oppose,
    /// A window opens for a talk with that player alone.
    Chat,
    /// The player is asked to be a friend.
    Friend,
    Mail,
    /// The player's whispers are turned away, or let in again.
    Block,
}

impl Entry {
    pub const ALL: [Entry; 10] = [Entry::Equipment, Entry::Deal, Entry::Invite, Entry::Guild, Entry::Ally, Entry::Oppose, Entry::Chat, Entry::Friend, Entry::Mail, Entry::Block];

    fn text(self) -> &'static str {
        match self {
            Entry::Equipment => "equipment",
            Entry::Deal => "deal",
            Entry::Invite => "invite",
            Entry::Guild => "guild",
            Entry::Ally => "ally",
            Entry::Oppose => "oppose",
            Entry::Chat => "chat",
            Entry::Friend => "friend",
            Entry::Mail => "mail",
            Entry::Block => "block",
        }
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        to.write_str(self.text())
    }
}

impl std::str::FromStr for Entry {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        Entry::ALL.into_iter().find(|entry| entry.text() == text).ok_or(())
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// A row of the menu.
        Do(Entry) = "player-do",
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Press};
    use crate::press::Named;

    #[test]
    fn every_row_is_read_back_from_what_it_is_written_as() {
        for press in Entry::ALL.map(Press::Do) {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
        assert_eq!(Press::parse("player-do", "duel"), None);
    }
}
