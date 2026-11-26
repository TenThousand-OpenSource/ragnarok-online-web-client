//! The reputation's window: its drop-down of groups, its search, its pages.

use super::*;
use crate::press::Step;
use crate::press::reputation::Press;
use crate::reputation::{self as rules, Filter};

impl App {
    /// The marks a script put on reputations go as the page is left (`UIReputeWnd::ClearUpdated`).
    fn reputation_unmarked(&mut self) {
        self.reputation.marked.clear();
    }

    /// A press of this system (`crate::press::reputation`).
    pub(super) fn reputation_pressed(&mut self, press: Press) {
        match press {
            // The menu's icon, the key, the cross and Close. The client makes the window anew each time: on
            // all the groups, its first page.
            Press::Window => {
                let state = &mut self.reputation;
                state.open = !state.open;
                (state.filter, state.group, state.page, state.choosing) = (Filter::All, None, 1, false);
                self.reputation_unmarked();
                self.render();
            }
            Press::Groups => {
                self.reputation.choosing = !self.reputation.choosing;
                self.redraw(&[&wnd::REPUTATION]);
            }
            // `UM_COMBOBOX_SELECTED`: the word sought is emptied, and the group's are listed from the first page.
            Press::Group(place) => {
                let group = place.checked_sub(1).filter(|group| *group < data::reputation().1.len());
                if place != 0 && group.is_none() {
                    return;
                }
                let state = &mut self.reputation;
                (state.group, state.filter, state.page, state.choosing) = (group, group.map_or(Filter::All, Filter::Group), 1, false);
                self.reputation_unmarked();
                set_field(&wnd::REPUTATION.field(), "");
                self.redraw(&[&wnd::REPUTATION]);
            }
            Press::Page(step) => {
                let count = self.reputation_listed().len();
                let page = if step == Step::More { self.reputation.page + 1 } else { self.reputation.page.saturating_sub(1) };
                self.reputation.page = rules::paged(count, page).0;
                self.reputation_unmarked();
                self.redraw(&[&wnd::REPUTATION]);
            }
        }
    }

    /// The word typed into the window's line is looked for in the reputations' names (`ID_SEARCH`); an empty
    /// line lists the group of the drop-down again.
    pub(in crate::web::app) fn reputation_searched(&mut self) {
        let sought = field(&wnd::REPUTATION.field());
        let state = &mut self.reputation;
        state.filter = if sought.is_empty() { state.group.map_or(Filter::All, Filter::Group) } else { Filter::Sought(sought) };
        state.page = 1;
        self.reputation_unmarked();
        self.redraw(&[&wnd::REPUTATION]);
    }
}
