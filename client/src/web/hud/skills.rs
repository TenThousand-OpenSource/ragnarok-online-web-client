//! The skill tree, a skill's description, and the hotkey bar; the button that tells of a job level gained, and
//! the box that asks the words of a Talkie Box or of Graffiti.

use super::*;
use crate::press::Step;
use crate::press::skills::{LevelStep, Press};

/// How wide the tabs of the jobs are, down the left of the skill window: the window's body begins where they
/// end (`UINewSkillListWnd::Paint`, its `left`).
const SKILL_TABS: i32 = tabs::SIDE_TABS;

impl App {
    /// The button that tells of a job level gained (`UINotifyJobLevelUpWnd`, which is the base level's button
    /// with another answer to its press): a press opens the skill window.
    pub(in crate::web) fn job_level_told(&self) -> String {
        pictured(0, 0, (43, 43), status::LEVEL_PICTURES, Press::JobLevelGained, "")
    }

    /// The box that asks the words of a Talkie Box (`UITalkboxTrapInputWnd`) or of Graffiti
    /// (`UIGraffiStrboxWnd`), 280 by 120: a title bar, the white panel and the bar of buttons; the client's
    /// message 436 at 16,22, black over a white copy of itself a pixel right and down; the line at 22,50 (240
    /// by 16, on `242, 242, 242`), of as many letters as the skill takes (`crate::skills::words_asked`); OK at
    /// 189,96 and Cancel at 234,96. Enter is its OK (`m_message0`). The Talkie Box's has the message 435 in
    /// its title bar, at 16,2; the box of Graffiti has none, and asks in the same words.
    pub(in crate::web) fn skill_words_window(&self) -> String {
        let Some((skill, ..)) = self.skills.writing else { return String::new() };
        let title = if skill == crate::skills::TALKIE_BOX { Self::words(435, "Record a message in the Talkie Box") } else { String::new() };
        [
            format!(r#"<form {} class="a" style="{}">"#, sent(Form::SkillWords), at(0, 0, 280, 120)),
            title_bar(280, ""),
            titled(16, 2, &title),
            panel(280, 120),
            format!(
                r#"<div class="a t" style="left:16px;top:22px;color:#000;text-shadow:1px 1px 0 #fff">{}</div>"#,
                escape(&Self::words(436, "Please type a message for the Talkie Box"))
            ),
            field(&windows::SKILL_WORDS.field(), (22, 50, 240, 16), "", crate::skills::words_asked(skill).unwrap_or(0), "background:#f2f2f2"),
            picture_press(189, 96, "btn_ok", Press::WordsOk),
            picture_press(234, 96, "btn_cancel", Press::WordsCancel),
            r#"<button type="submit" hidden></button></form>"#.to_owned()
        ].concat()
    }

    /// What a skill does, as the client shows it (`UISkillDescribeWnd`): a white box 304 wide inside a rounded
    /// line, as tall as the client's description of the skill needs at 18 pixels a line (the first line is the
    /// skill's name), with the close button in its corner. Only a text taller than the screen scrolls.
    pub(in crate::web) fn skill_info_window(&self) -> String {
        let Some(id) = self.skills.info else { return String::new() };
        let lines = data::skill_text(id).unwrap_or_else(|| vec![data::skill_name(id, || tr("Unknown Skill").to_owned())]);
        let height = 18 * lines.len() as i32 + 4 + 6;
        let description: String = lines
            .iter()
            .map(|line| format!(r#"<div style="height:18px;line-height:18px;white-space:nowrap">{}</div>"#, coloured(line)))
            .collect();
        [
            format!(r#"<div style="position:relative;width:304px;height:{height}px;max-height:calc(100vh - 50px);"#),
            format!(
                r#"background:#fff;border:1px solid #c3c3c3;border-radius:4px;box-sizing:border-box;overflow:hidden">
               <div class="t viewer" style="position:absolute;left:3px;top:3px;right:3px;bottom:3px;user-select:text">{description}</div>{}</div>"#,
                cross(304, Press::InfoClose)
            )
        ].concat()
    }

    /// The hotkey bar as the client draws it (`UIShortCutWnd`): one to four rows of nine, each row its frame 33
    /// under the last with its number at the left; no key is written on a slot (the client names the key only
    /// when a slot is pointed at). The buttons at the right take a row away and add one. An icon dragged from
    /// the inventory or the skill list fills a slot, and what is on a slot is dragged to another or off the bar.
    pub(in crate::web) fn hotkeys(&self) -> String {
        let rows = i32::from(self.skills.hotkey_rows) + 1;
        let mut bar = String::new();
        for row in 0..rows {
            bar.push_str(&art("shortcut/shortcut_frame.bmp", 0, row * 33, 290, 34));
            // The row's number, in the smaller text (`UIShortCutWnd::Paint`: (0, 11) at 6, 13).
            bar.push_str(&text_in(SMALL, 6, row * 33 + 13, "rgb(74,74,47)", &(row + 1).to_string()));
            for column in 0..9 {
                let slot = (row * 9 + column) as usize;
                // A skill the character has not leaves its slot empty to the eye and the hand.
                let hotkey = self.skills.hotkeys.get(slot).copied().unwrap_or_default();
                let lost = hotkey.skill && !crate::hotkeys::skill_shown(self.skill_level(hotkey.id as u16), hotkey.count as u16);
                let hotkey = if lost { Default::default() } else { hotkey };
                let (icon, count, title) = self.hotkey_look(hotkey);
                // A skill that waits to be used again has the shade of its wait over it (`hud/cooldowns.rs`).
                let count = if hotkey.skill && hotkey.id != 0 { format!("{count}{}", self.waiting(hotkey.id as u16)) } else { count };
                // The key the slot answers to now (the first table's places are the bar's slots), as the
                // Shortcut Settings write it: the player may have set another, or none, on any row.
                let key = crate::hotkeys::written(self.bound(crate::hotkeys::table::BAR, slot), self.keys.mac, data::key_name);
                let key = if key.is_empty() { key } else { format!("{key} ") };
                let skill = if hotkey.skill && hotkey.id != 0 { format!(r#" data-skill="{}""#, hotkey.id) } else { String::new() };
                // What is on a slot can be dragged to another, or off the bar to take it away.
                let skill = if hotkey.id != 0 { format!("{skill} {}", dragged(Drag::Slot(slot))) } else { skill };
                let used = used(Use::Slot(slot));
                bar.push_str(&format!(
                    r#"<div class="a slot" style="{}" data-slot="{slot}" {used}{skill} title="{key}{}">{icon}{count}</div>"#,
                    at(16 + 29 * column, row * 33 + 4, 24, 24),
                    escape(&title),
                ));
            }
        }
        for (name, step, y) in [("minus", Step::Less, rows * 33 - 28), ("plus", Step::More, rows * 33 - 13)] {
            bar.push_str(&format!(
                r#"<img class="a" style="right:2px;top:{y}px;cursor:pointer" src="{}" alt="" draggable="false" {}>"#,
                picture(&format!("shortcut/btn_{name}_normal.bmp")),
                pressed(Press::HotkeyRows(step))
            ));
        }
        bar
    }

    /// The SP a skill takes at a level, as the windows write it (`crate::skills::sp_shown`): the figure of the
    /// rules the server plays (`data::skill_sp`), and with no table what the server told of the level the
    /// skill is known at.
    pub(in crate::web) fn skill_sp(&self, id: u16, level: u16) -> u16 {
        let told = self.skills.known.iter().find(|skill| skill.id == id).map_or(0, |skill| skill.sp);
        crate::skills::sp_shown(data::skill_sp(id, level), told)
    }

    /// The level a skill is known at: none for one not known.
    pub(in crate::web) fn skill_level(&self, id: u16) -> u16 {
        self.skills.known.iter().find(|skill| skill.id == id).map_or(0, |skill| skill.level)
    }

    /// Where a skill's cell of the tree begins in the window, by its place among the cells (seven to a row,
    /// 68 apart, the rows 56 apart).
    fn skill_cell_at(place: u16) -> (i32, i32) {
        (i32::from(place % 7) * 68 + 72, i32::from(place / 7) * 56)
    }

    /// A hotkey's picture, the number under it, and what it says when pointed at.
    pub(in crate::web) fn hotkey_look(&self, hotkey: Hotkey) -> (String, String, String) {
        if hotkey.id == 0 {
            return (String::new(), String::new(), String::new());
        }
        if hotkey.skill {
            let (id, level) = (hotkey.id as u16, hotkey.count as u16);
            let name = data::skill_name(id, || format!("{} {}", tr("Skill"), hotkey.id));
            // What it says when pointed at (`ShortcutTooltip.cpp`, `UIQuickSlotWnd`): the level it is used at
            // and the SP of that level.
            (Self::skill_icon(id), count(hotkey.count), crate::skills::use_tip(&name, level, self.skill_sp(id, level)))
        } else {
            let carried = self.carried(hotkey.id);
            // An item that is used up stays on the bar, without its picture, until there is more of it.
            let icon = if carried == 0 { String::new() } else { Self::item_icon(hotkey.id, true) };
            let name = data::item(hotkey.id).map_or_else(|| format!("{} {}", tr("Item"), hotkey.id), |(name, _)| name);
            (icon, if carried == 0 { String::new() } else { count(carried) }, amount_tip(&name, carried))
        }
    }

    /// The skill window as the client draws it (`UINewSkillListWnd`, its tree of skills): 560 by 400, the tabs of
    /// the jobs down its left, the skills of the job in seven columns 68 apart and rows 56 apart, each with its
    /// name over it and its level under it, the character standing in the corner. (The tree writes no SP: the
    /// client's "Sp : n" is of its list of rows, `DrawSkillList`, which is not built here.)
    pub(in crate::web) fn skill_window(&self) -> String {
        let (width, height) = (560, 400);
        let job = self.own_figure().job;
        let pages = data::skill_pages(job);
        let page = self.skills.tab.min(pages.len().saturating_sub(1));
        let mut window = String::new();
        window.push_str(&Self::skill_window_frame(width, height));
        // The tabs: one for each job the character grew through, down the window's left from 6 under its top
        // (`UINewSkillListWnd::OnCreate`: a `UITabControl` of 15 round each tab's letters; `hud/tabs.rs`).
        // The tab of another page that has a skill the one looked at asks for is coloured
        // (`UpdateRequiredSkills`: `0xE1E7FC`).
        let looked = self.skills.info.filter(|looked| pages.get(page).is_some_and(|page| page.iter().any(|(_, skill)| skill == looked)));
        let asked: Vec<u16> = looked.map(|looked| data::skill_asks(job, looked).1.into_iter().map(|(skill, _)| skill).collect()).unwrap_or_default();
        let tabs: Vec<(String, Option<&str>, String)> = (0..pages.len())
            .map(|tab| {
                let ground = (tab != page && pages[tab].iter().any(|(_, skill)| asked.contains(skill))).then_some("#fce7e1");
                (tr(["1st", "2nd", "3rd", "4th"].get(tab).copied().unwrap_or("")).to_owned(), ground, pressed(Press::Tab(tab)))
            })
            .collect();
        window.push_str(&tabs::side_tabs(0, 6, 15, false, page, &tabs));
        // The cells, empty ones first.
        for cell in 0..42 {
            window.push_str(&art("basic_interface/no_skill.bmp", cell % 7 * 68 + 76, cell / 7 * 56 + 44, 24, 24));
        }
        let points = self.value(parameter::SKILL_POINTS);
        let shown = pages.get(page).map(Vec::as_slice).unwrap_or_default();
        for (place, id) in shown.iter().copied().filter(|(place, _)| *place < 42) {
            window.push_str(&self.skill_cell(place, id, points, shown));
        }
        // The character stands in the corner, its feet 25 from the bottom.
        window.push_str(&figure_canvas("pc-skill", (0, height - 130, 100, 130), (48 - 80, 130 - 25 - 137)));
        window.push_str(&text(80, height - 23, BLACK, &format!("{} : {points}", tr("Skill Point"))));
        for (name, x) in [("btn_apply", width - 100), ("btn_reset", width - 50)] {
            window.push_str(&format!(
                r#"<div class="a" style="{};background:url('{}');opacity:.5"></div>"#,
                at(x, height - 27, 42, 20),
                picture(&format!("{name}.bmp"))
            ));
        }
        window
    }

    /// The skill window's body: white between the bars, inside two rounded lines, with its title and buttons.
    fn skill_window_frame(width: i32, height: i32) -> String {
        let mut window = String::new();
        window.push_str(&format!(
            r#"<div class="a" style="{};background:#fff;border:1px solid #c3c3c3;border-radius:4px;box-sizing:border-box"></div>"#,
            at(SKILL_TABS, 0, width - SKILL_TABS, height)
        ));
        window.push_str(&format!(
            r#"<div class="a" style="{};border:1px solid #c3c3c3;border-radius:3px;box-sizing:border-box"></div>"#,
            at(SKILL_TABS + 2, 2, width - SKILL_TABS - 4, height - 4)
        ));
        window.push_str(&art("basic_interface/skill_upbar_left.bmp", SKILL_TABS, 0, 12, 20));
        window.push_str(&art("basic_interface/skill_upbar_mid.bmp", SKILL_TABS + 12, 0, width - SKILL_TABS - 24, 20));
        window.push_str(&art("basic_interface/skill_upbar_right.bmp", width - 12, 0, 12, 20));
        window.push_str(&art("basic_interface/skill_botbar_left.bmp", SKILL_TABS, height - 32, 12, 32));
        window.push_str(&art("basic_interface/skill_botbar_mid.bmp", SKILL_TABS + 12, height - 32, width - SKILL_TABS - 24, 32));
        window.push_str(&art("basic_interface/skill_botbar_right.bmp", width - 12, height - 32, 12, 32));
        window.push_str(&titled(40, 5, tr("Skill Tree")));
        window.push_str(&base_button(22, 5));
        window.push_str(&art("basic_interface/sys_mini_off.bmp", width - 30, 5, 11, 11));
        window.push_str(&press_button(width - 16, 5, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Window));
        window
    }

    /// The marks of what a skill asks for (`UINewSkillListWnd::UpdateRequiredSkills`, `DrawSkillTree`, its
    /// `m_values`): the skill's own cell and the cell of each skill it asks for on a pink ground (`0xCCC0FF`
    /// inside a line of `0xC1B6FF`), and by each asked for the level asked, black in a white edge at 30, 58
    /// ((0, 11), bold). The client shows them while the skill is pointed at, and keeps them after the right
    /// button (`m_showDetail`): here they follow the skill's picture and the page's style sheet shows them
    /// while it is pointed at (`.skill:hover + .needs`), and `kept` keeps them while its description is open.
    /// `page` is the page's skills by their places. The pink lies over the pictures here; the client tints
    /// the picture itself (`DrawAuxiliary(2, …)`, not read).
    fn skill_needs(id: u16, needs: &[(u16, u16)], kept: bool, page: &[(u16, u16)]) -> String {
        let mut marks = format!(r#"<div class="needs{}">"#, if kept { " shown" } else { "" });
        for (skill, level) in std::iter::once((id, 0)).chain(needs.iter().copied()) {
            let Some((place, _)) = page.iter().find(|(place, at)| *at == skill && *place < 42) else { continue };
            let (x, y) = Self::skill_cell_at(*place);
            marks.push_str(&format!(
                r#"<div class="a" style="{};background:#ffc0cc;outline:1px solid #ffb6c1;border-radius:2px;mix-blend-mode:multiply"></div>"#,
                at(x + 3, y + 43, 26, 26)
            ));
            if level > 0 {
                marks.push_str(&format!(
                    r#"<div class="a" style="left:{}px;top:{}px;{};color:#000;{}">{level}</div>"#,
                    x + 30,
                    y + 58,
                    SMALL_BOLD.style(),
                    outlined(WHITE)
                ));
            }
        }
        marks.push_str("</div>");
        marks
    }

    /// Under a skill's cell at (x, y): the level it is known at, if any, and for a skill that is used (`used`:
    /// the level it is used at) the two of them with the arrows that pick the second.
    fn skill_cell_level(id: u16, (x, y): (i32, i32), level: u16, used: Option<u16>, shade: &str) -> String {
        let mut window = String::new();
        if level > 0 {
            // "Used at / known at", the first in the client's pink when it is less than all (`0x7F3CFF`).
            let shown = match used {
                Some(used) => format!(r#"<span style="color:{}">{used}</span> / {level}"#, if used == level { shade } else { "#ff3c7f" }),
                None => level.to_string(),
            };
            window.push_str(&format!(
                r#"<div class="a" style="{};{};color:{shade};text-align:center;white-space:nowrap;{};pointer-events:none">{shown}</div>"#,
                at(x - 8, y + 70, 48, 13),
                SMALL_BOLD.style(),
                outlined(WHITE)
            ));
        }
        // The arrows at either side of the level pick the level it is used at, one down or up (the client's
        // `m_btnSkills` and `m_btnSkillUps`: 20 left of the cell and 55 further right, level with the text).
        if used.is_some() {
            for (arrow, step, across) in [("arw_left", Step::Less, -16), ("arw_right", Step::More, 39)] {
                window.push_str(&format!(
                    r#"<img class="a" style="left:{}px;top:{}px;cursor:pointer" src="{}" alt="" draggable="false" {}>"#,
                    x + across,
                    y + 70,
                    picture(&format!("basic_interface/{arrow}.bmp")),
                    pressed(Press::Level(LevelStep { skill: id, step }))
                ));
            }
        }
        window
    }

    /// One skill in its cell of the tree: its ground, picture, name, level, and the buttons by it. `page` is
    /// the page's skills by their places, for the marks of what the skill asks for.
    fn skill_cell(&self, place: u16, id: u16, points: i64, page: &[(u16, u16)]) -> String {
        let mut window = String::new();
        let (x, y) = Self::skill_cell_at(place);
        let known = data::skill(id);
        let learned = self.skills.known.iter().find(|skill| skill.id == id);
        let level = learned.map_or(0, |skill| skill.level);
        let name = known.as_ref().map_or_else(|| format!("{} {id}", tr("Skill")), |(_, name, _)| name.clone());
        let most = known.as_ref().map_or(level, |(.., most)| *most);
        let raisable = learned.is_some_and(|skill| skill.raisable) && points > 0 && level < most;
        // What it asks for by the rules the server plays (`data::skill_asks`), and what of that is lacking.
        let (job_level, needs) = data::skill_asks(self.own_figure().job, id);
        // The plus is offered from the table too, by a skill the server does not list (`plus_by_table`).
        let job = self.value(parameter::JOB_LEVEL);
        let opens = crate::skills::plus_by_table(raisable, points, level, job, (job_level, &needs), |skill| self.skill_level(skill));
        // A skill that can be raised stands on light blue; the others on white.
        let ground = if raisable { "background:#afc4f0;outline:1px solid #afc4f0;border-radius:2px" } else { "background:#fff" };
        window.push_str(&format!(r#"<div class="a" style="{};{ground}"></div>"#, at(x + 3, y + 43, 26, 26)));
        // Only a skill that is learned and does something can be used and put on the bar.
        let usable = learned.is_some_and(|skill| skill.level > 0 && !skill.passive());
        // The level it is used at: what the player picked with the arrows by its level, or all it has.
        let used = self.skills.use_level.get(&id).copied().filter(|used| (1..=level).contains(used)).unwrap_or(level);
        let drag = if usable {
            format!("{} {}", dragged(Drag::Skill(Leveled { id: u32::from(id), level: used })), super::used(Use::Skill(id)))
        } else {
            String::new()
        };
        let dim = if level == 0 { "filter:grayscale(1) opacity(.55);" } else { "" };
        let icon = Self::skill_icon(id);
        window.push_str(&format!(
            r#"<div class="a skill" style="{};{dim}" {drag} data-skill="{id}" title="{}">{icon}</div>"#,
            at(x + 4, y + 44, 24, 24),
            escape(&name)
        ));
        window.push_str(&Self::skill_needs(id, &needs, self.skills.info == Some(id), page));
        let shade = if level == 0 { "#787878" } else { "#666666" };
        // The name, over the cell, in the smaller text (`DrawSkillTree`: (0, 11), and the level under the cell
        // the same in bold): in the middle of 48 pixels, and one that is wider cut from their left edge to what
        // fits with three dots after it (`UIWindow::ClipTextToWidth`; the page's own mark for a text cut
        // stands for the dots). The whole name is what the picture says when pointed at.
        window.push_str(&[
            format!(
                r#"<div class="a" style="{};{};color:{shade};text-align:center;white-space:nowrap;overflow:hidden;"#,
                at(x - 8, y + 30, 48, 13),
                SMALL.style()
            ),
            format!(r#"text-overflow:ellipsis;pointer-events:none">{}</div>"#, escape(&name))
        ].concat());
        window.push_str(&Self::skill_cell_level(id, (x, y), level, usable.then_some(used), shade));
        // The client writes here, in red, how many levels are being added ("+N", `DrawSkillTree`: (0, 11), not
        // bold, at 30, 44); this client's plus that raises the skill at once stands in its place and its text.
        if raisable || opens {
            window.push_str(&format!(
                r#"<button type="button" class="a" style="{};padding:0;border:0;background:none;{};color:#f00;cursor:pointer" {} title="{}">+</button>"#,
                at(x + 30, y + 42, 12, 14),
                SMALL.style(),
                pressed(Press::Raise(id)),
                tr("Raise")
            ));
        }
        window
    }
}
