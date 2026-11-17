//! The Shortcut Settings window.

use super::*;
use crate::hotkeys;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn key_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            // From the window of game options: it opens, or closes if it was open (the client's
            // `ID_OPEN_HOTKEY`).
            "keys" => {
                self.keys.open = !self.keys.open;
                (self.keys.row, self.keys.notice) = (None, None);
                self.keys.edited.clear();
                self.render();
            }
            "keys-tab" => {
                (self.keys.tab, self.keys.page, self.keys.row, self.keys.notice) = (arg.parse().unwrap_or(0), 0, None, None);
                self.refresh_keys();
            }
            "keys-page" => {
                let pages = self.listed_keys(hotkeys::TABS[self.keys.tab.min(3)]).len().div_ceil(PAGE).max(1);
                self.keys.page = if arg == "+" { (self.keys.page + 1).min(pages - 1) } else { self.keys.page.saturating_sub(1) };
                (self.keys.row, self.keys.notice) = (None, None);
                self.refresh_keys();
            }
            "keys-row" => {
                (self.keys.row, self.keys.notice) = (arg.parse().ok(), None);
                self.refresh_keys();
            }
            "keys-reset" => {
                self.reset_keys();
                self.refresh_keys();
            }
            // Cancel drops what was set and leaves the window open, as the client's does.
            "keys-cancel" => {
                self.keys.edited.clear();
                (self.keys.row, self.keys.notice) = (None, None);
                self.refresh_keys();
            }
            "keys-ok" | "keys-close" => {
                if what == "keys-ok" {
                    self.keep_keys();
                }
                self.keys.edited.clear();
                (self.keys.open, self.keys.row, self.keys.notice) = (false, None, None);
                self.render();
            }
            _ => return false,
        }
        true
    }
}
