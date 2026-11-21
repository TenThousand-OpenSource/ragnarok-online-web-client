//! The presses of the guild: its window with its tabs, the list of members and the menu on one, the positions
//! and the skills, the emblem, the notice, the castles held with the menu on one, and the questions asked on
//! the way.

/// A row of a menu the right button opens in the guild window: on a member (the client's
/// `UIGuildMemberManageWnd`, `UM_MAKEMENU`) or on a guild allied or opposed (`UIGuildInfoManageWnd`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entry {
    /// The player's own row: leaving the guild.
    Leave,
    Expel,
    /// The member is invited into the player's party.
    Party,
    /// The words typed next go to that member alone.
    Chat,
    /// A letter to the member.
    Mail,
    /// The master hands the guild to the member.
    Hand,
    /// The alliance with a guild, or the opposition to it, is ended.
    Unrelate,
}

impl Entry {
    pub const ALL: [Entry; 7] = [Entry::Leave, Entry::Expel, Entry::Party, Entry::Chat, Entry::Mail, Entry::Hand, Entry::Unrelate];

    fn text(self) -> &'static str {
        match self {
            Entry::Leave => "leave",
            Entry::Expel => "expel",
            Entry::Party => "party",
            Entry::Chat => "chat",
            Entry::Mail => "mail",
            Entry::Hand => "hand",
            Entry::Unrelate => "unrelate",
        }
    }
}

worded!(Entry);

presses! {
    /// What can be pressed here.
    Press {
        /// The guild window is opened or closed: the menu's icon, the key, the cross in its corner.
        Window = "guild",
        /// A tab of the window, counted from the left.
        Tab(usize) = "guild-tab",
        /// A member in the list, as the list stands: it is picked.
        Member(usize) = "guild-member",
        /// The box under the list: members in the game are listed first, or not.
        OnlineFirst = "guild-online-first",
        /// A header of the list of members, counted from the left: the list is ordered by that column.
        Sort(usize) = "guild-sort",
        /// The button by a member's position (the member by its place in the list): the positions to pick from
        /// open under it.
        Combo(usize) = "guild-combo",
        /// A position picked there for that member.
        Position(u32) = "guild-position",
        /// The button that sends the positions the members were given.
        MembersOk = "guild-members-ok",
        /// A box of a position (by its number): whether it invites, expels, opens the storage.
        Invites(u32) = "guild-invites",
        Expels(u32) = "guild-expels",
        Stores(u32) = "guild-stores",
        /// The button that sends what was changed of the positions.
        PositionsOk = "guild-positions-ok",
        /// The button by the emblem: a file is picked to be the guild's emblem.
        Emblem = "guild-emblem",
        /// A skill in the list of the guild's skills, by its place: it is picked.
        Skill(usize) = "guild-skill",
        /// The button by a skill of that number that can be raised: a point of the guild goes into it.
        Raise(u16) = "guild-raise",
        /// The button that uses the skill picked.
        UseSkill = "guild-use",
        /// The notice's button: what is typed becomes the guild's notice.
        Notice = "guild-notice",
        /// The button that breaks the guild up.
        Disband = "guild-disband",
        /// What the guild asks (to join, why one leaves, why one is expelled): OK and Cancel.
        Yes = "guild-yes",
        No = "guild-no",
        /// A row of the menu on a member.
        Do(Entry) = "guild-do",
        /// A castle in the list of those the guild holds, by its line: it is picked.
        Castle(usize) = "guild-castle",
        /// The rows of the menu on a castle (the client's `UIListBoxAgitMenu`): going there, and asking for its
        /// economy and defence.
        CastleGo = "guild-castle-go",
        CastleAsk = "guild-castle-ask",
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Press};
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let mut all = vec![Press::Window, Press::Tab(3), Press::Member(2), Press::OnlineFirst, Press::Notice, Press::Disband, Press::Yes, Press::No];
        all.extend([
            Press::Sort(1), Press::Combo(2), Press::Position(3), Press::MembersOk, Press::Invites(1), Press::Expels(2),
            Press::Stores(3), Press::PositionsOk, Press::Emblem, Press::Skill(0), Press::Raise(10000), Press::UseSkill
        ]);
        all.extend(Entry::ALL.map(Press::Do));
        all.extend([Press::Castle(2), Press::CastleGo, Press::CastleAsk]);
        crate::press::testing::read_back(all);
    }

    #[test]
    fn a_row_the_menu_has_not_is_no_press() {
        assert_eq!(Press::parse("guild-do", "promote"), None);
    }
}
