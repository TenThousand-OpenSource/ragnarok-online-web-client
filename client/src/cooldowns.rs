//! The waits before skills may be used again, as the client keeps them (`CSession::m_requestTimers`) and draws
//! them over the pictures of the hotkey bar (`Source/UI/Cooldowns`): a shade that is wiped away clockwise.

/// The number the client keeps the wait after any skill under (`ApplyStatusChange`, for the state
/// EFST_POSTDELAY): it holds every skill back, and is drawn over every skill of the bar.
pub const ANY: u16 = 577;
/// How near its end a wait no longer holds a skill back, in milliseconds (`CSession::IsRequestReady`).
const GRACE: u32 = 100;

/// A wait: the skill (or `ANY`), when it ends and how long it is in all, in milliseconds of the page's clock.
/// One whose whole length is not known (nought) holds the skill back but is not drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wait {
    pub skill: u16,
    pub end: u32,
    pub duration: u32,
}

impl Wait {
    /// How far through it is at a moment, in milliseconds since it began.
    pub fn progress(&self, now: u32) -> u32 {
        self.duration.saturating_sub(self.end.saturating_sub(now))
    }
}

/// The waits running.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Waits(pub Vec<Wait>);

impl Waits {
    /// A wait begins. One for a skill takes the place of the one that skill had (`ApplyCooldownStart`); the
    /// wait after any skill is added to the others (`ApplyStatusChange`).
    pub fn begin(&mut self, wait: Wait) {
        if wait.skill != ANY {
            self.0.retain(|other| other.skill != wait.skill);
        }
        self.0.push(wait);
    }

    /// Those that are over are forgotten (the client does it as it draws: `PaintCooldowns`).
    pub fn prune(&mut self, now: u32) {
        self.0.retain(|wait| wait.end > now);
    }

    /// Whether a skill may be used at a moment (`CSession::IsRequestReady`): no wait of its own or after any
    /// skill has more than a tenth of a second left.
    pub fn ready(&self, skill: u16, now: u32) -> bool {
        !self.0.iter().any(|wait| (wait.skill == skill || wait.skill == ANY) && wait.end.saturating_sub(GRACE) > now)
    }

    /// The waits drawn over a skill's picture on the bar at a moment (`PaintCooldowns`): its own, and the one
    /// after any skill; only those still running whose length is known.
    pub fn over(&self, skill: u16, now: u32) -> impl Iterator<Item = &Wait> {
        self.0.iter().filter(move |wait| (wait.skill == skill || wait.skill == ANY) && wait.end > now && wait.duration != 0)
    }
}

/// The last stretch of a state's time, which the client draws anew in another colour (`CooldownState::Update`,
/// for the ones it draws "in reverse": the states at the side of the screen).
pub const LAST_STRETCH: u32 = 60_000;

/// How a state's time is drawn over its picture: the client lays a white shade over the part of the time that
/// has gone, clockwise from the top; when a minute is left it starts anew in orange, the whole turn being that
/// minute. Each of the two as `(length, how far through it is now)`, in milliseconds: the second is negative
/// while the orange turn has not begun. Nothing where the whole length is not known.
pub fn turns(duration: u32, remaining: u32) -> Option<((u32, u32), (u32, i64))> {
    (duration != 0).then(|| ((duration, duration.saturating_sub(remaining)), (LAST_STRETCH, i64::from(LAST_STRETCH) - i64::from(remaining))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wait_holds_its_skill_back_until_it_is_nearly_over() {
        let mut waits = Waits::default();
        waits.begin(Wait { skill: 28, end: 5000, duration: 3000 });
        assert!(!waits.ready(28, 2000));
        assert!(waits.ready(29, 2000));
        assert!(waits.ready(28, 4900));
        assert_eq!(waits.over(28, 2000).count(), 1);
        assert_eq!(waits.over(28, 5000).count(), 0);
        assert_eq!(waits.0[0].progress(3500), 1500);
        waits.prune(5000);
        assert!(waits.0.is_empty());
    }

    #[test]
    fn the_wait_after_any_skill_holds_all_back() {
        let mut waits = Waits::default();
        waits.begin(Wait { skill: ANY, end: 1000, duration: 1000 });
        waits.begin(Wait { skill: 28, end: 9000, duration: 9000 });
        assert!(!waits.ready(5, 500));
        assert_eq!(waits.over(28, 500).count(), 2);
        assert_eq!(waits.over(5, 500).count(), 1);
        // A new wait for the skill takes the old one's place.
        waits.begin(Wait { skill: 28, end: 2000, duration: 0 });
        assert_eq!(waits.0.len(), 2);
        assert!(!waits.ready(28, 1500));
        assert_eq!(waits.over(28, 1500).count(), 0);
    }

    #[test]
    fn a_states_time_turns_white_then_orange() {
        assert_eq!(turns(240_000, 200_000), Some(((240_000, 40_000), (60_000, -140_000))));
        assert_eq!(turns(30_000, 30_000), Some(((30_000, 0), (60_000, 30_000))));
        assert_eq!(turns(0, 5000), None);
    }
}
