//! The hits a blow is made of: how many land, how long after the first each lands, and as what each is shown.
//!
//! From the client's `CGameActor::ProcessHitMessage` (a blow, ZC_NOTIFY_ACT) and
//! `CGameActor::ProcessSkillMessage` (a skill that strikes, ZC_NOTIFY_SKILL): each hands the one struck one
//! `AM_WILL_BE_ATTACKED` message a hit, with the time it lands at and the message it becomes then.

/// The time between the hits of one blow, in milliseconds (the client's `g_multiHitInterval`).
pub const BETWEEN_HITS: f64 = 200.0;

/// What a hit is to the one struck as it lands: the client's message for it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum How {
    /// `AM_ATTACKED`: a plain blow.
    Plain,
    /// `AM_ATTACKEDNOMOTION`: a blow the one struck endures without flinching.
    Still,
    /// `AM_ATTACKED_CRITICAL`.
    Critical,
    /// `AM_ATTACKED_LUCKY`: the one struck dodged by luck; no number at all.
    Lucky,
    /// `AM_ATTACKED_SKILL`: the one hit of a skill.
    Skill,
    /// `AM_ATTACKEDNOMOTION_SKILL`.
    SkillStill,
    /// `AM_ATTACKED_MANYTIMES`: one of several hits, or of a combo: the total so far shows with it.
    Many,
    /// `AM_ATTACKED_MANYTIMES2`: a kick of the Taekwon classes: the total shows with the client's blue word.
    Kick,
    /// Message 148: one of several critical hits.
    ManyCritical,
}

impl How {
    /// Whether the one struck flinches (the client leaves it standing for the two "no motion" messages).
    pub fn flinches(self) -> bool {
        !matches!(self, How::Still | How::SkillStill | How::Lucky)
    }

    /// Whether it sounds as the hit of a skill does (`GetWeaponHitWaveName(-1)`), not as the striker's weapon.
    pub fn of_skill(self) -> bool {
        matches!(self, How::Skill | How::SkillStill)
    }
}

/// One hit of a blow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// How long after the blow's first hit lands this one does, in milliseconds.
    pub after: f64,
    pub damage: i32,
    /// The damage of the blow so far, which a monster struck shows over the hit's own number; 0 for a hit
    /// that shows none.
    pub total: i32,
    pub how: How,
}

/// The kinds of ZC_NOTIFY_ACT that are blows (the client's `DMG_NORMAL`, `DMG_ENDURE`, `DMG_MULTI_HIT`,
/// `DMG_MULTI_HIT_ENDURE`, `DMG_CRITICAL`, `DMG_LUCY_DODGE`, `DMG_MULTI_HIT_CRITICAL`): it shows no other, but
/// for picking up, sitting and standing (1, 2, 3).
pub fn is_blow(kind: u8) -> bool {
    matches!(kind, 0 | 4 | 8 | 9 | 10 | 11 | 13)
}

/// How many hits the client makes of the count the server sent: one unless the kind is of several hits, and
/// one for a count it does not believe (none, or over 36). With it the count the damage is shared by, which
/// the client works out before it looks at the kind.
fn counted(kind: u8, hits: i32) -> (i32, i32) {
    let shares = if hits <= 0 || hits > 36 { 1 } else { hits };
    (if matches!(kind, 8 | 9 | 13) { shares } else { 1 }, shares)
}

/// The hits of a blow (ZC_NOTIFY_ACT of a kind `is_blow` knows). `damage` is the whole blow's, `extra` what
/// the other hand adds (the packet's second damage), and `two_handed` whether the striker is one the client
/// shows that for: an NPC or a monster, an assassin, or a job over 4000.
pub fn of_act(kind: u8, damage: i32, hits: i32, extra: i32, two_handed: bool) -> Vec<Hit> {
    let (hits, _) = counted(kind, hits);
    let each = damage / hits;
    let how = match kind {
        4 | 9 => How::Still,
        10 => How::Critical,
        13 => How::ManyCritical,
        11 => How::Lucky,
        _ => How::Plain,
    };
    // A hit told the total so far becomes one of many, unless it is one of several critical hits.
    let many = if how == How::ManyCritical { how } else { How::Many };
    let first = if hits == 1 { Hit { after: 0.0, damage: each, total: 0, how } } else { Hit { after: 0.0, damage: each, total: each, how: many } };
    let mut all = vec![first];
    if extra != 0 && two_handed {
        // The other hits come together, half the usual time after the first; the other hand's 350 after it.
        all.extend((1..hits).map(|hit| Hit { after: BETWEEN_HITS / 2.0, damage: each, total: each * (hit + 1), how: many }));
        let after = BETWEEN_HITS / 2.0 + BETWEEN_HITS / 4.0 + BETWEEN_HITS;
        all.push(if hits == 1 { Hit { after, damage: extra, total: 0, how } } else { Hit { after, damage: extra, total: each * hits + extra, how: many } });
    } else {
        all.extend((1..hits).map(|hit| Hit { after: BETWEEN_HITS * f64::from(hit), damage: each, total: each * (hit + 1), how: many }));
    }
    all
}

/// The skills whose one hit shows a total as a combo's does, those whose hit is a kick, and those that land
/// late (`ProcessSkillMessage`; of the skills a classic server has).
const MO_COMBOFINISH: u16 = 273;
const CH_TIGERFIST: u16 = 371;
const KICKS_LATE: [u16; 2] = [415, 417];
const KICKS: [u16; 3] = [413, 419, 421];
const PR_TURNUNDEAD: u16 = 77;
const KN_PIERCE: u16 = 56;

/// The hits of a skill that strikes (ZC_NOTIFY_SKILL); none for a kind the client does not show (it reads
/// `DMG_ENDURE`, `DMG_SPLASH`, `DMG_SINGLE`, the three of several hits and `DMG_SPLASH_ENDURE`).
pub fn of_skill(skill: u16, kind: u8, damage: i32, hits: i32) -> Vec<Hit> {
    if !matches!(kind, 4 | 5 | 6 | 8 | 9 | 13 | 14) {
        return Vec::new();
    }
    let (hits, shares) = counted(kind, hits);
    let each = damage / shares;
    // A blow on everyone around lands once on each, with no total.
    if matches!(kind, 5 | 14) {
        return vec![Hit { after: 0.0, damage: each, total: 0, how: if kind == 5 { How::Skill } else { How::SkillStill } }];
    }
    let how = match kind {
        4 | 9 => How::Still,
        6 => How::SkillStill,
        13 => How::ManyCritical,
        _ => How::Skill,
    };
    let many = if how == How::ManyCritical { how } else { How::Many };
    let first = match skill {
        MO_COMBOFINISH | CH_TIGERFIST => Hit { after: 0.0, damage: each, total: each, how: many },
        skill if KICKS_LATE.contains(&skill) => Hit { after: 300.0, damage: each, total: each, how: How::Kick },
        skill if KICKS.contains(&skill) => Hit { after: 0.0, damage: each, total: each, how: How::Kick },
        _ => Hit { after: if skill == PR_TURNUNDEAD { 500.0 } else { 0.0 }, damage: each, total: if hits == 1 { 0 } else { each }, how: if hits == 1 { how } else { many } },
    };
    // Pierce's hits are 280 apart.
    let apart = if skill == KN_PIERCE { BETWEEN_HITS + 80.0 } else { BETWEEN_HITS };
    let mut all = vec![first];
    all.extend((1..hits).map(|hit| Hit { after: first.after + apart * f64::from(hit), damage: each, total: each * (hit + 1), how: many }));
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_blow_is_one_hit_with_no_total() {
        assert_eq!(of_act(0, 120, 1, 0, false), [Hit { after: 0.0, damage: 120, total: 0, how: How::Plain }]);
        // A count is believed only for the kinds of several hits.
        assert_eq!(of_act(0, 120, 3, 0, false).len(), 1);
        assert_eq!(of_act(10, 50, 1, 0, false)[0].how, How::Critical);
        assert_eq!(of_act(4, 50, 1, 0, false)[0].how, How::Still);
        assert_eq!(of_act(11, 0, 1, 0, false)[0].how, How::Lucky);
    }

    #[test]
    fn the_kinds_the_client_shows() {
        assert!([0, 4, 8, 9, 10, 11, 13].into_iter().all(is_blow));
        assert!(![1, 2, 3, 5, 6, 7, 12, 14].into_iter().any(is_blow));
    }

    #[test]
    fn several_hits_share_the_damage_and_count_it_up() {
        let hits = of_act(8, 300, 3, 0, false);
        assert_eq!(hits.iter().map(|hit| (hit.after, hit.damage, hit.total)).collect::<Vec<_>>(), [(0.0, 100, 100), (200.0, 100, 200), (400.0, 100, 300)]);
        assert!(hits.iter().all(|hit| hit.how == How::Many));
        assert!(of_act(13, 300, 2, 0, false).iter().all(|hit| hit.how == How::ManyCritical));
        // A count the client does not believe is one hit.
        assert_eq!(of_act(8, 300, 40, 0, false), [Hit { after: 0.0, damage: 300, total: 0, how: How::Plain }]);
    }

    #[test]
    fn the_other_hand_lands_350_after_the_first() {
        assert_eq!(of_act(0, 100, 1, 40, true), [Hit { after: 0.0, damage: 100, total: 0, how: How::Plain }, Hit { after: 350.0, damage: 40, total: 0, how: How::Plain }]);
        // Only for those the client shows it for.
        assert_eq!(of_act(0, 100, 1, 40, false).len(), 1);
        let hits = of_act(8, 200, 2, 30, true);
        assert_eq!(hits.iter().map(|hit| (hit.after, hit.total)).collect::<Vec<_>>(), [(0.0, 100), (100.0, 200), (350.0, 230)]);
    }

    #[test]
    fn a_skill_of_several_hits() {
        let bolts = of_skill(19, 8, 500, 5);
        assert_eq!(bolts.len(), 5);
        assert_eq!((bolts[4].after, bolts[4].damage, bolts[4].total, bolts[4].how), (800.0, 100, 500, How::Many));
        assert_eq!(of_skill(5, 6, 200, 1), [Hit { after: 0.0, damage: 200, total: 0, how: How::SkillStill }]);
        assert_eq!(of_skill(5, 8, 200, 1), [Hit { after: 0.0, damage: 200, total: 0, how: How::Skill }]);
        // Pierce's hits are 280 apart.
        assert_eq!(of_skill(KN_PIERCE, 8, 300, 3)[2].after, 560.0);
    }

    #[test]
    fn a_blow_on_everyone_around_shares_by_the_count_and_lands_once() {
        assert_eq!(of_skill(7, 5, 300, 3), [Hit { after: 0.0, damage: 100, total: 0, how: How::Skill }]);
        assert_eq!(of_skill(7, 14, 300, 1)[0].how, How::SkillStill);
    }

    #[test]
    fn the_kinds_of_a_skill_the_client_does_not_show() {
        assert!(of_skill(5, 0, 100, 1).is_empty());
        assert!(of_skill(5, 10, 100, 1).is_empty());
        assert!(of_skill(5, 11, 100, 1).is_empty());
    }

    #[test]
    fn combos_and_kicks_show_a_total_for_one_hit() {
        assert_eq!(of_skill(MO_COMBOFINISH, 6, 900, 1), [Hit { after: 0.0, damage: 900, total: 900, how: How::Many }]);
        assert_eq!(of_skill(415, 6, 400, 1), [Hit { after: 300.0, damage: 400, total: 400, how: How::Kick }]);
        assert_eq!(of_skill(413, 6, 400, 1)[0].how, How::Kick);
        assert_eq!(of_skill(PR_TURNUNDEAD, 6, 400, 1)[0].after, 500.0);
    }
}
