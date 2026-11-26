//! The achievements' windows: the window itself (the client's `UIAchievementWnd`, 750 by 468) with its upper
//! strip, its tabs and its summary; the achievements kept in sight (`UIAchTracingWnd`); the line across the
//! screen that tells of one completed; and the page of titles of the Equipment window. The list of a tab and
//! the pane of one achievement are in `achievements/list.rs`.
//!
//! The client writes its colours blue first: each is written here as the page takes it, with the client's
//! own number beside it.

use super::*;
use crate::achievements::{self as rules, SUB_HEIGHT, SUBS_SHOWN, TAB_HEIGHT, TABS, Shown};
use crate::press::achievements::{Press, Tab};

mod list;

/// The window's pictures (`유저인터페이스\achievement_re\`).
fn piece(name: &str) -> String {
    format!("achievement_re/{name}.bmp")
}

/// The dark blue of the window's headings (the client's 0xA55239) and the paler one of its small numbers
/// (0x635AA5), the blue of the texts of the pane (0xC9977D) and of the tabs (0x9C6B5A).
const HEADING: &str = "#3952a5";
const NUMBER: &str = "#a55a63";
const PALE: &str = "#7d97c9";
const TAB: &str = "#5a6b9c";

/// The colours of a gauge (`UICustomGageBar`): its fill, its ground, and its two rims, the outer first.
#[derive(Clone, Copy)]
struct Gauge(&'static str, &'static str, &'static str, &'static str);

/// The gauges of the upper strip (the class's own colours: 0xCE7B52, 0xFFFFFF, 0xEFE7E7, 0xADADAD) and those
/// of the summary and of the pane (0xC78A71, 0xFFFFFF, 0xFFFFFF, 0xDAA896).
const UPPER: Gauge = Gauge("#527bce", "#fff", "#e7e7ef", "#adadad");
const INNER: Gauge = Gauge("#718ac7", "#fff", "#fff", "#96a8da");

/// A gauge at a place (`UICustomGageBar::Paint`): two rims, the ground, and the fill from 2 in, by whole
/// hundredths of the gauge's width; nothing at all for a value past its most. Pointed at, it tells its
/// numbers after its words (`OnMouseHover`).
fn gauge((x, y, width, height): (i32, i32, i32, i32), value: u32, most: u32, colours: Gauge, words: &str) -> String {
    let Some(filled) = rules::filled(value, most, width) else { return String::new() };
    let Gauge(fill, ground, outer, inner) = colours;
    [
        format!(
            r#"<div class="a" style="{};box-sizing:border-box;border:1px solid {outer};background:{ground}" data-tip="{}">"#,
            at(x, y, width, height),
            escape(&format!("{words}( {value} / {most} )"))
        ),
        format!(r#"<div style="box-sizing:border-box;width:100%;height:100%;border:1px solid {inner};overflow:hidden">"#),
        format!(r#"<div style="width:{}px;height:100%;background:{fill}"></div></div></div>"#, filled.min(width - 2))
    ].concat()
}

/// A text with its middle in the middle of a space `width` wide that begins at `x` (`DrawCenteredText`).
fn centred(font: Font, x: i32, width: i32, y: i32, colour: &str, words: &str) -> String {
    text_centred(font, x + width / 2, y, colour, words)
}

impl App {
    /// What the server told of an achievement; of one it told nothing of, that it is not begun.
    pub(in crate::web) fn achievement_record(&self, id: u32) -> crate::net::achievements::Record {
        self.achievements.records.get(&id).copied().unwrap_or_default()
    }

    /// The day an achievement was completed, as the list writes it (`SessionRecord6144::SplitDate`: the day
    /// it is where the player is).
    fn achievement_date(at: u32) -> String {
        let date = js_sys::Date::new(&(f64::from(at) * 1000.0).into());
        rules::dated(date.get_full_year(), date.get_month() + 1, date.get_date())
    }

    /// The name of the tab that is open, as the upper strip writes it (`UIAchTab::Label`).
    fn achievement_tab_name(&self) -> String {
        let (line, subs) = TABS.get(self.achievements.group as usize).copied().unwrap_or(TABS[0]);
        let line = self.achievements.sub.and_then(|sub| subs.get(sub as usize).copied()).unwrap_or(line);
        Self::words(line, "").trim_end().to_owned()
    }

    /// The achievements' window (`UIAchievementWnd::OnCreate`): the title bar with its cross, the upper strip
    /// 46 high under it, the tabs 102 wide from 63 down, and beside them the summary, or the list of the tab
    /// that is open (320 wide) with the pane of the achievement picked (328 wide).
    pub(in crate::web) fn achievements_window(&self) -> String {
        let mut window = title_bar(750, &Self::words(2646, "Achievements"));
        window.push_str(&cross(750, Press::Window));
        window.push_str(&fill(0, 17, 750, 451, WHITE));
        window.push_str(&self.achievements_upper());
        window.push_str(&self.achievements_tabs());
        if self.achievements.group == 0 {
            window.push_str(&self.achievements_summary());
        } else {
            window.push_str(&self.achievements_list());
            window.push_str(&self.achievement_pane());
        }
        window
    }

    /// The upper strip (`UIAchUpperWnd::Paint`, at 0, 17): the rank in its circle, the points towards the next
    /// rank over their gauge, the score in the middle; and for a tab that lists achievements, the two round
    /// buttons of what the list shows, the tab's name with how many of its achievements are completed, and
    /// the gauge of that under the trophy.
    fn achievements_upper(&self) -> String {
        let state = &self.achievements;
        let standing = state.standing;
        let navy = "#082758";
        let mut strip = art(&piece("bg_upper"), 0, 0, 750, 46);
        strip.push_str(&gauge((47, 24, 84, 10), standing.points, standing.next, UPPER, &Self::words(3105, "Until next level")));
        strip.push_str(&text_ending_in(SMALL, 130, 750, 13, BLACK, &format!("{}/{}", standing.points, standing.next)));
        // (0x582708 and 0x44C58.)
        strip.push_str(&centred(BOLD, 353, 46, 6, navy, &standing.score.to_string()));
        strip.push_str(&centred(BOLD, 16, 15, 19, "#584c04", &standing.rank.to_string()));
        if state.group != 0 {
            let (done, all) = rules::counted(&data::achievements(), &state.records, state.group, state.sub);
            strip.push_str(&art(&piece("upper_trophy"), 575, 3, 39, 40));
            for (place, (line, otherwise)) in [(3106, "Complete"), (3107, "Incomplete")].into_iter().enumerate() {
                let y = 10 + 15 * place as i32;
                let on = (state.shown == Shown::Unfinished) == (place == 1);
                strip.push_str(&radio(150, y, on, true, "", Some(Press::Shown(place).into())));
                strip.push_str(&text(169, y + 1, BLACK, &Self::words(line, otherwise)));
            }
            strip.push_str(&text_in(SMALL, 614, 15, navy, &self.achievement_tab_name()));
            strip.push_str(&text_ending_in(SMALL, 737, 750, 15, navy, &format!("({done}/{all})")));
            strip.push_str(&gauge((613, 29, 124, 8), done as u32, all as u32, UPPER, ""));
        }
        format!(r#"<div class="a" style="{}">{strip}</div>"#, at(0, 17, 750, 46))
    }

    /// The tabs (`UIAchTab`, at 0, 63; `DrawGroup`): each 40 high, its name 11 in and bold, the open one on
    /// its own picture and in its own blue (0x9C1810); under the open one its own tabs, 26 high each and
    /// three at once, the picked one with its arrow, and the arrows that move them where there are more.
    fn achievements_tabs(&self) -> String {
        let state = &self.achievements;
        let open = state.group as usize;
        let mut column = art(&piece("bg_tab"), 0, 0, 102, 405);
        for (group, (line, subs)) in TABS.iter().enumerate() {
            let y = rules::tab_top(group, open);
            let picture = picture(&piece(if group == open { "tab_press" } else { "tab_out" }));
            let hover = if group == open { String::new() } else { format!(";--o:url('{}')", self::picture(&piece("tab_over"))) };
            column.push_str(&[
                format!(r#"<div class="a t b" style="{};--n:url('{picture}'){hover};"#, at(0, y, 100, TAB_HEIGHT)),
                format!(
                    r#"box-sizing:border-box;padding-left:11px;line-height:{TAB_HEIGHT}px;font-weight:bold;color:{}" {}>{}</div>"#,
                    if group == open { "#10189c" } else { TAB },
                    pressed(Press::Tab(Tab { group: group as u32, sub: None })),
                    escape(Self::words(*line, "").trim_end())
                )
            ].concat());
            if group == open {
                column.push_str(&self.achievement_subs(group, subs, y + TAB_HEIGHT));
            }
        }
        format!(r#"<div class="a" style="{}">{column}</div>"#, at(0, 63, 102, 405))
    }

    /// The tabs under the open one, from `top` down (`UIAchTab::DrawGroup`): those from the first shown on,
    /// three at most.
    fn achievement_subs(&self, group: usize, subs: &[usize], top: i32) -> String {
        let state = &self.achievements;
        let first = state.sub_first.min(subs.len().saturating_sub(SUBS_SHOWN));
        let mut rows = String::new();
        for (row, (sub, line)) in subs.iter().enumerate().skip(first).take(SUBS_SHOWN).enumerate() {
            let y = top + SUB_HEIGHT * row as i32;
            let picked = state.sub == Some(sub as u32);
            let ground = picture(&piece(if picked { "tab_sub_press" } else { "tab_sub_out" }));
            let hover = if picked { String::new() } else { format!(";--o:url('{}')", picture(&piece("tab_sub_over"))) };
            rows.push_str(&[
                format!(r#"<div class="a t b" style="{};--n:url('{ground}'){hover};"#, at(0, y, 100, SUB_HEIGHT)),
                // (0x9B6A59, picked or not.)
                format!(
                    r#"box-sizing:border-box;padding-left:26px;line-height:{SUB_HEIGHT}px;color:#596a9b" {}>{}</div>"#,
                    pressed(Press::Tab(Tab { group: group as u32, sub: Some(sub as u32) })),
                    escape(Self::words(*line, "").trim_end())
                )
            ].concat());
            if picked {
                rows.push_str(&art(&piece("tab_subarrow"), 12, y + 7, 10, 11));
            }
        }
        // More than three: the bar's two arrows at the tabs' left (`UIAchScrollBar`, 11 square each).
        if subs.len() > SUBS_SHOWN {
            let arrow = |name: &str, y: i32, step: crate::press::Step| {
                let picture = piece(name);
                pictured(0, y, (11, 11), [picture.as_str(); 3], Press::Subs(step), "")
            };
            rows.push_str(&arrow("scroll_v_arrow_up", top, crate::press::Step::Less));
            rows.push_str(&arrow("scroll_v_arrow_down", top + SUB_HEIGHT * SUBS_SHOWN as i32 - 11, crate::press::Step::More));
        }
        rows
    }

    /// The summary (`UIAchSummaryWnd::Paint`, at 102, 63): how many of all the achievements are completed
    /// over a gauge 558 wide, the same for each of the six groups in two columns, and the two achievements
    /// completed last, each with its picture, its points on the mark of one completed, its name (pressed, it
    /// opens the list on that achievement), what it says of itself and its day.
    fn achievements_summary(&self) -> String {
        let all = data::achievements();
        let records = &self.achievements.records;
        let mut pane = art(&piece("bg_summary"), 0, 0, 648, 404);
        let (done, total) = rules::counted(&all, records, 0, None);
        pane.push_str(&text_in(BOLD, 82, 27, HEADING, &Self::words(2679, "Achieved challenges")));
        pane.push_str(&text_ending_in(SMALL, 600, 648, 28, NUMBER, &format!("{done}/{total}")));
        pane.push_str(&gauge((43, 49, 558, 10), done as u32, total as u32, INNER, ""));
        for (place, (line, _)) in TABS.iter().enumerate().skip(1) {
            let (row, column) = (((place - 1) / 2 * 50) as i32, ((place - 1) % 2) as i32);
            let (done, total) = rules::counted(&all, records, place as u32, None);
            pane.push_str(&text_in(BOLD, column * 302 + 82, row + 107, HEADING, Self::words(*line, "").trim_end()));
            pane.push_str(&text_ending_in(SMALL, column * 304 + 296, 648, row + 108, NUMBER, &format!("{done}/{total}")));
            pane.push_str(&gauge((column * 304 + 43, row + 123, 254, 10), done as u32, total as u32, INNER, ""));
        }
        pane.push_str(&text_in(BOLD, 9, 292, TAB, &Self::words(2651, "Recent completed achievements")));
        let (first, second) = rules::latest(records);
        for (place, id) in [first, second].into_iter().enumerate() {
            let Some(info) = id.and_then(|id| all.iter().find(|info| info.id == id)) else { continue };
            pane.push_str(&self.achievement_recent(info, 260 + 316 * place as i32));
        }
        format!(r#"<div class="a" style="{}">{pane}</div>"#, at(102, 63, 648, 404))
    }

    /// One of the two completed last, in the summary: `x` is where its mark stands (`UIAchSummaryWnd::Paint`,
    /// and the names, which are buttons of text 198 to its left: `OnCreate`).
    fn achievement_recent(&self, info: &rules::Info, x: i32) -> String {
        let record = self.achievement_record(info.id);
        let mut card = art(&piece(&format!("icon_{}", info.icon.to_lowercase())), x - 242, 314, 38, 32);
        card.push_str(&art(&piece("badge_complete"), x, 316, 20, 30));
        card.push_str(&text_in(BOLD, x + 23, 325, HEADING, &info.points.to_string()));
        // The name: 0x997836, and 0x635AA5 under the pointer.
        card.push_str(&format!(
            r#"<div class="a t hov" style="left:{}px;top:325px;font-weight:bold;color:#367899;--over:{NUMBER}" {}>{}</div>"#,
            x - 198,
            pressed(Press::Show(info.id)),
            escape(&info.title)
        ));
        card.push_str(&text(x - 241, 365, PALE, &info.summary));
        if info.has_reward() && !record.rewarded {
            card.push_str(&art(&piece("list_rewardbox_not_receive"), x + 12, 360, 30, 23));
        }
        card.push_str(&format!(
            r#"<div class="a t" style="left:{}px;top:366px;color:{NUMBER};{};{}">{}</div>"#,
            x - 3,
            Font(0, 10, false).style(),
            outlined(WHITE),
            Self::achievement_date(record.completed_at)
        ));
        card
    }

    /// The achievements kept in sight (`UIAchTracingWnd::Paint`, 300 by 150 with no ground of its own): first
    /// those not completed, then the others, each a dot and its name in brackets, which opens or shuts the
    /// list of its goals when pressed, and at the right an arrow that opens the window on it; shut, it says
    /// how many goals there are; open, the goals stand under it 15 apart, one reached in green (0xDFF6F).
    /// (The colour of the names, which are buttons of text made with colour 0, is taken to be black over
    /// white as the count beside them is drawn: the button's own drawing was not read.) Only the names and
    /// the arrows take the pointer: what is under the rest of the window is pressed through it
    /// (`IsTransmitMouseInput`).
    pub(in crate::web) fn achievements_traced(&self) -> String {
        let all = data::achievements();
        let mut window = String::new();
        let mut y = 10;
        // The names are black over white and the goals white over black, each over a copy of itself a pixel
        // down and to the right.
        let (light, shade) = ("text-shadow:1px 1px 0 #fff", "text-shadow:1px 1px 0 #000");
        for completed in [false, true] {
            for traced in self.achievements.traced.iter().filter(|traced| self.achievement_record(traced.id).completed == completed) {
                let Some(info) = all.iter().find(|info| info.id == traced.id) else { continue };
                let record = self.achievement_record(info.id);
                let more = if traced.open { String::new() } else { format!(" <span style=\"pointer-events:none\">(+{})</span>", info.goals.len()) };
                window.push_str(&[
                    format!(r#"<div class="a t" style="left:10px;top:{y}px;color:{BLACK};font-weight:bold;{light}">"#),
                    format!(r#"<span style="pointer-events:auto" {}>● [{}]</span>{more}</div>"#, pressed(Press::Unfold(info.id)), escape(&info.title))
                ].concat());
                window.push_str(&format!(
                    r#"<div class="a t" style="left:280px;top:{y}px;pointer-events:auto;color:{BLACK};font-weight:bold;{light}" {}>▶</div>"#,
                    pressed(Press::Show(info.id))
                ));
                y += 15;
                if traced.open {
                    for place in 0..info.goals.len() {
                        let Some((words, reached)) = rules::traced_line(info, place, record.goals.get(place).copied().unwrap_or(0)) else { continue };
                        let colour = if reached { "#6fff0d" } else { WHITE };
                        window.push_str(&format!(r#"<div class="a t" style="left:20px;top:{y}px;color:{colour};{shade}">{}</div>"#, escape(&words)));
                        y += 15;
                    }
                }
                y += 5;
            }
        }
        window
    }

    /// The line that tells of an achievement completed (`UIAchBalloonText`, as wide as the screen and 50 high,
    /// 50 down it): white letters 20 high rimmed in black, in the middle.
    pub(in crate::web) fn achievement_told_line(&self) -> String {
        let Some((words, _)) = &self.achievements.balloon else { return String::new() };
        format!(
            r#"<div class="t" style="line-height:50px;text-align:center;color:{WHITE};{};{}">{}</div>"#,
            Font(0, 20, true).style(),
            outlined(BLACK),
            escape(words)
        )
    }

    /// The Equipment window's page of titles (`UIEquipWnd::Paint` for its third page, and its
    /// `UIListBoxTitle`), from the page's top at 37: a white ground 131 high, the title worn in its middle 20
    /// down, in bold a size larger, and in a grey frame 107 high the list of the titles to wear, 15 to a
    /// line on a ground of 225 grey: first the line that takes the title off (line 2686), then the titles by
    /// their names. The line picked stands on a band; a title gained since the window was opened is coloured
    /// (0x92C0FA). A double click on a line asks the server for it.
    pub(in crate::web) fn title_page(&self) -> String {
        let state = &self.achievements;
        let titles = rules::titles(&data::achievements(), &state.records, data::title_name);
        let mut page = fill(1, 35, 278, 131, WHITE);
        page.push_str(&centred(Font(0, 13, true), 1, 278, 40, BLACK, &data::title_name(state.title)));
        let lines: String = std::iter::once((0, Self::words(2686, "Disable Title")))
            .chain(titles.into_iter().map(|title| (title, data::title_name(title))))
            .map(|(title, name)| {
                let band = match (state.title_picked == Some(title), state.new_titles.contains(&title)) {
                    (true, _) => ";background:#c6d6f7",
                    (false, true) => ";background:#fac092",
                    _ => "",
                };
                format!(
                    r#"<div class="t" style="height:15px;line-height:14px;padding-left:3px;cursor:default{band}" {} {}>{}</div>"#,
                    pressed(Press::Title(title)),
                    used(Use::Title(title)),
                    escape(&name)
                )
            })
            .collect();
        page.push_str(&[
            format!(r#"<div class="a" style="{};box-sizing:border-box;border:1px solid #646464"></div>"#, at(2, 58, 276, 107)),
            format!(r#"<div class="a scroll" style="{};background:#e1e1e1" {}>{lines}</div>"#, at(3, 59, 274, 105), lines_stepped(15))
        ].concat());
        page
    }
}
