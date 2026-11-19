//! How heavy what is carried is, as the client tells of it: what the chat says as the weight passes the share
//! of the most that can be carried from which nothing is recovered by resting (the server tells that share:
//! `net::notices`) and 90 of a hundred (`CGameMode`'s `SP_WEIGHT` and `SP_MAXWEIGHT`).

/// The share from which the character can neither attack nor use skills (`CGameMode::IsOverWeight`).
pub const TOO_HEAVY: u8 = 90;

/// How many hundredths of the most that can be carried are carried, as the client counts them (a most of
/// nought counts as one).
pub fn percent(weight: i64, most: i64) -> i64 {
    weight * 100 / most.max(1)
}

/// A line the chat says of the weight: its number in the client's message table, whether it is a warning (in
/// red; else in the client's 0xFFFF00, which is blue and green), and whether the share is written into it (`%d`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub line: usize,
    pub warning: bool,
    pub with_share: bool,
}

const NO_RECOVERY: Line = Line { line: 3311, warning: true, with_share: true };
const NO_ATTACK: Line = Line { line: 307, warning: true, with_share: false };
const RECOVERY_AGAIN: Line = Line { line: 308, warning: false, with_share: false };
const ATTACK_AGAIN: Line = Line { line: 309, warning: false, with_share: false };

/// What is said when the weight carried changes from one share to another (`SP_WEIGHT`), `heavy` being the
/// share from which nothing is recovered: growing past it and past 90, each is warned of; falling under them,
/// each is told to be over.
pub fn weight_changed(before: i64, after: i64, heavy: u8) -> Vec<Line> {
    let (heavy, too_heavy) = (i64::from(heavy), i64::from(TOO_HEAVY));
    let mut lines = Vec::new();
    if before < after {
        if after >= heavy && before < heavy {
            lines.push(NO_RECOVERY);
        }
        if after >= too_heavy && before < too_heavy {
            lines.push(NO_ATTACK);
        }
    } else if before > after {
        if after < heavy && heavy <= before {
            lines.push(RECOVERY_AGAIN);
        }
        if after < too_heavy && before >= too_heavy {
            lines.push(ATTACK_AGAIN);
        }
    }
    lines
}

/// What is said when the most that can be carried grows (`SP_MAXWEIGHT`; nothing when it was nought or did not
/// grow, which the caller sees to): the client measures this one against 50 whatever the server's share is.
pub fn most_grew(before: i64, after: i64) -> Vec<Line> {
    let mut lines = Vec::new();
    if after < 50 {
        if before >= 50 {
            lines.push(RECOVERY_AGAIN);
        }
    } else if after >= 90 {
        return lines;
    }
    if before >= 90 {
        lines.push(ATTACK_AGAIN);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weight_passing_a_share_is_told_once() {
        assert_eq!(weight_changed(49, 50, 50), [NO_RECOVERY]);
        assert_eq!(weight_changed(50, 60, 50), []);
        assert_eq!(weight_changed(40, 95, 50), [NO_RECOVERY, NO_ATTACK]);
        assert_eq!(weight_changed(89, 90, 50), [NO_ATTACK]);
        assert_eq!(weight_changed(90, 89, 50), [ATTACK_AGAIN]);
        assert_eq!(weight_changed(95, 10, 50), [RECOVERY_AGAIN, ATTACK_AGAIN]);
        assert_eq!(weight_changed(50, 49, 50), [RECOVERY_AGAIN]);
        assert_eq!(weight_changed(60, 60, 50), []);
        // The server's share, where it tells another.
        assert_eq!(weight_changed(60, 70, 70), [NO_RECOVERY]);
        assert_eq!(weight_changed(49, 50, 70), []);
    }

    #[test]
    fn carrying_more_lifts_the_warnings() {
        assert_eq!(most_grew(95, 40), [RECOVERY_AGAIN, ATTACK_AGAIN]);
        assert_eq!(most_grew(95, 60), [ATTACK_AGAIN]);
        assert_eq!(most_grew(95, 92), []);
        assert_eq!(most_grew(60, 40), [RECOVERY_AGAIN]);
        assert_eq!(most_grew(40, 30), []);
    }

    #[test]
    fn the_share_is_counted() {
        assert_eq!((percent(450, 1000), percent(5, 0)), (45, 500));
    }
}
