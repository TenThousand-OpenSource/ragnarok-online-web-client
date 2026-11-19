//! The pane of one quest, beside the journal: of a quest of the server its description, what it hunts and what
//! it gives (the client's `CUIOngoingQuestInfo`); of one of the things the client recommends its three texts
//! over its picture (`CUIRecommendedQuestInfo`). Both are 342 by 412.

use super::*;

/// The pane's title, its texts, what a reward is worth, and the day a quest runs out (blue first in the
/// client: 0x55321D, 0x38302E, 0xA0551D, 0x161BD2).
const TITLE: &str = "#1d3255";
const WORTH: &str = "#1d55a0";
const DEADLINE: &str = "#d21b16";

/// Lines of text one under another as the client's `CUITextViewer` lays them, six pixels apart, each folded
/// at the viewer's width; colours and links written into a line are shown as `coloured` and
/// `crate::quests::shown` have them.
fn viewer_lines(lines: &[String]) -> String {
    viewer_lines_in("", lines)
}

/// The same in a font of the viewer's own (`style`, from `crate::font::style`).
fn viewer_lines_in(style: &str, lines: &[String]) -> String {
    lines.iter().map(|line| format!(r#"<div class="t" style="white-space:normal;margin-bottom:6px;{style}">{}</div>"#, coloured(&rules::shown(line)))).collect()
}

/// A plate under a text, laid half see-through so that the picture behind it shows through.
fn plate(name: &str, x: i32, y: i32, width: i32, height: i32) -> String {
    format!(r#"<div class="a" style="{};opacity:.5;background:url('{}') 0 0 / 100% 100% no-repeat"></div>"#, at(x, y, width, height), picture(name))
}

/// A part of the pane of a quest of the server (`OngoingQuestInfo::CUIContents`, 335 wide): the small Poring
/// 9 in and 9 down, the part's name 25 in, and from 32 in and 30 down what is in it; the part ends 9 under that.
fn part(name: &str, inside: &str) -> String {
    [
        format!(r#"<div style="position:relative;width:335px;min-height:30px;padding-bottom:9px;box-sizing:border-box">"#),
        format!(r#"{}{}<div style="padding:30px 13px 0 32px">{inside}</div></div>"#, art("renew_questui/img_poring.bmp", 9, 9, 10, 9), text(25, 9, BLACK, name))
    ].concat()
}

/// The line between two parts (`img_line.bmp`, 334 by 1, 1 in).
fn rule() -> String {
    format!(r#"<div style="margin-left:1px;width:334px;height:1px;background:url('{}')"></div>"#, picture("renew_questui/img_line.bmp"))
}

/// When a quest runs out, as the pane writes it under its parts (`RenewQuestOngoingInfo::GetDateString`):
/// the client's word for it (line 2195, "Deadline") and the day and time, by the player's own clock; nothing
/// for a quest that does not run out (the client writes it for a year after 1970).
fn deadline(at: u32) -> String {
    let when = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(f64::from(at) * 1000.0));
    crate::quests::deadline(
        &App::words(2195, "Deadline"),
        (when.get_full_year(), when.get_month() + 1, when.get_date()),
        (when.get_hours(), when.get_minutes())
    )
}

impl App {
    /// What both panes share (`OnCreate` of either): the title bar, the ground under it, the quest's title in
    /// bold in the middle 39 down, and Close.
    fn pane_frame(ground: &str, title: &str) -> String {
        format!(
            r#"{}{}{}<div class="a t" style="{};text-align:center;color:{TITLE};{};overflow:hidden">{}</div>{}"#,
            title_bar(PANE.0, &Self::words(3400, "Quest information")),
            base_button(3, 3),
            art(&format!("renew_questui/{ground}.bmp"), 0, 17, 342, 395),
            at(0, 39, 342, 15),
            BOLD.style(),
            escape(title),
            picture_press(288, 388, "basic_interface/btn_close", Press::PaneClose)
        )
    }

    /// The pane of the quest picked in the journal.
    pub(in crate::web) fn quest_pane(&self) -> String {
        match self.quests.pane {
            Some((number, true)) => Self::recommended_pane(number),
            Some((quest, false)) => self.ongoing_pane(quest),
            None => String::new(),
        }
    }

    /// The pane of a quest of the server. Under the title a list 342 by 322 from 62 down, of three parts with
    /// a line between them: the description (line 3401), the monsters with their counts (3402) and the reward
    /// (3403): the experience (3404, 3405) and the items (3406), each item in its box 38 across with its name
    /// and how many (`CUIRewardItem`; rows 41 apart). Under the list the day it runs out.
    /// (Not made: the picture of the NPC to go to, which the client stands at the right of the reward and
    /// whose press starts the navigation there (`UINpcSprWnd`); and the list's own scroll bar: the browser's
    /// thin one stands in.)
    fn ongoing_pane(&self, number: u32) -> String {
        let Some(quest) = self.quests.log.get(&number) else { return String::new() };
        let about = data::quest_text(number).unwrap_or_default();
        let complete = Self::words(2030, "Complete");
        let hunts: Vec<String> = quest.hunts.iter().map(|hunt| rules::hunt_text(hunt, &hunt.name, &complete)).collect();
        let worth = |y: i32, line: usize, otherwise: &str, value: &str| format!(
            "{}{}",
            text(32, y, BLACK, &Self::words(line, otherwise)),
            text(110, y, WORTH, value)
        );
        let items: String = about
            .rewards
            .iter()
            .enumerate()
            .map(|(place, (item, count))| {
                let top = 72 + place as i32 * 41;
                format!(
                    r#"{}<div class="a" style="{}">{}</div>{}{}"#,
                    art("renew_questui/img_questiocn.bmp", 110, top, 38, 38),
                    at(117, top + 7, 24, 24),
                    Self::item_icon(*item, true),
                    text(156, top + 5, BLACK, &data::item(*item).map(|(name, _)| name).unwrap_or_default()),
                    text(156, top + 20, WORTH, &Self::words(182, "%d ea.").replacen("%d", &count.to_string(), 1))
                )
            })
            .collect();
        // The reward's part is as tall as what is in it: the three labels, and the items under the third.
        let reward = format!(
            r#"<div style="position:relative;width:335px;height:{}px">{}{}{}{}{}{items}</div>"#,
            (93 + 9).max(72 + about.rewards.len() as i32 * 41 + 9),
            art("renew_questui/img_poring.bmp", 9, 9, 10, 9),
            text(25, 9, BLACK, &Self::words(3403, "Reward")),
            worth(30, 3404, "Base Exp", &about.base),
            worth(51, 3405, "Job Exp", &about.job),
            text(32, 72, BLACK, &Self::words(3406, "Item")),
        );
        format!(
            r#"{}<div class="a viewer" style="{};color:#2e3038">{}{}{}{}{reward}</div>{}"#,
            Self::pane_frame("bg_detail_quest", &self.quest_title(number)),
            at(0, 62, 342, 322),
            part(&Self::words(3401, "Description"), &viewer_lines(&about.lines)),
            rule(),
            part(&Self::words(3402, "Monster"), &viewer_lines(&hunts)),
            rule(),
            text_in(USUAL, 13, 393, DEADLINE, &deadline(quest.deadline))
        )
    }

    /// The pane of one of the things the client recommends: over its own picture (336 by 318, 3 in and 64
    /// down) its three texts, each in a box of its own that scrolls by itself (`CUIQuestInfo`, a
    /// `CUITextViewer` with a bar 7 wide): the first 306 by 120, the second 213 by 60 in bold with a white
    /// outline (0x2A2A2A, 6 down in its box), the third 182 by 76. Their font is named and 12 high (the client
    /// names Gulim; the page's face is used), which is smaller than the usual text: at the usual size the
    /// texts do not fit their boxes.
    /// Under the first text and under the third the client lays a plate (`topImage` 324 by 138, 9 in and 8
    /// down; `bottomImage` 208 by 92, 9 in and 221 down), so that they can be read over the picture. It names
    /// them `bg_quest01.png` and `bg_quest02.png`; the game's files have them as `.bmp` only (a skin has the
    /// `.png`), and those are laid here — whether the client itself falls back to them was not found. The
    /// `.png` the client names is half see-through (its pixels have an alpha of 128, as the one a skin brings
    /// shows), so that the picture behind is still seen; the `.bmp` is laid at that half here.
    /// (Not made: the picture of the NPC to go to; and what the signs written
    /// into some of these texts mean (`$$…$$`, `%%…%%`, `&*…*&`): they are shown as they are written.)
    fn recommended_pane(number: u32) -> String {
        let Some((_, about)) = data::recommended_quests().into_iter().find(|(known, _)| *known == number) else { return String::new() };
        let boxed = |(x, y, width, height): (i32, i32, i32, i32), style: &str, lines: &[String]| format!(
            r#"<div class="a viewer" style="{};{style}">{}</div>"#,
            at(x, 62 + y, width, height),
            viewer_lines_in(&crate::font::style(1, 12), lines)
        );
        format!(
            "{}{}{}{}{}{}{}",
            Self::pane_frame("bg_detail_recommended", &about.title),
            if about.picture.is_empty() { String::new() } else { art(&format!("renew_questui/{}", about.picture), 3, 64, 336, 318) },
            plate("renew_questui/bg_quest01.bmp", 9, 62 + 8, 324, 138),
            plate("renew_questui/bg_quest02.bmp", 9, 62 + 221, 208, 92),
            boxed((18, 17, 306, 120), "color:#000", &about.info[0]),
            boxed((16, 153, 213, 60), &format!("color:#2a2a2a;font-weight:bold;padding-top:6px;box-sizing:border-box;{}", outlined(WHITE)), &about.info[1]),
            boxed((22, 229, 182, 76), "color:#000", &about.info[2])
        )
    }
}
