//! The window of a talk with an NPC.

use super::*;

impl App {
    /// A press of this system (`crate::press::talk`).
    pub(super) fn talk_pressed(&mut self, press: crate::press::talk::Press) {
        use crate::press::talk::Press;
        let (Some(dialog), Some(socket)) = (&mut self.dialog, &self.socket) else { return };
        let npc = dialog.npc;
        match press {
            Press::Next => {
                dialog.button = None;
                dialog.stale = true;
                socket.send(&npc::next(npc));
            }
            Press::Close => {
                socket.send(&npc::close(npc));
                self.dialog = None;
            }
            Press::Pick(choice) => dialog.picked = choice,
            Press::Ok if !dialog.menu.is_empty() => {
                // The choices are counted from 1.
                socket.send(&npc::choose(npc, dialog.picked as u8 + 1));
                dialog.menu.clear();
                dialog.stale = true;
            }
            Press::Ok => {
                let typed = field("dialog-input");
                socket.send(&if dialog.asks == Some(true) { npc::words(npc, &typed) } else { npc::number(npc, typed.trim().parse().unwrap_or(0)) });
                dialog.asks = None;
                dialog.stale = true;
            }
            Press::Cancel => {
                // Turning a menu down ends the talk.
                socket.send(&npc::choose(npc, 255));
                self.dialog = None;
            }
        }
        self.redraw(&[&wnd::DIALOG]);
    }
}
