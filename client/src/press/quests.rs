//! The presses of the quests: the journal with its tabs, filters, search and rows, the pane of one quest, the
//! button that tells of a new quest, and the list of quests on the screen.

/// A row of the journal: a quest of the server, or one of the things the client recommends, by its number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    pub number: u32,
    pub recommended: bool,
}

impl std::fmt::Display for Row {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(to, "{}{}", if self.recommended { "r" } else { "" }, self.number)
    }
}

impl std::str::FromStr for Row {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        let (recommended, number) = match text.strip_prefix('r') {
            Some(number) => (true, number),
            None => (false, text),
        };
        Ok(Row { number: number.parse().map_err(|_| ())?, recommended })
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// The journal is opened or closed: its icon in the menu, and its Close button.
        Window = "quest",
        /// A tab down the journal's left side, counted from the top.
        Tab(usize) = "quest-tab",
        /// A filter of the journal's lower row, by the client's number for it.
        Filter(u32) = "quest-filter",
        /// The button by the journal's search: the quests are searched for what was typed.
        Search = "quest-search",
        /// A row of the journal: its pane is opened.
        Pick(Row) = "quest-pick",
        /// The box in a row's corner: whether the quest is in the list on the screen.
        Listed(u32) = "quest-listed",
        /// The lock in a row: a quest of the server is switched on or off, a recommended one put last or back.
        Lock(Row) = "quest-lock",
        /// The box under the journal: whether the list on the screen shows.
        Display = "quest-display",
        /// The pane of one quest is closed.
        PaneClose = "quest-pane-close",
        /// The button in the screen's corner that tells of a new quest: the journal is opened.
        Notice = "quest-notice",
    }
}

#[cfg(test)]
mod tests {
    use super::{Press, Row};
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let row = Row { number: 7122, recommended: false };
        let recommended = Row { number: 3, recommended: true };
        let all = vec![Press::Window, Press::Tab(2), Press::Filter(4), Press::Search, Press::Pick(row), Press::Pick(recommended), Press::Listed(7122), Press::Lock(row), Press::Lock(recommended), Press::Display, Press::PaneClose, Press::Notice];
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }

    #[test]
    fn a_row_is_written_with_its_kind() {
        assert_eq!(Row { number: 3, recommended: true }.to_string(), "r3");
        assert_eq!("7122".parse(), Ok(Row { number: 7122, recommended: false }));
        assert_eq!("rx".parse::<Row>(), Err(()));
    }
}
