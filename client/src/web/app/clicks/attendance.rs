//! The attendance check: its window asked for and closed, today's reward claimed (`crate::attendance`, the
//! client's `UICheckAttendanceWnd`).

use super::*;

impl App {
    /// The window is told how things stand and shown (`CGameMode::ReceiveServiceRequest` at
    /// `OUT_UI_ATTENDANCE`, and `ReceiveWindow328Result`; `UICheckAttendanceWnd::SendMsg`, `UM_SETINFO`). The
    /// days the event still runs are counted as the window is made (`OnCreate`), so only when it was closed.
    pub(in crate::web::app) fn attendance_told(&mut self, value: i32) {
        let table = data::attendance();
        if !self.attendance.open {
            let (year, month, day, hour) = crate::attendance::ends(table.last);
            let ends = js_sys::Date::new_with_year_month_day_hr(year, month as i32 - 1, day as i32, hour as i32);
            let now = js_sys::Date::new_0();
            // The player's clock is so far ahead of the world's (`getTimezoneOffset` counts the other way, in minutes).
            let ahead = -(now.get_timezone_offset() * 60.0) as i64;
            self.attendance.left = crate::attendance::days_left(((ends.get_time() - now.get_time()) / 1000.0) as i64, ahead);
        }
        (self.attendance.days, self.attendance.claimable) = crate::attendance::told(value, table.rewards.len());
        self.attendance.open = true;
        // The rewards are named and pictured once the gateway has told of them.
        data::want_items(table.rewards.iter().map(|reward| reward.1), || dispatch(Action::ItemsKnown));
        self.render();
    }

    /// A press of this system (`crate::press::attendance`).
    pub(super) fn attendance_pressed(&mut self, press: crate::press::attendance::Press) {
        use crate::press::attendance::Press;
        match press {
            // `UIMenuIconWnd::SendMsg`, `MENUICON_ATTENDANCE`: an open window closes; otherwise the server is
            // asked to open it (`CZ_UI_OPEN` with 5).
            Press::Window if self.attendance.open => self.attendance_pressed(Press::Close),
            Press::Window => self.send_on_map(&net::attendance::open()),
            Press::Close => {
                self.attendance.open = false;
                self.render();
            }
            // The cell of the day to claim (`OnLBtnUp`, `ID_OK`): asked of the server while it is to claim.
            Press::Claim => {
                if self.attendance.claimable {
                    self.send_on_map(&net::attendance::claim());
                }
            }
        }
    }
}
