//! The presses of the chat window (`UINewChatWnd`): its tabs, its lock and the buttons that change its size and
//! hide its log.

presses! {
    /// What can be pressed here.
    Press {
        /// A tab of the log is opened: which, counted from 0.
        Tab(usize) = "chat-tab",
        /// The lock that keeps the window where it is.
        Lock = "chat-lock",
        /// The button that changes how tall the log is.
        Size = "chat-size",
        /// The button that hides the log and brings it back.
        Mini = "chat-mini",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Tab(1), Press::Lock, Press::Size, Press::Mini] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
