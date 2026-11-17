//! What is said: NPC dialogs, the chat, whispers.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn talk_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            npc::ZC_SAY_DIALOG | npc::ZC_SAY_DIALOG2 | npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2 | npc::ZC_CLOSE_DIALOG | npc::ZC_MENU_LIST | npc::ZC_CLEAR_DIALOG | npc::ZC_OPEN_EDITDLG | npc::ZC_OPEN_EDITDLGSTR => {
                let Some(from) = npc::npc(id, packet) else { return true };
                // A script that ends after its window was closed says so once more: there is nothing to show.
                if self.dialog.is_none() && matches!(id, npc::ZC_CLOSE_DIALOG | npc::ZC_CLEAR_DIALOG | npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2) {
                    if id == npc::ZC_CLOSE_DIALOG {
                        if let Some(socket) = &self.socket {
                            socket.send(&npc::close(from));
                        }
                    }
                    return true;
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
                        if let Some(socket) = &self.socket {
                            socket.send(&npc::close(from));
                        }
                        self.dialog = None;
                        self.refresh_dialog();
                        return true;
                    }
                    npc::ZC_CLOSE_DIALOG => dialog.button = Some(false),
                    npc::ZC_CLEAR_DIALOG => dialog.lines.clear(),
                    npc::ZC_MENU_LIST => {
                        dialog.menu = npc::menu(packet).unwrap_or_default();
                        dialog.picked = 0;
                    }
                    asks => dialog.asks = Some(asks == npc::ZC_OPEN_EDITDLGSTR),
                }
                self.refresh_dialog();
            }
            // What the character says comes back from the server in green; what others say is white; the
            // server's announcements are yellow, or the colour they name.
            packets::ZC_NOTIFY_PLAYERCHAT | packets::ZC_NOTIFY_CHAT | packets::ZC_BROADCAST | packets::ZC_BROADCAST2 => {
                if let Some(words) = packets::chat_text(id, packet) {
                    // What is said near also hangs over the head of whoever says it.
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    match id {
                        packets::ZC_NOTIFY_PLAYERCHAT => world::said(own, own, &words),
                        packets::ZC_NOTIFY_CHAT => world::said(npc::npc(id, packet).unwrap_or(0), own, &words),
                        _ => {}
                    }
                    let colour = match id {
                        packets::ZC_NOTIFY_PLAYERCHAT => "#0f0".to_owned(),
                        packets::ZC_NOTIFY_CHAT => "#fff".to_owned(),
                        packets::ZC_BROADCAST2 => packets::broadcast_colour(packet).map_or("#ff0".to_owned(), |[red, green, blue]| format!("rgb({red},{green},{blue})")),
                        _ => "#ff0".to_owned(),
                    };
                    // What others say near is kept out of the chat's list when the player has it so (the client's
                    // `/notalkmsg` and `/notalkmsg2`); it still hangs over their heads.
                    let muted = id == packets::ZC_NOTIFY_CHAT && (self.option("TT_NOTALKMSG_ON_OFF") || self.option("TT_NOTALKMSG2_ON_OFF"));
                    if !muted {
                        self.chat_line(words, &colour);
                    }
                }
            }
            npc::ZC_WHISPER => {
                if let Some((name, words)) = npc::whispered(packet) {
                    self.chat_line(format!("( {} {name} ) : {words}", crate::i18n::tr("From")), "#ffff00");
                }
            }
            npc::ZC_ACK_WHISPER => {
                if packet.get(2).copied().unwrap_or(0) != 0 {
                    self.chat_line(crate::i18n::tr("There is no such character name or the user is offline.").to_owned(), "#ff8484");
                }
            }
            _ => return false,
        }
        true
    }
}
