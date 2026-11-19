//! Skills as rules, and no packets: which line of the client's message table says why a skill could not be
//! used, and which effect a notice of the server shows. (The packets that bring them are `net/skills.rs`.)

/// The causes the client answers with one line of its message table each (its `ReceiveSkillFailure`): the cause,
/// and the line's number.
const FAILURES: &[(u8, usize)] = &[
    (1, 202), (2, 203), (3, 808), (5, 233), (6, 239), (7, 246), (8, 247), (9, 580), (11, 1396), (12, 1397),
    (13, 1398), (14, 1399), (15, 1400), (16, 1401), (17, 1411), (18, 1412), (19, 1413), (20, 1414),
    (21, 1415), (22, 1416), (23, 1417), (24, 1425), (25, 1864), (26, 1427), (27, 1428), (28, 1929),
    (29, 1932), (30, 1937), (31, 1433), (32, 1434), (33, 1435), (34, 1948), (35, 1952), (36, 1951),
    (37, 1439), (40, 1450), (43, 1453), (50, 1462), (51, 1474), (52, 1475), (53, 1476), (54, 1477),
    (57, 1519), (60, 1522), (79, 1748), (80, 1823), (81, 1824), (82, 1825), (83, 1954), (84, 2466),
    (87, 2563), (88, 2626), (89, 2982), (93, 3482), (94, 3483), (95, 3529), (96, 3530), (97, 3533),
    (99, 3604), (100, 3643), (102, 3957), (103, 3958), (104, 3959), (105, 3960), (106, 3961), (107, 3962),
    (108, 3963), (109, 3964), (110, 3965), (111, 3968), (112, 3987),
];

/// The line of the client's message table said when a skill could not be used, by the skill, the value and the
/// cause; none for the causes this does not know (those the client fills an item's name or a number into).
/// A line with `%s` in it wants the skill's name.
pub fn failure(skill: u16, value: i32, cause: u8) -> Option<usize> {
    match cause {
        // Not learned far enough. For the Basic Skill the value says what was tried: trading, an emotion,
        // sitting, a chat room, a party, shouting, fighting players, joining a party.
        0 if skill == 1 => [(0, 159), (1, 160), (2, 161), (3, 162), (4, 163), (5, 164), (6, 165), (8, 1304)].iter().find(|(tried, _)| *tried == value).map(|(_, line)| *line),
        // Warp Portal, Steal and Envenom have lines of their own.
        0 => Some(match skill {
            27 => 214,
            50 => 205,
            52 => 207,
            _ => 204,
        }),
        // Still cooling down: the line names the skill.
        4 => Some(1460),
        10 => Some(285),
        _ => FAILURES.iter().find(|(known, _)| *known == cause).map(|(_, line)| *line),
    }
}

/// The effect ZC_NOTIFY_EFFECT (0x019b) shows for its kinds beyond the two levels gained, by the client's
/// numbers (`ReceivePartyMemberEffect`, `EffectTypes.h`): a refining that failed (2: `EF_REFINEFAIL`, 155) or
/// went well (3: `EF_REFINEOK`, 154), a potion brewed (5: `EF_PHARMACY_OK`, 305) or spoilt (6:
/// `EF_PHARMACY_FAIL`, 306), the levels of a super novice (7: `EF_ANGEL2`, 338; 8: `EF_JOBLVUP50`, 337) and of
/// a taekwon (9: `EF_ANGEL3`, 582).
pub fn notified_effect(kind: u32) -> Option<u32> {
    Some(match kind {
        2 => 155,
        3 => 154,
        5 => 305,
        6 => 306,
        7 => 338,
        8 => 337,
        9 => 582,
        _ => return None,
    })
}

/// The level a skill is used at one down or up from the one `used` now, between 1 and the level it is
/// `known` at; none at the level it is known at, of which nothing is kept.
pub fn use_level_stepped(used: u16, known: u16, more: bool) -> Option<u16> {
    let wanted = if more { (used + 1).min(known) } else { used.saturating_sub(1).max(1) };
    (wanted != known).then_some(wanted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notified_effect_is_the_clients_number_for_its_kind() {
        assert_eq!((notified_effect(2), notified_effect(3), notified_effect(5), notified_effect(6)), (Some(155), Some(154), Some(305), Some(306)));
        assert_eq!((notified_effect(7), notified_effect(8), notified_effect(9)), (Some(338), Some(337), Some(582)));
        assert_eq!((notified_effect(0), notified_effect(4)), (None, None));
    }

    #[test]
    fn a_failure_is_a_line_of_the_message_table() {
        // Not enough SP; the skill not learned far enough, with the lines Warp Portal, Steal and Envenom have.
        assert_eq!((failure(46, 0, 1), failure(46, 0, 0)), (Some(202), Some(204)));
        assert_eq!((failure(27, 0, 0), failure(50, 0, 0), failure(52, 0, 0)), (Some(214), Some(205), Some(207)));
        // The Basic Skill says what was tried: an emotion, joining a party; nothing for what it has no line for.
        assert_eq!((failure(1, 1, 0), failure(1, 8, 0), failure(1, 7, 0)), (Some(160), Some(1304), None));
        // Still cooling down; the last cause of the table; one the table does not have.
        assert_eq!((failure(46, 0, 4), failure(46, 0, 10), failure(46, 0, 112), failure(46, 0, 200)), (Some(1460), Some(285), Some(3987), None));
    }

    #[test]
    fn the_level_a_skill_is_used_at_stays_within_what_is_known() {
        assert_eq!((use_level_stepped(10, 10, false), use_level_stepped(9, 10, false), use_level_stepped(1, 10, false)), (Some(9), Some(8), Some(1)));
        // Back at the level it is known at, and past it: nothing is kept.
        assert_eq!((use_level_stepped(9, 10, true), use_level_stepped(10, 10, true)), (None, None));
        assert_eq!((use_level_stepped(1, 1, false), use_level_stepped(0, 3, false)), (None, Some(1)));
    }
}
