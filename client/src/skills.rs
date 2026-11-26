//! Skills as rules, and no packets: which line of the client's message table says why a skill could not be
//! used, which effect a notice of the server shows, the SP and the skills a skill asks for, and the pages of a
//! job's skill window. (The packets that bring them are `net/skills.rs`.)

pub mod aim;

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
        0 if skill == 1 => [(0, 159), (1, 160), (2, 161), (3, 162), (4, 163), (5, 164), (6, 165), (8, 1304)]
            .iter()
            .find(|(tried, _)| *tried == value)
            .map(|(_, line)| *line),
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

/// The two skills placed on the ground that ask for words first (`CPlayer::SendMsg`, `AM_REQ_GROUND_SKILL`:
/// `HT_TALKIEBOX`, 0x7D, and `RG_GRAFFITI`, 0xDC).
pub const TALKIE_BOX: u16 = 125;
pub const GRAFFITI: u16 = 220;

/// How many letters the box takes that asks a skill's words, for a skill that asks any (the `maximum` of the
/// line in `UITalkboxTrapInputWnd::OnCreate`, 78, and in `UIGraffiStrboxWnd::OnCreate`, 20). Whatever is
/// typed, 20 bytes of it are sent (`crate::net::skills::use_on_ground_with_words`).
pub fn words_asked(skill: u16) -> Option<usize> {
    match skill {
        TALKIE_BOX => Some(78),
        GRAFFITI => Some(20),
        _ => None,
    }
}

/// Whether one who used a skill puffs smoke where it stands, as one that hides does: Chase Walk (389) that
/// `worked` (`ST_CHASEWALK`, `GameModePackets.cpp:18990`: `AM_MAKE_SMOKE_FFECT` to the one who used it).
pub fn puffs_smoke(skill: u16, worked: bool) -> bool {
    skill == 389 && worked
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

/// The SP a skill takes at a level, of the SP of its levels as the table has them (`data::skill_sp`, the
/// client's `GetLevelUseSpAmount` of `skillinfo_f.lub`, here with the figures of the rules the server plays):
/// a table of one figure is for every level, and a level past the table's last takes the last. None for a
/// skill the table does not have, or for no level.
pub fn sp_at(levels: &[u16], level: u16) -> Option<u16> {
    levels.get(usize::from(level).checked_sub(1)?).or(levels.last()).copied()
}

/// The SP written by a skill used at a level (`UINewSkillListWnd::DrawSkillList`, `UISkillListWnd`: "Sp : n"):
/// the table's figure for that level, and where it has none, or says 0, what the server `told` of the level
/// the skill is known at.
pub fn sp_shown(table: Option<u16>, told: u16) -> u16 {
    table.filter(|amount| *amount != 0).unwrap_or(told)
}

/// What a skill on the hotkey bar says when pointed at (`SkillTooltipFlat`, `Contents/Hotkeys/HotkeyInfo.cpp`:
/// `"%s %s Use Lv %d (Sp : %d)"` after the key's name; `SkillUseTextFlat` is the same without a key).
pub fn use_tip(name: &str, level: u16, sp: u16) -> String {
    format!("{name} Use Lv {level} (Sp : {sp})")
}

/// Of the skills a skill asks for (each with its level), those not yet known that far.
pub fn missing(needs: &[(u16, u16)], level_of: impl Fn(u16) -> u16) -> Vec<(u16, u16)> {
    needs.iter().copied().filter(|(skill, level)| level_of(*skill) < *level).collect()
}

/// The points there are to spend on skills, of the figure the server tells (the character's value of them):
/// none for a figure under nothing, and no more than a count of levels holds.
pub fn points(told: i64) -> u16 {
    told.clamp(0, i64::from(u16::MAX)) as u16
}

/// Whether the plus is offered by a skill of the tree that the server does not list as one to raise
/// (`raisable`: then the plus is there anyway). The client takes a press on a skill it lacks the needs of as
/// on any other, and plans what it asks for with it (`UINewSkillListWnd::IncreaseSkillLevel`); the server
/// lists no such skill, so the plus is offered from the table of what skills ask for: with `points` to
/// spend, by a skill not known yet (`level` 0), to one of the `job_level` the skill `asks` for, who lacks
/// some of the skills it `asks` for (`missing`).
pub fn plus_by_table(raisable: bool, points: i64, level: u16, job_level: i64, asks: (u16, &[(u16, u16)]), level_of: impl Fn(u16) -> u16) -> bool {
    !raisable && points > 0 && level == 0 && job_level >= i64::from(asks.0) && !missing(asks.1, level_of).is_empty()
}

/// The skills to raise, one level each and in this order, when a skill is raised by one with `points` to
/// spend: first what it asks for and lacks, up to the level asked, then the skill itself, as far as the
/// points go. The client's `UINewSkillListWnd::IncreaseSkillLevel` plans the skill and what it asks for (its
/// `m_values`) so; here what those ask for in turn is raised too and everything asked for goes first, since
/// the server takes a level only of a skill whose own needs are met (this project's own, `docs/DEBTS.md`).
/// `level_of` is the level a skill is known at, `needs_of` what it asks for.
pub fn raises(skill: u16, points: u16, level_of: impl Fn(u16) -> u16, needs_of: impl Fn(u16) -> Vec<(u16, u16)>) -> Vec<u16> {
    fn plan(
        (skill, target, depth): (u16, u16, u8),
        (level_of, needs_of): (&dyn Fn(u16) -> u16, &dyn Fn(u16) -> Vec<(u16, u16)>),
        planned: &mut std::collections::HashMap<u16, u16>,
        order: &mut Vec<u16>,
    ) {
        let level = planned.get(&skill).copied().unwrap_or_else(|| level_of(skill));
        if level >= target || depth > 8 {
            return;
        }
        for (need, asked) in needs_of(skill) {
            plan((need, asked, depth + 1), (level_of, needs_of), planned, order);
        }
        order.extend(std::iter::repeat_n(skill, usize::from(target - level)));
        planned.insert(skill, target);
    }
    let mut order = Vec::new();
    plan((skill, level_of(skill).saturating_add(1), 0), (&level_of, &needs_of), &mut std::collections::HashMap::new(), &mut order);
    order.truncate(usize::from(points));
    order
}

/// The pages of a job's skill window, as the client lays them out (`skilltreeview.lub`): one for each job it
/// grew through, the first with the Novice's skills among its own. `tree` has, by job, the job it grew out of
/// and where its skills stand. Each page: the place of each skill (counting the cells row by row, seven to a
/// row) and the skill.
pub fn pages(job: u16, tree: &std::collections::HashMap<u16, (Option<u16>, Vec<(u16, u16)>)>) -> Vec<Vec<(u16, u16)>> {
    // From the job back to the Novice.
    let mut line = Vec::new();
    let mut at = Some(job);
    while let Some(job) = at.filter(|job| !line.contains(job) && line.len() < 8) {
        line.push(job);
        at = tree.get(&job).and_then(|(from, _)| *from).filter(|from| *from != job);
    }
    line.reverse();
    let mut pages: Vec<Vec<(u16, u16)>> = Vec::new();
    for (index, job) in line.iter().enumerate() {
        let slots = tree.get(job).map(|(_, slots)| slots.clone()).unwrap_or_default();
        // The Novice's page is the first job's too.
        if index == 1 && line[0] == 0 {
            pages[0].extend(slots);
        } else {
            pages.push(slots);
        }
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_skills_ask_for_words_before_they_are_placed() {
        assert_eq!((words_asked(125), words_asked(220)), (Some(78), Some(20)));
        // Any other skill of the ground is placed at once: Fire Wall, Warp Portal.
        assert_eq!((words_asked(18), words_asked(27)), (None, None));
    }

    #[test]
    fn a_skill_takes_the_sp_of_its_level() {
        // Bash: 8 to level 5, 15 from 6; Brandish Spear: 12 at every level.
        let bash = [8, 8, 8, 8, 8, 15, 15, 15, 15, 15];
        assert_eq!((sp_at(&bash, 1), sp_at(&bash, 6), sp_at(&bash, 10), sp_at(&bash, 11)), (Some(8), Some(15), Some(15), Some(15)));
        assert_eq!((sp_at(&[12], 1), sp_at(&[12], 10), sp_at(&[12], 0), sp_at(&[], 3)), (Some(12), Some(12), None, None));
        // Where the table has nothing, or 0, the server's figure stands.
        assert_eq!((sp_shown(Some(12), 24), sp_shown(None, 24), sp_shown(Some(0), 24)), (12, 24, 24));
        assert_eq!(use_tip("Bash", 6, 15), "Bash Use Lv 6 (Sp : 15)");
    }

    #[test]
    fn a_skill_raised_takes_what_it_asks_for_first() {
        // Brandish Spear (57) asks Riding 1 (63) and Spear Stab 3 (58); Spear Stab asks Pierce 5 (56); Riding
        // asks Endure 1 (8), which asks Provoke 5 (6).
        let needs = |skill: u16| match skill {
            57 => vec![(63, 1), (58, 3)],
            58 => vec![(56, 5)],
            63 => vec![(8, 1)],
            8 => vec![(6, 5)],
            _ => Vec::new(),
        };
        let known = |skill: u16| match skill {
            6 => 5,
            56 => 4,
            58 => 1,
            _ => 0,
        };
        assert_eq!(missing(&needs(57), known), [(63, 1), (58, 3)]);
        assert_eq!(missing(&needs(8), known), []);
        assert_eq!(raises(57, 20, known, needs), [8, 63, 56, 58, 58, 57]);
        // The plus by Brandish Spear, not known and lacking Riding: with a point, at the job level asked.
        assert!(plus_by_table(false, 1, 0, 30, (30, &needs(57)), known));
        assert!(!plus_by_table(false, 0, 0, 30, (30, &needs(57)), known) && !plus_by_table(false, 1, 0, 29, (30, &needs(57)), known));
        // Not by one known already, by one that lacks nothing, or twice by one the server lists.
        assert!(!plus_by_table(false, 1, 1, 30, (30, &needs(57)), known) && !plus_by_table(false, 1, 0, 30, (0, &needs(8)), known));
        assert!(!plus_by_table(true, 1, 0, 30, (30, &needs(57)), known));
        assert_eq!((points(-3), points(0), points(12), points(1 << 40)), (0, 0, 12, u16::MAX));
        // As far as the points go; with none, nothing.
        assert_eq!(raises(57, 3, known, needs), [8, 63, 56]);
        assert_eq!(raises(57, 0, known, needs), [] as [u16; 0]);
        // One that asks for nothing, or has all it asks for, goes up by one.
        assert_eq!((raises(6, 4, known, needs), raises(8, 4, known, needs)), (vec![6], vec![8]));
        // Skills that ask for each other end.
        let round = |skill: u16| vec![(if skill == 1 { 2 } else { 1 }, 1)];
        assert!(raises(1, 50, |_| 0, round).len() <= 50);
    }

    #[test]
    fn a_job_has_a_page_for_each_job_it_grew_through() {
        let tree: std::collections::HashMap<u16, (Option<u16>, Vec<(u16, u16)>)> = [
            (0, (None, vec![(0, 1)])),
            (1, (Some(0), vec![(1, 2), (2, 3)])),
            (7, (Some(1), vec![(0, 55)])),
            // One that grew out of itself, two that grew out of each other, one out of a job not in the table.
            (20, (Some(20), vec![(0, 9)])),
            (30, (Some(31), vec![(0, 10)])),
            (31, (Some(30), vec![(0, 11)])),
            (40, (Some(99), vec![(0, 12)])),
        ]
        .into_iter()
        .collect();
        assert_eq!(pages(0, &tree), [vec![(0, 1)]]);
        // The Novice's page is the first job's too.
        assert_eq!(pages(1, &tree), [vec![(0, 1), (1, 2), (2, 3)]]);
        assert_eq!(pages(7, &tree), [vec![(0, 1), (1, 2), (2, 3)], vec![(0, 55)]]);
        assert_eq!(pages(20, &tree), [vec![(0, 9)]]);
        assert_eq!(pages(30, &tree), [vec![(0, 11)], vec![(0, 10)]]);
        // A job the table does not have has an empty page, and so has one a job grew out of.
        assert_eq!(pages(5, &tree), [Vec::new()]);
        assert_eq!(pages(40, &tree), [Vec::new(), vec![(0, 12)]]);
        assert_eq!(pages(3, &std::collections::HashMap::new()), [Vec::new()]);
    }

    #[test]
    fn a_notified_effect_is_the_clients_number_for_its_kind() {
        assert_eq!((notified_effect(2), notified_effect(3), notified_effect(5), notified_effect(6)), (Some(155), Some(154), Some(305), Some(306)));
        assert_eq!((notified_effect(7), notified_effect(8), notified_effect(9)), (Some(338), Some(337), Some(582)));
        assert_eq!((notified_effect(0), notified_effect(4)), (None, None));
        assert!(puffs_smoke(389, true) && !puffs_smoke(389, false) && !puffs_smoke(51, true));
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
