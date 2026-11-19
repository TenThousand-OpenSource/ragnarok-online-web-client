//! How the numbers and words of a fight fly: where one is, how large and how faint, at a time of its life.
//!
//! From the client's `CMsgEffect::OnProcess`, kind by kind (`MET_…`), in its frames of 24 milliseconds; where
//! each begins is in `CMsgEffect::SendMsg` (`AM_SETPOS`, `AM_SETPOS2`) and `CGameActor::SpawnDamageDigits`.

/// How a thing flies off someone struck, by what it is (the client's kinds of `CMsgEffect`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flight {
    /// The number of a blow (`MET_NUMBER`; `MET_NUMBER_HALF`, one of several hits, flies the same).
    Number,
    /// The number of a critical blow (`MET_NUMBER_CRITICAL`): yellow, and it stays a little larger.
    Critical,
    /// The mark that goes with a critical blow (`MET_CRITICAL_EFFECT`): the pictures of an action of the
    /// numbers' sprite, one after another, greyed.
    Mark,
    /// The word for a blow that missed (`MET_MISS`), over the one who struck.
    Miss,
    /// The damage of several hits or of a combo so far (`MET_NUMBER_TOTAL`): it stands over the one struck,
    /// grows to three times its size and fades at the end.
    Total,
    /// The client's blue word that goes with the total of a kick (`MET_BLUE_EFFECT`).
    Word,
    /// The word for a blow dodged by luck (`MET_LUCKY_EFFECT`): an action of the numbers' sprite played once.
    Lucky,
    /// Life or spirit recovered (the client's kind 114): it rises straight up, shrinking to its size.
    Recovery,
}

/// Where something that flies is at a moment: how far over the feet of its owner, how many times its drift
/// aside (none for what rises straight), how large, and how much of it is left to see (0 to 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct At {
    pub up: f32,
    pub aside: f32,
    pub scale: f32,
    pub alpha: f32,
}

impl Flight {
    /// How many of the client's frames it lives; a word played once lives as long as its pictures last
    /// (`Flight::picture`).
    pub fn frames(self) -> f32 {
        match self {
            Flight::Number | Flight::Critical | Flight::Mark => 70.0,
            Flight::Miss => 80.0,
            Flight::Total | Flight::Word | Flight::Recovery => 120.0,
            Flight::Lucky => f32::INFINITY,
        }
    }

    /// Which picture of its action shows after `elapsed` frames, of `count`; none once a word played once is
    /// through. The digits and the words of one picture stay on their own.
    pub fn picture(self, elapsed: f32, count: usize) -> Option<usize> {
        match self {
            // One every two frames, staying on the last.
            Flight::Mark => Some(((elapsed * 0.5) as usize).min(count.saturating_sub(1))),
            // One every four (the speed `SpawnMsgEffect` leaves it), once through.
            Flight::Lucky => Some((elapsed / 4.0) as usize).filter(|picture| *picture < count.max(1)),
            _ => Some(0),
        }
    }

    /// Where it is after `elapsed` frames.
    pub fn at(self, elapsed: f32) -> At {
        // What rises and falls back as it drifts (the client's `MsgEffectDrift`).
        let drifting = |scale: f32, alpha: f32| At { up: 12.0 - 8.0 + (2.0 - elapsed / 30.0) * elapsed, aside: elapsed / 3.0 + 3.0, scale, alpha: alpha / 255.0 };
        // The total and its word fade over the last thirty frames.
        let fade = ((elapsed - 90.0).max(0.0) * 8.0).trunc();
        match self {
            Flight::Miss => At { up: 12.0 + elapsed * 0.54, aside: 0.0, scale: 1.0, alpha: (250.0 - elapsed * 3.0) / 255.0 },
            Flight::Number => drifting((5.0 - elapsed * 0.24).max(1.2), 250.0 - elapsed * 3.4),
            Flight::Critical => drifting((5.0 - elapsed * 0.24).max(1.3), 250.0 - elapsed * 3.4),
            Flight::Mark => drifting((3.0 - elapsed * 0.24 * 0.6).max(1.0), 255.0 - elapsed * 3.45),
            // It begins at half its size and has `elapsed * 0.18` added at every tick of the client's until it
            // is three times its size: counted here as one tick a frame.
            Flight::Total => At { up: 27.0 + elapsed * 0.1, aside: 0.0, scale: (0.5 + 0.09 * elapsed.trunc() * (elapsed.trunc() + 1.0)).min(3.0), alpha: (250.0 - fade).max(0.0) / 255.0 },
            Flight::Word => At { up: 27.0 + elapsed * 0.1, aside: 0.0, scale: 2.0, alpha: (210.0 - fade).max(0.0) / 255.0 },
            Flight::Lucky => At { up: 20.0, aside: 0.0, scale: 1.0, alpha: 1.0 },
            Flight::Recovery => At { up: 12.0 + elapsed * 0.18, aside: 0.0, scale: (5.0 - elapsed * 0.24).max(1.0), alpha: (250.0 - elapsed * 2.0).max(0.0) / 255.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_shrinks_from_five_times_its_size_and_is_gone_after_seventy_frames() {
        assert_eq!(Flight::Number.at(0.0), At { up: 4.0, aside: 3.0, scale: 5.0, alpha: 250.0 / 255.0 });
        assert_eq!(Flight::Number.at(30.0).scale, 1.2);
        assert_eq!(Flight::Critical.at(30.0).scale, 1.3);
        assert_eq!(Flight::Number.frames(), 70.0);
        // It rises for thirty frames and falls back.
        assert!(Flight::Number.at(30.0).up > Flight::Number.at(10.0).up && Flight::Number.at(60.0).up < Flight::Number.at(30.0).up);
    }

    #[test]
    fn a_total_grows_to_three_times_its_size_and_fades_at_the_end() {
        assert_eq!(Flight::Total.at(0.0).scale, 0.5);
        assert_eq!(Flight::Total.at(10.0).scale, 3.0);
        assert_eq!(Flight::Total.at(50.0), At { up: 32.0, aside: 0.0, scale: 3.0, alpha: 250.0 / 255.0 });
        assert_eq!(Flight::Total.at(100.0).alpha, 170.0 / 255.0);
        assert_eq!(Flight::Total.at(125.0).alpha, 0.0);
    }

    #[test]
    fn what_recovers_rises_straight_for_120_frames() {
        assert_eq!(Flight::Recovery.at(20.0), At { up: 15.6, aside: 0.0, scale: 1.0, alpha: 210.0 / 255.0 });
        assert_eq!(Flight::Recovery.frames(), 120.0);
    }

    #[test]
    fn the_word_of_luck_plays_its_pictures_once() {
        assert_eq!(Flight::Lucky.picture(0.0, 5), Some(0));
        assert_eq!(Flight::Lucky.picture(19.0, 5), Some(4));
        assert_eq!(Flight::Lucky.picture(20.0, 5), None);
        assert_eq!(Flight::Mark.picture(100.0, 5), Some(4));
    }
}
