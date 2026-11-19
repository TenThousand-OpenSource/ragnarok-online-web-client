//! The presses of the books: the four buttons of the window a book is read in (the client's `UIBookWnd`), and
//! the two that the window of what an item is has for a book that is carried (`UIItemCollectionWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// The bookmark (`ID_BOOKMARK`): the page open is kept for this book.
        Mark = "book-mark",
        /// The page before (`ID_PREVIEW`).
        Before = "book-before",
        /// The page after (`ID_NEXTVIEW`).
        After = "book-after",
        /// The cross (`ID_CLOSE`): the page is kept as the bookmark and the window closes.
        Close = "book-close",
        /// "Read" on the item that is being looked at (`ID_BOOKITEM`).
        Read = "book-read",
        /// "Auto Read" on it (`ID_AUTOBOOKITEM`): the book is said line by line.
        Aloud = "book-aloud",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Mark, Press::Before, Press::After, Press::Close, Press::Read, Press::Aloud];
        assert_eq!(all.len(), Press::NAMES.len(), "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
