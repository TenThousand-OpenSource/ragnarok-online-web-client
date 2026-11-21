//! The guild as rules: what the client decides of the castles a guild holds with no page and no packet (the
//! list of them in the guild window's first tab, `UIGuildInfoManageWnd::OnCreate`; the menu on one,
//! `UIListBoxAgit::OnRBtnDown`; the words of its two boxes, `UIListBoxAgitMenu::SendMsg` and
//! `CGameMode::ReceiveAgitNotice`). The packets are read in `net/guild/castles.rs`.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_opens_by_the_right_edge_of_the_line_pressed() {
        assert_eq!((castle_menu_place(0), castle_menu_place(3)), ((92, 2), (92, 41)));
        // Twelve lines are in sight.
        assert_eq!(CASTLE_LIST.1 / CASTLE_LINE, 12);
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
}
