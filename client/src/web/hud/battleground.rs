//! The battlegrounds on the page: the window that lists them (the client's `UIEntryQueueWnd`, 350 by 350), its
//! help (`UIEntryQueueHelpWnd`, 280 by 280), the window of the queue one waits in (`UIEntryQueueStandByWnd`,
//! 280 by 120), the one that asks whether to enter (`UIEntryQueueRequestWnd`, 300 by 250), the question
//! before applying, and on a battle's map the score (`CRenderer::DrawBattleFieldIcon`), the mark over one of
//! another team (`UIWordDisplayWnd`) and the team on the minimap.

use super::*;
use crate::press::battleground::{Press, Way};

/// The ground of the list of battlegrounds (240, 240, 240), and of the row picked in a list of the client's.
const LIST_GROUND: &str = "#f0f0f0";
const PICKED: &str = "#c6d6f7";

/// Texts in a box of the client's `UITextViewer`, one after another, each cut into lines of 38 letters as the
/// windows here ask (`UM_SETTEXT` with 38; `crate::message_box::lines`), a line 18 high from 3 in.
fn viewer((x, y, width, height): (i32, i32, i32, i32), texts: &[String]) -> String {
    let cut = |text: &String| if text.is_empty() { vec![String::new()] } else { crate::message_box::lines(text, 38) };
    let lines: String = texts.iter().flat_map(cut).map(|line| format!(r#"<div style="height:18px;white-space:pre">{}</div>"#, escape(&line))).collect();
    format!(
        r#"<div class="a t bar" style="{};line-height:18px;overflow:hidden auto"><div style="padding:0 0 0 3px">{lines}</div></div>"#,
        at(x, y, width, height)
    )
}

impl App {
    /// The mark the client puts over one of another team on a battleground (`UIWordDisplayWnd::Paint`:
    /// `전장적표시.bmp`, 24 square), as what a sign over a head is made of.
    pub(in crate::web) fn foe_mark() -> String {
        format!(r#"<img alt="" draggable="false" src="{}" style="width:24px;height:24px;pointer-events:none">"#, picture("전장적표시.bmp"))
    }

    /// The list (`UIEntryQueueWnd::OnCreate` and `Paint`): the title (line 1539 of the message table), "Battleground
    /// List" (2149) in grey at (10, 23), the battlegrounds in a box 130 by 270 at (10, 40), rows 16 high; the
    /// picture of the one picked at (157, 25); seven labels (2139 to 2145) from (157, 165), 20 apart, each
    /// with its value 5 after it; on the bar at the bottom the three ways to apply at 8, 101 and 194 and Help
    /// 62 from the right, all 24 from the bottom.
    pub(in crate::web) fn battleground_window(&self) -> String {
        let mut window = title_bar(350, &Self::words(1539, "Battle field entrance setting"));
        window.push_str(&base_button(3, 3));
        window.push_str(&cross(350, Press::Close));
        window.push_str(&panel(350, 350));
        window.push_str(&text(10, 23, "#c0c0c0", &Self::words(2149, "Battleground List")));
        let rows: String = data::battlegrounds()
            .iter()
            .map(|entry| {
                let ground = if self.battleground.current == Some(entry.id) { PICKED } else { "transparent" };
                format!(
                    r#"<div class="t" style="height:16px;padding-left:3px;white-space:nowrap;cursor:pointer;background:{ground}" {}>{}</div>"#,
                    pressed(Press::Row(entry.id)),
                    escape(&entry.shown)
                )
            })
            .collect();
        window.push_str(&format!(r#"<div class="a bar" style="{};background:{LIST_GROUND};overflow:hidden auto">{rows}</div>"#, at(10, 40, 130, 270)));
        if let Some(entry) = self.battleground.current.and_then(data::battleground) {
            window.push_str(&format!(
                r#"<img class="a" alt="" draggable="false" src="{}" style="left:157px;top:25px">"#,
                picture(&format!("battle_field/{}", entry.picture))
            ));
            window.push_str(&Self::battleground_values(&entry));
        }
        for (place, (way, name)) in [(Way::Alone, "apply_individual"), (Way::Party, "apply_party"), (Way::Guild, "apply_guild")].into_iter().enumerate() {
            window.push_str(&picture_press_sized(8 + 93 * place as i32, 326, (85, 20), &format!("battle_field/{name}"), Press::Apply(way)));
        }
        window.push_str(&picture_press_sized(288, 326, (54, 20), "battle_field/btn_help", Press::Help));
        window
    }

    /// The seven lines about the battleground picked (`UIEntryQueueWnd::SelectQueue`): its name, its goal,
    /// how many fight on each side (line 2150), the level it takes (2151 to 2154), and what is won, drawn
    /// and lost.
    fn battleground_values(entry: &crate::battleground::Entry) -> String {
        let (line, numbers) = entry.level_line();
        let numbers: Vec<String> = numbers.iter().map(u32::to_string).collect();
        let level = crate::numbers::filled(&Self::words(line, ""), &numbers.iter().map(String::as_str).collect::<Vec<_>>());
        let sides = crate::numbers::filled(&Self::words(2150, "%d VS %d"), &[&entry.allies.to_string(), &entry.enemies.to_string()]);
        let values = [&entry.shown, &entry.goal, &sides, &level, &entry.win, &entry.draw, &entry.loss];
        let fallbacks = ["Name:", "Goal:", "Format:", "Level:", "Win:", "Draw:", "Loss:"];
        values
            .into_iter()
            .enumerate()
            .map(|(row, value)| {
                [
                    format!(r#"<div class="a t" style="left:157px;top:{}px;width:190px;white-space:nowrap;overflow:hidden">"#, 165 + 20 * row as i32),
                    format!(r#"{}<span style="margin-left:5px">{}</span></div>"#, escape(&Self::words(2139 + row, fallbacks[row])), escape(value)),
                ]
                .concat()
            })
            .collect()
    }

    /// The help (`UIEntryQueueHelpWnd::OnCreate`): the title (line 2125), the rules of applying (2116 to 2124)
    /// in a box 270 by 225 at (10, 22), Close at (234, 256).
    pub(in crate::web) fn battleground_help(&self) -> String {
        let lines: Vec<String> = (2116..=2124).map(|line| Self::words(line, "")).collect();
        [
            title_bar(280, &Self::words(2125, "Request help battle position")),
            panel(280, 280),
            viewer((10, 22, 270, 225), &lines),
            picture_press(234, 256, "btn_close", Press::HelpClose),
        ]
        .concat()
    }

    /// The window of the queue (`UIEntryQueueStandByWnd::Create` and `SetInfo`): the title (line 2135); in a
    /// box 260 by 65 at (10, 22) the battleground's name (2136), how many it needs (2137) and one's place
    /// (2138), once the server has told them; the button that leaves the queue at (159, 96) and Close at
    /// (234, 96).
    pub(in crate::web) fn battleground_standby(&self) -> String {
        let lines: Vec<String> = match &self.battleground.waiting {
            Some((name, needed, place)) => vec![
                Self::words(2136, "Battlefield name:%s").replacen("%s", name, 1),
                Self::words(2137, "Persons required:%d").replacen("%d", &needed.to_string(), 1),
                Self::words(2138, "Your position:%d").replacen("%d", &place.to_string(), 1),
            ],
            None => Vec::new(),
        };
        [
            title_bar(280, &Self::words(2135, "Standby position")),
            base_button(3, 3),
            panel(280, 120),
            viewer((10, 22, 260, 65), &lines),
            picture_press_sized(159, 96, (68, 20), "battle_field/cancel_waiting", Press::Leave),
            picture_press(234, 96, "btn_close", Press::StandbyClose),
        ]
        .concat()
    }

    /// The window that asks whether to enter (`UIEntryQueueRequestWnd::Create` and `Paint`): the title (line
    /// 2133); the battle that begins (2126 with the battleground's name), the question (2127) and the note
    /// (2128 to 2131) in a box 280 wide at (10, 22); OK at (209, 226) and Cancel at (254, 226); and at the
    /// left of the bottom how many of the twenty seconds are left (2134).
    pub(in crate::web) fn battleground_entry(&self) -> String {
        let Some(since) = self.battleground.entry else { return String::new() };
        let shown = self.battleground.current.and_then(data::battleground).map(|entry| entry.shown).unwrap_or_default();
        let mut lines = vec![Self::words(2126, "%s battle begins.").replacen("%s", &shown, 1), Self::words(2127, ""), String::new()];
        lines.extend((2128..=2131).map(|line| Self::words(line, "")));
        let left = crate::battleground::entry_seconds(crate::web::app::now().wrapping_sub(since)).max(0);
        [
            title_bar(300, &Self::words(2133, "Battlefield position request")),
            base_button(3, 3),
            panel(300, 250),
            viewer((10, 22, 280, 200), &lines),
            text(10, 230, BLACK, &Self::words(2134, "Accept standby time:%d seconds").replacen("%d", &left.to_string(), 1)),
            picture_press(209, 226, "btn_ok", Press::Enter),
            picture_press(254, 226, "btn_cancel", Press::Decline),
        ]
        .concat()
    }

    /// The question before applying (`UIEntryQueueWnd::ConfirmApply`): the battleground's name and whether
    /// one means to fight alone, as a party or as a guild (lines 2146 to 2148).
    pub(in crate::web) fn battleground_question(&self) -> String {
        let (Some(way), Some(entry)) = (self.battleground.asks, self.battleground.current.and_then(data::battleground)) else { return String::new() };
        confirm_box(&format!("{} {}", entry.shown, Self::words(crate::battleground::question(way), "")), Press::Yes, Press::No)
    }

    /// The score of a battle in the corner of the screen (`CRenderer::DrawBattleFieldIcon`): the two teams'
    /// emblems, 48 square, 280 left of the right edge at 26 and 84 down, and beside each its points in the
    /// digits of `RankFont` (`crate::battleground::score`). Nothing until the server has told a score, or
    /// until the digits are read.
    pub(in crate::web) fn score_board(&self) -> String {
        use crate::battleground::{EMBLEMS, EMBLEM_LEFT, EMBLEM_SIDE, score};
        let Some((first, second)) = self.battleground.score else { return String::new() };
        let digits = score(first, second);
        if digits.is_empty() || self.pvp.digits.is_empty() {
            return String::new();
        }
        let emblems = EMBLEMS.map(|(name, top)| {
            [
                format!(r#"<img alt="" draggable="false" src="{}" "#, crate::web::image_url(&format!("data/texture/{name}"))),
                format!(r#"style="left:calc(100% + {EMBLEM_LEFT}px);top:{top}px;width:{EMBLEM_SIDE}px;height:{EMBLEM_SIDE}px;opacity:.8">"#),
            ]
            .concat()
        });
        let digits: String = digits
            .into_iter()
            .filter_map(|(action, x, y)| {
                let (picture, _, origin) = self.pvp.digits.get(action)?;
                Some(format!(
                    r#"<img alt="" draggable="false" src="{picture}" style="left:calc(100% + {:.0}px);top:{:.0}px">"#,
                    x as f32 - origin.0,
                    y as f32 - origin.1
                ))
            })
            .collect();
        emblems.concat() + &digits
    }

    /// The others of the team on the minimap, each a mark as a party member's is (the client keeps both in
    /// one list: `MM_ADDPARTYPOS`).
    pub(in crate::web) fn minimap_team(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        self.battleground
            .places
            .values()
            .filter_map(|(x, y)| {
                let (x, y) = crate::web::view::minimap_spot(game, self.minimap_zoom, (f32::from(*x), f32::from(*y)))?;
                Some([
                    format!(r#"<span style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:4px;height:4px;margin:-3px "#),
                    r#"0 0 -3px;background:#493a37;border:1px solid rgba(255,255,255,.6)"></span>"#.to_owned()
                ].concat())
            })
            .collect()
    }
}
