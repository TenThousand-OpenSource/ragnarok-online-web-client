//! Looking at another player's equipment: the server's answer, the player's own switch for it as the server
//! keeps it, and the lines of the message table the server has said (the client's
//! `CGameMode::ReceiveEquipWindowMicroscope6`, `ReceiveFeedOption`, `ReceiveEquipmentViewState` and `Zc_Msg`).

use super::*;
use crate::net::viewequip;

/// The lines `Zc_Msg` says in red; any other is white.
const SAID_IN_RED: [usize; 5] = [1221, 1222, 1879, 1880, 2182];

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn viewequip_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // What another wears: its window opens. While it is open the answer closes it instead, as in the
            // client (`if (!CloseWindow(WID_EQUIPWND_VIEWOTHER_1)) EnsureWindow(...)`).
            viewequip::ZC_EQUIPWIN_MICROSCOPE => {
                self.viewequip.seen = if self.viewequip.seen.is_some() { None } else { viewequip::seen(packet) };
                if let Some(seen) = &self.viewequip.seen {
                    data::want_items(seen.items.iter().map(|item| item.id), || dispatch(Action::ItemsKnown));
                }
                self.redraw(&[&wnd::VIEW_EQUIP]);
            }
            // The switch that lets others look was set: the chat says how it stands, and the box of the
            // Equipment window shows it. The other settings this packet tells of are not made here.
            crate::net::config::ZC_CONFIG => match crate::net::config::config(packet) {
                Some((viewequip::OPEN_EQUIPMENT, on)) => self.equipment_open_told(on),
                _ => return false,
            },
            // The same, told on entering the game.
            viewequip::ZC_CONFIG_NOTIFY => {
                if let Some(on) = viewequip::open(packet) {
                    self.equipment_open_told(on);
                }
            }
            // A line of the message table: that the other's equipment is not open to be looked at comes so.
            crate::net::notices::ZC_MSG => {
                if let Some((line, words)) = crate::net::notices::said(packet).map(|line| (line, Self::words(line, ""))).filter(|(_, words)| !words.is_empty()) {
                    self.chat_line(words, if SAID_IN_RED.contains(&line) { RED } else { WHITE });
                }
            }
            _ => return false,
        }
        true
    }

    /// Whether others may look at the player's equipment, as the server has it now: lines 1359 and 1358 of
    /// the message table, in the client's `0x64FFFF`.
    fn equipment_open_told(&mut self, on: bool) {
        self.viewequip.allowed = on;
        let words = if on { Self::words(1359, "Equipment information open to the public.") } else { Self::words(1358, "Equipment information not open to the public.") };
        self.chat_line(words, PALE_YELLOW);
        self.redraw(&[&wnd::EQUIPMENT]);
    }
}
