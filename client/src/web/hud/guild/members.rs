//! The guild window's second tab: the members (`UIGuildMemberManageWnd`). Every place and size is the
//! client's, from its `OnCreate` and `Paint`.

use super::*;

/// The widths of the columns of the members (`g_guildMemberColumnWidths`).
const MEMBER_COLUMNS: [i32; 6] = [98, 65, 53, 39, 39, 76];
/// A member's row: 36 and the 8 the client adds (`m_rowSpacing`), five of them in sight.
const MEMBER_ROW: i32 = 44;
/// The colour of the header the members are ordered by (the client's 0xEBD8CE, blue first).
const SORTED: &str = "#ced8eb";

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
                Some((by, down)) if by == column => format!(
                    r#"<img alt="" draggable="false" src="{}" style="margin-top:2px">"#,
                    picture(if down { "guild/guild_arrow_down.bmp" } else { "guild/guild_arrow_up.bmp" })
                ),
                _ => String::new(),
            };
            tab.push_str(&[
                format!(
                    r#"<div class="a t" style="{};padding:4px 0 0 4px;display:flex;align-items:flex-start;overflow:hidden;"#,
                    at(x + 6, 47, MEMBER_COLUMNS[column], 19)
                ),
                format!(
                    r#"cursor:default;color:{BLACK};background:{}" {}>{}{arrow}</div>"#,
                    if arrow.is_empty() { "transparent" } else { SORTED },
                    pressed(Press::Sort(column)),
                    escape(&Self::words(line, otherwise))
                )
            ].concat());
            x += MEMBER_COLUMNS[column] + 1;
        }
        let members = self.guild_listed();
        let mut rows: String = members.iter().enumerate().map(|(row, member)| self.guild_member_row(row, member)).collect();
        rows.push_str(&self.guild_position_combo());
        tab.push_str(&format!(
            r#"<div class="a scroll" style="{};overflow-x:hidden"><div style="position:relative;height:{}px">{rows}</div></div>"#,
            at(6, 67, WIDTH - 12, 5 * MEMBER_ROW),
            members.len() as i32 * MEMBER_ROW
        ));
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
            Self::words(3011, "%y.%m.%d")
                .replace("%y", &format!("{:02}", date.get_full_year() % 100))
                .replace("%m", &format!("{:02}", date.get_month() + 1))
                .replace("%d", &format!("{:02}", date.get_date()))
        };
        let head = match self.guild.heads.get(&crate::web::figure::head_sprite(member.job, member.sex as u8, member.hair)).filter(|_| member.online) {
            Some(Some((picture, (width, _)))) => format!(
                r#"<img alt="" draggable="false" src="{picture}" style="position:absolute;left:{}px;top:3px;pointer-events:none">"#,
                22 - *width as i32 / 2
            ),
            _ => String::new(),
        };
        let combo = if self.guild.master {
            pictured_by(Convention::ABC, 144, 12, (18, 18), "basic_interface/txtbox_btn", Press::Combo(row), "")
        } else {
            String::new()
        };
        // The columns start 6 into the window, as the rows do.
        let right = |edge: i32, words: String| text_ending(edge - 6, 375, 16, BLACK, &words);
        [
            [
                format!(r#"<div style="position:absolute;left:0;top:{}px;width:375px;height:44px;border-bottom:1px solid "#, row as i32 * MEMBER_ROW),
                format!(r#"{LINE};background:{ground}" {} {}>"#, pressed(Press::Member(row)), menued(Menu::GuildMember(row)))
            ].concat(),
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
        [
            format!(
                r#"<div class="viewer" style="position:absolute;left:82px;top:{}px;width:150px;height:{}px;"#,
                row as i32 * MEMBER_ROW + MEMBER_ROW - 2,
                (self.guild.positions.len() as i32 * 16).clamp(16, 80) + 2
            ),
            format!(r#"background:#fff;border:1px solid #9a9a9a;z-index:2">{options}</div>"#)
        ].concat()
    }
}
