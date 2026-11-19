//! The hotkey bar as the server keeps it: its list, told on coming in.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn hotkeys_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The server sends the list twice on coming in: the hotkey bar (tab 0), then the second bar (tab 1),
            // which this client does not show yet. Only the first is this bar; taking the second for it too
            // emptied the bar at every login.
            net::inventory::ZC_SHORTCUT_KEY_LIST => {
                if net::inventory::hotkeys_bar(packet) == Some(0) {
                    self.skills.hotkeys = net::inventory::hotkeys(packet);
                    // What the bar's skills are called and which pictures are theirs.
                    for hotkey in self.skills.hotkeys.clone().iter().filter(|hotkey| hotkey.skill && hotkey.id != 0) {
                        data::want_skill(hotkey.id as u16, || dispatch(Action::ItemsKnown));
                    }
                    self.items_changed();
                }
            }
            _ => return false,
        }
        true
    }
}
