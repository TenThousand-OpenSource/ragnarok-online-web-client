//! The achievements as rules (the client's `CAchievementMgr`, `UIAchievementWnd` and what `CSession` keeps of
//! them): which achievements a tab lists and in what order, how many of them are completed, what a goal says
//! and in which colour, the two completed last, the achievements kept in sight, and how a gauge is filled.
//! The table itself is the client's `System\achievement_list.lub`, given by the gateway.

use std::collections::BTreeMap;

use crate::net::achievements::Record;

/// One goal of an achievement (`AchievementReward`: the file's `resource`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Goal {
    pub text: String,
    /// How many are to be reached, for an achievement that counts.
    pub count: u32,
    /// The achievement its line leads to when pressed (the file's `shortcut`, `UIAchTV::OnLBtnDown`); 0 for
    /// none.
    pub link: u32,
}

/// An achievement as the client's table has it (`AchievementInfo`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Info {
    pub id: u32,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub points: u32,
    /// What completing it gives: an item, a title, a state; 0 for none of that kind.
    pub item: u32,
    pub title_id: u32,
    pub buff: u32,
    pub goals: Vec<Goal>,
    /// Its goals are counted and shown with gauges (`UI_Type` 1), or are only done or not (0).
    pub counted: bool,
    /// The picture of its kind: `achievement_re\icon_<this>.bmp`.
    pub icon: String,
    /// The tab it stands under and the tab under that one (`SetAchieveIDByTab`: `major`, `minor`).
    pub group: u32,
    pub sub: u32,
}

impl Info {
    /// Whether completing it gives anything (`CAchievementMgr::HasReward`).
    pub fn has_reward(&self) -> bool {
        self.item > 0 || self.title_id > 0 || self.buff > 0
    }
}

/// The tabs down the window's left (`UIAchievementWnd::CreateTab`): each the line of the message table that
/// names it, and the lines of the tabs under it. The first is the summary.
pub const TABS: [(usize, &[usize]); 7] = [
    (2656, &[]),
    (2657, &[2658, 2659, 2660]),
    (2661, &[2662, 2663, 2664, 2665, 2666, 2667]),
    (2668, &[2669, 2670]),
    (2671, &[2672, 2673]),
    (2674, &[2675, 2676, 2677]),
    (2678, &[]),
];

/// A tab is 40 high, a tab under it 26, and three of those show at once (`CreateTab`, `UIAchTab`).
pub const TAB_HEIGHT: i32 = 40;
pub const SUB_HEIGHT: i32 = 26;
pub const SUBS_SHOWN: usize = 3;

/// How far down the tabs' column the tab of a group begins, with the tabs under the open one shown
/// (`UIAchTab::DrawGroup`).
pub fn tab_top(group: usize, open: usize) -> i32 {
    let pushed = if group > open { TABS.get(open).map_or(0, |(_, subs)| subs.len().min(SUBS_SHOWN)) as i32 * SUB_HEIGHT } else { 0 };
    group as i32 * TAB_HEIGHT + pushed
}

/// The achievements of a tab, in the order of their numbers (`CAchievementMgr::GetGroupAchievements`): those
/// of one tab under the group, or of all of them when none is picked.
pub fn of_tab(all: &[Info], group: u32, sub: Option<u32>) -> Vec<&Info> {
    let mut found: Vec<&Info> = all.iter().filter(|info| info.group == group && sub.is_none_or(|sub| info.sub == sub)).collect();
    found.sort_by_key(|info| info.id);
    found
}

/// How many achievements a tab has, and how many of them are completed (`GetCompletedCount`,
/// `GetAchievementCount`): the summary counts every achievement there is.
pub fn counted(all: &[Info], records: &BTreeMap<u32, Record>, group: u32, sub: Option<u32>) -> (usize, usize) {
    let done = |info: &Info| records.get(&info.id).is_some_and(|record| record.completed);
    if group == 0 {
        return (all.iter().filter(|info| done(info)).count(), all.len());
    }
    let tab = of_tab(all, group, sub);
    (tab.iter().filter(|info| done(info)).count(), tab.len())
}

/// Which of a tab's achievements the list shows (the two round buttons over it).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Shown {
    /// Those completed, by when, then the others.
    #[default]
    Completed,
    /// Those completed whose reward was not taken yet, by when, then those not completed.
    Unfinished,
}

/// The list of a tab, in its order (`UIAchListWnd::SetAchievements`).
pub fn listed(tab: &[&Info], records: &BTreeMap<u32, Record>, shown: Shown) -> Vec<u32> {
    let record = |info: &Info| records.get(&info.id).copied().unwrap_or_default();
    let mut first: Vec<(u32, u32)> = tab
        .iter()
        .filter(|info| {
            let record = record(info);
            record.completed && (shown == Shown::Completed || (info.has_reward() && !record.rewarded))
        })
        .map(|info| (record(info).completed_at, info.id))
        .collect();
    first.sort_by_key(|(at, _)| *at);
    first.into_iter().map(|(_, id)| id).chain(tab.iter().filter(|info| !record(info).completed).map(|info| info.id)).collect()
}

/// The two achievements completed last, the latest first (`CSession::FindLatestCompletedAchievements`, which
/// goes through them in the order of their numbers).
pub fn latest(records: &BTreeMap<u32, Record>) -> (Option<u32>, Option<u32>) {
    let (mut first, mut second) = (None::<(u32, u32)>, None::<(u32, u32)>);
    for (id, record) in records.iter().filter(|(_, record)| record.completed) {
        let at = record.completed_at;
        if first.is_none_or(|(best, _)| best < at) {
            (second, first) = (first, Some((at, *id)));
        } else if second.is_none_or(|(best, _)| best < at) {
            second = Some((at, *id));
        }
    }
    (first.map(|(_, id)| id), second.map(|(_, id)| id))
}

/// The colour of a goal's line (`UIAchViewWnd::UpdateInfo`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tone {
    /// Reached: (125, 151, 201), and the line is underlined.
    Reached,
    /// Not begun: (54, 87, 153).
    Waiting,
    /// A count under way: (230, 137, 65).
    Counting,
}

/// A goal as the pane of an achievement writes it.
#[derive(Debug, Clone, PartialEq)]
pub struct GoalLine {
    pub words: String,
    pub tone: Tone,
    /// The gauge under a counted goal: how far, of how many.
    pub gauge: Option<(u32, u32)>,
}

/// What the pane says of a goal that has come so far (`UIAchViewWnd::UpdateInfo`): one that counts is
/// written with its numbers and has a gauge.
pub fn goal_line(info: &Info, place: usize, progress: u32) -> Option<GoalLine> {
    let goal = info.goals.get(place)?;
    Some(if info.counted {
        GoalLine {
            words: format!("{} ( {} / {} )", goal.text, progress, goal.count),
            tone: if progress == goal.count { Tone::Reached } else if progress == 0 { Tone::Waiting } else { Tone::Counting },
            gauge: Some((progress, goal.count)),
        }
    } else {
        GoalLine { words: goal.text.clone(), tone: if progress > 0 { Tone::Reached } else { Tone::Waiting }, gauge: None }
    })
}

/// Where a goal's line and its gauge stand in the pane, from its top: the lines of goals that are only done
/// or not are 19 apart from 97, with their dot at 103; those that count are 36 apart, the line at 105, its
/// dot at 111 and the gauge at 123 (`UIAchViewWnd::UpdateInfo`, `Paint`).
pub fn goal_places(counted: bool, place: usize) -> (i32, i32, i32) {
    let place = place as i32;
    if counted { (105 + 36 * place, 111 + 36 * place, 123 + 36 * place) } else { (97 + 19 * place, 103 + 19 * place, 0) }
}

/// How many pixels of a gauge `width` wide are filled (`UICustomGageBar::Paint`: whole hundredths of the
/// width); none for a gauge the client does not draw at all, one whose value is past its most.
pub fn filled(value: u32, most: u32, width: i32) -> Option<i32> {
    if value > most {
        return None;
    }
    let percent = if most == 0 { 0.0 } else { value as f32 * 100.0 / most as f32 };
    Some(if percent > 0.0 { percent as i32 * width / 100 } else { 0 })
}

/// A date as the list writes it (`"%4d.%2d.%2d"`).
pub fn dated(year: u32, month: u32, day: u32) -> String {
    format!("{year:4}.{month:2}.{day:2}")
}

/// How many achievements can be kept in sight at once (`CSession::ToggleTrackedAchievement`).
pub const KEPT_IN_SIGHT: usize = 3;

/// An achievement kept in sight (`SessionRecord6178`, the client's `UIAchTracingWnd`).
#[derive(Debug, Clone, PartialEq)]
pub struct Traced {
    pub id: u32,
    /// Its goals are listed under its name.
    pub open: bool,
}

/// What pressing the picture of a row of the list did to those kept in sight.
#[derive(Debug, PartialEq)]
pub enum Toggled {
    /// It is kept in sight now, or no longer.
    Changed,
    /// There are three already: the client says its line 2680 in the chat.
    Full,
}

/// Keeps an achievement in sight, or lets go of one that is (`CSession::ToggleTrackedAchievement`).
pub fn toggle(traced: &mut Vec<Traced>, id: u32) -> Toggled {
    if let Some(at) = traced.iter().position(|one| one.id == id) {
        traced.remove(at);
    } else if traced.len() >= KEPT_IN_SIGHT {
        return Toggled::Full;
    } else {
        traced.push(Traced { id, open: false });
    }
    Toggled::Changed
}

/// A goal of an achievement kept in sight, as its window writes it, and whether it is reached
/// (`CSession::RefreshTrackedAchievement`): the goal's words are cut at 40 bytes, the numbers of a count
/// kept whole.
pub fn traced_line(info: &Info, place: usize, progress: u32) -> Option<(String, bool)> {
    let goal = info.goals.get(place)?;
    let cut = |most: usize| {
        let mut end = most.min(goal.text.len());
        while !goal.text.is_char_boundary(end) {
            end -= 1;
        }
        &goal.text[..end]
    };
    Some(if info.counted {
        let count = format!("({}/{})", progress, goal.count);
        let words = if goal.text.len() + count.len() > 40 {
            format!("- {}...{count}", cut(40usize.saturating_sub(count.len())))
        } else {
            format!("- {} {count}", goal.text)
        };
        (words, progress == goal.count)
    } else {
        (if goal.text.len() > 40 { format!("- {}...", cut(40)) } else { format!("- {}", goal.text) }, progress > 0)
    })
}

/// The titles the character may wear, in the order the client lists them (`CSession::AddAchievementJobTitle`:
/// sorted by what each is called): those of the achievements whose reward was taken.
pub fn titles(all: &[Info], records: &BTreeMap<u32, Record>, called: impl Fn(u32) -> String) -> Vec<u32> {
    let mut titles: Vec<u32> = Vec::new();
    for info in all.iter().filter(|info| info.title_id > 0 && records.get(&info.id).is_some_and(|record| record.rewarded)) {
        if !titles.contains(&info.title_id) {
            titles.push(info.title_id);
        }
    }
    titles.sort_by_key(|title| called(*title));
    titles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(id: u32, group: u32, sub: u32) -> Info {
        Info { id, group, sub, ..Info::default() }
    }

    fn done(at: u32, rewarded: bool) -> Record {
        Record { completed: true, completed_at: at, rewarded, ..Record::default() }
    }

    #[test]
    fn a_tab_lists_its_own_by_number_and_a_group_all_under_it() {
        let all = [info(30, 1, 1), info(10, 1, 0), info(20, 2, 0), info(5, 1, 1)];
        assert_eq!(of_tab(&all, 1, Some(1)).iter().map(|info| info.id).collect::<Vec<_>>(), [5, 30]);
        assert_eq!(of_tab(&all, 1, None).iter().map(|info| info.id).collect::<Vec<_>>(), [5, 10, 30]);
        let records = BTreeMap::from([(10, done(5, false)), (20, done(6, false)), (30, Record::default())]);
        assert_eq!(counted(&all, &records, 1, None), (1, 3));
        assert_eq!(counted(&all, &records, 1, Some(1)), (0, 2));
        assert_eq!(counted(&all, &records, 0, None), (2, 4));
    }

    #[test]
    fn the_list_has_the_completed_first_by_when() {
        let mut all = [info(1, 1, 0), info(2, 1, 0), info(3, 1, 0), info(4, 1, 0)];
        all[1].item = 501;
        all[3].title_id = 1013;
        let records = BTreeMap::from([(2, done(90, false)), (3, done(50, false)), (4, done(70, true))]);
        let tab = of_tab(&all, 1, None);
        assert_eq!(listed(&tab, &records, Shown::Completed), [3, 4, 2, 1]);
        // Only what still has a reward to take, then what is not completed.
        assert_eq!(listed(&tab, &records, Shown::Unfinished), [2, 1]);
    }

    #[test]
    fn the_two_completed_last() {
        let records = BTreeMap::from([(1, done(10, false)), (2, done(30, false)), (3, Record::default()), (4, done(20, false))]);
        assert_eq!(latest(&records), (Some(2), Some(4)));
        assert_eq!(latest(&BTreeMap::from([(7, done(1, false))])), (Some(7), None));
        assert_eq!(latest(&BTreeMap::new()), (None, None));
    }

    #[test]
    fn a_goal_says_how_far_it_is() {
        let mut one = info(1, 1, 0);
        one.goals = vec![Goal { text: "Talk".to_owned(), count: 0, link: 0 }];
        assert_eq!(goal_line(&one, 0, 0), Some(GoalLine { words: "Talk".to_owned(), tone: Tone::Waiting, gauge: None }));
        assert_eq!(goal_line(&one, 0, 1).unwrap().tone, Tone::Reached);
        assert_eq!(goal_line(&one, 1, 1), None);
        let mut counts = info(2, 1, 0);
        (counts.counted, counts.goals) = (true, vec![Goal { text: "Poring".to_owned(), count: 10, link: 0 }]);
        assert_eq!(goal_line(&counts, 0, 3), Some(GoalLine { words: "Poring ( 3 / 10 )".to_owned(), tone: Tone::Counting, gauge: Some((3, 10)) }));
        assert_eq!(goal_line(&counts, 0, 10).unwrap().tone, Tone::Reached);
        assert_eq!(goal_line(&counts, 0, 0).unwrap().tone, Tone::Waiting);
        assert_eq!(goal_places(false, 2), (135, 141, 0));
        assert_eq!(goal_places(true, 1), (141, 147, 159));
    }

    #[test]
    fn a_gauge_fills_by_whole_hundredths() {
        assert_eq!(filled(1, 3, 254), Some(33 * 254 / 100));
        assert_eq!(filled(0, 0, 84), Some(0));
        assert_eq!(filled(5, 5, 84), Some(84));
        assert_eq!(filled(6, 5, 84), None);
    }

    #[test]
    fn a_date_is_written_in_its_columns() {
        assert_eq!(dated(2026, 3, 9), "2026. 3. 9");
        assert_eq!(dated(2026, 11, 26), "2026.11.26");
    }

    #[test]
    fn three_are_kept_in_sight_at_most() {
        let mut traced = Vec::new();
        for id in 1..=3 {
            assert_eq!(toggle(&mut traced, id), Toggled::Changed);
        }
        assert_eq!(toggle(&mut traced, 4), Toggled::Full);
        assert_eq!(toggle(&mut traced, 2), Toggled::Changed);
        assert_eq!(traced.iter().map(|one| one.id).collect::<Vec<_>>(), [1, 3]);
    }

    #[test]
    fn a_goal_in_sight_is_cut_at_forty() {
        let mut one = info(1, 1, 0);
        one.goals = vec![Goal { text: "a".repeat(45), count: 0, link: 0 }, Goal { text: "Short".to_owned(), count: 0, link: 0 }];
        assert_eq!(traced_line(&one, 0, 0), Some((format!("- {}...", "a".repeat(40)), false)));
        assert_eq!(traced_line(&one, 1, 2), Some(("- Short".to_owned(), true)));
        let mut counts = info(2, 1, 0);
        counts.counted = true;
        counts.goals = vec![Goal { text: "b".repeat(38), count: 100, link: 0 }, Goal { text: "Poring".to_owned(), count: 5, link: 0 }];
        assert_eq!(traced_line(&counts, 0, 7), Some((format!("- {}...(7/100)", "b".repeat(33)), false)));
        assert_eq!(traced_line(&counts, 1, 5), Some(("- Poring (5/5)".to_owned(), true)));
    }

    #[test]
    fn the_titles_are_those_rewarded_by_name() {
        let mut all = [info(1, 1, 0), info(2, 1, 0), info(3, 1, 0)];
        (all[0].title_id, all[1].title_id, all[2].title_id) = (1002, 1001, 1003);
        let records = BTreeMap::from([(1, done(1, true)), (2, done(2, true)), (3, done(3, false))]);
        let called = |title: u32| if title == 1002 { "Ace".to_owned() } else { "Zed".to_owned() };
        assert_eq!(titles(&all, &records, called), [1002, 1001]);
    }

    #[test]
    fn the_tabs_under_the_open_one_push_the_rest_down() {
        assert_eq!(tab_top(1, 2), 40);
        assert_eq!(tab_top(3, 2), 120 + 3 * 26);
        assert_eq!(tab_top(2, 1), 80 + 3 * 26);
        assert_eq!(tab_top(6, 0), 240);
    }
}
