//! The presses of the chat window (`UINewChatWnd`): its tabs, its lock, the small buttons at the right of the
//! tabs, the buttons that change its size and hide its log; of a tab's own window (`UISubChatHisWnd`); of the
//! window that sets what a tab lists (`UIBattleMsgOptionWnd`); and a link to an item in a line.

presses! {
    /// What can be pressed here.
    Press {
        /// A tab of the log is opened: which, counted from 0 among all the tabs.
        Tab(usize) = "chat-tab",
        /// The lock that keeps the window where it is.
        Lock = "chat-lock",
        /// The button that changes how tall the log is.
        Size = "chat-size",
        /// The button that hides the log and brings it back.
        Mini = "chat-mini",
        /// The small buttons of the tabs' line: the open tab is taken away (`ID_MINI`), a tab is added
        /// (`ID_ADD_CHAT_TAB`), the open tab goes into a window of its own (`ID_STICK_OFF`), and what it lists
        /// is set (`ID_OPTION`).
        Remove = "chat-remove",
        Add = "chat-add",
        Detach = "chat-detach",
        Options = "chat-options",
        /// The buttons of a tab's own window, by the tab: it is taken away, put back among the tabs
        /// (`ID_STICK_ON`), and what it lists is set.
        OwnRemove(usize) = "chat-own-remove",
        Attach(usize) = "chat-attach",
        OwnOptions(usize) = "chat-own-options",
        /// A row of the window that sets what a tab lists: lines of that kind are listed, or no more.
        Lists(usize) = "chat-lists",
        /// Its box that has the tab list every kind, and its close button.
        ListsAll = "chat-lists-all",
        OptionsClose = "chat-options-close",
        /// A link to an item in a line: what is known of the item is shown.
        Link(u32) = "chat-link",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Tab(1), Press::Lock, Press::Size, Press::Mini, Press::Remove, Press::Add, Press::Detach, Press::Options, Press::OwnRemove(2), Press::Attach(3), Press::OwnOptions(4), Press::Lists(16), Press::ListsAll, Press::OptionsClose, Press::Link(501)];
        assert_eq!(all.len(), Press::NAMES.len(), "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
