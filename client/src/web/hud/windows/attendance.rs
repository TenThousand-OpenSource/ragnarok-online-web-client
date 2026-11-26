//! The entry of the list of windows for the attendance check.

use super::*;

/// The attendance check (`UICheckAttendanceWnd`): 488 by 413, in the middle of the screen
/// (`UIWindowMgr::EnsureWindow`, `WID_CHECKATTENDANCEWND`). Escape closes it as its "Close" does: that the
/// client's Escape does so was not found in its code and is taken from what its other framed windows do.
pub(in crate::web) static ATTENDANCE: Window = Window::usual(
    "attendance",
    |app| app.attendance.open,
    |_, view| framed(format!("{};height:413px", centred(view, 488, 413))),
    App::attendance_window,
    |app| app.pressed(crate::press::attendance::Press::Close),
);
