//! The bridge's debug delay (`--lag`, `--jitter`): when a piece that comes is passed on.

use std::time::{Duration, Instant};

/// A debug delay on everything the bridge passes, each way: a fixed time and a random part up to `jitter` more,
/// drawn anew for every piece that comes. What comes later is never passed on sooner than what came before
/// it, as on a TCP connection. None by default: it is for looking at walking and fighting as a player far away
/// has them, on one's own machine, and makes the game worse for everyone who plays through the gateway.
#[derive(Clone, Copy, Default)]
pub struct Lag {
    pub fixed: Duration,
    pub jitter: Duration,
}

impl Lag {
    /// When a piece that comes at `now` is passed on, the one before it being due at `last`. `chance` is any
    /// number: it picks the random part.
    fn due(&self, now: Instant, last: Option<Instant>, chance: u64) -> Instant {
        let random = Duration::from_millis(chance % (self.jitter.as_millis() as u64 + 1));
        let due = now + self.fixed + random;
        last.map_or(due, |last| due.max(last))
    }

    /// The same with a number drawn here; at once when there is no delay.
    pub(super) fn due_now(&self, last: Option<Instant>) -> Instant {
        let now = Instant::now();
        if self.fixed.is_zero() && self.jitter.is_zero() {
            return now;
        }
        let mut chance = [0u8; 8];
        let _ = getrandom::fill(&mut chance);
        self.due(now, last, u64::from_le_bytes(chance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_delayed_piece_is_due_after_the_fixed_time_and_a_part_of_the_jitter() {
        let lag = Lag { fixed: Duration::from_millis(150), jitter: Duration::from_millis(100) };
        let now = Instant::now();
        assert_eq!(lag.due(now, None, 0), now + Duration::from_millis(150));
        assert_eq!(lag.due(now, None, 100), now + Duration::from_millis(250));
        assert_eq!(lag.due(now, None, 101), now + Duration::from_millis(150));
        assert_eq!(Lag::default().due(now, None, 12_345), now);
    }

    #[test]
    fn what_comes_later_is_never_due_sooner() {
        let lag = Lag { fixed: Duration::from_millis(150), jitter: Duration::from_millis(100) };
        let now = Instant::now();
        let first = lag.due(now, None, 100);
        // Ten milliseconds later a piece draws no jitter at all: it waits for the one before it.
        assert_eq!(lag.due(now + Duration::from_millis(10), Some(first), 0), first);
        assert_eq!(lag.due(now + Duration::from_millis(200), Some(first), 0), now + Duration::from_millis(350));
    }
}
