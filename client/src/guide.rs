//! The Adventure Guide as rules (the client's `CUIAdventureGuide`, `CAdventureGuideMgr`): what a character of
//! a level is offered, which kind of a tab shows, and the lines of the message table a monster's element,
//! race and size are written with. The window is `web/hud/guide.rs`, the tables `web/data/guide.rs`.

/// Whether something for the levels from `least` to `most` is for a character of that `level`.
pub fn in_levels(least: u32, most: u32, level: i64) -> bool {
    (i64::from(least)..=i64::from(most)).contains(&level)
}

/// The kind shown of a tab's kinds (each its number, its name and the levels it is for): the one picked if
/// the character's level is shown it, or the first of those the level is shown.
pub fn kind_shown(kinds: &[(u32, String, u32, u32)], picked: Option<u32>, level: i64) -> Option<u32> {
    let shown = |kind: &&(u32, String, u32, u32)| in_levels(kind.2, kind.3, level);
    kinds.iter().filter(shown).find(|kind| Some(kind.0) == picked).or_else(|| kinds.iter().find(shown)).map(|kind| kind.0)
}

/// The client's words for a monster's element, race and size, by what the server's database calls them: lines
/// of the client's message table (those its navigation shows a monster with).
pub fn element_line(element: &str) -> usize {
    match element {
        "Water" => 2263,
        "Earth" => 2264,
        "Fire" => 2265,
        "Wind" => 2266,
        "Poison" => 2267,
        "Holy" => 2268,
        "Dark" => 2269,
        "Ghost" => 2270,
        "Undead" => 2271,
        _ => 2272,
    }
}

pub fn race_line(race: &str) -> usize {
    match race {
        "Undead" => 2276,
        "Brute" => 2277,
        "Plant" => 2278,
        "Insect" => 2279,
        "Fish" => 2280,
        "Demon" => 2281,
        "Demihuman" => 2282,
        "Angel" => 2283,
        "Dragon" => 2284,
        _ => 2285,
    }
}

pub fn size_line(size: &str) -> usize {
    match size {
        "Medium" => 2273,
        "Large" => 2274,
        _ => 2275,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_level_is_offered_what_is_for_it() {
        assert!(in_levels(1, 10, 1) && in_levels(1, 10, 10));
        assert!(!in_levels(1, 10, 0) && !in_levels(1, 10, 11));
        assert!(!in_levels(20, 10, 15));
    }

    #[test]
    fn the_kind_picked_shows_while_the_level_is_shown_it() {
        let kinds = [(1, "one".to_owned(), 1, 10), (2, "two".to_owned(), 5, 20), (3, "three".to_owned(), 15, 99)];
        assert_eq!(kind_shown(&kinds, None, 7), Some(1));
        assert_eq!(kind_shown(&kinds, Some(2), 7), Some(2));
        // One picked that is not for the level gives way to the first that is; so does one that is no kind.
        assert_eq!(kind_shown(&kinds, Some(3), 7), Some(1));
        assert_eq!(kind_shown(&kinds, Some(9), 17), Some(2));
        assert_eq!(kind_shown(&kinds, Some(1), 100), None);
        assert_eq!(kind_shown(&[], None, 1), None);
    }

    #[test]
    fn a_monster_is_worded_by_the_lines_of_the_message_table() {
        assert_eq!((element_line("Water"), element_line("Undead"), element_line("Neutral")), (2263, 2271, 2272));
        assert_eq!((race_line("Undead"), race_line("Dragon"), race_line("Formless")), (2276, 2284, 2285));
        assert_eq!((size_line("Medium"), size_line("Large"), size_line("Small")), (2273, 2274, 2275));
    }
}
