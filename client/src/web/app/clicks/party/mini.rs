//! The small windows of single members of the party (`crate::party::mini`, the client's `UIMiniPartyWnd`):
//! one put on the screen and taken off it, let go where it was moved to, the windows put in order, a press on
//! one, the menu on one, and their keeping in the browser.

use super::*;
use crate::party::mini::Windows;

/// Where the browser keeps the small windows, as the client writes them to `savedata\MiniPartyInfo.lua`.
const KEPT: &str = "roweb.miniparty";

impl App {
    /// The stage's size, in its own measure.
    fn stage() -> (f64, f64) {
        let (view, scale) = (crate::web::page_size(), crate::web::stage_scale());
        (view.0 / scale, view.1 / scale)
    }

    /// The small windows changed (one more, one less, another cell): they are kept in the browser under the
    /// character's name, none is where an earlier one of its place was dragged to, and the screen is written
    /// again, for they stand elsewhere now.
    fn party_mini_changed(&mut self) {
        let name = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
        store(KEPT, self.party.mini.written(&name).as_deref());
        for window in wnd::PARTY_MINI {
            forget_moved(window.id);
        }
        self.render();
    }

    /// The party's members are known or changed: the small windows kept for this character are read once
    /// (`CSession::LoadMiniPartyInfo`), only those of members still in the party stay
    /// (`UIWindowMgr::RebuildMiniPartyWindows`), and with no party there are none
    /// (`ClearMiniPartyRecords` as the player leaves). What they show is drawn anew.
    pub(in crate::web::app) fn party_mini_follows(&mut self) {
        let Some(name) = self.game.as_ref().map(|game| game.name.clone()) else { return };
        if self.party.members.is_empty() {
            return;
        }
        let was = self.party.mini.clone();
        if self.party.mini_of != name {
            self.party.mini = Windows::read(&stored(KEPT).unwrap_or_default(), &name);
            self.party.mini_of = name;
        }
        let members: Vec<u32> = self.party.members.iter().map(|member| member.account).collect();
        self.party.mini.keep(&members);
        if self.party.mini != was {
            return self.party_mini_changed();
        }
        self.redraw(&wnd::PARTY_MINI);
    }

    /// The player is of no party any more: the small windows go, from the browser too.
    pub(in crate::web::app) fn party_mini_cleared(&mut self) {
        self.party.mini = Windows::default();
        self.party_mini_changed();
    }

    /// "Add small party window" of the menu on a member (`UIMessengerGroupWnd`, the menu's 0x3C): the member
    /// gets a window in the first free cell, unless it has one.
    pub(super) fn party_mini_added(&mut self, account: u32) {
        let grid = self.party_mini_grid(Self::stage());
        if self.party.mini.add(account, &grid) {
            self.party_mini_changed();
        }
    }

    /// "Remove small party window" (the menu's 0x3D, and `MENUID_CLOSE_MINIPARTY` of the small window's own
    /// menu: `UIWindowMgr::CloseMiniPartyWindow`).
    pub(super) fn party_mini_removed(&mut self, account: u32) {
        if self.party.mini.remove(account) {
            self.party_mini_changed();
        }
    }

    /// The button "Sort Mini Party Window" of the party window (`UIMessengerGroupWnd`'s 0x16C,
    /// `UIWindowMgr::RearrangeMiniPartyWindows`).
    pub(super) fn party_mini_sorted(&mut self) {
        let grid = self.party_mini_grid(Self::stage());
        self.party.mini.sort(&grid);
        self.party_mini_changed();
    }

    /// A small window was let go where it was moved to (`UIMiniPartyWnd::OnLBtnUp`): it goes into the nearest
    /// cell, changing places with a window that is there.
    pub(in crate::web::app) fn party_mini_dropped(&mut self, place: usize) {
        let (Some(window), Some((account, _))) = (wnd::PARTY_MINI.get(place), self.party.mini.0.get(place).copied()) else { return };
        let Some((x, y, ..)) = Self::window_rect(window.id) else { return };
        let grid = self.party_mini_grid(Self::stage());
        self.party.mini.dropped(account, (x, y), &grid);
        self.party_mini_changed();
    }

    /// A member was taken out of the party window with the pointer and let go elsewhere
    /// (`UIMessengerGroupWnd::DropMember`): the member gets a small window, in the first free cell.
    pub(in crate::web::app) fn party_mini_taken(&mut self, row: usize) {
        if let Some(account) = self.party.members.get(row).filter(|member| !member.offline).map(|member| member.account) {
            self.party_mini_added(account);
        }
    }

    /// A window of the game was pressed: on a member's small window the press picks that member for the
    /// skill being aimed (`UIMiniPartyInfoWnd::OnLBtnDown`).
    pub(in crate::web::app) fn party_mini_pressed(&mut self, window: &str) {
        let Some(place) = wnd::PARTY_MINI.iter().position(|mini| mini.id == window) else { return };
        if let Some(row) = self.party_mini_member(place).map(|(row, _)| row) {
            self.party_member_picked(row);
        }
    }

    /// A member was pressed, in the party window's list or on its small window: the skill being aimed is
    /// used on that member (`CGameMode::TargetActorByAid`, from `UIMessengerGroupWnd::OnLBtnDown` and
    /// `UIMiniPartyInfoWnd::OnLBtnDown`): one who is online, while the character lives, with a skill of
    /// help aimed at someone (for any other skill the client asks more: Shift held, or a map where players
    /// fight; that is not done here); the player's own row takes only a skill that may be used on oneself.
    pub(super) fn party_member_picked(&mut self, row: usize) {
        let Some((account, offline)) = self.party.members.get(row).map(|member| (member.account, member.offline)) else { return };
        let Some(skill) = self.skills.aimed.and_then(|(id, _)| self.skills.known.iter().find(|known| known.id == id)) else { return };
        let fits = if account == self.own_account() { skill.on_friend() } else { skill.target != 2 && crate::skills::aim::support(skill.id) };
        if offline || self.hero.dead || !fits {
            return;
        }
        world::aim(None, 0);
        self.skill_aimed(Some(account), (0, 0));
        self.aim_words();
    }

    /// The right button on a member's small window (`UIMiniPartyInfoWnd::OnRBtnUp`): the menu on that member
    /// opens there, ending with taking the window off.
    pub(in crate::web::app) fn party_mini_menu_opens(&mut self, row: usize, at: (f32, f32)) {
        if row < self.party.members.len() {
            (self.party.menu, self.party.menu_mini) = (Some((Self::on_stage(at), row)), true);
            self.render();
        }
    }
}
