//! The reputation as rules (the client's `CReputeMgr` and `UIReputeWnd`): which reputations the window lists
//! for a group or a word sought, its pages, the points of a group, and how a reputation's points are drawn as
//! lights and a bar. The tables are the client's `ContentData\Repute\*.bson`, given by the gateway.

/// When a reputation is listed (`ReputeVisibility`, the table's `Invisible`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    /// `VISIBLE_TRUE`: always.
    Always,
    /// `VISIBLE_FALSE`: never.
    Never,
    /// Anything else (`VISIBLE_EXIST`): once the character has points of it.
    WithPoints,
}

impl Visibility {
    /// As the table writes it (`CReputeMgr::InfoInserter`).
    pub fn of(written: &str) -> Self {
        match written {
            "VISIBLE_TRUE" => Self::Always,
            "VISIBLE_FALSE" => Self::Never,
            _ => Self::WithPoints,
        }
    }
}

/// A reputation as the client's table has it (`ReputeInfo`), with the points the server told.
#[derive(Debug, Clone, PartialEq)]
pub struct Repute {
    pub id: u64,
    pub name: String,
    /// The most points it may have for and against.
    pub most: u64,
    pub least: u64,
    pub visibility: Visibility,
}

/// A group of reputations (`ReputeGroup`).
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: u64,
    pub name: String,
    pub reputes: Vec<u64>,
}

/// How many reputations a page of the window shows (`UIReputeWnd::m_perPage`).
pub const PER_PAGE: usize = 8;

/// The points of one reputation, 0 for one the server told nothing of.
fn points_of(points: &[(u64, i64)], id: u64) -> i64 {
    points.iter().find(|(kind, _)| *kind == id).map_or(0, |(_, points)| *points)
}

fn seen(repute: &Repute, points: &[(u64, i64)]) -> bool {
    repute.visibility == Visibility::Always || (repute.visibility == Visibility::WithPoints && points_of(points, repute.id) != 0)
}

/// What the window lists.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// Every group's (the first line of the drop-down, line 3829 of the message table).
    All,
    /// One group's, by its place among the groups.
    Group(usize),
    /// Those whose name has the word typed.
    Sought(String),
}

/// The reputations the window lists, as numbers (`CReputeMgr::GetAllReputes`, `GetGroupReputes`,
/// `FindReputes`): group by group in the order of the groups' numbers, or for a word sought in the order of
/// the reputations' own. One that stands in two groups is listed twice under "all", as the client lists it.
pub fn listed(reputes: &[Repute], groups: &[Group], points: &[(u64, i64)], filter: &Filter) -> Vec<u64> {
    let known = |id: &u64| reputes.iter().find(|repute| repute.id == *id).filter(|repute| seen(repute, points)).map(|repute| repute.id);
    match filter {
        Filter::All => groups.iter().flat_map(|group| group.reputes.iter().filter_map(known)).collect(),
        Filter::Group(place) => groups.get(*place).map(|group| group.reputes.iter().filter_map(known).collect()).unwrap_or_default(),
        Filter::Sought(word) => reputes.iter().filter(|repute| repute.name.contains(word.as_str()) && seen(repute, points)).map(|repute| repute.id).collect(),
    }
}

/// The points of all the reputations listed together: what the window writes beside "GROUP :" for a group
/// (`UIReputeWnd::FilterByGroup`).
pub fn total(listed: &[u64], points: &[(u64, i64)]) -> i64 {
    listed.iter().map(|id| points_of(points, *id)).sum()
}

/// How many pages so many reputations make, and the page asked for brought within them, counted from 1
/// (`UIReputeWnd::SetPage`).
pub fn paged(count: usize, page: usize) -> (usize, usize) {
    let pages = if count == 0 { 1 } else { (count - 1) / PER_PAGE + 1 };
    (page.clamp(1, pages), pages)
}

/// How a reputation's points are drawn (`UIReputeWnd::Paint`): a light for every thousand it may reach, so
/// many of them lit, in red for points against; the bar under them filled by what is over the last whole
/// thousand, of 299 pixels; and that remainder, written into line 3828 ("%u / 1000 P").
#[derive(Debug, PartialEq)]
pub struct Drawn {
    pub lights: u64,
    pub lit: u64,
    pub against: bool,
    pub bar: i32,
    pub remainder: u64,
}

pub fn drawn(repute: &Repute, points: i64) -> Drawn {
    let (against, amount) = (points < 0, points.unsigned_abs());
    let remainder = amount % 1000;
    Drawn {
        lights: if against { repute.least } else { repute.most } / 1000,
        lit: amount / 1000,
        against,
        bar: (remainder as f32 / 1000.0 * 299.0) as i32,
        remainder,
    }
}

/// The points of a reputation among those the server told.
pub fn points(told: &[(u64, i64)], id: u64) -> i64 {
    points_of(told, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repute(id: u64, name: &str, visibility: Visibility) -> Repute {
        Repute { id, name: name.to_owned(), most: 3000, least: 5000, visibility }
    }

    fn tables() -> (Vec<Repute>, Vec<Group>) {
        let reputes = vec![
            repute(1, "Orc Village", Visibility::Always),
            repute(2, "Goblin Village", Visibility::Always),
            repute(3, "Illusion Pass", Visibility::WithPoints),
            repute(4, "Hidden", Visibility::Never),
        ];
        let groups =
            vec![Group { id: 1, name: "Friends".to_owned(), reputes: vec![2, 1, 9] }, Group { id: 2, name: "Passes".to_owned(), reputes: vec![3, 4] }];
        (reputes, groups)
    }

    #[test]
    fn the_table_says_when_one_is_listed() {
        assert_eq!(Visibility::of("VISIBLE_TRUE"), Visibility::Always);
        assert_eq!(Visibility::of("VISIBLE_FALSE"), Visibility::Never);
        assert_eq!(Visibility::of("VISIBLE_EXIST"), Visibility::WithPoints);
    }

    #[test]
    fn lists_what_is_to_be_seen() {
        let (reputes, groups) = tables();
        // With no points told, as on the classic server: only those always listed.
        assert_eq!(listed(&reputes, &groups, &[], &Filter::All), [2, 1]);
        let points = [(3, -1500), (4, 10), (1, 2500)];
        assert_eq!(listed(&reputes, &groups, &points, &Filter::All), [2, 1, 3]);
        assert_eq!(listed(&reputes, &groups, &points, &Filter::Group(1)), [3]);
        assert_eq!(listed(&reputes, &groups, &points, &Filter::Group(5)), [0u64; 0]);
        assert_eq!(listed(&reputes, &groups, &points, &Filter::Sought("Village".to_owned())), [1, 2]);
        assert_eq!(total(&[2, 1, 3], &points), 1000);
    }

    #[test]
    fn eight_to_a_page() {
        assert_eq!(paged(0, 1), (1, 1));
        assert_eq!(paged(8, 2), (1, 1));
        assert_eq!(paged(9, 2), (2, 2));
        assert_eq!(paged(9, 0), (1, 2));
    }

    #[test]
    fn points_are_lights_and_a_bar() {
        let one = repute(1, "Orc Village", Visibility::Always);
        assert_eq!(drawn(&one, 2500), Drawn { lights: 3, lit: 2, against: false, bar: 149, remainder: 500 });
        assert_eq!(drawn(&one, -1250), Drawn { lights: 5, lit: 1, against: true, bar: 74, remainder: 250 });
        assert_eq!(drawn(&one, 0), Drawn { lights: 3, lit: 0, against: false, bar: 0, remainder: 0 });
    }
}
