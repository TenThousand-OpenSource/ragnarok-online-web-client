//! The guild's windows: the guild window with its six tabs (the client's `UIGuildWnd` and what is made of it:
//! `UIGuildInfoManageWnd` here, and a file to each of the others in `guild/`: `UIGuildMemberManageWnd`,
//! `UIGuildPositionManageWnd`, `UIGuildSkillWnd`, `UIGuildBanishedMemberWnd`, `UIGuildNoticeWnd`), the table
//! two of them are laid out in, the menus the right button opens in it, the castles held (`guild/castles.rs`),
//! and what the guild asks (`UIJoinGuildAcceptWnd`, `UIAllyGuildAcceptWnd`, `UIGuildLeaveReasonDescWnd`,
//! `UIGuildBanReasonDescWnd`).
//!
//! The window is 400 by 317 (`g_defaultPlacementWidth`, `g_defaultPlacementHeight`) and has no words in its
//! title bar: the client draws it with an empty title (`UIGuildWnd::Paint`: `DrawTitleBar(false, "", 0)`). Its
//! grounds are colours the client reads out of `colorchip.bmp` (`CSkinMgr::PaletteChannels`), written out here
//! as that file has them: the strip the tabs stand on (10, 2), a tab that is open and the panel (2, 2), a tab
//! that is not and the lines (14, 6), what a pick marks (6, 2), a skill picked that is not used (14, 2).

use super::*;
use crate::net::guild::Member;
use crate::press::guild::{Entry, Press};
use crate::press::things::Menu;
use crate::web::app::{GuildAsk, GuildMenu};
use crate::web::parts::menued;

mod castles;
mod expelled;
mod members;
mod notice;
mod positions;
mod skills;

pub(in crate::web) use castles::{CASTLE_ASK, CASTLE_TOLD};

pub(in crate::web) const WIDTH: i32 = 400;
pub(in crate::web) const HEIGHT: i32 = 317;
const STRIP: &str = "#b4b4b4";
const PANEL: &str = "#fff";
const LINE: &str = RIM;
const MARK: &str = "#709fed";
const PASSIVE_MARK: &str = "#70d7ed";
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
    let inside: String = rows.iter()
        .enumerate()
        .map(|(row, inside)| format!(
            r#"<div style="position:absolute;left:0;top:{}px;width:{}px;height:19px">{inside}</div>"#,
            row * 19,
            WIDTH - 18
        ))
        .collect();
    table.push_str(&format!(
        r#"<div class="a scroll" style="{};overflow-x:hidden" {}><div style="position:relative;height:{}px">{inside}</div></div>{}"#,
        at(0, 65, WIDTH - 6, 11 * 19),
        rows_stepped(19, 11),
        rows.len() * 19,
        // The tab's bar (`UIGuildWnd.cpp`, the tabs' `OnCreate`: `Configure(width - 15, 63, 1, height - 87)`).
        scroll_bar(WIDTH - 15, 63, 11 * 19, 19)
    ));
    table
}

impl App {
    /// The guild window: the title bar with its cross, the six tabs on their strip, the panel, and what the
    /// tab that is open shows (`UIGuildWnd::Paint`, `CreateTabs`: each tab 64 wide and 66 from the last, its
    /// words fitted into 62). The bar the client draws along the bottom (`DrawButtonBar(2, 0, 1)`) is taken to
    /// be the one every window's buttons stand on.
    ///
    /// This project's own, asked for by its owner: a tab the server does not let the player open is written in
    /// grey and is no button (the pointer over it stays the plain one). The client draws all six alike and
    /// does nothing when such a tab is pressed (`UM_CHANGE_GUILD_TAB`).
    pub(in crate::web) fn guild_window(&self) -> String {
        let mut window = title_bar(WIDTH, "");
        window.push_str(&cross(WIDTH, Press::Window));
        window.push_str(&fill(0, 17, WIDTH, 24, STRIP));
        let tabs = [(340, "Guild Info"), (341, "Member"), (342, "Position"), (343, "Guild Skill"), (344, "Expel List"), (345, "Announcement")];
        for (tab, (line, otherwise)) in tabs.into_iter().enumerate() {
            let x = tab as i32 * 66;
            let opens = self.guild_tab_opens(tab);
            let press = if opens { pressed(Press::Tab(tab)) } else { String::new() };
            window.push_str(&format!(
                r#"<div class="a" style="{};background:{}" {press}>{}</div>"#,
                at(x + 1, 18, 64, 23),
                if self.guild.tab == tab { PANEL } else { LINE },
                fitted(2, 4, 62, if opens { BLACK } else { GREY }, &Self::words(line, otherwise))
            ));
        }
        window.push_str(&fill(0, 41, WIDTH, HEIGHT - 41, LINE));
        window.push_str(&fill(1, 41, WIDTH - 2, HEIGHT - 42, PANEL));
        window.push_str(&button_bar(WIDTH, HEIGHT));
        window.push_str(&match self.guild.tab {
            0 => self.guild_info(),
            1 => self.guild_members(),
            2 => self.guild_positions(),
            3 => self.guild_skills(),
            4 => self.guild_expelled(),
            _ => self.guild_notice_tab(),
        });
        window
    }

    /// The picture of a guild's emblem at a place, 24 pixels each way; nothing until it is read.
    fn emblem_at(&self, guild: u32, version: u32, x: i32, y: i32) -> String {
        match self.guild.emblems.get(&(guild, version)).filter(|picture| !picture.is_empty()) {
            Some(picture) => format!(r#"<img class="a" alt="" draggable="false" src="{picture}" style="{};image-rendering:pixelated">"#, at(x, y, 24, 24)),
            None => String::new(),
        }
    }

    /// The first tab (`UIGuildInfoManageWnd::Paint`): the guild's name, level, master, how many are in it and
    /// how many of them in the game, their average level, its castles; at the right its experience, its
    /// emblem with the master's button that changes it (`btn_edit`, 8 from the right edge, 69 down), and the
    /// guilds allied and opposed.
    ///
    /// How many are in the game is written as the client writes it for a service that is not Korea's: the
    /// sign `grp_online.bmp` 8 after the words of the members and 2 over their line, and the number after it.
    ///
    /// The castles are the list of `guild/castles.rs`, which lies over what the server words of them.
    fn guild_info(&self) -> String {
        let guild = &self.guild;
        let info = guild.info.clone().unwrap_or_default();
        let name = if info.name.is_empty() { guild.name.clone() } else { info.name.clone() };
        let line = |words: (usize, &str), value: String| format!("{} : {value}", Self::words(words.0, words.1));
        // How many are in the game is counted from the members once they are known: the server tells of each
        // coming and going, and of the guild's figures only when asked.
        let online = if guild.members.is_empty() { info.online as usize } else { guild.members.iter().filter(|member| member.online).count() };
        let mut tab = [
            text(8, 49, BLACK, &line((328, "Guild Name"), name)),
            text(8, 65, BLACK, &line((329, "Guild Level"), info.level.to_string())),
            text(8, 81, BLACK, &line((330, "Guild Master"), info.master.clone())),
            [
                format!(
                    r#"<div class="a t" style="left:8px;top:97px;display:flex;color:{BLACK}">{}"#,
                    escape(&line((331, "Members"), format!("{} / {}", guild.members.len().max(online), info.most)))
                ),
                format!(
                    r#"<img alt="" draggable="false" src="{}" style="width:15px;height:15px;margin:-2px 0 0 8px">"#,
                    picture("basic_interface/grp_online.bmp")
                ),
                format!(r#"<span style="white-space:pre"> {}</span></div>"#, online)
            ].concat(),
            text(8, 113, BLACK, &line((332, "Avg.lvl of Members"), info.average_level.to_string())),
            self.guild_castles(),
            // The experience is written in red from level 50 (the client's 0xFF there).
            text(200, 49, if info.level >= 50 { "#ff0000" } else { BLACK }, &line((335, "EXP"), info.exp.to_string())),
            text(200, 73, BLACK, &Self::words(336, "Emblem")),
            text(200, 139, BLACK, &Self::words(338, "Alliances")),
            text(200, 213, BLACK, &Self::words(339, "Antagonists")),
            fill(299, 64, 26, 26, MARK),
            self.emblem_at(guild.id, guild.emblem, 300, 65),
        ]
        .concat();
        tab.push_str(&self.guild_related());
        if guild.master {
            tab.push_str(&picture_press(WIDTH - 42 - 8, 69, "btn_edit", Press::Emblem));
            // The button that breaks the guild up, 15 from the right and 5 from the bottom.
            tab.push_str(&picture_press_sized(WIDTH - 64 - 15, HEIGHT - 20 - 5, (64, 20), "btn_disBand", Press::Disband));
        }
        tab
    }

    /// The guilds allied and the guilds opposed, each list on its grey, three rows of 16 (`UIGuildInfoManageWnd::
    /// Paint`). The right button picks a row over the width the client takes it on, 201 to 287, whether a
    /// guild is in the row or not (`OnRBtnUp`), and the row picked that has a guild stays marked, 80 wide from
    /// 204, until another is picked or the tab is left. The left button and the pointer over a row do nothing
    /// there, as in the client; the rules are `crate::guild`'s.
    fn guild_related(&self) -> String {
        use crate::guild::{RELATED_MARK, RELATED_PICK, RELATED_ROW, RELATED_ROWS, related_place, related_slot};
        let mut lists = String::new();
        for (opposed, top) in [(false, 155), (true, 229)] {
            lists.push_str(&fill(200, top, 168, 48, LINE));
            for row in 0..RELATED_ROWS {
                let place = related_place(self.guild.related.iter().map(|(.., against)| *against), opposed, row);
                let name = place.and_then(|place| self.guild.related.get(place)).map(|(_, name, _)| name.as_str());
                let marked = name.is_some() && self.guild.related_row == Some((opposed, row));
                lists.push_str(&format!(
                    r#"<div class="a" style="{}" {}><div class="a t" style="{};overflow:visible;color:{BLACK};background:{}">{}</div></div>"#,
                    at(RELATED_PICK.0, top + 2 + row as i32 * RELATED_ROW, RELATED_PICK.1, RELATED_ROW),
                    menued(Menu::GuildRelated(related_slot(opposed, row))),
                    at(RELATED_MARK.0 - RELATED_PICK.0, 0, RELATED_MARK.1, RELATED_ROW),
                    if marked { MARK } else { "transparent" },
                    escape(name.unwrap_or_default())
                ));
            }
        }
        lists
    }

    /// The menu the right button opened in the guild window: on a member (`guild_member_menu`), or on a guild
    /// allied or opposed, where the master has the one row that ends it (`UIGuildInfoManageWnd`, `UM_MAKEMENU`:
    /// line 351, "Delete").
    pub(in crate::web) fn guild_menu(&self) -> String {
        match self.guild.menu {
            Some((_, GuildMenu::Member(row))) => self.guild_listed().get(row).map(|member| self.guild_member_menu(member)).unwrap_or_default(),
            Some((_, GuildMenu::Related(_))) if self.guild.master => menu_row(&Self::words(351, "Delete"), Press::Do(Entry::Unrelate)),
            _ => String::new(),
        }
    }

    /// The menu on a member (`UIGuildMemberManageWnd`, `UM_MAKEMENU`): on the player's own row leaving the
    /// guild, unless the player is its master; on another's, for the master expelling and handing the guild
    /// over (line 2923), for one whose position expels expelling (never the master); then for one in the game
    /// the invitation into the player's party (for its leader, when the member is not in it) and the talk
    /// between two; and a letter.
    fn guild_member_menu(&self, member: &Member) -> String {
        let master = self.guild.master_character;
        if member.character == self.own_character() {
            return if member.character == master { String::new() } else { menu_row(&Self::words(508, "Leave Guild"), Press::Do(Entry::Leave)) };
        }
        let mut rows = String::new();
        if master == self.own_character() {
            rows.push_str(&menu_row(&Self::words(509, "Expel"), Press::Do(Entry::Expel)));
            rows.push_str(&menu_row(&Self::words(2923, "Assign Guild Leader"), Press::Do(Entry::Hand)));
        } else if self.guild.expels && member.character != master {
            rows.push_str(&menu_row(&Self::words(509, "Expel"), Press::Do(Entry::Expel)));
        }
        if member.online {
            if self.leads_party() && !self.party.members.iter().any(|known| known.name == member.name) {
                rows.push_str(&menu_row(&Self::words(1330, "Invite to party"), Press::Do(Entry::Party)));
            }
            rows.push_str(&menu_row(&Self::words(2043, "Open 1:1 Chat"), Press::Do(Entry::Chat)));
        }
        rows.push_str(&menu_row(&Self::words(2808, "Send a mail"), Press::Do(Entry::Mail)));
        rows
    }

    /// How large the box of what the guild asks is: the client's message box, 280 by 120, for a question
    /// answered with OK or Cancel; 280 by 80 for the reason of leaving or expelling
    /// (`UIWindowMgr::EnsureWindow`); the two boxes of the castles have their own sizes (`guild/castles.rs`).
    pub(in crate::web) fn guild_ask_size(&self) -> (i32, i32) {
        match self.guild.asks {
            Some(GuildAsk::Leave | GuildAsk::Expel(..)) => (280, 80),
            Some(GuildAsk::Castle(_)) => CASTLE_ASK,
            Some(GuildAsk::Invested(_)) => CASTLE_TOLD,
            _ => (280, 120),
        }
    }

    /// What the guild asks. In the client's message box, as the party's invitation is: a guild's invitation
    /// (its name in brackets, then line 377), another guild's wish for an alliance (the same with line 393,
    /// `UIAllyGuildAcceptWnd`), and whether the guild is handed to a member (line 2924, into which the client
    /// writes the member's name twice). The reason of leaving or of expelling in a box of its own
    /// (`UIGuildLeaveReasonDescWnd::OnCreate`): the words 15,6, the reason typed 15,22 on grey 232 with 39
    /// letters at most, OK and Cancel 101 and 55 from the right, 32 from the bottom. Its ground is the
    /// client's `DrawBorder`, taken here to be white with a grey rim.
    pub(in crate::web) fn guild_question(&self) -> String {
        if let Some(castles) = self.guild_castle_box() {
            return castles;
        }
        let (width, height) = self.guild_ask_size();
        let boxed = |words: String| message_box(&escape(&words), Press::Yes, Press::No);
        let words = match &self.guild.asks {
            // (The guild's name with stars for a word that is not let be written: `UIJoinGuildAcceptWnd`.)
            Some(GuildAsk::Invited(_, name)) => return boxed(
                format!("({}){}", data::starred(name), Self::words(377, " Guild has sent you an invitation. Would you like to join this Guild?"))
            ),
            Some(GuildAsk::Ally(_, name)) =>
                return boxed(format!("({name}){}", Self::words(393, " Guild is asking for an Alliance with them. Do you accept?"))),
            Some(GuildAsk::Hand(_, _, name)) => return boxed(
                Self::words(2924, "Are sure want to assign %s as guild leader? After assigned your position will become %s")
                    .replace("%s", name)
            ),
            Some(GuildAsk::Leave) => Self::words(523, "Please enter the reason of Withdrawal."),
            Some(GuildAsk::Expel(..)) => Self::words(524, "Please enter the reason of Expulsion."),
            Some(GuildAsk::Castle(_) | GuildAsk::Invested(_)) | None => return String::new(),
        };
        [
            format!(
                r#"<div class="a" style="{};background:#fff;border:1px solid #9a9a9a;box-sizing:border-box"></div>{}"#,
                at(0, 0, width, height),
                fitted(15, 6, width - 30, BLACK, &words)
            ),
            format!(r#"<input id="guild-reason" class="a in" style="{};background:#e8e8e8;box-sizing:border-box" "#, at(15, 22, width - 32, 16)),
            format!(
                r#"type="text" maxlength="39" autocomplete="off">{}{}"#,
                picture_press(width - 101, height - 32, "btn_ok", Press::Yes),
                picture_press(width - 55, height - 32, "btn_cancel", Press::No)
            )
        ].concat()
    }
}
