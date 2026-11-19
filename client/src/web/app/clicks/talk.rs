//! The window of a talk with an NPC.

use super::*;

impl App {
    /// A press of this system (`crate::press::talk`).
    pub(super) fn talk_pressed(&mut self, press: crate::press::talk::Press) {
        use crate::press::talk::Press;
        // A link to an item (`CCreateLineInfo::OnItemLink`: `UM_ITEMEXPLANATION`, as one that is identified).
        if let Press::Item(item) = press {
            return self.handle(Action::Info(item, true));
        }
        let (Some(dialog), Some(socket)) = (&mut self.dialog, &self.socket) else { return };
        let npc = dialog.npc;
        match press {
            Press::Next => {
                dialog.button = None;
                dialog.stale = true;
                socket.send(&net::talk::next(npc));
            }
            Press::Close => {
                socket.send(&net::talk::close(npc));
                self.dialog = None;
            }
            Press::Pick(choice) => dialog.picked = choice,
            Press::Item(_) => {}
            Press::Ok if !dialog.menu.is_empty() => {
                // The choices are counted from 1.
                socket.send(&net::talk::choose(npc, dialog.picked as u8 + 1));
                dialog.menu.clear();
                dialog.stale = true;
            }
            Press::Ok => {
                let typed = field("dialog-input");
                socket.send(&if dialog.asks == Some(true) { net::talk::words(npc, &typed) } else { net::talk::number(npc, typed.trim().parse().unwrap_or(0)) });
                dialog.asks = None;
                dialog.stale = true;
            }
            Press::Cancel => {
                // Turning a menu down ends the talk.
                socket.send(&net::talk::choose(npc, 255));
                self.dialog = None;
            }
        }
        self.redraw(&[&wnd::DIALOG]);
    }
}
