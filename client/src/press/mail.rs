//! The presses of the mail (the client's RODEX): the notice of mail not read, the mailbox with its tabs, list
//! and search, a letter being read, a letter being written, and what they ask before they act.

use super::Step;

presses! {
    /// What can be pressed here.
    Press {
        /// The mailbox is opened or closed: the menu's icon, the notice of mail not read, the cross.
        Window = "mail",
        /// A tab of the mailbox: the three kinds of letters by their numbers (`net::mail::tab`), and 3 for
        /// what was searched for.
        Tab(u8) = "mail-tab",
        /// The mailbox is asked for anew.
        Refresh = "mail-refresh",
        /// A letter is to be written, to nobody yet.
        Write = "mail-write",
        /// The box of a row of the list, counted from the top: ticked or not.
        Check(usize) = "mail-check",
        /// The title of a row: the letter is read.
        Title(usize) = "mail-title",
        /// The sender of a row: a letter is written back.
        Sender(usize) = "mail-sender",
        /// The button of a row from someone not known: the letter is sent back.
        GiveBack(usize) = "mail-give-back",
        /// The two buttons under the list: Delete All and Claim All, or Delete and Claim for the rows ticked.
        Delete = "mail-delete",
        Claim = "mail-claim",
        /// The page before, or the next.
        Page(Step) = "mail-page",
        /// What is searched in: the titles, or the senders.
        SearchIn(bool) = "mail-search-in",
        /// The search's button: the form it stands in is sent.
        Search = "mail-search",
        /// The letter being read: its cross.
        ReadClose = "mail-read-close",
        /// Its things, and its zeny, are taken.
        TakeItems = "mail-take-items",
        TakeZeny = "mail-take-zeny",
        /// It is deleted.
        ReadDelete = "mail-read-delete",
        /// A letter is written back to its sender.
        Reply = "mail-reply",
        /// It is sent back to its sender.
        ReadGiveBack = "mail-read-give-back",
        /// The letter being written: its cross.
        WriteClose = "mail-write-close",
        /// The receiver typed is asked about.
        Validate = "mail-validate",
        Send = "mail-send",
        /// What the mail asks: OK and Cancel.
        Yes = "mail-yes",
        No = "mail-no",
        /// How many of a thing go into the letter: OK.
        CountOk = "mail-count-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::{Named, Step};

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [
            Press::Window,
            Press::Tab(3),
            Press::Refresh,
            Press::Write,
            Press::Check(5),
            Press::Title(0),
            Press::Sender(2),
            Press::GiveBack(1),
            Press::Delete,
            Press::Claim,
            Press::Page(Step::More),
            Press::Page(Step::Less),
            Press::SearchIn(true),
            Press::Search,
            Press::ReadClose,
            Press::TakeItems,
            Press::TakeZeny,
            Press::ReadDelete,
            Press::Reply,
            Press::ReadGiveBack,
            Press::WriteClose,
            Press::Validate,
            Press::Send,
            Press::Yes,
            Press::No,
            Press::CountOk,
        ];
        crate::press::testing::read_back(all);
    }

    #[test]
    fn a_row_that_is_no_number_is_no_press() {
        assert_eq!(Press::parse("mail-title", "first"), None);
    }
}
