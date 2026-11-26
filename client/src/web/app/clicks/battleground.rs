//! The battlegrounds: what the menu's icon, the list's window and the windows of the queue do when pressed
//! (the client's `UIMenuIconWnd::SendMsg` with `MENUICON_BATTLE`, `UIEntryQueueWnd::SendMsg`,
//! `UIEntryQueueStandByWnd::SendMsg`, `UIEntryQueueRequestWnd::SendMsg`).

use super::*;
use crate::battleground::Standing;
use crate::net::battleground;
use crate::press::battleground::{Press, Way};

impl App {
    /// A press of this system (`crate::press::battleground`).
    pub(super) fn battleground_pressed(&mut self, press: Press) {
        match press {
            Press::Window => self.battleground_icon(),
            Press::Close => self.battleground.open = false,
            Press::Row(id) => self.battleground.current = Some(id),
            Press::Apply(way) => self.battleground_apply(way),
            Press::Help => self.battleground.help = !self.battleground.help,
            Press::HelpClose => self.battleground.help = false,
            Press::Yes => {
                if let (Some(way), Some(entry)) = (self.battleground.asks.take(), self.battleground_current()) {
                    self.send_on_map(&battleground::apply(crate::battleground::code(way), &entry.name));
                }
            }
            Press::No => self.battleground.asks = None,
            // `SendCancel`: only the server's answer closes the window.
            Press::Leave => {
                if let Some(entry) = self.battleground_current() {
                    self.send_on_map(&battleground::cancel(&entry.name));
                }
            }
            Press::StandbyClose => self.battleground.standby = false,
            Press::Enter | Press::Decline => self.battleground_answered(press == Press::Enter),
        }
        self.redraw(&wnd::BATTLEGROUND_WINDOWS);
    }

    /// The menu's icon: one who waits in a queue asks the server for the place in it (whose answer shows the
    /// window of the queue and closes the list); the list opens, or closes if it was open. The first of the
    /// list is picked as it opens (`UIEntryQueueWnd::OnCreate`).
    fn battleground_icon(&mut self) {
        if self.battleground.ready {
            if let Some(entry) = self.battleground_current() {
                self.send_on_map(&battleground::ask_place(&entry.name));
            }
        }
        self.battleground.open = !self.battleground.open;
        if self.battleground.open {
            self.battleground.current = data::battlegrounds().first().map(|entry| entry.id);
        }
    }

    /// One of the three buttons under the list: the client says in a box why one cannot apply so, or asks
    /// whether it is meant.
    fn battleground_apply(&mut self, way: Way) {
        let Some(entry) = self.battleground_current() else { return };
        let own = self.own_account();
        let party = &self.party.members;
        let standing = Standing {
            battlefield: self.pvp.zone.battlefield,
            delayed: self.statuses.on.iter().any(|state| state.state == crate::battleground::APPLY_DELAY),
            party: party.len(),
            leader: party.iter().any(|member| member.account == own && member.leader),
            party_online: party.iter().filter(|member| !member.offline).count(),
            guild: self.guild.id != 0,
            master: self.guild.master,
            guild_online: self.guild.members.iter().filter(|member| member.online).count(),
        };
        match crate::battleground::refused(way, &standing, entry.allies) {
            Some(line) => {
                self.message = Some(Self::words(line, ""));
                self.render();
            }
            None => self.battleground.asks = Some(way),
        }
    }

    /// The answer whether to enter the battle that is ready (`SendAdmissionReply`): both windows of the queue
    /// close; a yes says in the chat that the others are waited for, a no ends the waiting.
    fn battleground_answered(&mut self, enter: bool) {
        if self.battleground.entry.take().is_none() {
            return;
        }
        if let Some(entry) = self.battleground_current() {
            self.send_on_map(&battleground::admit(enter, &entry.name, &self.battleground.lobby));
        }
        if enter {
            self.chat_line(Self::words(2132, "Waiting for the opponents."), PALE_YELLOW);
        } else {
            self.battleground.ready = false;
        }
        self.battleground.standby = false;
    }
}
