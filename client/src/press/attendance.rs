//! The presses of the attendance check (the client's `UICheckAttendanceWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon: the window closes if it is open; otherwise the server is asked to open it.
        Window = "attendance",
        /// "Close" under the days, and Escape: the window closes.
        Close = "attendance-close",
        /// The cell of the day that is to be claimed: today's reward is asked for.
        Claim = "attendance-claim",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back([Press::Window, Press::Close, Press::Claim]);
    }
}
