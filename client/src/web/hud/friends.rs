//! The friends list's windows: the Friend side of the window it shares with the party (the client's
//! `UIMessengerGroupWnd` with `mode` 0: `hud/party.rs` draws its other side and hands over to here), the menu
//! on a friend, the Friend Setup window (`UIFriendOptionWnd`), and what the list asks in a message box
//! (`UIJoinFriendAcceptWnd` and the client's `Confirm`).

use super::*;
use crate::net::friends::Friend;
use crate::friends::{MOST, colour};
use crate::press::friends::{Entry, Press};
use crate::press::things::Menu;
use crate::web::app::FriendAsk;
use crate::web::parts::menued;

/// A friend's row is 23 tall (`UM_REFRESHITEM` with `mode` 0).
const ROW: i32 = 23;
/// The Friend Setup window's size (`UIWindowMgr::EnsureWindow`, `WID_FRIENDOPTIONWND`).
pub(super) const SETUP: (i32, i32) = (213, 90);

/// A button of the bar that cannot be pressed: its one picture, and what it says when pointed at.
fn bar_still(x: i32, y: i32, name: &str, tip: &str) -> String {
    format!(
        r#"<div class="a" style="{};background:url('{}') 0 0 no-repeat" data-tip="{}"></div>"#,
        at(x, y, 26, 20),
        picture(&format!("basic_interface/{name}.bmp")),
        escape(tip)
    )
}

impl App {
    /// The size of the window's Friend side (`UM_RESIZE`): as wide as the Party side, 273 or 243, and of 326 or
    /// 179 less what is left over of a row of 23 under the title and over the 58 of the bars.
    pub(in crate::web) fn friends_size(&self) -> (i32, i32) {
        if self.party.small { (243, 173) } else { (273, 311) }
    }

    /// How many rows show (`UM_RESIZE`: the height less 180, in rows, and five more).
    fn friends_rows(&self) -> i32 {
        (self.friends_size().1 - 180).max(0) / ROW + 5
    }

    /// The window's Friend side (`UIMessengerGroupWnd::Paint` with `mode` 0): the title with how many friends
    /// of the forty there may be, in the large form the two pictures of the client behind the list, the
    /// friends a row each, under them the bar of buttons, and at the bottom the two tabs.
    pub(in crate::web) fn friends_window(&self) -> String {
        let (width, height) = self.friends_size();
        let friend = Self::words(102, "Friend");
        let mut window = title_bar(width, &format!("{friend}({}/{MOST})", self.friends.list.len()));
        // The buttons of the title and the ground are the window's own, whichever side it shows.
        window.push_str(&self.messenger_frame(width, height));
        if !self.party.small {
            window.push_str(&art("renewalparty/img_friend1.bmp", 147, 36, 112, 130));
            window.push_str(&art("renewalparty/img_friend2.bmp", 99, 193, 172, 74));
        }
        let rows: String = self.friends.list.iter().enumerate().map(|(row, friend)| self.friend_row(row, friend)).collect();
        window.push_str(&format!(
            r#"<div class="a scroll" style="{};overflow-x:hidden" {}>{rows}</div>"#,
            at(1, 17, width - 2, self.friends_rows() * ROW),
            rows_stepped(ROW, self.friends_rows())
        ));
        window.push_str(&Self::friends_bar(width, height - 40));
        window.push_str(&radio_marked(4, height - 16, true, true, &friend, ""));
        window.push_str(&radio_marked(64, height - 16, false, true, &Self::words(103, "Party"), &pressed(Press::Party)));
        window
    }

    /// A friend's row: the mark of one who is in the game 2,4 into it, and the name 19,6 into it in the colour
    /// of its place in the list, on the pick's blue when it is the one picked. (The row starts 5 into the
    /// window in its large form and 4 in its small one, as the party's pictures do.)
    fn friend_row(&self, row: usize, friend: &Friend) -> String {
        let x = if self.party.small { 4 } else { 5 };
        let mut inside = String::new();
        if !friend.offline {
            inside.push_str(&art("basic_interface/grp_online.bmp", x + 2, 4, 15, 15));
        }
        // The client fills 16 high from 2 left of the name to a pixel past it.
        inside.push_str(&format!(
            r#"<div class="a t" style="left:{}px;top:4px;height:16px;line-height:16px;padding:0 1px 0 2px;white-space:nowrap;color:{};{}">{}</div>"#,
            x + 17,
            colour(row),
            if self.friends.picked == Some(row) { "background:#c6d6f7" } else { "" },
            escape(&friend.name)
        ));
        format!(
            r#"<div style="position:relative;height:{ROW}px;cursor:default" {} {} {}>{inside}</div>"#,
            pressed(Press::Row(row)),
            menued(Menu::Friend(row)),
            used(Use::Friend(row))
        )
    }

    /// The bar of buttons under the list (`OnCreate`, `LayoutButtons` and `UM_REFRESHITEM` with `mode` 0), 26
    /// apart from 5: sending a message and the friend's information, which the client itself keeps switched
    /// off; between them the talk with the friend picked; the Friend Setup; removing the friend; and the lock.
    fn friends_bar(width: i32, y: i32) -> String {
        [
            bar_back(width, y),
            bar_still(5, y, "mesbtn_01_disable", &Self::words(98, "Send Message")),
            bar_button(31, y, "mesbtn_02", Press::Talk, &Self::words(99, "1:1 Chat")),
            bar_still(57, y, "mesbtn_03_disable", &Self::words(100, "Information")),
            bar_button(83, y, "mesbtn_04", Press::Setup, &Self::words(355, "Friend Setup")),
            bar_button(109, y, "mesbtn_05", Press::Remove, &Self::words(351, "Delete")),
            // The lock is the window's own, as on its Party side: not made there yet.
            bar_button(135, y, "mesbtn_07", crate::press::party::Press::Lock, &Self::words(1627, "Deactivate lock function")),
        ]
        .concat()
    }

    /// The menu the right button opens on a friend (`UM_MAKEMENU` with `mode` 0): the talk between two, a
    /// letter, removing the friend, and for a leader of a party, when the friend is in the game and not of
    /// the party, the invitation into it.
    pub(in crate::web) fn friend_menu(&self) -> String {
        let Some(friend) = self.friends.menu.and_then(|(_, row)| self.friends.list.get(row)) else { return String::new() };
        let mut rows = menu_row(&Self::words(360, "Open 1:1 Chat"), Press::Do(Entry::Chat));
        rows.push_str(&menu_row(&Self::words(2808, "Send a mail"), Press::Do(Entry::Mail)));
        rows.push_str(&menu_row(&Self::words(351, "Delete"), Press::Do(Entry::Remove)));
        if !friend.offline && self.leads_party() && !self.party.members.iter().any(|member| member.name == friend.name) {
            rows.push_str(&menu_row(&Self::words(1330, "Request to Join Party").replace("%s", &friend.name), Press::Do(Entry::Invite)));
        }
        rows
    }

    /// The Friend Setup window (`UIFriendOptionWnd`, 213 by 90): three boxes 6 in and 16 apart from 21 down,
    /// for whether a whisper of one who is no friend opens a window of its own, whether a friend's does, and
    /// whether a window opened so chimes.
    pub(in crate::web) fn friend_setup_window(&self) -> String {
        let (width, height) = SETUP;
        let talks = &self.whispers;
        [
            title_bar(width, &Self::words(355, "Friend Setup")),
            panel(width, height),
            cross(width, Press::SetupClose),
            check_words(6, 21, talks.strangers, &Self::words(361, "Open 1:1 Chat between Strangers"), Press::Option(0)),
            check_words(6, 37, talks.friends, &Self::words(359, "Open 1:1 Chat between Friends"), Press::Option(1)),
            check_words(6, 53, talks.sound, &Self::words(362, "Alarm when receive a 1:1 Chat"), Press::Option(2)),
        ]
        .concat()
    }

    /// What the friends list asks, in the client's message box (280 by 120, laid out as the party's): whether
    /// to be the friend of one who asks, and whether to remove a friend.
    pub(in crate::web) fn friend_question(&self) -> String {
        let words = match &self.friends.asks {
            Some(FriendAsk::Wanted(_, _, name)) => Self::words(818, "(%s) wishes to be friends with you. Will you accept?").replace("%s", name),
            Some(FriendAsk::Remove(..)) => Self::words(356, "Are you sure that you want to delete it?"),
            None => return String::new(),
        };
        message_box(&escape(&words), Press::Yes, Press::No)
    }
}
