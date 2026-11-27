//! The guild as rules: what the client decides of the castles a guild holds with no page and no packet (the
//! list of them in the guild window's first tab, `UIGuildInfoManageWnd::OnCreate`; the menu on one,
//! `UIListBoxAgit::OnRBtnDown`; the words of its two boxes, `UIListBoxAgitMenu::SendMsg` and
//! `CGameMode::ReceiveAgitNotice`), and of the guilds allied and opposed in that tab (which row of the two lists
//! the right button picks, `UIGuildInfoManageWnd::OnRBtnUp`), and which of a master's rows the menu on another
//! player has (`relation_rows`). The packets are read in `net/guild/castles.rs`.

/// The list of castles (`m_agitList`): 95 by 158, a line 13 high, twelve lines in sight.
pub const CASTLE_LIST: (i32, i32) = (95, 158);
pub const CASTLE_LINE: i32 = 13;
/// The menu on a castle (`m_agitMenu`): 120 by 35, its two rows 16 high as every list box's are.
pub const CASTLE_MENU: (i32, i32) = (120, 35);
/// A castle's economy and its defence are told out of this many (the 100 `ReceiveAgitNotice` writes twice).
pub const INVESTED_MOST: i32 = 100;

/// Where the menu opens for the line of the list that the right button was pressed on, counted from the
/// list's own corner: 3 short of the list's right edge, 2 under the top of the line (`UIListBoxAgit::OnRBtnDown`:
/// `width + x - 3`, `y - y % lineHeight + 2` for a press `y` down the list). `line` is counted from the first
/// line in sight.
pub fn castle_menu_place(line: usize) -> (i32, i32) {
    (CASTLE_LIST.0 - 3, line as i32 * CASTLE_LINE + 2)
}

/// The guilds allied and the guilds opposed are two lists of three rows each, 16 to a row
/// (`UIGuildInfoManageWnd::Paint`: the rows from 157 and from 231 down, none drawn from 205 and 279 on).
pub const RELATED_ROWS: usize = 3;
pub const RELATED_ROW: i32 = 16;
/// Where the right button picks a row of either list, across the window: from 201 to 287, both taken
/// (`OnRBtnUp`: `x < 201 || x > 287` is no pick), which is 3 before and 4 after the 80 the pick is marked on
/// (`Paint`: `FillRegion(204, y, 80, 16)`). The rest of the grey the lists lie on, 168 wide, picks nothing.
pub const RELATED_PICK: (i32, i32) = (201, 87);
pub const RELATED_MARK: (i32, i32) = (204, 80);

/// A row of the two lists as one number, for what is written on the page: the three of the allied, then the
/// three of the opposed (the client keeps a row for each list, `m_allyRow` and `m_hostileRow`, and never both).
pub fn related_slot(opposed: bool, row: usize) -> usize {
    if opposed { RELATED_ROWS + row } else { row }
}

/// The list and the row a number of `related_slot` is; none for a number that is no row.
pub fn related_row(slot: usize) -> Option<(bool, usize)> {
    (slot < 2 * RELATED_ROWS).then_some((slot >= RELATED_ROWS, slot % RELATED_ROWS))
}

/// The place, among the related guilds as the server told them (each given as whether it is opposed), of the
/// guild a row of a list shows: the lists are filled in the server's order, each with its own kind
/// (`UIGuildInfoManageWnd::Paint`, and `SendMsg`'s `UM_MENU_SELECTED`, which counts the same way). None for
/// a row no guild is in: the right button on it marks nothing and opens no menu (`OnRBtnUp`: the menu is made
/// only when the list holds more guilds than the row's number).
pub fn related_place(related: impl IntoIterator<Item = bool>, opposed: bool, row: usize) -> Option<usize> {
    let mut places = related.into_iter().enumerate().filter(|(_, against)| *against == opposed).map(|(place, _)| place);
    if row < RELATED_ROWS { places.nth(row) } else { None }
}

/// Which of a master's two rows the menu on another player has: asking that player's guild for an alliance,
/// and setting it as an antagonist. The client's rule (`CGameMode::OpenActorMenu`,
/// `Source/Contents/World/WorldObjects.cpp:814-828`) offers both to a master on anyone who is not of the
/// master's guild, and looks at nothing else. Decided here against it (the owner's decision, 2026-11-21,
/// `docs/DEBTS.md`): a relation that stands already is not offered (no alliance with a guild already allied,
/// no opposing of one already opposed), and neither row on a player of no guild, where there is nothing to
/// relate to. `own` and `theirs` are the two guilds (0 for none), `related` the guilds the master's own is
/// related to, each with whether it is opposed.
pub fn relation_rows(own: u32, master: bool, theirs: u32, related: impl IntoIterator<Item = (u32, bool)>) -> (bool, bool) {
    if own == 0 || !master || theirs == 0 || theirs == own {
        return (false, false);
    }
    let (mut allied, mut opposed) = (false, false);
    for (_, against) in related.into_iter().filter(|(guild, _)| *guild == theirs) {
        if against { opposed = true } else { allied = true }
    }
    (!allied, !opposed)
}

/// A line of the message table as it is written into a box: what the table writes as `\n` ends a line.
fn lined(line: &str) -> String {
    line.replace("\\n", "\n")
}

/// What the box asks before the character is moved to a castle: line 3631 of the message table with the
/// castle's name, the zeny it costs and the zeny it costs during a siege, in that order.
pub fn castle_question(line: &str, name: &str, cost: u32, siege_cost: u32) -> String {
    lined(&crate::numbers::filled(line, &[name, &cost.to_string(), &siege_cost.to_string()]))
}

/// What the box says of a castle's economy and defence: line 3632 of the message table with the castle's
/// name, then the defence and the economy, each out of 100 — the defence first, though the packet has the
/// economy first (`ReceiveAgitNotice(castle, economy, defense)` writes `name, defense, 100, economy, 100`).
pub fn castle_investment(line: &str, name: &str, economy: i32, defense: i32) -> String {
    let most = INVESTED_MOST.to_string();
    lined(&crate::numbers::filled(line, &[name, &defense.to_string(), &most, &economy.to_string(), &most]))
}

/// The part of the guild's experience a member gave, in hundredths, as the members' tab writes it
/// (`GuildInfo::UpdateMemberExpRatios`, `Source/Contents/Guild/GuildInfo.cpp`): what the member gave against
/// what all the members listed gave together, not against the guild's own experience; nothing while none gave.
pub fn exp_ratio(given: u32, all: impl IntoIterator<Item = u32>) -> f32 {
    let total: f64 = all.into_iter().map(f64::from).sum();
    if total == 0.0 { 0.0 } else { given as f32 * 100.0 / total as f32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_master_is_offered_only_a_relation_that_does_not_stand() {
        let related = [(7, false), (8, true), (9, false), (9, true)];
        // A guild not related: both; one allied: only opposing; one opposed: only the alliance; both: neither.
        assert_eq!(relation_rows(1, true, 5, related), (true, true));
        assert_eq!(relation_rows(1, true, 7, related), (false, true));
        assert_eq!(relation_rows(1, true, 8, related), (true, false));
        assert_eq!(relation_rows(1, true, 9, related), (false, false));
        // Nothing on a player of no guild or of the master's own, for one who is no master, or with no guild.
        assert_eq!((relation_rows(1, true, 0, related), relation_rows(1, true, 1, related)), ((false, false), (false, false)));
        assert_eq!((relation_rows(1, false, 5, related), relation_rows(0, true, 5, related)), ((false, false), (false, false)));
    }

    #[test]
    fn the_menu_opens_by_the_right_edge_of_the_line_pressed() {
        assert_eq!((castle_menu_place(0), castle_menu_place(3)), ((92, 2), (92, 41)));
        // Twelve lines are in sight.
        assert_eq!(CASTLE_LIST.1 / CASTLE_LINE, 12);
    }

    #[test]
    fn a_row_of_the_related_lists_is_found_by_its_list_and_its_number() {
        // Allied, opposed, allied, allied, allied (a fourth, which no row shows), opposed.
        let related = [false, true, false, false, false, true];
        assert_eq!(related_place(related, false, 0), Some(0));
        assert_eq!(related_place(related, false, 1), Some(2));
        assert_eq!(related_place(related, false, 2), Some(3));
        assert_eq!(related_place(related, false, 3), None);
        assert_eq!(related_place(related, true, 0), Some(1));
        assert_eq!(related_place(related, true, 1), Some(5));
        // A row no guild is in picks nothing.
        assert_eq!(related_place(related, true, 2), None);
        assert_eq!(related_place([], false, 0), None);
    }

    #[test]
    fn the_rows_of_both_lists_are_numbered_through() {
        for (opposed, row) in [(false, 0), (false, 2), (true, 0), (true, 2)] {
            assert_eq!(related_row(related_slot(opposed, row)), Some((opposed, row)));
        }
        assert_eq!((related_slot(false, 2), related_slot(true, 0)), (2, 3));
        assert_eq!(related_row(6), None);
        // The pick is taken a little wider than it is marked, and far narrower than the grey of the list.
        assert_eq!((RELATED_PICK.0 + RELATED_PICK.1 - 1, RELATED_MARK.0 + RELATED_MARK.1), (287, 284));
    }

    #[test]
    fn the_question_names_the_castle_and_both_costs() {
        let line = "Would you like to move to %s Management Territory?\\n%d Zeny will be consumed with every move.\\n(%d Zeny will be \
            consumed during WoE times.)";
        assert_eq!(
            castle_question(line, "Kriemhild", 100, 100_000),
            "Would you like to move to Kriemhild Management Territory?\n100 Zeny will be consumed with every move.\n(100000 Zeny will be \
            consumed during WoE times.)"
        );
    }

    #[test]
    fn the_defence_is_written_before_the_economy() {
        let line = "\\nManagement Territory: %s\\n\t\\nDefense: %d / %d\\nCommercial: %d / %d";
        assert_eq!(
            castle_investment(line, "Bright Arbor", 35, 7),
            "\nManagement Territory: Bright Arbor\n\t\nDefense: 7 / 100\nCommercial: 35 / 100"
        );
    }

    /// `GuildInfo::UpdateMemberExpRatios`: against the sum of the members', and none of nothing.
    #[test]
    fn a_member_gave_its_part_of_what_all_members_gave() {
        assert_eq!(exp_ratio(30, [30, 10, 0]), 75.0);
        assert_eq!(exp_ratio(0, [30, 10, 0]), 0.0);
        assert_eq!(exp_ratio(0, [0, 0]), 0.0);
        assert_eq!(format!("{:3.0}%", exp_ratio(1, [1, 2])), " 33%");
    }
}
