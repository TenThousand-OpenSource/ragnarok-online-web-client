//! Counting attempts per key (an address, an account name) inside a sliding window of time.
//!
//! The counts live in memory: they start empty when the gateway restarts, and are not shared between gateways.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Past this many keys, the ones with nothing left inside the window are thrown out.
const SWEEP_AT: usize = 10_000;

pub struct Limiter {
    max: usize,
    window: Duration,
    hits: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl Limiter {
    /// Allows `max` hits per key in any stretch of `window`.
    pub fn new(max: usize, window: Duration) -> Self {
        Self { max, window, hits: Mutex::new(HashMap::new()) }
    }

    /// How long the key stays blocked, or `None` while it has attempts left.
    pub fn blocked_for(&self, key: &str, now: Instant) -> Option<Duration> {
        let mut hits = self.hits.lock().unwrap();
        let recent = hits.get_mut(key)?;
        self.forget_old(recent, now);
        if recent.is_empty() {
            hits.remove(key);
            return None;
        }
        // Blocked until the oldest hit that still counts leaves the window.
        (recent.len() >= self.max).then(|| (recent[recent.len() - self.max] + self.window).saturating_duration_since(now))
    }

    /// Counts one attempt.
    pub fn hit(&self, key: &str, now: Instant) {
        let mut hits = self.hits.lock().unwrap();
        if hits.len() >= SWEEP_AT {
            hits.retain(|_, recent| recent.back().is_some_and(|last| now.saturating_duration_since(*last) < self.window));
        }
        let recent = hits.entry(key.to_owned()).or_default();
        self.forget_old(recent, now);
        recent.push_back(now);
    }

    /// Counts one attempt if the key has any left; otherwise says how long it stays blocked.
    pub fn attempt(&self, key: &str, now: Instant) -> Result<(), Duration> {
        match self.blocked_for(key, now) {
            Some(wait) => Err(wait),
            None => {
                self.hit(key, now);
                Ok(())
            }
        }
    }

    /// Forgets the key's attempts: after a success that proves who is trying.
    pub fn clear(&self, key: &str) {
        self.hits.lock().unwrap().remove(key);
    }

    fn forget_old(&self, recent: &mut VecDeque<Instant>, now: Instant) {
        while recent.front().is_some_and(|first| now.saturating_duration_since(*first) >= self.window) {
            recent.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: Duration = Duration::from_secs(60);

    #[test]
    fn blocks_after_the_allowed_attempts() {
        let limiter = Limiter::new(3, 10 * MINUTE);
        let start = Instant::now();
        for attempt in 0..3 {
            assert_eq!(limiter.attempt("ip", start + Duration::from_secs(attempt)), Ok(()));
        }
        // The fourth waits for the first to leave the window.
        assert_eq!(limiter.attempt("ip", start + Duration::from_secs(3)), Err(10 * MINUTE - Duration::from_secs(3)));
        assert_eq!(limiter.blocked_for("ip", start + 5 * MINUTE), Some(5 * MINUTE));
    }

    #[test]
    fn attempts_come_back_as_the_window_slides() {
        let limiter = Limiter::new(2, 10 * MINUTE);
        let start = Instant::now();
        limiter.hit("ip", start);
        limiter.hit("ip", start + 4 * MINUTE);
        assert!(limiter.blocked_for("ip", start + 9 * MINUTE).is_some());
        // The first hit is ten minutes old: one attempt is free again, and the second hit still counts.
        assert_eq!(limiter.attempt("ip", start + 10 * MINUTE), Ok(()));
        assert_eq!(limiter.blocked_for("ip", start + 10 * MINUTE), Some(4 * MINUTE));
        assert_eq!(limiter.blocked_for("ip", start + 21 * MINUTE), None);
    }

    #[test]
    fn keys_are_counted_apart() {
        let limiter = Limiter::new(1, MINUTE);
        let now = Instant::now();
        assert_eq!(limiter.attempt("one", now), Ok(()));
        assert!(limiter.attempt("one", now).is_err());
        assert_eq!(limiter.attempt("two", now), Ok(()));
    }

    #[test]
    fn a_refused_attempt_does_not_extend_the_block() {
        let limiter = Limiter::new(1, MINUTE);
        let start = Instant::now();
        limiter.hit("ip", start);
        for second in 1..60 {
            assert!(limiter.attempt("ip", start + Duration::from_secs(second)).is_err());
        }
        assert_eq!(limiter.attempt("ip", start + MINUTE), Ok(()));
    }

    #[test]
    fn clearing_gives_all_attempts_back() {
        let limiter = Limiter::new(2, MINUTE);
        let now = Instant::now();
        limiter.hit("ip", now);
        limiter.hit("ip", now);
        assert!(limiter.blocked_for("ip", now).is_some());
        limiter.clear("ip");
        assert_eq!(limiter.blocked_for("ip", now), None);
    }

    #[test]
    fn old_keys_are_swept_out() {
        let limiter = Limiter::new(5, MINUTE);
        let start = Instant::now();
        for key in 0..SWEEP_AT {
            limiter.hit(&key.to_string(), start);
        }
        limiter.hit("late", start + 2 * MINUTE);
        assert_eq!(limiter.hits.lock().unwrap().len(), 1);
    }
}
