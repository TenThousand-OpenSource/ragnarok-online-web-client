//! What the app keeps of the attendance check.

/// The attendance check (`UICheckAttendanceWnd`).
#[derive(Default)]
pub(in crate::web) struct Attendance {
    pub open: bool,
    /// The days attended, and whether today's reward is still to be claimed (`m_checkedDays`, `m_blinking`).
    pub days: usize,
    pub claimable: bool,
    /// The days the event still runs, counted as the window opens (`m_daysLeft`).
    pub left: u32,
}
