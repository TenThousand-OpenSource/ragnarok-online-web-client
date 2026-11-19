//! What is said: NPC dialogs and the chat. (Whispers: `incoming/whispers.rs`.)

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn talk_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            npc::ZC_SAY_DIALOG | npc::ZC_SAY_DIALOG2 | npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2 | npc::ZC_CLOSE_DIALOG | npc::ZC_MENU_LIST | npc::ZC_CLEAR_DIALOG | npc::ZC_OPEN_EDITDLG | npc::ZC_OPEN_EDITDLGSTR => self.npc_spoke(id, packet),
            packets::ZC_NOTIFY_PLAYERCHAT | packets::ZC_NOTIFY_CHAT => self.chat_heard(id, packet),
            // A line in a colour of its own (`CGameMode::ReceiveActorColorChat`): into the chat, and over the
            // head of whoever says it when they are in sight. The server refuses some things this way.
            npc::ZC_NPC_CHAT => {
                if let Some((who, [red, green, blue], words)) = npc::coloured_line(packet) {
                    world::said(who, self.own_account(), &words);
                    self.chat_line(words, &format!("rgb({red},{green},{blue})"));
                }
            }
            _ => return false,
        }
        true
    }

    fn npc_spoke(&mut self, id: u16, packet: &[u8]) {
        let Some(from) = npc::npc(id, packet) else { return };
        // A script that ends after its window was closed says so once more: there is nothing to show.
        if self.dialog.is_none() && matches!(id, npc::ZC_CLOSE_DIALOG | npc::ZC_CLEAR_DIALOG | npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2) {
            if id == npc::ZC_CLOSE_DIALOG {
                self.send(&npc::close(from));
            }
            return;
        }
        let dialog = self.dialog.get_or_insert_with(Dialog::default);
        dialog.npc = from;
        match id {
            npc::ZC_SAY_DIALOG | npc::ZC_SAY_DIALOG2 => {
                if std::mem::take(&mut dialog.stale) {
                    dialog.lines.clear();
                }
                dialog.lines.extend(npc::said(id, packet));
            }
            npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2 => dialog.button = Some(true),
            // A window with nothing said in it has nothing to close: it is closed for the player.
            npc::ZC_CLOSE_DIALOG if dialog.lines.is_empty() && dialog.menu.is_empty() && dialog.asks.is_none() => {
                self.send(&npc::close(from));
                self.dialog = None;
                self.redraw(&[&wnd::DIALOG]);
                return;
            }
            npc::ZC_CLOSE_DIALOG => dialog.button = Some(false),
            npc::ZC_CLEAR_DIALOG => dialog.lines.clear(),
            npc::ZC_MENU_LIST => {
                dialog.menu = npc::menu(packet).unwrap_or_default();
                dialog.picked = 0;
            }
            asks => dialog.asks = Some(asks == npc::ZC_OPEN_EDITDLGSTR),
        }
        self.redraw(&[&wnd::DIALOG]);
    }

    /// What the character says comes back from the server in green; what others say is white. (The
    /// server's announcements are read in `incoming/broadcast.rs`.)
    fn chat_heard(&mut self, id: u16, packet: &[u8]) {
        if let Some(words) = packets::chat_text(id, packet) {
            // What is said near also hangs over the head of whoever says it.
            let own = self.own_account();
            match id {
                packets::ZC_NOTIFY_PLAYERCHAT => world::said(own, own, &words),
                packets::ZC_NOTIFY_CHAT => world::said(npc::npc(id, packet).unwrap_or(0), own, &words),
                _ => {}
            }
            let colour = if id == packets::ZC_NOTIFY_PLAYERCHAT { "#0f0" } else { "#fff" };
            // What others say near is kept out of the chat's list when the player has it so (the client's
            // `/notalkmsg` and `/notalkmsg2`); it still hangs over their heads.
            let muted = id == packets::ZC_NOTIFY_CHAT && (self.option("TT_NOTALKMSG_ON_OFF") || self.option("TT_NOTALKMSG2_ON_OFF"));
            if !muted {
                self.chat_line_of(said::CHAT, words, colour);
            }
        }
    }
}
