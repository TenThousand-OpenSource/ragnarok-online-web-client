//! The list of the achievements of a tab (the client's `UIAchListWnd`) and the pane of the one picked
//! (`UIAchViewWnd`).

use super::*;
use crate::achievements::{Info, Tone};

/// A row of the list is 81 high: 405 for five (`UIAchListWnd::Create`).
const ROW: i32 = 81;

impl App {
    /// The achievements the list shows now, in its order (`UIAchievementWnd::SelectTab`).
    pub(in crate::web) fn achievements_listed(&self) -> Vec<u32> {
        let all = data::achievements();
        let state = &self.achievements;
        rules::listed(&rules::of_tab(&all, state.group, state.sub), &state.records, state.shown)
    }

    /// The list (`UIAchListWnd::Paint`, at 102, 63, 320 by 405 with its bar of 11 at the right): five rows of
    /// 81 show at once, and the wheel moves it three rows.
    pub(super) fn achievements_list(&self) -> String {
        let all = data::achievements();
        let rows: String = self
            .achievements_listed()
            .into_iter()
            .filter_map(|id| all.iter().find(|info| info.id == id))
            .enumerate()
            .map(|(row, info)| self.achievement_row(info, row as i32))
            .collect();
        [
            format!(r#"<div class="a" style="{}">{}"#, at(102, 63, 320, 405), art(&piece("bg_list"), 0, 0, 309, 405)),
            format!(
                r#"<div class="a viewer achbar" style="{}" {}><div style="position:relative;height:{}px">{rows}</div></div></div>"#,
                at(0, 0, 320, 405),
                rows_stepped(ROW, 5),
                ROW * self.achievements_listed().len() as i32
            )
        ].concat()
    }

    /// A row of the list (`UIAchListWnd::Paint`): its ground by whether it is picked and completed; the
    /// picture of its kind 8 in and 3 down, with a pin on it while it is kept in sight (the picture is the
    /// button that keeps it so: `UIAchListWnd::Create`, 30 square at 5, 5); its name at 53, 14 in bold; its
    /// points in the middle of 258 to 298, on the mark of one completed; what it says of itself at 10, 54;
    /// the box of a reward not taken yet at 263, 49; and the day it was completed, small and rimmed in white,
    /// its middle at 276.
    fn achievement_row(&self, info: &Info, row: i32) -> String {
        let state = &self.achievements;
        let record = self.achievement_record(info.id);
        let picked = state.picked == Some(info.id);
        let ground = match (record.completed, picked) {
            (true, true) => "list_complete_press",
            (true, false) => "list_complete_out",
            (false, true) => "list_press",
            (false, false) => "list_out",
        };
        let mut line = art(&piece(ground), 0, 0, 309, ROW);
        line.push_str(&art(&piece(&format!("icon_{}", info.icon.to_lowercase())), 8, 3, 38, 32));
        if state.traced.iter().any(|traced| traced.id == info.id) {
            line.push_str(&art(&piece("list_pin"), 8, 3, 12, 11));
        }
        line.push_str(&format!(r#"<div class="a" style="{}" {}></div>"#, at(5, 5, 30, 30), pressed(Press::Trace(info.id))));
        line.push_str(&text_in(BOLD, 53, 14, HEADING, &info.title));
        if record.completed {
            line.push_str(&art(&piece("badge_complete"), 250, 5, 20, 30));
        }
        line.push_str(&centred(BOLD, 258, 40, 14, HEADING, &format!("{:3}", info.points)));
        line.push_str(&text(10, 54, HEADING, &info.summary));
        if info.has_reward() && !record.rewarded {
            let name = if record.completed { "list_rewardbox_not_receive" } else { "list_rewardbox_default" };
            line.push_str(&art(&piece(name), 263, 49, 30, 23));
        }
        if record.completed {
            line.push_str(&format!(
                r#"<div class="a t" style="left:228px;top:55px;width:96px;text-align:center;color:{NUMBER};{};{}">{}</div>"#,
                Font(0, 10, false).style(),
                outlined(WHITE),
                Self::achievement_date(record.completed_at)
            ));
        }
        format!(
            r#"<div class="a" style="{};overflow:hidden"{} {}>{line}</div>"#,
            at(0, row * ROW, 309, ROW),
            if picked { " data-picked" } else { "" },
            pressed(Press::Row(info.id))
        )
    }

    /// The pane of the achievement picked (`UIAchViewWnd::Paint` and `UpdateInfo`, at 422, 63, 328 by 405):
    /// its name at 15, 14 in bold, the stamp of one completed at 264, 3, what is told of it from 12, 34 in
    /// lines of 50 letters (pale blue rimmed in white), its goals, what it gives, and the button that takes
    /// the reward.
    pub(super) fn achievement_pane(&self) -> String {
        let all = data::achievements();
        let mut pane = art(&piece("bg_detail"), 0, 0, 328, 405);
        if let Some(info) = self.achievements.picked.and_then(|id| all.iter().find(|info| info.id == id)) {
            let record = self.achievement_record(info.id);
            if record.completed {
                pane.push_str(&art(&piece("detail_stamp"), 264, 3, 64, 54));
            }
            pane.push_str(&text_in(BOLD, 15, 14, HEADING, &info.title));
            pane.push_str(&format!(
                r#"<div class="a t" style="{};overflow:hidden;line-height:18px;white-space:pre;color:{PALE};{}">{}</div>"#,
                at(15, 36, 297, 56),
                outlined(WHITE),
                escape(&crate::broadcast::wrapped(&info.description, 50).join("\n"))
            ));
            pane.push_str(&Self::achievement_goals(info, &record.goals));
            pane.push_str(&Self::achievement_rewards(info));
            if info.has_reward() && !record.rewarded {
                pane.push_str(&Self::achievement_reward_button(record.completed));
            }
        }
        format!(r#"<div class="a" style="{}">{pane}</div>"#, at(422, 63, 328, 405))
    }

    /// The goals of an achievement in its pane (`UIAchViewWnd::UpdateInfo`): each a dot at 20 and its words
    /// from 28, cut into lines of 56 letters; one reached is underlined; a goal that counts says how far it
    /// is and has its gauge, 254 wide, under it; one that leads to another achievement opens the window on
    /// that one when pressed (`UIAchTV::OnLBtnDown`).
    fn achievement_goals(info: &Info, progress: &[u32]) -> String {
        let mut goals = String::new();
        for place in 0..info.goals.len().min(crate::net::achievements::GOALS) {
            let Some(goal) = rules::goal_line(info, place, progress.get(place).copied().unwrap_or(0)) else { continue };
            let (line, dot, bar) = rules::goal_places(info.counted, place);
            // (0xC9977D, 0x995736 and 0x4189E6.)
            let (colour, reached) = match goal.tone {
                Tone::Reached => (PALE, true),
                Tone::Waiting => ("#365799", false),
                Tone::Counting => ("#e68941", false),
            };
            let link = info.goals[place].link;
            goals.push_str(&art(&piece("detail_bullet"), 20, dot, 5, 5));
            goals.push_str(&[
                format!(r#"<div class="a t" style="left:31px;top:{}px;max-width:277px;max-height:36px;overflow:hidden;line-height:18px;"#, line + 2),
                format!(
                    r#"white-space:pre;color:{colour}{}" {}>{}</div>"#,
                    if reached { ";text-decoration:underline" } else { "" },
                    if link > 0 { pressed(Press::Show(link)) } else { String::new() },
                    escape(&crate::broadcast::wrapped(&goal.words, 56).join("\n"))
                )
            ].concat());
            if let Some((value, most)) = goal.gauge {
                goals.push_str(&gauge((28, bar, 254, 10), value, most, INNER, ""));
            }
        }
        goals
    }

    /// What an achievement gives, in a row 304 down the pane and about its middle (`UIAchViewWnd::Paint`): a
    /// title, a state, an item, each on its own picture 62 square and 70 apart; the title and the state are
    /// told when pointed at, the item is drawn 20, 15 into its picture and described at the right button.
    fn achievement_rewards(info: &Info) -> String {
        let mut boxes: Vec<String> = Vec::new();
        if info.title_id > 0 {
            boxes.push(format!(r#"data-tip="{}">"#, escape(&data::title_name(info.title_id))));
        }
        if info.buff > 0 {
            let says = data::status_text(info.buff).and_then(|text| text.lines.first().map(|(line, _)| line.clone())).unwrap_or_default();
            boxes.push(format!(r#"data-tip="{}">"#, escape(&says)));
        }
        if info.item > 0 {
            boxes.push(format!(
                r#"><div class="a cell" style="{}" {} title="{}">{}</div>"#,
                at(20, 15, 24, 24),
                item_marks(info.item, true),
                escape(&called(info.item)),
                Self::item_icon(info.item, true)
            ));
        }
        let names = [(info.title_id, "reward_title"), (info.buff, "reward_buff"), (info.item, "reward_item")];
        let count = boxes.len() as i32;
        let left = ((328 - count * 8 - count * 62 + 8) as f64 * 0.5) as i32;
        names
            .into_iter()
            .filter(|(has, _)| *has > 0)
            .zip(boxes)
            .enumerate()
            .map(|(place, ((_, name), inside))| {
                format!(
                    r#"<div class="a" style="{};background:url('{}') no-repeat" {inside}</div>"#,
                    at(left + 70 * place as i32, 304, 62, 62),
                    picture(&piece(name))
                )
            })
            .collect()
    }

    /// The button that takes the reward (`UIAchViewWnd::OnCreate`: `btn_receive_*`, 148 by 20 at 90, 372):
    /// there while the achievement has a reward not taken, and pressed only once it is completed.
    fn achievement_reward_button(completed: bool) -> String {
        if !completed {
            return art(&piece("btn_receive_disable"), 90, 372, 148, 20);
        }
        let pictures = ["btn_receive_out", "btn_receive_over", "btn_receive_press"].map(piece);
        pictured(90, 372, (148, 20), [pictures[0].as_str(), pictures[1].as_str(), pictures[2].as_str()], Press::Reward, "")
    }
}
