//! The Shortcut Settings window.

use super::*;
use crate::hotkeys;

impl App {
    /// A press of this system (`crate::press::hotkeys`).
    pub(super) fn key_pressed(&mut self, press: crate::press::hotkeys::Press) {
        use crate::press::Step;
        use crate::press::hotkeys::Press;
        match press {
            // From the window of game options: it opens, or closes if it was open (the client's
            // `ID_OPEN_HOTKEY`).
            Press::Open => {
                self.keys.open = !self.keys.open;
                (self.keys.row, self.keys.notice) = (None, None);
                self.keys.edited.clear();
                self.render();
            }
            Press::Tab(tab) => {
                (self.keys.tab, self.keys.page, self.keys.row, self.keys.notice) = (tab, 0, None, None);
                self.redraw(&[&wnd::KEYS]);
            }
            Press::Page(step) => {
                let pages = self.listed_keys(hotkeys::TABS[self.keys.tab.min(3)]).len().div_ceil(PAGE).max(1);
                self.keys.page = if step == Step::More { (self.keys.page + 1).min(pages - 1) } else { self.keys.page.saturating_sub(1) };
                (self.keys.row, self.keys.notice) = (None, None);
                self.redraw(&[&wnd::KEYS]);
            }
            Press::Row(row) => {
                (self.keys.row, self.keys.notice) = (Some(row), None);
                self.redraw(&[&wnd::KEYS]);
            }
            Press::Reset => {
                self.reset_keys();
                self.redraw(&[&wnd::KEYS]);
            }
            // Cancel drops what was set and leaves the window open, as the client's does.
            Press::Cancel => {
                self.keys.edited.clear();
                (self.keys.row, self.keys.notice) = (None, None);
                self.redraw(&[&wnd::KEYS]);
            }
            Press::Ok | Press::Close => {
                if press == Press::Ok {
                    self.keep_keys();
                }
                self.keys.edited.clear();
                (self.keys.open, self.keys.row, self.keys.notice) = (false, None, None);
                self.render();
            }
        }
    }
}
