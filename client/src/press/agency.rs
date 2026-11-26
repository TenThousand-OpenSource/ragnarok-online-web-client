//! The presses of the Adventurer's Agency: the menu's icon, the board of notices (the client's
//! `UIAdvenPartyBoardWnd`), the window a notice is put up and changed in (`UIRegisterPartyWnd`), the one a
//! notice is read in before asking to join (`UIApplyForPartyWnd`), and the question to a leader whether to let
//! someone in (`UIRequestJoinPartyWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon: the board opens, asking for its first page, or closes if it was open.
        Window = "agency",
        /// The board's cross and its Close button.
        Close = "agency-close",
        /// One of the four roles of the search: wanted, or not.
        Role(usize) = "agency-role",
        /// The box of purposes of the search: its list opens or closes; and a purpose of the list.
        Purposes = "agency-purposes",
        Purpose(i32) = "agency-purpose",
        /// Search, Reset, and the button that asks for the first page anew.
        Search = "agency-search",
        Reset = "agency-reset",
        Reload = "agency-reload",
        /// The arrows of the pages.
        Before = "agency-before",
        After = "agency-after",
        /// A notice of the page, counted from the top.
        Row(usize) = "agency-row",
        /// The player's own notice above the list: it is changed.
        Own = "agency-own",
        /// The strip above the list while the player has no notice: one is put up.
        Recruit = "agency-recruit",
        /// "Cancel Recruitment": the player's notice is taken down.
        Stop = "agency-stop",
        /// The window a notice is written in: a role, the box of purposes and a purpose of it, Register or
        /// Edit, and Close (its cross too).
        NoticeRole(usize) = "agency-notice-role",
        NoticePurposes = "agency-notice-purposes",
        NoticePurpose(i32) = "agency-notice-purpose",
        NoticeOk = "agency-notice-ok",
        NoticeClose = "agency-notice-close",
        /// The window a notice is read in: a talk with its leader, asking to join, and Close (its cross too).
        Talk = "agency-talk",
        Join = "agency-join",
        ReadClose = "agency-read-close",
        /// The question to a leader: let in, refuse (its cross too), and the box that blocks this player's
        /// requests from then on.
        Accept = "agency-accept",
        Refuse = "agency-refuse",
        Block = "agency-block",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let board = [Press::Window, Press::Close, Press::Role(3), Press::Purposes, Press::Purpose(6), Press::Search, Press::Reset, Press::Reload];
        let list = [Press::Before, Press::After, Press::Row(9), Press::Own, Press::Recruit, Press::Stop];
        let notice = [Press::NoticeRole(1), Press::NoticePurposes, Press::NoticePurpose(2), Press::NoticeOk, Press::NoticeClose];
        let rest = [Press::Talk, Press::Join, Press::ReadClose, Press::Accept, Press::Refuse, Press::Block];
        crate::press::testing::read_back(board.into_iter().chain(list).chain(notice).chain(rest));
    }
}
