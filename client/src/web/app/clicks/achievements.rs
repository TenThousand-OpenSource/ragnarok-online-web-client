//! The achievements: what the window's tabs, rows and buttons do, the achievements kept in sight, and the
//! titles of the Equipment window.

use super::*;
use crate::achievements::{self as rules, Shown, SUBS_SHOWN, TABS, Toggled};
use crate::net::achievements;
use crate::press::achievements::{Press, Tab};

impl App {
    /// A tab is opened (`UIAchievementWnd::SelectTab`): its list is made anew and the first of it picked
    /// (`UIAchListWnd::SetAchievements`).
    fn achievement_tab(&mut self, tab: Tab) {
        if tab.group as usize >= TABS.len() {
            return;
        }
        let state = &mut self.achievements;
        if state.group != tab.group {
            state.sub_first = 0;
        }
        (state.group, state.sub) = (tab.group, tab.sub);
        self.achievements.picked = self.achievements_listed().first().copied();
        self.achievement_items();
    }

    /// The item the achievement picked gives is asked of the gateway, for its picture and its name.
    fn achievement_items(&self) {
        let all = data::achievements();
        let item = self.achievements.picked.and_then(|id| all.iter().find(|info| info.id == id)).map(|info| info.item).filter(|item| *item > 0);
        data::want_items(item.into_iter(), || dispatch(Action::ItemsKnown));
    }

    /// The window opens on an achievement (`UIAchievementWnd::SelectAchievement`): on the tab it stands
    /// under, with it picked and in sight in the list. One the table has under no tab opens nothing.
    fn achievement_shown(&mut self, id: u32) {
        let all = data::achievements();
        let Some(info) = all.iter().find(|info| info.id == id) else { return };
        if !self.achievements.open {
            (self.achievements.open, self.achievements.told, self.achievements.shown) = (true, false, Shown::Completed);
        }
        self.achievement_tab(Tab { group: info.group, sub: Some(info.sub) });
        let subs = TABS.get(info.group as usize).map_or(0, |(_, subs)| subs.len());
        self.achievements.sub_first = (info.sub as usize).min(subs.saturating_sub(SUBS_SHOWN));
        self.achievements.picked = Some(id);
        self.achievement_items();
        self.render();
        Self::achievement_in_sight();
    }

    /// The list is moved so that the row picked shows (`SelectAchievement` sets the list's first row).
    fn achievement_in_sight() {
        let Some(window) = crate::web::by_id(wnd::ACHIEVEMENTS.id) else { return };
        let (Ok(Some(list)), Ok(Some(row))) = (window.query_selector(".viewer"), window.query_selector("[data-picked]")) else { return };
        if let Some(row) = row.dyn_ref::<web_sys::HtmlElement>() {
            list.set_scroll_top(row.offset_top());
        }
    }

    /// A press of this system (`crate::press::achievements`).
    pub(super) fn achievements_pressed(&mut self, press: Press) {
        match press {
            // The menu's icon, the key, the cross: the window opens on its summary, as the client makes it
            // anew each time (`UIAchievementWnd`), or closes. Opened, the icon's mark goes.
            Press::Window => {
                let state = &mut self.achievements;
                state.open = !state.open;
                if state.open {
                    (state.group, state.sub, state.sub_first, state.shown, state.picked, state.told) = (0, None, 0, Shown::Completed, None, false);
                }
                self.render();
            }
            Press::Tab(tab) => {
                self.achievement_tab(tab);
                self.redraw(&[&wnd::ACHIEVEMENTS]);
            }
            Press::Subs(step) => {
                let subs = TABS.get(self.achievements.group as usize).map_or(0, |(_, subs)| subs.len());
                let first = self.achievements.sub_first;
                let last = subs.saturating_sub(SUBS_SHOWN);
                self.achievements.sub_first = if step == crate::press::Step::More { (first + 1).min(last) } else { first.saturating_sub(1) };
                self.redraw(&[&wnd::ACHIEVEMENTS]);
            }
            // The round buttons: the tab's list is made anew (`UIAchUpperWnd::SendMsg`).
            Press::Shown(which) => {
                self.achievements.shown = if which == 1 { Shown::Unfinished } else { Shown::Completed };
                self.achievement_tab(Tab { group: self.achievements.group, sub: self.achievements.sub });
                self.redraw(&[&wnd::ACHIEVEMENTS]);
            }
            Press::Row(id) => {
                self.achievements.picked = Some(id);
                self.achievement_items();
                self.redraw(&[&wnd::ACHIEVEMENTS]);
            }
            // `CSession::ToggleTrackedAchievement`: three at most; a fourth is refused in the chat (line
            // 2680, the client's 0x64FFFF).
            Press::Trace(id) => {
                if rules::toggle(&mut self.achievements.traced, id) == Toggled::Full {
                    self.chat_line(Self::words(2680, "You cannot add anymore."), "#ffff64");
                }
                self.redraw(&[&wnd::ACHIEVEMENTS, &wnd::ACHIEVEMENTS_TRACED]);
            }
            // CZ_REQ_ACH_REWARD: only for one that is completed and whose reward was not taken, as the
            // client's button is only then to be pressed.
            Press::Reward => {
                let all = data::achievements();
                let Some(info) = self.achievements.picked.and_then(|id| all.iter().find(|info| info.id == id)) else { return };
                let record = self.achievement_record(info.id);
                if info.has_reward() && record.completed && !record.rewarded {
                    self.send_on_map(&achievements::ask_reward(info.id));
                }
            }
            Press::Show(id) => self.achievement_shown(id),
            Press::Unfold(id) => {
                if let Some(traced) = self.achievements.traced.iter_mut().find(|traced| traced.id == id) {
                    traced.open = !traced.open;
                }
                self.redraw(&[&wnd::ACHIEVEMENTS_TRACED]);
            }
            Press::Title(title) => {
                self.achievements.title_picked = Some(title);
                self.redraw(&[&wnd::EQUIPMENT]);
            }
        }
    }

    /// A double click on a line of the list of titles (`UIListBoxID::OnLBtnDblClk`, the list 480 of
    /// `UIEquipWnd`): CZ_REQ_CHANGE_TITLE asks for that title, or with 0 for none. Only a title the character
    /// has is asked for: the list has no other.
    pub(in crate::web::app) fn title_used(&mut self, title: u32) {
        let has = title == 0 || rules::titles(&data::achievements(), &self.achievements.records, data::title_name).contains(&title);
        if has {
            self.achievements.title_picked = Some(title);
            self.send_on_map(&achievements::wear_title(title));
            self.redraw(&[&wnd::EQUIPMENT]);
        }
    }

    /// The Equipment window was closed, or left its page of titles for good: the titles gained are no longer
    /// new (`UIWindowMgr::CloseWindow`, `WID_EQUIPWND`).
    pub(in crate::web::app) fn titles_seen(&mut self) {
        self.achievements.new_titles.clear();
    }
}
