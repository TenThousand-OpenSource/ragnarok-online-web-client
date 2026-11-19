//! The guild: what its window's tabs, rows and buttons do, the menus in it, the answers to what it asks, and
//! what is typed for it (the client's `UIGuildWnd` and its tabs, `UIGuildLeaveReasonDescWnd`,
//! `UIGuildBanReasonDescWnd`, `UIJoinGuildAcceptWnd`, and the `MM_*` arms of `CGameMode::SendMsg` they call).
//! What the master does with the guild is in `guild/manage.rs`.

use super::*;
use crate::net::guild;

mod manage;

impl App {
    /// A press of the guild (`crate::press::guild`).
    pub(super) fn guild_pressed(&mut self, press: crate::press::guild::Press) {
        use crate::press::guild::Press;
        match press {
            // The window is for one who is of a guild. (For anyone else the client opens a helper for finding
            // one, `UIGuildHelperWnd`, which is not made here: nothing opens.)
            Press::Window => {
                self.guild.open = !self.guild.open && self.guild.id != 0;
                self.guild_tab_left();
                if self.guild.open {
                    // Which tabs may be opened is asked once (`UIGuildWnd::CreateTabs`).
                    if self.guild.tabs.is_none() {
                        self.send_on_map(&guild::ask_tabs());
                    }
                    self.guild_tab_asked();
                }
                self.render();
            }
            Press::Tab(tab) => {
                if self.guild_tab_opens(tab) {
                    self.guild_tab_left();
                    self.guild.tab = tab;
                    self.guild_tab_asked();
                    self.render();
                }
            }
            Press::Member(row) => {
                (self.guild.picked, self.guild.menu, self.guild.combo) = (Some(row), None, None);
                self.render();
            }
            Press::OnlineFirst => {
                self.guild.online_first = !self.guild.online_first;
                (self.guild.picked, self.guild.menu, self.guild.combo) = (None, None, None);
                self.render();
            }
            // A header pressed orders the list by its column, from the least up; pressed again, from the
            // greatest down (`UIGuildMemberManageWnd::SortMembers`).
            Press::Sort(column) => {
                self.guild.sort = Some((column, matches!(self.guild.sort, Some((by, false)) if by == column)));
                (self.guild.picked, self.guild.menu, self.guild.combo) = (None, None, None);
                self.render();
            }
            Press::Combo(row) => self.guild_combo_pressed(row),
            Press::Position(position) => self.guild_position_picked(position),
            Press::MembersOk => self.guild_members_sent(),
            Press::Invites(id) | Press::Expels(id) | Press::Stores(id) => self.guild_right_pressed(id, &press),
            Press::PositionsOk => self.guild_positions_sent(),
            Press::Emblem => self.guild_emblem_asked(),
            Press::Skill(row) => {
                self.guild.skill = Some(row);
                self.redraw(&[&wnd::GUILD]);
            }
            Press::Raise(skill) => self.guild_skill_raised(skill),
            Press::UseSkill => self.guild_skill_used(),
            Press::Notice => self.guild_notice_sent(),
            // Breaking the guild up takes its name typed after `/breakguild`: the button puts the command
            // into the chat's line for the name to be typed. (The client's own button warns that what lies in
            // the guild's storage is lost, line 2564, and opens a small window of its own for the name,
            // `WID_CREATEGUILDWND_MODE_1`, which is not made here.)
            Press::Disband => {
                set_field("chat-input", "/breakguild ");
                if let Some(line) = element("chat-input").and_then(|line| line.dyn_into::<web_sys::HtmlElement>().ok()) {
                    let _ = line.focus();
                }
            }
            Press::Yes | Press::No => self.guild_answered(press == Press::Yes),
            Press::Do(entry) => self.guild_menu_row(entry),
        }
    }

    /// Whether a tab opens for the player (`UM_CHANGE_GUILD_TAB`: the members, the positions, the expelled and
    /// the notice each have a bit of what the server said with ZC_ACK_GUILD_MENUINTERFACE; the first tab and
    /// the skills open for everyone). Until the server has said, those four are shut.
    pub(in crate::web) fn guild_tab_opens(&self, tab: usize) -> bool {
        const NEEDS: [u32; 6] = [0, 1, 2, 0, 16, 128];
        NEEDS.get(tab).is_some_and(|needs| *needs == 0 || self.guild.tabs.unwrap_or(0) & needs != 0)
    }

    /// What a tab holds only while it is open is dropped: the pick, the menus, and what the master changed and
    /// did not send (the client makes each tab's window anew when it opens).
    fn guild_tab_left(&mut self) {
        let guild = &mut self.guild;
        (guild.picked, guild.menu, guild.combo, guild.skill) = (None, None, None, None);
        guild.position_edits.clear();
        guild.member_moves.clear();
    }

    /// Asks the server for what the tab now open shows (each tab's `OnCreate`: the first asks for the
    /// expelled, the guild's numbers and the members; the others for their own).
    fn guild_tab_asked(&mut self) {
        use guild::menu::*;
        let asks: &[u32] = match self.guild.tab {
            0 => &[EXPELLED, INFO, MEMBERS],
            1 => &[MEMBERS],
            2 => &[POSITIONS],
            3 => &[SKILLS],
            4 => &[EXPELLED],
            _ => &[],
        };
        for what in asks {
            self.send_on_map(&guild::ask(*what));
        }
        self.guild_heads_wanted();
    }

    /// The members as the window lists them (`UIGuildMemberManageWnd::Paint`, `SortMembers`): ordered by the
    /// column whose header was pressed (the name, the position's number, the job's name, the level, the part
    /// given and what was given, which go alike), and then those in the game first, when the player has it so.
    pub(in crate::web) fn guild_listed(&self) -> Vec<&guild::Member> {
        let mut members: Vec<&guild::Member> = self.guild.members.iter().collect();
        if let Some((column, down)) = self.guild.sort {
            members.sort_by(|a, b| {
                let order = match column {
                    0 => a.name.cmp(&b.name),
                    1 => a.position.cmp(&b.position),
                    2 => data::job_name(a.job).cmp(&data::job_name(b.job)),
                    3 => a.level.cmp(&b.level),
                    _ => a.given.cmp(&b.given),
                };
                if down { order.reverse() } else { order }
            });
        }
        if self.guild.online_first {
            members.sort_by_key(|member| !member.online);
        }
        members
    }

    /// The right button in the guild window, on a member or on a guild allied or opposed: it is marked and its
    /// menu opens there. A menu that would have no rows does not open.
    pub(in crate::web::app) fn guild_menu_opens(&mut self, on: GuildMenu, at: (f32, f32)) {
        let known = match on {
            GuildMenu::Member(row) => row < self.guild.members.len(),
            GuildMenu::Related(row) => row < self.guild.related.len() && self.guild.master,
        };
        if known {
            if let GuildMenu::Member(row) = on {
                self.guild.picked = Some(row);
            }
            (self.guild.menu, self.guild.combo) = (Some((Self::on_stage(at), on)), None);
            self.render();
        }
    }

    /// Puts the menu of the guild window away, if it is open.
    pub(in crate::web::app) fn close_guild_menu(&mut self) {
        if self.guild.menu.take().is_some() {
            self.render();
        }
    }

    /// A row of a menu of the guild window was pressed (`UM_MENU_SELECTED`).
    fn guild_menu_row(&mut self, entry: crate::press::guild::Entry) {
        use crate::press::guild::Entry;
        let Some((_, on)) = self.guild.menu.take() else { return };
        match on {
            GuildMenu::Related(row) => {
                // The master ends an alliance or an opposition (`MM_REQ_DELETE_RELATED_GUILD`).
                if let (Entry::Unrelate, Some((number, _, opposed))) = (entry, self.guild.related.get(row).cloned()) {
                    self.send_on_map(&guild::unrelate(number, opposed));
                }
            }
            GuildMenu::Member(row) => {
                let Some(member) = self.guild_listed().get(row).map(|member| (*member).clone()) else { return self.render() };
                match entry {
                    Entry::Leave => self.guild.asks = Some(GuildAsk::Leave),
                    Entry::Expel => self.guild.asks = Some(GuildAsk::Expel(member.account, member.character)),
                    Entry::Hand => self.guild.asks = Some(GuildAsk::Hand(member.account, member.character, member.name.clone())),
                    Entry::Party => self.party_invite(&member.name),
                    Entry::Chat => self.whisper_to(&member.name),
                    // A letter is written to the member (`MM_OPEN_WRITE_RODEX` with the name).
                    Entry::Mail => self.mail_write_to(&member.name),
                    Entry::Unrelate => {}
                }
            }
        }
        self.render();
    }

    /// OK or Cancel of what the guild asks.
    fn guild_answered(&mut self, yes: bool) {
        let Some(ask) = self.guild.asks.take() else { return };
        let reason = field("guild-reason");
        match ask {
            GuildAsk::Invited(number, _) => self.send_on_map(&guild::answer(number, yes)),
            GuildAsk::Ally(account, _) => self.send_on_map(&guild::answer_ally(account, yes)),
            GuildAsk::Leave if yes => self.send_on_map(&guild::leave(self.guild.id, self.own_account(), self.own_character(), reason.trim())),
            // Expelling is for the master and for a position that allows it (`MM_REQ_BAN_GUILD`).
            GuildAsk::Expel(account, character) if yes && (self.guild.master || self.guild.expels) => self.send_on_map(&guild::expel(self.guild.id, account, character, reason.trim())),
            // The guild goes to the member: the first position is the master's (`MM_REQ_CHANGE_MEMBERPOS`).
            GuildAsk::Hand(account, character, _) if yes && self.guild.master => self.send_on_map(&guild::move_members(&[(account, character, 0)])),
            GuildAsk::Leave | GuildAsk::Expel(..) | GuildAsk::Hand(..) => {}
        }
        self.redraw(&[&wnd::GUILD_ASKS]);
    }

    /// What was typed into the chat, when it is the guild's: a line for the guild (`$` before it, or after
    /// `/gc` and `/guildchat`), or one of its commands, as `CSession::GetTalkType` tells them: `/guild name`
    /// makes one, `/breakguild name` breaks the player's own up, `/guildinvite name` invites. `false` for
    /// anything else.
    pub(in crate::web::app) fn guild_typed(&mut self, words: &str) -> bool {
        if let Some(line) = words.strip_prefix('$') {
            self.guild_say(line);
            return true;
        }
        let (command, rest) = words.split_once(' ').map_or((words, ""), |(command, rest)| (command, rest.trim()));
        // A name is one word (the client's `count == 2`); a line of chat and the name to invite are all the rest.
        let one = !rest.is_empty() && !rest.contains(' ');
        match command.to_lowercase().as_str() {
            "/gc" | "/guildchat" if !rest.is_empty() => self.guild_say(rest),
            "/guild" if one => self.send_on_map(&guild::make(self.own_character(), rest)),
            // Only the master breaks it up, and only by its own name: anything else is refused before it is
            // sent (`MM_REQ_DISORGANIZE_GUILD`, whose message box says line 366).
            "/breakguild" if one => {
                if self.guild.master && rest == self.guild.name {
                    self.send_on_map(&guild::break_up(rest));
                } else if self.guild.master {
                    self.message = Some(Self::words(366, "You have failed to disband the guild."));
                    self.render();
                }
            }
            "/guildinvite" if !rest.is_empty() => self.guild_invite(rest),
            _ => return false,
        }
        true
    }

    /// The row of the menu on another player that invites them into the guild (`MM_REQ_JOIN_GUILD`).
    pub(in crate::web::app) fn guild_invite_seen(&mut self, who: u32) {
        self.send_on_map(&guild::invite(who, self.own_account(), self.own_character()));
    }

    /// A line for the guild.
    fn guild_say(&mut self, line: &str) {
        if let (false, Some(name)) = (line.is_empty() || self.guild.id == 0, self.game.as_ref().map(|game| game.name.clone())) {
            self.send_on_map(&guild::say(&name, line));
        }
    }

    /// `/guildinvite name` (`MM_REQ_JOIN_GUILD2`): for one whose position invites; a name of 24 letters or
    /// more is refused in a message box; the chat says whom the invitation went to, in the client's 0xFFFF00.
    fn guild_invite(&mut self, name: &str) {
        if self.guild.id == 0 || !self.guild.invites {
            return;
        }
        if name.len() >= 24 {
            self.message = Some(Self::words(531, "Name is too long. Please enter a name up to 23 english characters."));
            return self.render();
        }
        self.send_on_map(&guild::invite_named(name));
        self.chat_line(Self::words(382, "Send (%s) a Guild Invitation").replacen("%s", name, 1), CYAN);
    }
}

