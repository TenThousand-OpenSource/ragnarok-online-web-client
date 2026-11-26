//! The windows beside the navigation's own: its help (the client's `UINavigationHelpWnd`), the sets of
//! arrows that mark a way (`UINavigationroadiconWnd`), the list of the route (`UINavigationRuideWnd`), and
//! the question whether to give a navigation up.

use super::*;

/// The lines of the help, by their numbers in the client's message table; nought is an empty line
/// (`UINavigationHelpWnd::OnCreate`).
const HELP: [usize; 31] = [
    2241, 0, 2242, 2243, 2244, 0, 2245, 2246, 2247, 0, 2248, 0, 2249, 0, 2250, 0, 2251, 0, 2252, 0, 2253, 2254, 0, 2255, 2256, 0, 3216, 0, 3217,
    3218, 0,
];

impl App {
    /// The help (310 by 200): the title (line 2240) with the cross, and the lines on grey (210, 210, 210) in
    /// a space 300 by 173 at (5, 22) that the wheel moves.
    pub(in crate::web) fn navigation_help_window(&self) -> String {
        let lines: String = HELP
            .iter()
            .map(|line| if *line == 0 { "&nbsp;".to_owned() } else { escape(&data::message(*line).unwrap_or_default()) })
            .map(|line| format!(r#"<div style="min-height:16px;line-height:16px">{line}</div>"#))
            .collect();
        [
            title_bar(310, &Self::words(2240, "How to Use Navigation")),
            cross(310, Press::Help),
            panel_ground_round(0, 17, 310, 183),
            format!(
                r#"<div class="a t scroll" style="{};overflow-y:auto;background:rgb(210,210,210);white-space:normal;color:{BLACK};padding:2px 3px;box-sizing:border-box" {}>{lines}</div>"#,
                at(5, 22, 300, 173),
                lines_stepped(16)
            ),
        ].concat()
    }

    /// The sets of arrows (158 by 102): the title (line 3203) with the cross, and under it the picture of the
    /// eight sets, four to a row, the one in use framed (`img_roadIocn_bg_select.bmp`, 32 square, 36 apart
    /// from 10 across; the rows at 24 and 60). A press on a set takes it (`OnLBtnUp`: each 30 square from
    /// (11, 25) and from (11, 63)).
    pub(in crate::web) fn navigation_icons_window(&self) -> String {
        let icon = i32::from(self.navigation.guide.icon.clamp(1, 8));
        let (column, top) = (if icon > 4 { icon - 4 } else { icon }, if icon < 5 { 24 } else { 60 });
        let sets: String = (1..=8)
            .map(|set: i32| {
                let (column, top) = ((set - 1) % 4, if set < 5 { 25 } else { 63 });
                format!(r#"<div class="a" style="{};cursor:pointer" {}></div>"#, at(11 + column * 36, top, 30, 30), pressed(Press::Icon(set as u8)))
            })
            .collect();
        [
            title_bar(158, &Self::words(3203, "Navigation guide icon set")),
            cross(158, Press::Icons),
            art(&format!("{ART}bg_roadiocn_select.bmp"), 0, 17, 158, 85),
            art(&format!("{ART}img_roadiocn_bg_select.bmp"), column * 36 - 26, top, 32, 32),
            sets,
        ].concat()
    }

    /// The list of the route (150 by 338, `RefreshRoute`): the title (line 3204) with the cross, and a list
    /// 146 by 293 at (2, 20): every way out of the route with its map and its cell, the first marked as the
    /// start, and last where the navigation ends.
    pub(in crate::web) fn navigation_route_window(&self) -> String {
        let Some(atlas) = data::atlas() else { return String::new() };
        let guide = &self.navigation.guide;
        let mut rows: String = guide
            .nodes
            .iter()
            .map(|link| &atlas.links[*link])
            .enumerate()
            .map(|(place, link)| {
                let words = format!("{} ({} {})", atlas.maps[link.map].name, link.x, link.y);
                list_row(if place == 0 { "start" } else { "riute" }, &words, false, Press::Route)
            })
            .collect();
        if let Some((map, cell)) = guide.goal {
            let name = &atlas.maps[map].name;
            let words = if cell.0 > 0 { format!("{name} ({} {})", cell.0, cell.1) } else { name.clone() };
            rows.push_str(&list_row("target", &words, false, Press::Route));
        }
        [
            fill(0, 0, 150, 338, "#ccc"),
            title_bar(150, &Self::words(3204, "navigation UI")),
            cross(150, Press::Route),
            panel(150, 338),
            list((2, 20, 146, 293), WHITE, &rows),
        ].concat()
    }

    /// "Do you want to cancel navigation?" (line 2239), in the client's box.
    pub(in crate::web) fn navigation_question(&self) -> String {
        confirm_box(&Self::words(2239, "Do you want to cancel navigation?"), Press::Yes, Press::No)
    }
}
