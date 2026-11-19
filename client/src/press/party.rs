//! The presses of the party: its window with the list of members and the bar of buttons under it, the menu on a
//! member, the windows that set the party up and invite into it, and the questions asked on the way.

/// A row of the menu the right button opens on a member (the client's `UIMessengerGroupWnd`, `UM_MAKEMENU`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entry {
    /// A letter to the member.
    Mail,
    /// The player's own row: leaving the party.
    Leave,
    /// The words typed next go to that member alone.
    Chat,
    Expel,
    /// The party is handed to that member.
    HandOver,
    /// The member's small window is put on the screen, or taken off it.
    AddSmall,
    RemoveSmall,
}

impl Entry {
    pub const ALL: [Entry; 7] = [Entry::Mail, Entry::Leave, Entry::Chat, Entry::Expel, Entry::HandOver, Entry::AddSmall, Entry::RemoveSmall];

    fn text(self) -> &'static str {
        match self {
            Entry::Mail => "mail",
            Entry::Leave => "leave",
            Entry::Chat => "chat",
            Entry::Expel => "expel",
            Entry::HandOver => "hand-over",
            Entry::AddSmall => "add-small",
            Entry::RemoveSmall => "remove-small",
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
        /// The party window is opened or closed: the menu's icon, the cross in its corner.
        Window = "party",
        /// The button by the cross: the window in its small form, or back in its large one.
        Size = "party-size",
        /// A member in the list, counted from the top: it is picked.
        Row(usize) = "party-row",
        /// The bar's buttons: the Adventurer's Agency, the party's setup, the lock, making a party, leaving
        /// it, inviting into it, putting the small windows in order.
        Agency = "party-agency",
        Setup = "party-setup",
        Lock = "party-lock",
        Create = "party-create",
        Leave = "party-leave",
        Invite = "party-invite",
        Sort = "party-sort",
        /// The Friend tab under the list.
        Friends = "party-friends",
        /// The setup window: how experience is shared (0 each its own, 1 evenly), who may pick items up (0 the
        /// one who earned them, 1 anyone of the party), OK and Cancel.
        Experience(u32) = "party-experience",
        Pickup(u8) = "party-pickup",
        SetupOk = "party-setup-ok",
        SetupCancel = "party-setup-cancel",
        /// The window that asks whom to invite: OK and Cancel.
        InviteOk = "party-invite-ok",
        InviteCancel = "party-invite-cancel",
        /// What the party asks (to join, to leave, to expel, to hand over): OK and Cancel.
        Yes = "party-yes",
        No = "party-no",
        /// A row of the menu on a member.
        Do(Entry) = "party-do",
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Press};
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let mut all = vec![
            Press::Window, Press::Size, Press::Row(3), Press::Agency, Press::Setup, Press::Lock, Press::Create, Press::Leave, Press::Invite, Press::Sort, Press::Friends,
            Press::Experience(1), Press::Pickup(0), Press::SetupOk, Press::SetupCancel, Press::InviteOk, Press::InviteCancel, Press::Yes, Press::No,
        ];
        all.extend(Entry::ALL.map(Press::Do));
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }

    #[test]
    fn a_row_the_menu_has_not_is_no_press() {
        assert_eq!(Press::parse("party-do", "promote"), None);
    }
}
