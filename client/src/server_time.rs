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
mod tests;
