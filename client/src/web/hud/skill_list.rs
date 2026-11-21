//! The list of a homunculus's skills and of a mercenary's: the client's `UISkillListWnd`, one window class for
//! both (its `skillTab` 0 and 1), a row to a skill.

use super::*;
use crate::net::inventory::Skill;
use crate::press::skills::LevelStep;
use crate::press::{Press, Step};

/// What the list's buttons do for the one whose list it is.
pub(super) struct Listed {
    pub close: Press,
    pub pick: fn(usize) -> Press,
    pub uses: Press,
    pub level: fn(LevelStep) -> Press,
    /// The button by a skill that can be raised, for one whose skills are raised.
    pub raise: Option<fn(u16) -> Press>,
}

/// How tall the list is with that many skills. The client opens it 280 by 120, two rows, and the player pulls
/// it taller by its corner, a row of 32 at a time up to seven (`UM_RESIZE`); here, with no corner to pull, it
/// is as tall as its skills need within those bounds. This client's choice.
pub(super) fn list_height(skills: usize) -> i32 {
    skills.clamp(2, 7) as i32 * 32 + 38
}

impl App {
    /// The window (`UISkillListWnd::Paint`, `OnCreate`, `UpdateRowButtons`): the title bar; down the sides the
    /// inventory's edges; each row its cell with the skill's picture 44 in, the name 112 in, under it the
    /// level (the level it is used at in red when it is not the whole) with the arrows that pick it, and what
    /// it costs or that it is passive; the button that raises it 78 in; at the bottom the points left, Use
    /// and Close. The picked row has the client's band behind it. `points` are the skill points to spend,
    /// for one who has such.
    pub(super) fn skill_list(&self, title: &str, skills: &[Skill], picked: Option<usize>, points: Option<u16>, does: Listed) -> String {
        let (width, height) = (280, list_height(skills.len()));
        let mut window = title_bar(width, title);
        window.push_str(&cross(width, does.close.clone()));
        window.push_str(&fill(0, 17, width, height - 38, WHITE));
        window.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') repeat-y"></div>"#,
            at(0, 17, 20, height - 38),
            picture("basic_interface/itemwin_left.bmp")
        ));
        window.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') repeat-y"></div>"#,
            at(width - 20, 17, 20, height - 38),
            picture("basic_interface/itemwin_right.bmp")
        ));
        window.push_str(&format!(
            r#"<div class="a" style="{};background:url('{}') repeat-y"></div>"#,
            at(40, 17, 32, height - 38),
            picture("basic_interface/itemwin_mid.bmp")
        ));
        window.push_str(&button_bar(width, height));
        for (row, skill) in skills.iter().take(7).enumerate() {
            window.push_str(&self.skill_row(row, skill, picked == Some(row), points.unwrap_or(0) > 0, &does));
        }
        if let Some(points) = points {
            window.push_str(&text(16, height - 20, BLACK, &Self::words(284, "Skill Point : %d").replace("%d", &points.to_string())));
        }
        window.push_str(&picture_press(width - 107, height - 24, "btn_use", does.uses));
        window.push_str(&picture_press(width - 61, height - 24, "btn_close", does.close));
        window
    }

    /// One skill of the list. The client lets the level be picked for the skills its tables mark so
    /// (`CSkillInfo::UsesLevel`); those tables are not read here, and every skill that is used has the arrows.
    fn skill_row(&self, row: usize, skill: &Skill, picked: bool, points: bool, does: &Listed) -> String {
        let top = row as i32 * 32;
        let name = data::skill_name(skill.id, || format!("{} {}", tr("Skill"), skill.id));
        let usable = skill.level > 0 && !skill.passive();
        let used = self.skills.use_level.get(&skill.id).copied().filter(|used| (1..=skill.level).contains(used)).unwrap_or(skill.level);
        let mut line = String::new();
        // The whole row is pressed to pick it; the band of the picked one is 164 by 28 from 104, 19.
        line.push_str(&format!(r#"<div class="a" style="{};cursor:default" {}></div>"#, at(40, top + 17, 228, 32), pressed((does.pick)(row))));
        if picked {
            line.push_str(&format!(r#"<div class="a" style="{};background:#c6d6f7;pointer-events:none"></div>"#, at(104, top + 19, 164, 28)));
        }
        let handles = if usable {
            format!("{} {}", dragged(Drag::Skill(Leveled { id: u32::from(skill.id), level: used })), super::used(Use::Skill(skill.id)))
        } else {
            String::new()
        };
        let icon = Self::skill_icon(skill.id);
        line.push_str(&format!(
            r#"<div class="a" style="{}" {handles} {} data-skill="{}" title="{}">{icon}</div>"#,
            at(44, top + 21, 24, 24),
            pressed((does.pick)(row)),
            skill.id,
            escape(&name)
        ));
        // A skill not learned yet is named in grey (120) and says nothing more.
        line.push_str(&format!(
            r#"<div class="a t" style="{};color:{};white-space:nowrap;overflow:hidden;pointer-events:none">{}</div>"#,
            at(112, top + 22, 148, 14),
            if skill.level == 0 { "#787878" } else { BLACK },
            escape(&name)
        ));
        if skill.level > 0 {
            if usable {
                line.push_str(&text(124, top + 34, BLACK, "Lv :"));
                line.push_str(&format!(
                    r#"<div class="a t" style="left:149px;top:{}px;color:{};white-space:pre;pointer-events:none">{used:2}</div>"#,
                    top + 34,
                    if used == skill.level { BLACK } else { RED }
                ));
                line.push_str(&format!(
                    r#"<div class="a t" style="left:168px;top:{}px;white-space:pre;pointer-events:none">/ {:2}</div>"#,
                    top + 34,
                    skill.level
                ));
                for (arrow, step, x) in [("arw_left", Step::Less, 112), ("arw_right", Step::More, 192)] {
                    let file = format!("basic_interface/{arrow}.bmp");
                    line.push_str(&pictured(x, top + 34, (11, 11), [&file, &file, &file], (does.level)(LevelStep { skill: skill.id, step }), ""));
                }
            } else {
                line.push_str(&text(113, top + 34, BLACK, &format!("Lv : {}", skill.level)));
            }
            // The SP of the level it is used at (`UISkillListWnd`: `UseSpAmount` of that level where it has
            // a figure, else the server's), by the rules the server plays (`crate::skills::sp_shown`).
            let costs = if skill.passive() {
                Self::words(286, "Passive")
            } else {
                format!("Sp : {}", crate::skills::sp_shown(data::skill_sp(skill.id, used), skill.sp))
            };
            line.push_str(&text(212, top + 34, BLACK, &costs));
        }
        if let (Some(raise), true, true) = (does.raise, points, skill.raisable) {
            line.push_str(&pictured_by(Convention::ABC, 78, top + 21, (24, 24), "basic_interface/skill_up", raise(skill.id), ""));
        }
        line
    }
}
