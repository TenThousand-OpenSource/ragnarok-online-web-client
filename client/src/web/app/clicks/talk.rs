//! The window of a talk with an NPC.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn talk_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "dialog-next" | "dialog-close" | "dialog-ok" | "dialog-cancel" | "dialog-pick" => {
                let (Some(dialog), Some(socket)) = (&mut self.dialog, &self.socket) else { return true };
                let npc = dialog.npc;
                match what {
                    "dialog-next" => {
                        dialog.button = None;
                        dialog.stale = true;
                        socket.send(&npc::next(npc));
                    }
                    "dialog-close" => {
                        socket.send(&npc::close(npc));
                        self.dialog = None;
                    }
                    "dialog-pick" => dialog.picked = arg.parse().unwrap_or(0),
                    "dialog-ok" if !dialog.menu.is_empty() => {
                        // The choices are counted from 1.
                        socket.send(&npc::choose(npc, dialog.picked as u8 + 1));
                        dialog.menu.clear();
                        dialog.stale = true;
                    }
                    "dialog-ok" => {
                        let typed = field("dialog-input");
                        socket.send(&if dialog.asks == Some(true) { npc::words(npc, &typed) } else { npc::number(npc, typed.trim().parse().unwrap_or(0)) });
                        dialog.asks = None;
                        dialog.stale = true;
                    }
                    _ => {
                        // Turning a menu down ends the talk.
                        socket.send(&npc::choose(npc, 255));
                        self.dialog = None;
                    }
                }
                self.refresh_dialog();
            }
            _ => return false,
        }
        true
    }
}
