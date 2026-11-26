//! What is said: NPC dialogs and the chat. (Whispers: `incoming/whispers.rs`.)

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn talk_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            net::talk::ZC_SAY_DIALOG | net::talk::ZC_SAY_DIALOG2 | net::talk::ZC_WAIT_DIALOG
            | net::talk::ZC_WAIT_DIALOG2 | net::talk::ZC_CLOSE_DIALOG | net::talk::ZC_MENU_LIST
            | net::talk::ZC_CLEAR_DIALOG | net::talk::ZC_OPEN_EDITDLG
            | net::talk::ZC_OPEN_EDITDLGSTR => self.npc_spoke(id, packet),
            packets::ZC_NOTIFY_PLAYERCHAT | packets::ZC_NOTIFY_CHAT => self.chat_heard(id, packet),
            // A line in a colour of its own (`CGameMode::ReceiveActorColorChat`): into the chat, and over the
            // head of whoever says it when they are in sight. The server refuses some things this way.
            net::talk::ZC_NPC_CHAT => {
                if let Some((who, [red, green, blue], words)) = net::talk::coloured_line(packet) {
                    world::said(who, self.own_account(), &words);
                    self.chat_line(words, &said::colour::css([red, green, blue]));
                }
            }
            _ => return false,
        }
        true
    }

    fn npc_spoke(&mut self, id: u16, packet: &[u8]) {
        let Some(from) = net::talk::npc(id, packet) else { return };
        // A script that ends after its window was closed says so once more: there is nothing to show.
        if self.dialog.is_none() && matches!(
            id,
            net::talk::ZC_CLOSE_DIALOG | net::talk::ZC_CLEAR_DIALOG | net::talk::ZC_WAIT_DIALOG | net::talk::ZC_WAIT_DIALOG2
        ) {
            if id == net::talk::ZC_CLOSE_DIALOG {
                self.send(&net::talk::close(from));
            }
            return;
        }
        let dialog = self.dialog.get_or_insert_with(Dialog::default);
        dialog.npc = from;
        match id {
            net::talk::ZC_SAY_DIALOG | net::talk::ZC_SAY_DIALOG2 => {
                if std::mem::take(&mut dialog.stale) {
                    dialog.lines.clear();
                }
                dialog.lines.extend(net::talk::said(id, packet));
            }
            net::talk::ZC_WAIT_DIALOG | net::talk::ZC_WAIT_DIALOG2 => dialog.button = Some(true),
            // A window with nothing said in it has nothing to close: it is closed for the player.
            net::talk::ZC_CLOSE_DIALOG if dialog.lines.is_empty() && dialog.menu.is_empty() && dialog.asks.is_none() => {
                self.send(&net::talk::close(from));
                self.dialog = None;
                self.redraw(&[&wnd::DIALOG]);
                return;
            }
            net::talk::ZC_CLOSE_DIALOG => dialog.button = Some(false),
            net::talk::ZC_CLEAR_DIALOG => dialog.lines.clear(),
            net::talk::ZC_MENU_LIST => {
                dialog.menu = net::talk::menu(packet).unwrap_or_default();
                dialog.picked = 0;
            }
            asks => dialog.asks = Some(asks == net::talk::ZC_OPEN_EDITDLGSTR),
        }
        let asked = matches!(id, net::talk::ZC_OPEN_EDITDLG | net::talk::ZC_OPEN_EDITDLGSTR);
        self.redraw(&[&wnd::DIALOG]);
        // The box that asks has the keyboard as it opens (`UINpcEditDialogWnd::OnCreate` and
        // `UINpcTextEditDialogWnd::OnCreate`: `g_windowMgr.SetFocus(edit)`), so what is typed goes into it and
        // Enter sends it; the 0 a number begins with is typed over. (A page gives no line drawn after it was
        // opened the keyboard by itself.)
        if asked {
            if let Some(line) = element("dialog-input").and_then(|line| line.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                let _ = line.focus();
                line.select();
            }
        }
    }

    /// What the character says comes back from the server in green; what others say is white. (The
    /// server's announcements are read in `incoming/broadcast.rs`.)
    fn chat_heard(&mut self, id: u16, packet: &[u8]) {
        if let Some(words) = packets::chat_text(id, packet) {
            // What is said near also hangs over the head of whoever says it.
            let own = self.own_account();
            // Stars for a word that is not let be written, in the character's own words and in those of a
            // player in sight (`ReceiveBroadcast`, `ReceiveActorChat`; `crate::manner`).
            let speaker = packets::chat_speaker(packet).unwrap_or(0);
            let player = id == packets::ZC_NOTIFY_PLAYERCHAT || world::player_name(speaker).is_some();
            let words = if player { data::starred(&words) } else { words };
            match id {
                packets::ZC_NOTIFY_PLAYERCHAT => world::said(own, own, &words),
                packets::ZC_NOTIFY_CHAT => world::said(speaker, own, &words),
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
