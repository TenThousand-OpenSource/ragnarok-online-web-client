//! What the server says of the attendance check (`crate::net::attendance`), and of the windows it opens.

use super::*;
use crate::net::attendance;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn attendance_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // `CGameMode::ReceiveServiceRequest`: the server opens a window. The attendance check is told its
            // number; the bank (window 0, an NPC's `openbank`) is asked for as its icon asks, unless it is
            // open (`OpenServiceWindow`). The other windows the server opens so are left to the list of
            // packets not read: a tip of the Tipbox (5), a quest's information (6), the stylist (1), the
            // windows against macros (2, 3) and the grade enchanting (8).
            attendance::ZC_UI_OPEN => match attendance::opened(packet) {
                Some((attendance::OUT_ATTENDANCE, value)) => self.attendance_told(value),
                Some((0, _)) => {
                    if !self.bank.open {
                        self.pressed(crate::press::bank::Press::Window);
                    }
                }
                _ => return false,
            },
            // `CGameMode::ReceiveWindow328Result`: today is attended, and the open window shows it; or the
            // server refused, which is said in the chat (line 3472) and closes the window.
            attendance::ZC_ACK_CHECK_ATTENDANCE => match attendance::answered(packet) {
                Some((0, days)) if self.attendance.open => self.attendance_told(crate::attendance::attended(days)),
                Some((1, _)) => {
                    self.say_filled(said::GENERAL, 3472, &[], RED);
                    self.attendance.open = false;
                    self.render();
                }
                _ => {}
            },
            _ => return false,
        }
        true
    }
}
