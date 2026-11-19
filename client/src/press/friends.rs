//! The presses of the friends list: the Friend side of the window it shares with the party (the client's
//! `UIMessengerGroupWnd` with `mode` 0), the menu on a friend, the Friend Setup window, and the questions asked
//! on the way.

/// A row of the menu the right button opens on a friend (`UIMessengerGroupWnd`, `UM_MAKEMENU`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entry {
    /// A window opens for a talk with that friend alone.
    Chat,
    /// A letter to the friend.
    Mail,
    /// The friend is one no more.
    Remove,
    /// The friend is invited into the party.
    Invite,
}

impl Entry {
    pub const ALL: [Entry; 4] = [Entry::Chat, Entry::Mail, Entry::Remove, Entry::Invite];

    fn text(self) -> &'static str {
        match self {
            Entry::Chat => "chat",
            Entry::Mail => "mail",
            Entry::Remove => "remove",
            Entry::Invite => "invite",
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
        /// The Party tab under the list of friends: the window shows the party again.
        Party = "friends-party",
        /// A friend in the list, counted from the top: it is picked.
        Row(usize) = "friends-row",
        /// The bar's buttons: the talk with the friend picked, the Friend Setup window, removing the friend.
        Talk = "friends-talk",
        Setup = "friends-setup",
        Remove = "friends-remove",
        /// The Friend Setup window: its cross, and its boxes (0 a whisper of one who is no friend opens a
        /// window, 1 a friend's does, 2 a window opened so chimes).
        SetupClose = "friends-setup-close",
        Option(u8) = "friends-option",
        /// What the friends list asks (to be someone's friend, to remove one): OK and Cancel.
        Yes = "friends-yes",
        No = "friends-no",
        /// A row of the menu on a friend.
        Do(Entry) = "friends-do",
    }
}

#[cfg(test)]
mod tests {
    use super::{Entry, Press};
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let mut all = vec![Press::Party, Press::Row(3), Press::Talk, Press::Setup, Press::Remove, Press::SetupClose, Press::Option(2), Press::Yes, Press::No];
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
        assert_eq!(Press::parse("friends-do", "block"), None);
    }
}
