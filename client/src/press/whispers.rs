//! The presses of a talk with one player in a window of its own (the client's `UIWhisperWnd`): its cross, and
//! the menu the right button opens on it (which window is meant is its place in the list of windows); and the OK
//! of the box that asks the away message.

presses! {
    /// What can be pressed here.
    Press {
        /// The cross of a window: the talk is put away.
        Close(usize) = "whisper-close",
        /// The menu's row that asks the other to be a friend (the client's
        /// `MENUID_ADD_TO_FRIEND_CHAR_LIST_FROM_WINDOW`).
        Friend = "whisper-friend",
        /// OK of the box that asks the away message (the client's `UIAutoMessageWnd`, `/am`).
        AwayOk = "away-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Close(3), Press::Friend, Press::AwayOk] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
        assert_eq!(Press::parse("whisper-close", "first"), None);
    }
}
