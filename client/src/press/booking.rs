//! The presses of the old party booking: the list of ads (the client's `UISeekPartyListWnd`) and its help
//! (`UIPartyBookingHelpWnd`), the window an ad is registered in (`UISeekPartyMBWnd`), the window of one's own
//! ad (`UISeekPartyWnd`) and the one its jobs are changed in (`UIJobListWnd`).

/// The boxes that drop a list to pick of (`UIComboBoxDefWnd`): the list's four, and the three of registering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Region,
    Map,
    Group,
    Job,
    OwnRegion,
    OwnMap,
    OwnGroup,
}

impl Choice {
    pub const ALL: [Choice; 7] = [Choice::Region, Choice::Map, Choice::Group, Choice::Job, Choice::OwnRegion, Choice::OwnMap, Choice::OwnGroup];

    fn text(self) -> &'static str {
        match self {
            Choice::Region => "region",
            Choice::Map => "map",
            Choice::Group => "group",
            Choice::Job => "job",
            Choice::OwnRegion => "own-region",
            Choice::OwnMap => "own-map",
            Choice::OwnGroup => "own-group",
        }
    }
}

worded!(Choice);

presses! {
    /// What can be pressed here.
    Press {
        /// The list's OK and its cross: it closes, and keeps how its box "Notice" stood.
        Close = "booking-close",
        /// The button by the cross of a window that makes it small, or large again: the list's, the
        /// registering's, one's own ad's.
        Size = "booking-size",
        RegisterSize = "booking-register-size",
        OwnSize = "booking-own-size",
        /// The list's Search.
        Search = "booking-search",
        /// A row of the list, by the number of its ad: it is picked.
        Row(u32) = "booking-row",
        /// The button that opens a talk with the leader of the ad picked.
        Talk = "booking-talk",
        /// The word "Recruit" at the list's bottom: the window of registering opens, or closes.
        Recruit = "booking-recruit",
        /// The list's Help, and the cross of the help.
        Help = "booking-help",
        HelpClose = "booking-help-close",
        /// The list's box "Notice".
        Alarm = "booking-alarm",
        /// A box that drops its list, and a line of the list that is dropped.
        Opens(Choice) = "booking-opens",
        Pick(usize) = "booking-pick",
        /// The window of registering: a job ticked, OK, and Cancel or the cross.
        Tick(i16) = "booking-tick",
        Register = "booking-register",
        RegisterClose = "booking-register-close",
        /// The window of one's own ad: Cancel or the cross takes the ad away; the other button opens the
        /// window its jobs are changed in.
        Withdraw = "booking-withdraw",
        Jobs = "booking-jobs",
        /// That window: a job ticked by its place, OK, and the cross.
        JobTick(usize) = "booking-job-tick",
        JobsOk = "booking-jobs-ok",
        JobsClose = "booking-jobs-close",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let boxes = Choice::ALL.map(Press::Opens);
        let list = [Press::Close, Press::Size, Press::Search, Press::Row(7), Press::Talk, Press::Recruit, Press::Help, Press::HelpClose, Press::Alarm];
        let register = [Press::Pick(3), Press::Tick(13), Press::Register, Press::RegisterClose, Press::RegisterSize];
        let own = [Press::Withdraw, Press::Jobs, Press::OwnSize, Press::JobTick(2), Press::JobsOk, Press::JobsClose];
        crate::press::testing::read_back(boxes.into_iter().chain(list).chain(register).chain(own));
    }
}
