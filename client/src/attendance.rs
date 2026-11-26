//! The attendance check as rules (the client's `UICheckAttendanceWnd`): what the number the server sends says
//! of the days attended, where each day's cell stands in the window, how the client counts the days the event
//! still runs, and what its lines are filled with.

/// How many days a row of the window holds, how far apart the cells stand and where the first one is
/// (`UICheckAttendanceWnd::Paint`, `SlotAt`): cells of 58 by 60, 62 apart across and 64 down, from (26, 95).
pub const ROW: usize = 5;
pub const CELL: (i32, i32) = (58, 60);
const FIRST: (i32, i32) = (26, 95);
const APART: (i32, i32) = (62, 64);

/// How many days have been attended, and whether today's reward is still to be claimed, by the number the
/// server sends as the window opens and by how many rewards the client's table has
/// (`UICheckAttendanceWnd::SendMsg`, `UM_SETINFO`): the tens are the days, and a last figure of nought says
/// that today was not attended yet; once every reward is taken there is nothing left to claim.
pub fn told(value: i32, rewards: usize) -> (usize, bool) {
    let days = (value / 10).max(0) as usize;
    (days, value == (value / 10) * 10 && rewards > days)
}

/// What the window is told after the server took today's attendance, by the days it counts now
/// (`CGameMode::ReceiveWindow328Result`): those days, and today attended.
pub fn attended(days: i32) -> i32 {
    days * 10 + 1
}

/// Where the cell of a day stands in the window, by the day's place from nought (`Paint`: its picture, the
/// reward at 17 and 14 into it, "Day n" 45 under its top).
pub fn cell(place: usize) -> (i32, i32) {
    (FIRST.0 + (place % ROW) as i32 * APART.0, FIRST.1 + (place / ROW) as i32 * APART.1)
}

/// The month and the day of a date written as the client's table writes one (`20260219`).
pub fn month_and_day(date: u32) -> (u32, u32) {
    (date % 10000 / 100, date % 100)
}

/// The moment the event ends as the window counts it (`UICheckAttendanceWnd::OnCreate`): three in the
/// afternoon of the last day, as a year, a month and a day with that hour.
pub fn ends(last: u32) -> (u32, u32, u32, u32) {
    (last / 10000, last % 10000 / 100, last % 100, 15)
}

/// The days left that the window shows, by the seconds from now to that moment and by how far the player's
/// clock is ahead of the world's, in seconds (`OnCreate`: the client takes the seconds left for a moment of
/// its own, as if counted from 1970, and shows the day of the year that moment falls on by the local clock):
/// nothing once the moment has passed.
pub fn days_left(seconds: i64, ahead: i64) -> u32 {
    if seconds <= 0 {
        return 0;
    }
    let mut days = (seconds + ahead).div_euclid(86_400);
    let mut year = 1970;
    loop {
        let long = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if days < long {
            return days.max(0) as u32;
        }
        days -= long;
        year += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_server_s_number_is_the_days_and_whether_today_is_still_to_claim() {
        assert_eq!(told(0, 20), (0, true));
        assert_eq!(told(1, 20), (0, false));
        assert_eq!(told(30, 20), (3, true));
        assert_eq!(told(31, 20), (3, false));
        // Every reward taken: nothing is left to claim, whatever the last figure.
        assert_eq!(told(200, 20), (20, false));
        assert_eq!(told(attended(4), 20), (4, false));
    }

    #[test]
    fn the_cells_stand_five_to_a_row() {
        assert_eq!(cell(0), (26, 95));
        assert_eq!(cell(4), (274, 95));
        assert_eq!(cell(5), (26, 159));
        assert_eq!(cell(19), (274, 287));
    }

    #[test]
    fn a_date_of_the_table_is_read() {
        assert_eq!(month_and_day(20260219), (2, 19));
        assert_eq!(ends(20260318), (2026, 3, 18, 15));
    }

    #[test]
    fn the_days_left_are_those_of_the_seconds_left_by_the_local_clock() {
        assert_eq!(days_left(0, 0), 0);
        assert_eq!(days_left(-5, 32_400), 0);
        assert_eq!(days_left(3 * 86_400 + 10, 0), 3);
        // Nine hours ahead, as in Korea: a day more from fifteen hours on.
        assert_eq!(days_left(3 * 86_400 + 15 * 3600, 32_400), 4);
        assert_eq!(days_left(3600, -7200), 0);
        // More than a year left: the day of that later year.
        assert_eq!(days_left(366 * 86_400, 0), 1);
    }
}
