//! The Shortcut Settings window.

use super::*;

impl App {
    /// A press of this system (`crate::press::hotkeys`).
    pub(super) fn hotkeys_pressed(&mut self, press: crate::press::hotkeys::Press) {
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
                let pages = self.listed_keys(crate::hotkeys::TABS[self.keys.tab.min(3)]).len().div_ceil(PAGE).max(1);
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

    /// A slot of the bar is used, by a key or by the mouse: it stands for its item or its skill.
    pub(in crate::web::app) fn use_slot(&mut self, slot: usize) {
        match self.skills.hotkeys.get(slot).copied().filter(|hotkey| hotkey.id != 0) {
            Some(hotkey) if hotkey.skill => self.use_skill(hotkey.id as u16, Some(hotkey.count)),
            // The first of that item the character carries without wearing it.
            Some(hotkey) => match self.bag.first_unworn(hotkey.id) {
                Some(item) => self.use_item(item.index),
                None => return,
            },
            None => return,
        }
    }

    /// Puts what was dragged on a slot of the hotkey bar, or empties the slot (for nothing dragged), and tells
    /// the server, which keeps the bar.
    pub(in crate::web::app) fn set_hotkey(&mut self, slot: usize, dragged: Option<Drag>) {
        let hotkey = match dragged {
            // Not every item may go on the bar (`Item::fits_hotkeys`): anything else is let go of and nothing
            // happens.
            Some(Drag::Item(id)) => {
                let fits = self.bag.items.iter().find(|item| item.id == id).is_some_and(Item::fits_hotkeys);
                if !fits {
                    return;
                }
                Hotkey { skill: false, id, count: 0 }
            }
            // A thing worn goes on the bar as the item it is, by the same rule.
            Some(Drag::Worn(index)) => match self.bag.at(index).filter(|item| item.fits_hotkeys()) {
                Some(item) => Hotkey { skill: false, id: item.id, count: 0 },
                None => return,
            },
            Some(Drag::Skill(skill)) => Hotkey { skill: true, id: skill.id, count: skill.level },
            // A thing of a letter being written does not go on the bar.
            Some(Drag::Attached(_)) => return,
            // Nor does a row of a shop, or a thing that lies in the storage.
            Some(Drag::Offer(_) | Drag::Basket(_) | Drag::Stored(_)) => return,
            // Nor a thing of the cart, or of a merchant's shop (the client lets go of what was taken up in the
            // cart there and nothing happens).
            Some(Drag::Carted(_) | Drag::Stock(_) | Drag::Stalled(_) | Drag::Vended(_) | Drag::Wanted(_)) => return,
            None => Hotkey::default(),
            // From another slot of the bar: what was there moves here, and that slot is left empty.
            Some(Drag::Slot(from)) => {
                let Some(moved) = self.skills.hotkeys.get(from).copied().filter(|moved| moved.id != 0 && from != slot && slot < net::inventory::HOTKEYS) else { return };
                self.skills.hotkeys.resize(net::inventory::HOTKEYS, Hotkey::default());
                self.skills.hotkeys[from] = Hotkey::default();
                self.send(&net::inventory::set_hotkey(from as u16, Hotkey::default()));
                moved
            }
        };
        if slot >= net::inventory::HOTKEYS {
            return;
        }
        self.skills.hotkeys.resize(net::inventory::HOTKEYS, Hotkey::default());
        self.skills.hotkeys[slot] = hotkey;
        self.send(&net::inventory::set_hotkey(slot as u16, hotkey));
        self.refresh_items();
    }
}
