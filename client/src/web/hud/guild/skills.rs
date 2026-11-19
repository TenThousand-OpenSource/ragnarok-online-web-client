//! The guild window's fourth tab: the guild's skills (`UIGuildSkillWnd`). Every place and size is the
//! client's, from its `OnCreate` and `Paint`.

use super::*;

impl App {
    /// The guild's skills as its window lists them (`UIGuildSkillWnd`, `UM_REFRESHITEM`): those learnt and
    /// those that can be raised.
    pub(in crate::web) fn guild_skills_listed(&self) -> Vec<&crate::net::guild::Skill> {
        self.guild.skills.iter().filter(|skill| skill.raisable || skill.level != 0).collect()
    }

    /// The fourth tab (`UIGuildSkillWnd::Paint`, `OnCreate`, `UpdateSkillButtons`): each skill 34 high with its
    /// picture 44 in, its name 112 in (grey 120 for one not learnt), under the name its level and what it
    /// costs or that it is passive (only for one learnt); the one picked on its mark (164 by 28, 104 in: grey
    /// for one not learnt, the paler blue for a passive one); for the master, while the guild has points, the
    /// button that raises one that can be raised (`skill_up_a.bmp`, 78 in); at the bottom the buttons that use
    /// the skill picked and close the window (46 and 92 in, 24 from the bottom) and the points left to give.
    ///
    /// A learnt skill that is used can be dragged from here to the hotkey bar, as in the client
    /// (`UIGuildSkillWnd::OnLBtnDown`). The right button on a skill shows its description at once, as in this
    /// client's skill window; the client opens a menu of the one row "View Skill Info" first.
    pub(super) fn guild_skills(&self) -> String {
        let rows: String = self.guild_skills_listed().iter().enumerate().map(|(row, skill)| self.guild_skill_row(row, skill)).collect();
        [
            format!(
                r#"<div class="a scroll" style="{};overflow-x:hidden"><div style="position:relative;height:{}px">{rows}</div></div>"#,
                at(0, 43, WIDTH - 6, 7 * 34),
                self.guild_skills_listed().len() * 34
            ),
            picture_press(46, HEIGHT - 24, "btn_use", Press::UseSkill),
            picture_press(92, HEIGHT - 24, "btn_close", Press::Window),
            text(WIDTH - 100, HEIGHT - 20, BLACK, &format!("SkillPoint : {}", self.guild.skill_points)),
        ]
        .concat()
    }

    /// A skill's row, its places counted from the row's corner (the window's 0, 43).
    fn guild_skill_row(&self, row: usize, skill: &crate::net::guild::Skill) -> String {
        let y = row as i32 * 34;
        let name = data::skill(skill.id).map(|(_, name, _)| name).filter(|name| !name.is_empty()).unwrap_or_else(|| skill.name.clone());
        let mark = match (self.guild.skill == Some(row), skill.level, skill.sp) {
            (false, ..) => "transparent",
            (true, 0, _) => STRIP,
            (true, _, 0) => PASSIVE_MARK,
            (true, ..) => MARK,
        };
        // Only one learnt that is used goes onto the hotkey bar.
        let drag = if skill.level != 0
            && skill.kind != 0 { dragged(Drag::Skill(Leveled { id: u32::from(skill.id), level: skill.level })) } else { String::new() };
        let mut line = format!(
            r#"<div style="position:absolute;left:40px;top:{y}px;width:{}px;height:34px" {} {drag} data-skill="{}">"#,
            WIDTH - 60,
            pressed(Press::Skill(row)),
            skill.id
        );
        line.push_str(&fill(64, 0, 164, 28, mark));
        line.push_str(&[
            format!(
                r#"<img class="a" alt="" draggable="false" src="{}" style="left:4px;top:2px;pointer-events:none" "#,
                picture(&format!("item/{}.bmp", skill.name.to_lowercase()))
            ),
            format!(r#"onerror="this.remove()">"#)
        ].concat());
        line.push_str(&text(72, 3, if skill.level == 0 { GREY } else { BLACK }, &name));
        if skill.level != 0 {
            line.push_str(&text(73, 15, BLACK, &format!("Lv : {}", skill.level)));
            line.push_str(&text(172, 15, BLACK, &if skill.kind == 0 { Self::words(286, "Passive") } else { format!("Sp : {}", skill.sp) }));
        }
        if self.guild.master && self.guild.skill_points != 0 && skill.raisable {
            line.push_str(&pictured_by(Convention::ABC, 38, 2, (24, 24), "basic_interface/skill_up", Press::Raise(skill.id), ""));
        }
        line.push_str("</div>");
        line
    }
}
