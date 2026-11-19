//! The guild window's tabs after the first: the members (`UIGuildMemberManageWnd`), the positions
//! (`UIGuildPositionManageWnd`), the skills (`UIGuildSkillWnd`), the expelled (`UIGuildBanishedMemberWnd`) and
//! the notice (`UIGuildNoticeWnd`). Every place and size is the client's, from each one's `OnCreate` and `Paint`.

use super::*;
use crate::net::guild::Position;

/// The widths of the columns of the members, the positions and the expelled (`g_guildMemberColumnWidths`,
/// `g_guildPositionColumnWidths`, `g_guildBanishedColumnWidths`).
const MEMBER_COLUMNS: [i32; 6] = [98, 65, 53, 39, 39, 76];
const POSITION_COLUMNS: [i32; 6] = [50, 100, 60, 60, 60, 60];
const EXPELLED_COLUMNS: [i32; 2] = [82, 292];
/// A member's row: 36 and the 8 the client adds (`m_rowSpacing`), five of them in sight.
const MEMBER_ROW: i32 = 44;
/// The colour of the header the members are ordered by (the client's 0xEBD8CE, blue first).
const SORTED: &str = "#ced8eb";
/// The grey of the fields typed into (242, 242, 242).
const FIELD: &str = "#f2f2f2";

/// A table of rows 19 high (`UIGuildPositionManageWnd::Paint`, `UIGuildBanishedMemberWnd::Paint`): the grey it
/// lies on, twelve rows of white cells with a pixel of the grey between each two, the headers in the first,
/// and under it what the rows hold, eleven in sight. The grey ends 18 from the window's right edge: the
/// client's scroll bar stands there.
fn table(widths: &[i32], headers: &[String], rows: &[String]) -> String {
    let mut table = fill(4, 45, WIDTH - 22, HEIGHT - 19 - 69, LINE);
    for row in 0..12 {
        let mut x = 0;
        for width in widths {
            table.push_str(&fill(x + 5, row * 19 + 46, *width, 18, PANEL));
            x += width + 1;
        }
    }
    let mut x = 0;
    for (header, width) in headers.iter().zip(widths) {
        table.push_str(&text(x + 8, 49, BLACK, header));
        x += width + 1;
    }
    let inside: String = rows.iter().enumerate().map(|(row, inside)| format!(r#"<div style="position:absolute;left:0;top:{}px;width:{}px;height:19px">{inside}</div>"#, row * 19, WIDTH - 18)).collect();
    table.push_str(&format!(r#"<div class="a scroll" style="{};overflow-x:hidden"><div style="position:relative;height:{}px">{inside}</div></div>"#, at(0, 65, WIDTH - 6, 11 * 19), rows.len() * 19));
    table
}

impl App {
    /// The second tab (`UIGuildMemberManageWnd::Paint`): the frame with its lines, the headers (pressed, each
    /// orders the list by its column, and the one it is ordered by is tinted and has an arrow after its
    /// word), the members a row of 44 each, five in sight; under them the box that lists those in the game
    /// first, and for the master, once a member was given another position, the button that sends it
    /// (`btn_ok`, 61 from the right and 23 from the bottom).
    pub(super) fn guild_members(&self) -> String {
        let mut tab = format!(r#"<div class="a" style="{};border:1px solid {LINE};box-sizing:border-box"></div>"#, at(5, 46, 377, 241));
        tab.push_str(&fill(5, 66, 377, 1, LINE));
        let headers = [(407, "Name"), (503, "Position"), (504, "Job"), (408, "Level"), (506, "Tax"), (507, "Contribution")];
        let mut x = 0;
        for (column, (line, otherwise)) in headers.into_iter().enumerate() {
            if column > 0 {
                tab.push_str(&fill(x + 5, 46, 1, 241, LINE));
            }
            let arrow = match self.guild.sort {
                Some((by, down)) if by == column => format!(r#"<img alt="" draggable="false" src="{}" style="margin-top:2px">"#, picture(if down { "guild/guild_arrow_down.bmp" } else { "guild/guild_arrow_up.bmp" })),
                _ => String::new(),
            };
            tab.push_str(&format!(
                r#"<div class="a t" style="{};padding:4px 0 0 4px;display:flex;align-items:flex-start;overflow:hidden;cursor:default;color:{BLACK};background:{}" {}>{}{arrow}</div>"#,
                at(x + 6, 47, MEMBER_COLUMNS[column], 19),
                if arrow.is_empty() { "transparent" } else { SORTED },
                pressed(Press::Sort(column)),
                escape(&Self::words(line, otherwise))
            ));
            x += MEMBER_COLUMNS[column] + 1;
        }
        let members = self.guild_listed();
        let mut rows: String = members.iter().enumerate().map(|(row, member)| self.guild_member_row(row, member)).collect();
        rows.push_str(&self.guild_position_combo());
        tab.push_str(&format!(r#"<div class="a scroll" style="{};overflow-x:hidden"><div style="position:relative;height:{}px">{rows}</div></div>"#, at(6, 67, WIDTH - 12, 5 * MEMBER_ROW), members.len() as i32 * MEMBER_ROW));
        tab.push_str(&check_words(12, HEIGHT - 18, self.guild.online_first, &Self::words(2864, "Show guild member login status"), Press::OnlineFirst));
        if self.guild.master && !self.guild.member_moves.is_empty() {
            tab.push_str(&picture_press(WIDTH - 61, HEIGHT - 23, "btn_ok", Press::MembersOk));
        }
        tab
    }

    /// A member's row (`UIGuildMemberManageWnd::DrawMember`), 375 by 43 over its line: pale green for one in
    /// the game, the pick's colour for the one picked. The places are the client's, counted from the row's own
    /// corner (the window's 6, 67): the head of one in the game, its middle 22 in and its top 3 down; the name
    /// 37, 12, fitted into 51; the position 103, 16, fitted into 49, and for the master the button that opens
    /// the positions to pick from 144, 12 (it lies over the position's end in the client too); the job 169, 16,
    /// fitted into 39; the level, the part of its experience the member gives and how much that came to,
    /// each ending 4 before its column does; and under them, in the smaller text and grey 64, when the member
    /// was last in the game, 2, 33. That last line is not cut at the column, nor is it in the client: with the
    /// English words of line 3012 it runs past the first column.
    ///
    /// The part given is counted against the guild's experience as the server last told it; how the client
    /// counts its `expRatio` was not found. The head is the hair's sprite without its colour (the client takes
    /// the palette of the hair's colour).
    fn guild_member_row(&self, row: usize, member: &Member) -> String {
        let ground = match (self.guild.picked == Some(row), member.online) {
            (true, _) => MARK,
            (false, true) => "#e2ffe4",
            (false, false) => "transparent",
        };
        let whole = self.guild.info.as_ref().map_or(0, |info| info.exp);
        let part = if whole > 0 { f64::from(member.given) * 100.0 / f64::from(whole) } else { 0.0 };
        let seen = if member.seen == 0 {
            String::new()
        } else {
            let date = js_sys::Date::new(&(f64::from(member.seen) * 1000.0).into());
            Self::words(3011, "%y.%m.%d").replace("%y", &format!("{:02}", date.get_full_year() % 100)).replace("%m", &format!("{:02}", date.get_month() + 1)).replace("%d", &format!("{:02}", date.get_date()))
        };
        let head = match self.guild.heads.get(&crate::web::figure::head_sprite(member.job, member.sex as u8, member.hair)).filter(|_| member.online) {
            Some(Some((picture, (width, _)))) => format!(r#"<img alt="" draggable="false" src="{picture}" style="position:absolute;left:{}px;top:3px;pointer-events:none">"#, 22 - *width as i32 / 2),
            _ => String::new(),
        };
        let combo = if self.guild.master { pictured(144, 12, (18, 18), ["basic_interface/txtbox_btn_a.bmp", "basic_interface/txtbox_btn_b.bmp", "basic_interface/txtbox_btn_c.bmp"], Press::Combo(row), "") } else { String::new() };
        // The columns start 6 into the window, as the rows do.
        let right = |edge: i32, words: String| text_ending(edge - 6, 375, 16, BLACK, &words);
        [
            format!(r#"<div style="position:absolute;left:0;top:{}px;width:375px;height:44px;border-bottom:1px solid {LINE};background:{ground}" {} {}>"#, row as i32 * MEMBER_ROW, pressed(Press::Member(row)), menued(Menu::GuildMember(row))),
            head,
            fitted(37, 12, MEMBER_COLUMNS[0] / 2 + 2, BLACK, &member.name),
            fitted(MEMBER_COLUMNS[0] + 5, 16, MEMBER_COLUMNS[1] - 16, BLACK, &self.guild.position_name(self.guild.position_of(member))),
            combo,
            fitted(MEMBER_COLUMNS[0] + MEMBER_COLUMNS[1] + 6, 16, 39, BLACK, &data::job_name(member.job)),
            right(260, member.level.to_string()),
            right(300, format!("{part:3.0}%")),
            right(377, member.given.to_string()),
            text_in(SMALL, 2, 33, "#404040", &Self::words(3012, "Access date: %s").replacen("%s", &seen, 1)),
            "</div>".to_owned(),
        ]
        .concat()
    }

    /// The positions to pick from for a member, opened by the button in its row (`ID_OPEN_MEMBER_POS_COMBO`):
    /// a list 150 wide and no taller than five rows of 16, 88 into the window and under the row.
    fn guild_position_combo(&self) -> String {
        let Some(row) = self.guild.combo else { return String::new() };
        let options: String = self.guild.positions.iter().map(|position| menu_row(&position.name, Press::Position(position.id))).collect();
        format!(
            r#"<div class="viewer" style="position:absolute;left:82px;top:{}px;width:150px;height:{}px;background:#fff;border:1px solid #9a9a9a;z-index:2">{options}</div>"#,
            row as i32 * MEMBER_ROW + MEMBER_ROW - 2,
            (self.guild.positions.len() as i32 * 16).clamp(16, 80) + 2
        )
    }

    /// The third tab (`UIGuildPositionManageWnd::Paint`): each position's number, what it is called, whether it
    /// invites, expels and opens the guild's storage, and what part of a member's experience it takes. For the
    /// master the name and the tax are fields (30 and 2 letters, on grey 242) and the boxes are pressed; once
    /// something was changed the button that sends it shows (`btn_ok`, 61 from the right, 23 from the bottom).
    pub(super) fn guild_positions(&self) -> String {
        let headers = [(510, "Rank"), (511, "Position Title"), (512, "Invite"), (513, "Expel"), (2499, "Storage"), (514, "Tax %")].map(|(line, otherwise)| Self::words(line, otherwise));
        let rows: Vec<String> = self.guild.positions.iter().map(|position| self.guild_position_row(&self.guild.position_shown(position))).collect();
        let mut tab = table(&POSITION_COLUMNS, &headers, &rows);
        if self.guild.master {
            let hidden = if self.guild.position_edits.is_empty() { " hidden" } else { "" };
            tab.push_str(&format!(r#"<div id="guild-positions-ok"{hidden}>{}</div>"#, picture_press(WIDTH - 61, HEIGHT - 23, "btn_ok", Press::PositionsOk)));
        }
        tab
    }

    /// A position's row, its places counted from the row's corner (the window's 0, 65): the number 8, 3; the
    /// name 59, 1 in 96; the boxes 160, 221 and 282, 3; the tax 343, 1 (for the master a field 26 wide, with
    /// the sign of hundredths 373, 3).
    fn guild_position_row(&self, position: &Position) -> String {
        let [number, name, invite, expel, storage] = [POSITION_COLUMNS[0], POSITION_COLUMNS[1], POSITION_COLUMNS[2], POSITION_COLUMNS[3], POSITION_COLUMNS[4]];
        let tax = number + name + invite + expel + storage + 13;
        let id = position.id;
        let master = self.guild.master;
        let box_of = |x: i32, on: bool, press: Press| check(x, 3, on, &if master { pressed(press) } else { String::new() });
        // What is typed shows the button that sends it.
        let field = |what: &str, x: i32, width: i32, most: usize, value: &str| {
            format!(
                r#"<input id="guild-{what}-{id}" class="a in" style="{};background:{FIELD}" type="text" maxlength="{most}" autocomplete="off" value="{}" oninput="document.getElementById('guild-positions-ok').hidden=false">"#,
                at(x, 1, width, 16),
                escape(&Self::guild_written(&format!("guild-{what}-{id}")).unwrap_or_else(|| value.to_owned()))
            )
        };
        [
            text(8, 3, BLACK, &id.to_string()),
            if master { field("position", number + 9, name - 4, 30, &position.name) } else { fitted(number + 9, 1, name - 4, BLACK, &position.name) },
            box_of(number + name + 10, position.invites, Press::Invites(id)),
            box_of(number + name + invite + 11, position.expels, Press::Expels(id)),
            box_of(number + name + invite + expel + 12, position.storage, Press::Stores(id)),
            if master { [field("tax", tax, storage / 2 - 4, 2, &position.tax.to_string()), text(tax + POSITION_COLUMNS[5] / 2, 3, BLACK, "%")].concat() } else { text(tax, 1, BLACK, &format!("{} %", position.tax)) },
        ]
        .concat()
    }

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
            format!(r#"<div class="a scroll" style="{};overflow-x:hidden"><div style="position:relative;height:{}px">{rows}</div></div>"#, at(0, 43, WIDTH - 6, 7 * 34), self.guild_skills_listed().len() * 34),
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
        let drag = if skill.level != 0 && skill.kind != 0 { dragged(Drag::Skill(Leveled { id: u32::from(skill.id), level: skill.level })) } else { String::new() };
        let mut line = format!(r#"<div style="position:absolute;left:40px;top:{y}px;width:{}px;height:34px" {} {drag} data-skill="{}">"#, WIDTH - 60, pressed(Press::Skill(row)), skill.id);
        line.push_str(&fill(64, 0, 164, 28, mark));
        line.push_str(&format!(r#"<img class="a" alt="" draggable="false" src="{}" style="left:4px;top:2px;pointer-events:none" onerror="this.remove()">"#, picture(&format!("item/{}.bmp", skill.name.to_lowercase()))));
        line.push_str(&text(72, 3, if skill.level == 0 { GREY } else { BLACK }, &name));
        if skill.level != 0 {
            line.push_str(&text(73, 15, BLACK, &format!("Lv : {}", skill.level)));
            line.push_str(&text(172, 15, BLACK, &if skill.kind == 0 { Self::words(286, "Passive") } else { format!("Sp : {}", skill.sp) }));
        }
        if self.guild.master && self.guild.skill_points != 0 && skill.raisable {
            line.push_str(&pictured(38, 2, (24, 24), ["basic_interface/skill_up_a.bmp", "basic_interface/skill_up_b.bmp", "basic_interface/skill_up_c.bmp"], Press::Raise(skill.id), ""));
        }
        line.push_str("</div>");
        line
    }

    /// The fifth tab (`UIGuildBanishedMemberWnd::Paint`): those expelled, and why: the name 8, 3 into its row,
    /// fitted into 78, the reason 91, 3, fitted into 288. The client has nothing here that takes one off the
    /// list.
    pub(super) fn guild_expelled(&self) -> String {
        let headers = [Self::words(407, "Name"), Self::words(462, "The Reason of Expulsion")];
        let rows: Vec<String> = self.guild.expelled.iter().map(|(name, reason)| [fitted(8, 3, EXPELLED_COLUMNS[0] - 4, BLACK, name), fitted(EXPELLED_COLUMNS[0] + 9, 3, EXPELLED_COLUMNS[1] - 4, BLACK, reason)].concat()).collect();
        table(&EXPELLED_COLUMNS, &headers, &rows)
    }

    /// The sixth tab (`UIGuildNoticeWnd::OnCreate`, `Paint`): the notice's title (a field at 50, 66, the
    /// window's width less 60 by 16, 60 letters) and what it says (a field of many lines at 5, 104, the width
    /// less 10 by 156, 119 letters), both on grey 242; under the text how much of the 119 is used, ending 7
    /// from the right edge at 266; and for the master the button that sends them (`btn_send`, 46 from the
    /// right, 24 from the bottom). There is no other button: the fields are typed into as they stand.
    ///
    /// The count is of bytes, as the client's and the server's limits are; the title is cut to 59 and the text
    /// to 119 as it is sent (`MM_GUILD_NOTICE`, rAthena's `MAX_GUILDMES1` 60 and `MAX_GUILDMES2` 120 with
    /// their closing zero). Decided here: one who is not the master cannot type into them (the client lets
    /// anyone type and sends nothing).
    pub(super) fn guild_notice_tab(&self) -> String {
        let locked = if self.guild.master { "" } else { " readonly" };
        let notice = Self::guild_written("guild-notice").filter(|_| self.guild.master).unwrap_or_else(|| self.guild.notice.clone());
        let subject = Self::guild_written("guild-subject").filter(|_| self.guild.master).unwrap_or_else(|| self.guild.subject.clone());
        let mut tab = [
            text(5, 67, BLACK, &format!("{}:", Self::words(515, "Title"))),
            format!(r#"<input id="guild-subject" class="a in" style="{};background:{FIELD}" type="text" maxlength="59" autocomplete="off" value="{}"{locked}>"#, at(50, 66, WIDTH - 60, 16), escape(&subject)),
            text(5, 86, BLACK, &Self::words(516, "Contents")),
            format!(
                r#"<textarea id="guild-notice" class="a in viewer" style="{};resize:none;background:{FIELD};line-height:15px;white-space:pre-wrap;word-break:break-all" maxlength="119" oninput="document.getElementById('guild-letters').textContent=new TextEncoder().encode(this.value).length"{locked}>{}</textarea>"#,
                at(5, 104, WIDTH - 10, 156),
                escape(&notice)
            ),
            format!(r#"<div class="a t" style="right:7px;top:266px;color:{BLACK}">[<span id="guild-letters">{}</span>/119]</div>"#, notice.len()),
        ]
        .concat();
        if self.guild.master {
            tab.push_str(&picture_press(WIDTH - 46, HEIGHT - 24, "btn_send", Press::Notice));
        }
        tab
    }
}
