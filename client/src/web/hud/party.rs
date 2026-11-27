//! The party's windows: the list of members with the bar of buttons under it (the Party side of the client's
//! `UIMessengerGroupWnd`), the menu on a member, what the party asks in a message box (`UIJoinPartyAcceptWnd`
//! and the client's `Confirm`), and the members' marks on the minimap. What the window shares with its Friend
//! side is `hud/messenger.rs`; the windows beside it, which set a party up and invite into it, are
//! `party/side.rs`.

use super::*;
use crate::net::party::Member;
use crate::party::map_name;
use crate::press::party::{Entry, Press};
use crate::press::things::Menu;
use crate::web::app::{PartyAsk, PartySide};
use crate::web::parts::menued;

mod side;
mod mini;
use mini::life_gauge;

/// How the party window is laid out in its two forms (`UIMessengerGroupWnd::OnCreate`, `UM_RESIZE`): the
/// large one is 273 by 326 less what is left over of a row, the small one 243 by 179 less the same; a row is
/// 34 or 27 tall with the job's picture 5,5 or 4,1 into it.
struct Layout {
    width: i32,
    height: i32,
    row: i32,
    rows: i32,
    icon: (i32, i32),
}

const LARGE: Layout = Layout { width: 273, height: 296, row: 34, rows: 7, icon: (5, 5) };
const SMALL_FORM: Layout = Layout { width: 243, height: 166, row: 27, rows: 4, icon: (4, 1) };

impl App {
    fn party_layout(&self) -> &'static Layout {
        if self.party.small { &SMALL_FORM } else { &LARGE }
    }

    /// The party window's size now, for its place on the screen.
    pub(in crate::web) fn party_size(&self) -> (i32, i32) {
        // The window's Friend side is a little taller: its rows are of another height (`hud/friends.rs`).
        if self.friends.shown {
            return self.friends_size();
        }
        let layout = self.party_layout();
        (layout.width, layout.height)
    }

    /// The party window (`UIMessengerGroupWnd::Paint` with `mode` 1): the title with the party's name, the
    /// members a row each, under them the bar of buttons, and at the bottom the two tabs and how many of
    /// twelve places are taken.
    pub(in crate::web) fn party_window(&self) -> String {
        // With its Friend tab pressed the window lists the friends (the client's `mode` 0).
        if self.friends.shown {
            return self.friends_window();
        }
        let layout = self.party_layout();
        let (width, height) = (layout.width, layout.height);
        let party = Self::words(103, "Party");
        let mut window = title_bar(width, &if self.party.name.is_empty() { party.clone() } else { format!("{party}({})", self.party.name) });
        window.push_str(&self.messenger_frame(width, height));
        let rows: String = self.party.members.iter().enumerate().map(|(row, member)| self.party_row(layout, row, member)).collect();
        window.push_str(&format!(
            r#"<div class="a scroll" style="{};overflow-x:hidden" {}>{rows}</div>{}"#,
            at(1, 17, width - 2, layout.rows * layout.row),
            rows_stepped(layout.row, layout.rows),
            // The window's bar (`UIMessengerGroupWnd::OnCreate`: `Configure(width - 15, 17, 1, height - 58)`).
            scroll_bar(width - 15, 17, layout.rows * layout.row, layout.row)
        ));
        window.push_str(&self.party_bar(layout));
        // The tabs: the Friend one turns the window to the list of friends (`hud/friends.rs`).
        window.push_str(&radio(4, height - 16, false, true, &Self::words(102, "Friend"), Some(Press::Friends.into())));
        window.push_str(&radio(64, height - 16, true, true, &party, None));
        // How many are in the party, of the twelve it holds: written only when someone is.
        if !self.party.members.is_empty() {
            window.push_str(&text(width - 96, height - 16, BLACK, &Self::words(3231, "Cap")));
            window.push_str(&art("renewalparty/bg_partymember.bmp", width - 58, height - 18, 38, 15));
            window.push_str(&text(width - 51, height - 16, "#fff", &format!("{}/12", self.party.members.len())));
        }
        window
    }

    /// A member's row: the picture of its job (its dead one for a dead member) with the leader's crown over
    /// it, the level and the name with the map in brackets, the bar of its life with the numbers beside it,
    /// and at the right whether it is online. Colours are the client's: 0x493A37 for the name and the same
    /// number read the other way round for the level, 0x6F3007 for the player's own name as leader, 0xA38A86
    /// for one who is offline.
    fn party_row(&self, layout: &Layout, row: usize, member: &Member) -> String {
        let own = self.own_account() == member.account;
        let (x, y) = layout.icon;
        let dead = self.party.dead.contains(&member.account) || (own && self.hero.dead);
        let mut inside = String::new();
        if self.party.picked == Some(row) {
            inside.push_str(&fill(0, 2, layout.width, layout.row, "#c6d6f7"));
        }
        // How the client draws the picture of a member who is offline was not read (`UIJobIconButton`): it is
        // faded here.
        inside.push_str(&format!(
            r#"<img class="a{}" alt="" draggable="false" style="left:{x}px;top:{y}px" src="{}" title="{}" onerror="this.remove()">"#,
            if member.offline { " dim" } else { "" },
            picture(&format!("renewalparty/icon_jobs_{}{}.bmp", member.job, if dead { "_die" } else { "" })),
            escape(&data::job_name(member.job))
        ));
        if member.leader {
            inside.push_str(&art("renewalparty/ico_partyCrown.bmp", x + 2, y - 8, 21, 10));
        }
        let (level_colour, name_colour) = match (member.offline, own && member.leader) {
            (true, _) => ("#868aa3", "#868aa3"),
            (false, true) => ("#373a49", "#6f3007"),
            (false, false) => ("#373a49", "#493a37"),
        };
        let name = if !member.offline && !member.map.is_empty() {
            format!("{}({})", member.name, data::map_name(map_name(&member.map)))
        } else {
            member.name.clone()
        };
        inside.push_str(&[
            format!(r#"<div class="a t" style="left:{}px;top:{}px;white-space:nowrap;{}">"#, x + 28, y + 3, BOLD.style()),
            format!(r#"<span style="color:{level_colour}">Lv.{}</span>"#, member.level),
            format!(r#"<span style="display:inline-block;vertical-align:top;margin-left:5px;max-width:{}px;overflow:hidden;"#, layout.width * 8 / 10 - 80),
            format!(r#"color:{name_colour}">{}</span></div>"#, escape(&name))
        ].concat());
        if let Some((left, most)) = self.party_life(member) {
            let top = layout.row * 76 / 100;
            inside.push_str(&life_gauge(47, top, 60, left, most));
            inside.push_str(&text_in(Font(0, 10, false), 47 + 66, top - 2, BLACK, &format!("{left}/{most}")));
        }
        let status = if member.offline { "off" } else if own { "me" } else { "on" };
        inside.push_str(&art(&format!("renewalparty/icon_party_{status}.bmp"), layout.width * 8 / 10, layout.row * 65 / 100, 26, 11));
        // A member who is online and has no small window is taken out of the list with the pointer, to get
        // one (`UIMessengerGroupWnd::OnLBtnDown`, `UIDragMiniPartyWnd`).
        let taken = if member.offline || self.party.mini.has(member.account) { String::new() } else { dragged(Drag::Member(row)) };
        format!(
            r#"<div style="position:relative;height:{}px;cursor:default" {} {} {taken}>{inside}</div>"#,
            layout.row,
            pressed(Press::Row(row)),
            menued(Menu::Member(row))
        )
    }

    /// A member's life, left and most, where its gauge shows: the life of a member is known once the server
    /// has told it (it does for those in sight), the player's own is the character's, and one who is offline
    /// has no gauge.
    fn party_life(&self, member: &Member) -> Option<(i32, i32)> {
        let life = if self.own_account() == member.account {
            Some((self.value(parameter::HP) as i32, self.value(parameter::MAX_HP) as i32))
        } else {
            self.party.life.get(&member.account).copied()
        };
        life.filter(|(_, most)| !member.offline && *most > 0)
    }

    /// The bar of buttons under the list (`OnCreate` and `LayoutButtons`): the Adventurer's Agency, the setup,
    /// the lock, and then either the making of a party or inviting, leaving and the small windows' order. In
    /// the Party side the client hides its second, third and fifth buttons, and the rest close up.
    fn party_bar(&self, layout: &Layout) -> String {
        let y = layout.height - 40;
        let mut bar = bar_back(layout.width, y);
        bar.push_str(&bar_button(5, y, "mesbtn_partymaster_01", Press::Agency, &Self::words(3514, "Adventurer's Agency")));
        bar.push_str(&bar_button(31, y, "mesbtn_04", Press::Setup, &Self::words(101, "Party Setup")));
        bar.push_str(&bar_button(57, y, "mesbtn_07", Press::Lock, &Self::words(1627, "Deactivate lock function")));
        if self.party.members.is_empty() {
            bar.push_str(&bar_button(83, y, "mesbtn_08", Press::Create, &Self::words(2054, "Create Party")));
        } else {
            bar.push_str(&bar_button(83, y, "mesbtn_010", Press::Invite, &Self::words(2056, "Party Invitation")));
            bar.push_str(&bar_button(109, y, "mesbtn_09", Press::Leave, &Self::words(2055, "Leave Party")));
            bar.push_str(&bar_button(135, y, "mesbtn_011", Press::Sort, &Self::words(3983, "Sort Mini Party Window")));
        }
        bar
    }

    /// The menu the right button opens on a member (`UIMessengerGroupWnd`'s `UM_MAKEMENU`): a letter; for the
    /// player's own row leaving the party, for another's the talk between two and, for the leader, expelling
    /// and handing the party over; and for one who is online the small window of that member, put on the
    /// screen and taken off it. Opened on a member's small window (`UIMiniPartyInfoWnd`'s `UM_MAKEMENU`) it
    /// ends with taking that window off, whether the member is online or not.
    pub(in crate::web) fn party_menu(&self) -> String {
        let Some(member) = self.party.menu.and_then(|(_, row)| self.party.members.get(row)) else { return String::new() };
        let own = self.own_account() == member.account;
        let mut rows = menu_row(&Self::words(2808, "Send a mail"), Press::Do(Entry::Mail));
        if own {
            rows.push_str(&menu_row(&Self::words(96, "Leave party"), Press::Do(Entry::Leave)));
        } else {
            rows.push_str(&menu_row(&Self::words(360, "Open 1:1 Chat"), Press::Do(Entry::Chat)));
            if self.leads_party() {
                rows.push_str(&menu_row(&Self::words(97, "Expel from party"), Press::Do(Entry::Expel)));
                rows.push_str(&menu_row(&Self::words(1532, "Assign Party Leader"), Press::Do(Entry::HandOver)));
            }
        }
        if !member.offline && !self.party.menu_mini {
            rows.push_str(&menu_row(&Self::words(3101, "Add small party window"), Press::Do(Entry::AddSmall)));
        }
        if !member.offline || self.party.menu_mini {
            rows.push_str(&menu_row(&Self::words(3102, "Remove small party window"), Press::Do(Entry::RemoveSmall)));
        }
        rows
    }

    /// What the party asks, in the client's message box (280 by 120, the words 10,22, OK 189,96 and Cancel
    /// 234,96 as `UIExchangeAcceptWnd::OnCreate` lays out the boxes of this kind): whether to join the party
    /// that invites, to leave, to expel, to hand the party over.
    pub(in crate::web) fn party_question(&self) -> String {
        let words = match &self.party.asks {
            Some(PartyAsk::Invited(_, name)) => format!("({name}){}", Self::words(94, " Party has sent you an invitation. Would you like to join?")),
            Some(PartyAsk::Leave) => Self::words(357, "Are you sure that you want to leave?"),
            Some(PartyAsk::Disband) => Self::words(3257, "There is no party member that can be delegated. Do you want to disband the party instead?"),
            Some(PartyAsk::Expel(_)) => Self::words(363, "Are you sure that you want to expel this player?"),
            Some(PartyAsk::HandOver(_)) => Self::words(1533, "Switch Party Leader?"),
            None => return String::new(),
        };
        message_box(&escape(&words), Press::Yes, Press::No)
    }

    /// The members' marks on the minimap (`CGameMode::DrawMiniMapMembers`): for each other member on this map
    /// a square 4 across in the member's colour, with a rim of white let through six parts in ten. After
    /// them, as the client draws them, the marks of the guild's members and of the boss (`hud/marks.rs`).
    pub(in crate::web) fn minimap_party(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        let own = self.own_account();
        let members: String = self
            .party
            .members
            .iter()
            .filter(|member| member.account != own && !member.offline && map_name(&member.map) == map_name(&game.map))
            .filter_map(|member| {
                let (x, y) = self.party.places.get(&member.account)?;
                let (x, y) = crate::web::view::minimap_spot(game, self.minimap_zoom, (f32::from(*x), f32::from(*y)))?;
                Some([
                    format!(r#"<span title="{}" style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:4px;height:4px;margin:-3px "#, escape(&member.name)),
                    format!(r#"0 0 -3px;background:#493a37;border:1px solid rgba(255,255,255,.6)"></span>"#)
                ].concat())
            })
            .collect();
        members + &self.minimap_team() + &self.minimap_server_marks()
    }
}
