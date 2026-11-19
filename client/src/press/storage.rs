//! The presses of the storage's window (the Kafra's, or a guild's): its tabs, the buttons that show a tab in a
//! window of its own, the order of its lists, its search and its "close".

presses! {
    /// What can be pressed here.
    Press {
        /// A tab down the window's side is opened: which, counted from 0.
        Tab(usize) = "storage-tab",
        /// One of the eight small buttons: the things of that tab are shown in a window of their own, or no more.
        Apart(usize) = "storage-apart",
        /// The box that says how the lists are ordered: its three choices are shown, or put away.
        Sorting = "storage-sorting",
        /// One of the three orders is chosen: which, counted from 0.
        Sort(usize) = "storage-sort",
        /// The button beside the field of the search. The field is a form and the button sends it, so the press
        /// does nothing.
        Search = "storage-search",
        /// The window of what was found is closed.
        FoundClose = "storage-found-close",
        /// The storage is closed.
        Close = "storage-close",
        /// The OK of the box that asks how many. The box is a form and its OK sends it, so the press does nothing.
        CountOk = "storage-count-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Tab(5), Press::Apart(7), Press::Sorting, Press::Sort(2), Press::Search, Press::FoundClose, Press::Close, Press::CountOk];
        assert_eq!(all.len(), Press::NAMES.len());
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
