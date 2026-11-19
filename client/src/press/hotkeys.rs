//! The presses of the Shortcut Settings window (the client's `UIHotKeyWnd`): its tabs, its pages, its rows and
//! its buttons (see `crate::press`).

use super::Step;

presses! {
    /// What can be pressed here.
    Press {
        /// From the window of game options: the window opens, or closes if it was open.
        Open = "keys",
        /// A tab of the window, by its place: the table of keys it shows.
        Tab(usize) = "keys-tab",
        /// The arrows at the top right: the next page of a long table, and the one before.
        Page(Step) = "keys-page",
        /// A row of the page, by its place on it: its key is to be set.
        Row(usize) = "keys-row",
        /// Reset: every key as the client has it.
        Reset = "keys-reset",
        /// Cancel: what was set is dropped, and the window stays.
        Cancel = "keys-cancel",
        /// OK: what was set is kept, and the window closes.
        Ok = "keys-ok",
        /// Close, and the cross of the title bar: the window closes, and what was set is dropped.
        Close = "keys-close",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [
            Press::Open, Press::Tab(2), Press::Page(Step::More), Press::Page(Step::Less), Press::Row(35), Press::Reset,
            Press::Cancel, Press::Ok, Press::Close
        ];
        assert_eq!(all.len(), Press::NAMES.len() + 1, "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
