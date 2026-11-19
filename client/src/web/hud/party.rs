//! The party's windows: the list of members with the bar of buttons under it (the Party side of the client's
//! `UIMessengerGroupWnd`), the menu on a member, the window that sets the party up or makes one
//! (`UIPartySettingWnd`), the one that asks whom to invite (`UIPartyInvitationToWnd`), what the party asks in a
//! message box (`UIJoinPartyAcceptWnd` and the client's `Confirm`), and the members' marks on the minimap.

use super::*;
use crate::net::party::{Member, map_name};
use crate::press::party::{Entry, Press};
use crate::press::things::Menu;
use crate::web::app::{PartyAsk, PartySide};
use crate::web::parts::menued;

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

/// A button of the bar under the list: 26 by 20, its three pictures `name.bmp`, `name_a.bmp`, `name_b.bmp`.
pub(super) fn bar_button(x: i32, y: i32, name: &str, press: impl Into<crate::press::Press>, tip: &str) -> String {
    let file = |state: &str| format!("basic_interface/{name}{state}.bmp");
    pictured(x, y, (26, 20), [&file(""), &file("_a"), &file("_b")], press, &format!(r#" data-tip="{}""#, escape(tip)))
}

/// One of the client's round buttons with its words 13 to the right (`UIRadioBtn::Paint`): on or off, and grey
/// when it cannot be pressed; `press` is what pressing it does, if anything.
fn radio(x: i32, y: i32, on: bool, enabled: bool, words: &str, press: Option<Press>) -> String {
    radio_marked(x, y, on, enabled, words, &press.map(pressed).unwrap_or_default())
}

/// The same button with what marks it as pressable written already (`pressed`), or nothing: for the Friend
/// side of the window, whose presses are its own (`hud/friends.rs`).
pub(super) fn radio_marked(x: i32, y: i32, on: bool, enabled: bool, words: &str, mark: &str) -> String {
    format!(
        r#"<div class="a t" style="left:{x}px;top:{y}px;height:12px;padding-left:13px;white-space:nowrap;cursor:default;color:{};background:url('{}') 0 0 no-repeat" {mark}>{}</div>"#,
        if enabled { BLACK } else { "#b5b5b5" },
        picture(if on { "radiobtn_on.bmp" } else { "radiobtn_off.bmp" }),
        escape(words)
    )
}

/// The strip the bar's buttons stand on, `y` down a window that wide (`UIMessengerGroupWnd::Paint`: its two
/// ends 7 wide and the middle between them).
pub(super) fn bar_back(width: i32, y: i32) -> String {
    format!(
        r#"{}<div class="a" style="{};background:url('{}') 0 0 repeat-x"></div>{}"#,
        art("basic_interface/mesbtn_left.bmp", 0, y, 7, 20),
        at(7, y, width - 14, 20),
        picture("basic_interface/mesbtn_mid.bmp"),
        art("basic_interface/mesbtn_right.bmp", width - 7, y, 7, 20)
    )
}

impl App {
    /// What the window has under its title on both of its sides, the Party one and the Friend one
    /// (`hud/friends.rs`): the button by the cross that makes it small, or large again (`ID_MINI`,
    /// `ID_RESTORE_PARTY_LIST`), the cross, the white ground of the list, and the bar its tabs stand on.
    pub(super) fn messenger_frame(&self, width: i32, height: i32) -> String {
        [
            if self.party.small {
                pictured(width - 28, 3, (10, 10), ["renewalparty/btn_titlebar_maximum_out.bmp", "renewalparty/btn_titlebar_maximum_over.bmp", "renewalparty/btn_titlebar_maximum_press.bmp"], Press::Size, "")
            } else {
                press_button(width - 28, 3, 11, "basic_interface/sys_base_off.bmp", "basic_interface/sys_base_on.bmp", Press::Size)
            },
            press_button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::Window),
            format!(r#"<div class="a" style="{};background:#fff;border:1px solid #c0c0c0;box-sizing:border-box"></div>"#, at(0, 17, width, height - 38)),
            art("basic_interface/btnbar_left.bmp", 0, height - 21, 21, 21),
            art("basic_interface/btnbar_mid.bmp", 21, height - 21, width - 42, 21),
            art("basic_interface/btnbar_right.bmp", width - 21, height - 21, 21, 21),
        ]
        .concat()
    }

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
        window.push_str(&format!(r#"<div class="a scroll" style="{};overflow-x:hidden">{rows}</div>"#, at(1, 17, width - 2, layout.rows * layout.row)));
        window.push_str(&self.party_bar(layout));
        // The tabs: the Friend one turns the window to the list of friends (`hud/friends.rs`).
        window.push_str(&radio(4, height - 16, false, true, &Self::words(102, "Friend"), Some(Press::Friends)));
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
        let own = self.account.as_ref().map_or(0, |account| account.account_id) == member.account;
        let (x, y) = layout.icon;
        let dead = self.party.dead.contains(&member.account) || (own && self.hero.dead);
        let mut inside = String::new();
        if self.party.picked == Some(row) {
            inside.push_str(&format!(r#"<div class="a" style="{};background:#c6d6f7"></div>"#, at(0, 2, layout.width, layout.row)));
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
        let name = match data::map_sign(map_name(&member.map)).filter(|_| !member.offline && !member.map.is_empty()) {
            Some(sign) => format!("{}({})", member.name, sign.name),
            None if !member.offline && !member.map.is_empty() => format!("{}({})", member.name, map_name(&member.map)),
            None => member.name.clone(),
        };
        inside.push_str(&format!(
            r#"<div class="a t" style="left:{}px;top:{}px;white-space:nowrap;{}"><span style="color:{level_colour}">Lv.{}</span><span style="display:inline-block;vertical-align:top;margin-left:5px;max-width:{}px;overflow:hidden;color:{name_colour}">{}</span></div>"#,
            x + 28,
            y + 3,
            BOLD.style(),
            member.level,
            layout.width * 8 / 10 - 80,
            escape(&name)
        ));
        // The life of a member is known once the server has told it (it does for those in sight); the
        // player's own is the character's.
        let life = if own { Some((self.value(parameter::HP) as i32, self.value(parameter::MAX_HP) as i32)) } else { self.party.life.get(&member.account).copied() };
        if let Some((left, most)) = life.filter(|(_, most)| !member.offline && *most > 0) {
            let top = layout.row * 76 / 100;
            let full = (i64::from(left.clamp(0, most)) * 58 / i64::from(most)) as i32;
            inside.push_str(&format!(
                r#"<div class="a" style="{};background:#424242;border:1px solid #10189c;box-sizing:border-box"><div style="width:{full}px;height:3px;background:{}"></div></div>"#,
                at(47, top, 60, 5),
                if left * 4 < most { "#ff0000" } else { "#10ef21" }
            ));
            inside.push_str(&text_in(Font(0, 10, false), 47 + 66, top - 2, BLACK, &format!("{left}/{most}")));
        }
        let status = if member.offline { "off" } else if own { "me" } else { "on" };
        inside.push_str(&art(&format!("renewalparty/icon_party_{status}.bmp"), layout.width * 8 / 10, layout.row * 65 / 100, 26, 11));
        format!(r#"<div style="position:relative;height:{}px;cursor:default" {} {}>{inside}</div>"#, layout.row, pressed(Press::Row(row)), menued(Menu::Member(row)))
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

    /// The menu the right button opens on a member (`UM_MAKEMENU`): a letter; for the player's own row leaving
    /// the party, for another's the talk between two and, for the leader, expelling and handing the party
    /// over; and for one who is online the small window of that member.
    pub(in crate::web) fn party_menu(&self) -> String {
        let Some(member) = self.party.menu.and_then(|(_, row)| self.party.members.get(row)) else { return String::new() };
        let own = self.account.as_ref().map_or(0, |account| account.account_id) == member.account;
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
        if !member.offline {
            rows.push_str(&menu_row(&Self::words(3101, "Add small party window"), Press::Do(Entry::AddSmall)));
            rows.push_str(&menu_row(&Self::words(3102, "Remove small party window"), Press::Do(Entry::RemoveSmall)));
        }
        rows
    }

    /// How large the window beside the party window is now: the setup 130 by 245, the making of a party 130
    /// by 220 with a name to type or 130 by 180 without, the invitation 150 by 100 (`UIWindowMgr::
    /// EnsureWindow`).
    pub(in crate::web) fn party_side_size(&self) -> (i32, i32) {
        match self.party.side {
            Some(PartySide::Setup) => (130, 245),
            Some(PartySide::Create) => (130, 220),
            Some(PartySide::Invite) => (150, 100),
            Some(PartySide::Named) | None => (130, 180),
        }
    }

    /// The window beside the party window: the setup, or the invitation.
    pub(in crate::web) fn party_side_window(&self) -> String {
        let Some(side) = self.party.side else { return String::new() };
        let (width, height) = self.party_side_size();
        if side == PartySide::Invite {
            // `UIPartyInvitationToWnd`: the words 10,22, the name typed 20,42 on grey 230, OK and Cancel.
            return format!(
                r#"{}{}{}<input id="party-invitee" class="a in" style="{};background:#e6e6e6;box-sizing:border-box" type="text" maxlength="23" autocomplete="off">{}{}"#,
                title_bar(width, &Self::words(2056, "Party Invitation")),
                panel(width, height),
                text(10, 22, BLACK, &Self::words(2058, "Player Name:")),
                at(20, 42, 120, 20),
                picture_press(width - 92, height - 24, "btn_ok", Press::InviteOk),
                picture_press(width - 46, height - 24, "btn_cancel", Press::InviteCancel)
            );
        }
        // `UIPartySettingWnd`: what its rows stand under is its `m_mode`, 70 for a party there is (with how
        // experience is shared on top), 50 for one being made (with its name on top), 5 for one whose name is
        // given already.
        let mode = match side {
            PartySide::Setup => 70,
            PartySide::Create => 50,
            _ => 5,
        };
        let draft = self.party.draft;
        let (leads, new) = (self.leads_party(), self.party.members.is_empty());
        let mut window = title_bar(width, &if side == PartySide::Create { Self::words(2054, "Create Party") } else { Self::words(291, "Party Setup") });
        window.push_str(&panel(width, height));
        window.push_str(&press_button(width - 14, 3, 11, "basic_interface/sys_close_off.bmp", "basic_interface/sys_close_on.bmp", Press::SetupCancel));
        if side == PartySide::Setup {
            window.push_str(&text(10, 22, BLACK, &Self::words(292, "How to share EXP")));
            window.push_str(&radio(20, 42, draft.experience == 0, leads, &Self::words(287, "Each Take "), Some(Press::Experience(0))));
            window.push_str(&radio(20, 62, draft.experience == 1, leads, &Self::words(288, "Even Share"), Some(Press::Experience(1))));
        } else if side == PartySide::Create {
            window.push_str(&text(10, 22, BLACK, &Self::words(2057, "Party Name:")));
            window.push_str(&format!(r#"<input id="party-name" class="a in" style="{};background:#e6e6e6;box-sizing:border-box" type="text" maxlength="23" autocomplete="off" value="{}">"#, at(20, 38, 100, 20), escape(&self.party.making)));
        }
        window.push_str(&text(10, mode + 17, BLACK, &Self::words(293, "How to share Items")));
        window.push_str(&radio(20, mode + 37, !draft.pickup, new || leads, &Self::words(289, "Each Take"), Some(Press::Pickup(0))));
        window.push_str(&radio(20, mode + 57, draft.pickup, new || leads, &Self::words(290, "Party Share"), Some(Press::Pickup(1))));
        // The one way items are handed out: the client has this single button, always on.
        window.push_str(&text(10, mode + 82, BLACK, &Self::words(738, "Item sharing type")));
        window.push_str(&radio(20, mode + 102, true, new || leads, &Self::words(739, "Individual"), None));
        window.push_str(&picture_press(width - 92, height - 24, "btn_ok", Press::SetupOk));
        window.push_str(&picture_press(width - 46, height - 24, "btn_cancel", Press::SetupCancel));
        window
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
        message_box(Asked::Offer, &escape(&words), Press::Yes, Press::No)
    }

    /// The members' marks on the minimap (`CGameMode::DrawMiniMapMembers`): for each other member on this map
    /// a square 4 across in the member's colour, with a rim of white let through six parts in ten. After
    /// them, as the client draws them, the marks of the guild's members and of the boss (`hud/marks.rs`).
    pub(in crate::web) fn minimap_party(&self) -> String {
        let Some(game) = &self.game else { return String::new() };
        let own = self.account.as_ref().map_or(0, |account| account.account_id);
        let members: String = self
            .party
            .members
            .iter()
            .filter(|member| member.account != own && !member.offline && map_name(&member.map) == map_name(&game.map))
            .filter_map(|member| {
                let (x, y) = self.party.places.get(&member.account)?;
                let (x, y) = crate::web::view::minimap_spot(game, self.minimap_zoom, (f32::from(*x), f32::from(*y)))?;
                Some(format!(
                    r#"<span title="{}" style="position:absolute;left:{x:.2}%;top:{y:.2}%;width:4px;height:4px;margin:-3px 0 0 -3px;background:#493a37;border:1px solid rgba(255,255,255,.6)"></span>"#,
                    escape(&member.name)
                ))
            })
            .collect();
        members + &self.minimap_server_marks()
    }
}
