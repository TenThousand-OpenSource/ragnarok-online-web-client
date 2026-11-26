//! The presses of the battlegrounds: the menu's icon, the window that lists them (the client's
//! `UIEntryQueueWnd`), its help (`UIEntryQueueHelpWnd`), the window of the queue one waits in
//! (`UIEntryQueueStandByWnd`) and the one that asks whether to enter (`UIEntryQueueRequestWnd`).

/// The three ways to apply, each a button of the list's window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    Alone,
    Party,
    Guild,
}

impl Way {
    pub const ALL: [Way; 3] = [Way::Alone, Way::Party, Way::Guild];

    fn text(self) -> &'static str {
        match self {
            Way::Alone => "alone",
            Way::Party => "party",
            Way::Guild => "guild",
        }
    }
}

worded!(Way);

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon: the list opens, or closes if it was open; one who waits in a queue asks for the
        /// place in it first.
        Window = "battleground",
        /// The cross of the list.
        Close = "battleground-close",
        /// A battleground of the list, by its number: it is picked.
        Row(u32) = "battleground-row",
        /// The three buttons under the list: to apply alone, as a party, as a guild.
        Apply(Way) = "battleground-apply",
        /// The list's Help button, and the help's Close.
        Help = "battleground-help",
        HelpClose = "battleground-help-close",
        /// The question before applying: yes and no.
        Yes = "battleground-yes",
        No = "battleground-no",
        /// The window of the queue: to leave the queue, and to put the window away.
        Leave = "battleground-leave",
        StandbyClose = "battleground-standby-close",
        /// The battle is ready: to enter, and not to.
        Enter = "battleground-enter",
        Decline = "battleground-decline",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let ways = Way::ALL.map(Press::Apply);
        let plain = [Press::Window, Press::Close, Press::Row(3), Press::Help, Press::HelpClose, Press::Yes, Press::No];
        let queue = [Press::Leave, Press::StandbyClose, Press::Enter, Press::Decline];
        crate::press::testing::read_back(ways.into_iter().chain(plain).chain(queue));
    }
}
