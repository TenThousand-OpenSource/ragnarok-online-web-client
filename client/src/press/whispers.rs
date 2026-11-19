//! The presses of a talk with one player in a window of its own (the client's `UIWhisperWnd`): its cross, and
//! the menu the right button opens on it. Which window is meant is its place in the list of windows.

presses! {
    /// What can be pressed here.
    Press {
        /// The cross of a window: the talk is put away.
        Close(usize) = "whisper-close",
        /// The menu's row that asks the other to be a friend (the client's
        /// `MENUID_ADD_TO_FRIEND_CHAR_LIST_FROM_WINDOW`).
        Friend = "whisper-friend",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Close(3), Press::Friend] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
        assert_eq!(Press::parse("whisper-close", "first"), None);
    }
}
