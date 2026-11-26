//! The small window of a member of the party (the client's `UIMiniPartyWnd` with the `UIMiniPartyInfoWnd` in
//! it): 174 by 52, standing in a cell of the grid `crate::party::mini` lays over the screen.

use super::*;
use crate::party::mini;

/// How tall the client's strip of menu icons is with more than twenty icons, as it has now
/// (`UIMenuIconWnd::OnCreate`: `Resize(220, 238)`), folded away or not.
const MENU_TALL: i32 = 238;

impl App {
    /// The member whose small window is the one of that place among the small windows, with the member's
    /// place in the party's list.
    pub(in crate::web) fn party_mini_member(&self, place: usize) -> Option<(usize, &Member)> {
        let (account, _) = self.party.mini.0.get(place)?;
        self.party.members.iter().enumerate().find(|(_, member)| member.account == *account)
    }

    /// The grid the small windows stand in on a stage of that size: new ones go under the height of the
    /// strip of menu icons (`UIWindowMgr::GetMiniPartyUsableArea` takes the height of `WID_MENUICONWND`).
    pub(in crate::web) fn party_mini_grid(&self, view: (f64, f64)) -> mini::Grid {
        mini::Grid::of((view.0 as i32, view.1 as i32), Some(MENU_TALL))
    }

    /// Where the small window of that place stands, if there is one and its cell is on the screen.
    pub(in crate::web) fn party_mini_place(&self, place: usize, view: (f64, f64)) -> Option<(i32, i32)> {
        self.party_mini_member(place)?;
        self.party_mini_grid(view).place(self.party.mini.0.get(place)?.1)
    }

    /// A member's small window (`UIMiniPartyWnd::Paint`, `UIMiniPartyInfoWnd::Paint`): its back
    /// (`img_partymini_bg.tga`, drawn three times over for a member who is online and once for one who is
    /// not, so that it shows more faintly then); the picture of the member's job (its dead one for a dead
    /// member) at 7, 6, or at 7, 8 under the crown that the leader has at 9, 0; at 35, 9 the level, the name
    /// and the map in brackets, in white (in 0xAAAAAA for one offline), cut to six tenths of the window's
    /// width; and at 35, 23 the gauge of the member's life, 38 hundredths of the width by 5 (`UIPcGage`),
    /// for a member who is online and whose life is known: the player's own, and that of one in sight. The
    /// right button opens the menu on the member (`OnRBtnUp`); a press picks the member for the skill being
    /// aimed (`OnLBtnDown`, `clicks/party/mini.rs`). The client shows the whole text by the pointer when it
    /// is cut; here the page's own tip has it always.
    pub(in crate::web) fn party_mini_window(&self, place: usize) -> String {
        let Some((row, member)) = self.party_mini_member(place) else { return String::new() };
        let own = self.own_account() == member.account;
        let dead = self.party.dead.contains(&member.account) || (own && self.hero.dead);
        let back = format!(r#"<img class="a" alt="" draggable="false" style="left:0;top:0" src="{}">"#, picture("renewalparty/img_partymini_bg.tga"));
        let mut inside = back.repeat(if member.offline { 1 } else { 3 });
        inside.push_str(&format!(
            r#"<img class="a{}" alt="" draggable="false" style="left:7px;top:{}px" src="{}" onerror="this.remove()">"#,
            if member.offline { " dim" } else { "" },
            if member.leader { 8 } else { 6 },
            picture(&format!("renewalparty/icon_jobs_{}{}.bmp", member.job, if dead { "_die" } else { "" })),
        ));
        if member.leader {
            inside.push_str(&art("renewalparty/ico_partyCrown.bmp", 9, 0, 21, 10));
        }
        let label = mini::label(member.level, &member.name, &data::map_name(map_name(&member.map)));
        inside.push_str(&format!(
            r#"<div class="a t" style="left:35px;top:9px;width:{}px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;color:{};{}">{}</div>"#,
            mini::SIZE.0 * 6 / 10,
            if member.offline { "#aaaaaa" } else { WHITE },
            BOLD.style(),
            escape(&label)
        ));
        if let Some((left, most)) = self.party_life(member) {
            inside.push_str(&life_gauge(35, 23, mini::SIZE.0 * 38 / 100, left, most));
        }
        format!(
            r#"<div class="a" style="{};cursor:default" title="{}" {}>{inside}</div>"#,
            at(0, 0, mini::SIZE.0, mini::SIZE.1),
            escape(&label),
            menued(Menu::MiniMember(row))
        )
    }
}

/// The gauge of a member's life as the party's windows have it (`UIPcGage::Paint`: a rim in 0x10189C about a
/// ground in 0x424242, and over that as much of the width as there is life left, green, or red under a
/// quarter: `mini::gauge`).
pub(super) fn life_gauge(x: i32, y: i32, width: i32, left: i32, most: i32) -> String {
    let (full, low) = mini::gauge(width, left, most);
    [
        format!(r#"<div class="a" style="{};background:#424242;border:1px solid #10189c;box-sizing:border-box">"#, at(x, y, width, 5)),
        format!(r#"<div style="width:{full}px;height:3px;background:{}"></div></div>"#, if low { "#ff0000" } else { "#10ef21" })
    ].concat()
}
