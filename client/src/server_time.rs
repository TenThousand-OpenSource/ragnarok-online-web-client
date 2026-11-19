//! The server's clock as the client keeps it: everyone's walk is shown by this clock, not from the moment its
//! packet came, so a packet that comes late shows the walker where it is by then.
//!
//! From the client's `CSession`: `SetServerTime` (the clock is set from a time the server tells),
//! `GetServerTime` (the time now: 72 milliseconds behind what was told, so that a walk told of a little late has
//! still to begin) and `SyncServerTime` (every walk told corrects the clock). The client sets the clock from
//! ZC_ACCEPT_ENTER and from the first ZC_NOTIFY_TIME on a map, and corrects it with every later ZC_NOTIFY_TIME
//! and with the time a walk set out at in every packet that tells of one (ZC_NOTIFY_MOVE, ZC_NOTIFY_PLAYERMOVE
//! and the packets that show someone walking into sight).
//!
//! The client counts in whole milliseconds that run round after 2^32; here the clock is a line that does not
//! run round, and a time told is put on it nearest to the time now (`Clock::of`).

/// How far behind the time told the clock is read (`GetServerTime`).
pub const BEHIND: f64 = 72.0;

/// A time told this much before the clock's is late (`SyncServerTime`), and the clock goes back by as much
/// when the fourth late one in a row has come.
const LATE: f64 = 144.0;
const LATE_IN_A_ROW: u8 = 4;

/// The clock: how far the local time is ahead of the server's, and how many times told in a row were late.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Clock {
    ahead: f64,
    late: u8,
}

impl Clock {
    /// The server tells its time (`SetServerTime`): the clock is set from it. `local` is the local time now,
    /// in milliseconds.
    pub fn set(&mut self, local: f64, told: u32) {
        *self = Self { ahead: local - f64::from(told), late: 0 };
    }

    /// The server's time now (`GetServerTime`).
    pub fn now(&self, local: f64) -> f64 {
        local - self.ahead - BEHIND
    }

    /// A time the server tells, on the clock's line: the one of its kind (they run round after 2^32) nearest
    /// to the time now.
    pub fn of(&self, local: f64, told: u32) -> f64 {
        let now = self.now(local).floor();
        now + f64::from(told.wrapping_sub(now as i64 as u32) as i32)
    }

    /// A time the server tells corrects the clock (`SyncServerTime`): one ahead of the clock sets it at once;
    /// one more than 144 behind it is late, and at the fourth late one in a row the clock goes back by 144;
    /// any other leaves it.
    pub fn sync(&mut self, local: f64, told: u32) {
        let told = self.of(local, told);
        let read = self.now(local) + BEHIND;
        if read < told {
            *self = Self { ahead: local - told, late: 0 };
        } else if read > told + LATE {
            self.late += 1;
            if self.late >= LATE_IN_A_ROW {
                // The client writes `timeGetTime() + 72 - current`, which is what it had and 144 more.
                *self = Self { ahead: self.ahead + LATE, late: 0 };
            }
        } else {
            self.late = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(local: f64, told: u32) -> Clock {
        let mut clock = Clock::default();
        clock.set(local, told);
        clock
    }

    #[test]
    fn the_clock_is_read_72_behind_what_the_server_told() {
        let clock = set(5_000.0, 100_000);
        assert_eq!(clock.now(5_000.0), 99_928.0);
        assert_eq!(clock.now(5_250.0), 100_178.0);
    }

    #[test]
    fn a_time_ahead_of_the_clock_sets_it_at_once() {
        // Set from a packet that took long to come: the next comes quicker, and tells a later time than the
        // clock has.
        let mut clock = set(5_000.0, 100_000);
        clock.sync(6_000.0, 101_090);
        assert_eq!(clock.now(6_000.0), 101_018.0);
        // One that tells just the clock's time, or a little less, changes nothing.
        let was = clock;
        clock.sync(7_000.0, 102_090);
        clock.sync(8_000.0, 103_000);
        assert_eq!(clock, was);
    }

    #[test]
    fn the_fourth_late_time_in_a_row_takes_the_clock_back_by_144() {
        let mut clock = set(5_000.0, 100_000);
        // Each of these was 200 on its way longer than the one the clock was set from.
        for at in [6_000.0, 7_000.0, 8_000.0] {
            clock.sync(at + 200.0, 100_000 + at as u32 - 5_000);
            assert_eq!(clock.now(9_000.0), 103_928.0);
        }
        clock.sync(9_200.0, 104_000);
        assert_eq!(clock.now(9_200.0), 104_128.0 - 144.0);
    }

    #[test]
    fn a_time_that_is_not_late_starts_the_count_anew() {
        let mut clock = set(5_000.0, 100_000);
        for at in [6_000.0, 7_000.0, 8_000.0] {
            clock.sync(at + 200.0, 100_000 + at as u32 - 5_000);
        }
        // 144 behind is not yet late.
        clock.sync(9_144.0, 104_000);
        for at in [10_000.0, 11_000.0, 12_000.0] {
            clock.sync(at + 200.0, 100_000 + at as u32 - 5_000);
        }
        assert_eq!(clock.now(13_000.0), 107_928.0);
    }

    #[test]
    fn a_time_told_after_the_servers_count_ran_round_is_still_the_nearest() {
        let mut clock = set(1_000.0, u32::MAX - 100);
        let before = clock.now(1_000.0);
        // 300 later the server's count has run round to 199.
        assert_eq!(clock.of(1_300.0, 199), before + 72.0 + 300.0);
        clock.sync(1_300.0, 250);
        assert_eq!(clock.now(1_300.0), before + 300.0 + 51.0);
    }
}
