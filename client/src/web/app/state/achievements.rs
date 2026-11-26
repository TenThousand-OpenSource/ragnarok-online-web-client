//! What the app keeps of the achievements and the titles.

use std::collections::BTreeMap;

use crate::achievements::{Shown, Traced};
use crate::net::achievements::{Record, Standing};

/// The achievements (the client's `CSession::achievements` and what goes with them, `UIAchievementWnd`,
/// `UIAchTracingWnd`) and the title worn.
#[derive(Default)]
pub(in crate::web) struct Achievements {
    pub open: bool,
    /// What the server told of each achievement begun or completed, by its number.
    pub records: BTreeMap<u32, Record>,
    /// The score, the rank and the points towards the next.
    pub standing: Standing,
    /// The tab open: 0 the summary; and the tab under it, if one is picked.
    pub group: u32,
    pub sub: Option<u32>,
    /// The first of the tabs under the open one that shows, where there are more than three (`UIAchTab::scrollPos`).
    pub sub_first: usize,
    /// Which of the two round buttons over the list is on.
    pub shown: Shown,
    /// The achievement picked in the list.
    pub picked: Option<u32>,
    /// Those kept in sight, in the order they were taken.
    pub traced: Vec<Traced>,
    /// The menu's icon wears its mark: one was completed and the window not opened since.
    pub told: bool,
    /// The line across the screen that tells of one completed, and when it goes (`CGameMode`'s
    /// `achievementBalloon`, ten seconds), by the page's clock.
    pub balloon: Option<(String, f64)>,
    /// The character's own names were asked of the server, for the title told with them.
    pub own_asked: bool,
    /// The title worn (`CSession::jobTitle`); 0 for none.
    pub title: u32,
    /// The Equipment window shows its page of titles.
    pub titles_page: bool,
    /// The line picked in the list of titles: a title, or 0 for the line that takes the title off.
    pub title_picked: Option<u32>,
    /// The titles gained since the Equipment window was last closed: their lines are coloured
    /// (`CSession::achievementIds`, `UIListBoxTitle::Paint`).
    pub new_titles: Vec<u32>,
}
