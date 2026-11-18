//! The presses of the Adventure Guide (the client's `CUIAdventureGuide`): its icon and close button, its tabs,
//! the kinds under a tab and the rows of a page (see `crate::press`).

presses! {
    /// What can be pressed here.
    Press {
        /// The guide's icon in the menu, and the window's own close button: it opens, or closes if it was open.
        Open = "guide",
        /// A tab down the left, or a row of the first page, by the tab's place.
        Tab(usize) = "guide-tab",
        /// A kind under the open tab, by the number the guide's tables give it.
        Kind(u32) = "guide-kind",
        /// A row of a page, by what the page knows it by: a quest's or an item's number, or a monster's name in
        /// the guide's tables.
        Pick(String) = "guide-pick",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Open, Press::Tab(6), Press::Kind(12), Press::Pick("1002".to_owned()), Press::Pick("RED PLANT".to_owned())];
        assert_eq!(all.len(), Press::NAMES.len() + 1, "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
