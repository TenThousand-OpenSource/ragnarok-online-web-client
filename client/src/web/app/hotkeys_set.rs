//! Which key does what for this player, and the setting of it in the Shortcut Settings window.

use super::*;

/// Where the player's own keys are kept in the browser: `table:place:key:with`, one after another.
const KEPT: &str = "roweb.keys";
/// How many entries a page of the window shows: two columns of eighteen rows.
pub(in crate::web) const PAGE: usize = 36;

impl App {
    /// Reads what the browser keeps: the player's own keys, and what kind of keyboard this is.
    pub(super) fn load_keys(&mut self) {
        self.keys.mac = web_sys::window().and_then(|window| window.navigator().platform().ok()).is_some_and(|platform| platform.starts_with("Mac") || platform.starts_with("iP"));
        self.keys.own = stored(KEPT)
            .unwrap_or_default()
            .split(';')
            .filter_map(|entry| {
                let numbers: Vec<u16> = entry.split(':').filter_map(|number| number.parse().ok()).collect();
                (numbers.len() == 4).then(|| ((usize::from(numbers[0]), usize::from(numbers[1])), Chord { key: numbers[2], with: numbers[3] }))
            })
            .collect();
    }

    /// The keys an entry answers to: what was set in the open window, or what the player set earlier, or the
    /// client's own. On a Mac, an entry the client gives a key a MacBook lacks has this client's key in its
    /// place (`hotkeys::mac_instead`), or none.
    pub(in crate::web) fn bound(&self, table: usize, place: usize) -> Chord {
        if let Some(chord) = self.keys.edited.get(&(table, place)).or_else(|| self.keys.own.get(&(table, place))) {
            return *chord;
        }
        self.unset(table, place)
    }

    /// The keys an entry has before the player sets any.
    fn unset(&self, table: usize, place: usize) -> Chord {
        let own = data::hotkeys(table).get(place).map(|(_, chord)| *chord).unwrap_or_default();
        if self.keys.mac && (hotkeys::missing_on_mac(own.key) || hotkeys::missing_on_mac(own.with)) {
            return hotkeys::mac_instead(table, place).unwrap_or_default();
        }
        own
    }

    /// The entry a chord is bound to, of the interface, the Shortcut List's lines and the hotkey bars: its
    /// table and place.
    pub(super) fn bound_to(&self, chord: Chord) -> Option<(usize, usize)> {
        if chord.key == 0 {
            return None;
        }
        [hotkeys::table::INTERFACE, hotkeys::table::MACROS, hotkeys::table::BAR, hotkeys::table::BAR_2].into_iter().find_map(|table| (0..data::hotkeys(table).len()).find(|place| self.bound(table, *place) == chord).map(|place| (table, place)))
    }

    /// The places of a table the window lists, in order: those with a name, and of the interface those the
    /// client does not hide.
    pub(in crate::web) fn listed_keys(&self, table: usize) -> Vec<usize> {
        data::hotkeys(table).iter().enumerate().filter(|(place, (name, _))| !name.is_empty() && !(table == hotkeys::table::INTERFACE && hotkeys::HIDDEN.contains(place))).map(|(place, _)| place).collect()
    }

    /// A key was pressed while a row of the window is picked: the row takes it (the client's
    /// `AssignSelectedHotkey`). Escape takes the row's key away. A key held with others (Alt, Ctrl, Shift) is
    /// not a key by itself. Another entry that had the same keys loses them: the client asks first, this
    /// does not.
    pub(super) fn assign_key(&mut self, chord: Chord) {
        let table = hotkeys::TABS[self.keys.tab.min(3)];
        let Some(place) = self.keys.row.and_then(|row| self.listed_keys(table).get(self.keys.page * PAGE + row).copied()) else { return };
        if matches!(chord.key, hotkeys::SHIFT | hotkeys::CTRL | hotkeys::ALT | 0) {
            return;
        }
        self.keys.notice = None;
        if chord.key == 27 {
            self.keys.edited.insert((table, place), Chord::default());
        } else if self.keys.mac && hotkeys::missing_on_mac(chord.key) {
            self.keys.notice = Some(crate::i18n::tr("This keyboard does not have that key.").to_owned());
        } else {
            // A key another entry has is taken from it only if the player says so (the client's
            // `CheckHotkeyConflict`, which asks with its message 1489). The two hotkey bars may share a key.
            let bars = [hotkeys::table::BAR, hotkeys::table::BAR_2];
            for other in 0..4 {
                if bars.contains(&table) && bars.contains(&other) && other != table {
                    continue;
                }
                for at in 0..data::hotkeys(other).len() {
                    if self.bound(other, at) != chord {
                        continue;
                    }
                    // Its own key again: nothing changes.
                    if (other, at) != (table, place) {
                        self.ask = Some(Ask::Rebind((table, place), (other, at), chord));
                        self.render();
                    }
                    return;
                }
            }
            self.keys.edited.insert((table, place), chord);
        }
        self.redraw(&[&wnd::KEYS]);
    }

    /// The player agreed: the key is taken from the entry that had it and given to the one it was pressed for.
    /// Taken from a slot of the first hotkey bar for something that is not on that bar, it is taken from the
    /// second bar too (as the client's `CheckHotkeyConflict` does).
    pub(super) fn rebind(&mut self, to: (usize, usize), from: (usize, usize), chord: Chord) {
        self.keys.edited.insert(from, Chord::default());
        if from.0 == hotkeys::table::BAR && to.0 != hotkeys::table::BAR {
            for at in 0..data::hotkeys(hotkeys::table::BAR_2).len() {
                if self.bound(hotkeys::table::BAR_2, at) == chord {
                    self.keys.edited.insert((hotkeys::table::BAR_2, at), Chord::default());
                }
            }
        }
        self.keys.edited.insert(to, chord);
    }

    /// OK in the window: what was set there is the player's from now on, and is kept in the browser.
    pub(super) fn keep_keys(&mut self) {
        let edited = std::mem::take(&mut self.keys.edited);
        for ((table, place), chord) in edited {
            // What is the same as it would be unset is not kept: it follows the client's table again.
            if chord == self.unset(table, place) {
                self.keys.own.remove(&(table, place));
            } else {
                self.keys.own.insert((table, place), chord);
            }
        }
        let kept: Vec<String> = self.keys.own.iter().map(|((table, place), chord)| format!("{table}:{place}:{}:{}", chord.key, chord.with)).collect();
        store(KEPT, (!kept.is_empty()).then(|| kept.join(";")).as_deref());
    }

    /// Reset in the window: every entry of every table goes back to the keys it has unset. OK keeps that.
    pub(super) fn reset_keys(&mut self) {
        self.keys.edited = self.keys.own.keys().map(|(table, place)| ((*table, *place), self.unset(*table, *place))).collect();
        self.keys.row = None;
    }
}
