//! The skill tree, a skill's description, and the hotkey bar.

use super::*;
use crate::press::Step;
use crate::press::skills::{LevelStep, Press};

/// How wide the tabs of the jobs are, down the left of the skill window: the window's body begins where they
/// end (`UINewSkillListWnd::Paint`, its `left`).
const SKILL_TABS: i32 = tabs::SIDE_TABS;

impl App {
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
                let hotkey = self.skills.hotkeys.get(slot).copied().unwrap_or_default();
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

    /// A hotkey's picture, the number under it, and what it says when pointed at.
    pub(in crate::web) fn hotkey_look(&self, hotkey: Hotkey) -> (String, String, String) {
        if hotkey.id == 0 {
            return (String::new(), String::new(), String::new());
        }
        if hotkey.skill {
            let name = data::skill_name(hotkey.id as u16, || format!("{} {}", tr("Skill"), hotkey.id));
            (Self::skill_icon(hotkey.id as u16), count(hotkey.count), format!("{name} Lv {}", hotkey.count))
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
    /// name over it and its level under it, the character standing in the corner.
    pub(in crate::web) fn skill_window(&self) -> String {
        let (width, height) = (560, 400);
        let job = self.own_figure().job;
        let pages = data::skill_pages(job);
        let page = self.skills.tab.min(pages.len().saturating_sub(1));
        let mut window = String::new();
        window.push_str(&Self::skill_window_frame(width, height));
        // The tabs: one for each job the character grew through, down the window's left from 6 under its top
        // (`UINewSkillListWnd::OnCreate`: a `UITabControl` of 15 round each tab's letters; `hud/tabs.rs`).
        let tabs: Vec<(String, Option<&str>, String)> = (0..pages.len())
            .map(|tab| (tr(["1st", "2nd", "3rd", "4th"].get(tab).copied().unwrap_or("")).to_owned(), None, pressed(Press::Tab(tab))))
            .collect();
        window.push_str(&tabs::side_tabs(0, 6, 15, false, page, &tabs));
        // The cells, empty ones first.
        for cell in 0..42 {
            window.push_str(&art("basic_interface/no_skill.bmp", cell % 7 * 68 + 76, cell / 7 * 56 + 44, 24, 24));
        }
        let points = self.value(parameter::SKILL_POINTS);
        for (place, id) in pages.get(page).into_iter().flatten().copied().filter(|(place, _)| *place < 42) {
            window.push_str(&self.skill_cell(place, id, points));
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

    /// One skill in its cell of the tree: its ground, picture, name, level, and the buttons by it.
    fn skill_cell(&self, place: u16, id: u16, points: i64) -> String {
        let mut window = String::new();
        let (x, y) = (i32::from(place % 7) * 68 + 72, i32::from(place / 7) * 56);
        let known = data::skill(id);
        let learned = self.skills.known.iter().find(|skill| skill.id == id);
        let level = learned.map_or(0, |skill| skill.level);
        let name = known.as_ref().map_or_else(|| format!("{} {id}", tr("Skill")), |(_, name, _)| name.clone());
        let most = known.as_ref().map_or(level, |(.., most)| *most);
        let raisable = learned.is_some_and(|skill| skill.raisable) && points > 0 && level < most;
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
            r#"<div class="a" style="{};{dim}" {drag} data-skill="{id}" title="{}">{icon}</div>"#,
            at(x + 4, y + 44, 24, 24),
            escape(&name)
        ));
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
        if level > 0 {
            // "Used at / known at", the first in the client's pink when it is less than all (`0x7F3CFF`).
            let shown = if usable {
                format!(r#"<span style="color:{}">{used}</span> / {level}"#, if used == level { shade } else { "#ff3c7f" })
            } else {
                level.to_string()
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
        if usable {
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
        // The client writes here, in red, how many levels are being added ("+N", `DrawSkillTree`: (0, 11), not
        // bold, at 30, 44); this client's plus that raises the skill at once stands in its place and its text.
        if raisable {
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
