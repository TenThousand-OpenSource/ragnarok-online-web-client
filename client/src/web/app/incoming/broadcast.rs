//! The server's announcements (ZC_BROADCAST and its coloured form): said in the chat, and shown for twenty
//! seconds across the top of the screen (the client's `CGameMode::ReceiveTaggedBroadcast`,
//! `ReceiveColorBroadcast` and `UIBroadcastBalloon`).

use super::*;
use crate::net::broadcast::{self, Announcement};

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn broadcast_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let announced = match id {
            broadcast::ZC_BROADCAST => broadcast::plain(packet),
            broadcast::ZC_BROADCAST2 => broadcast::coloured(packet),
            _ => return false,
        };
        // One sent by a character whose whispers the player has turned away is dropped.
        if let Some(mut announced) = announced.filter(|announced| !announced.from.as_ref().is_some_and(|from| self.commands.blocked.contains(from))) {
            // (With stars for a word that is not let be written: `ReceiveTaggedBroadcast`,
            // `ReceiveColorBroadcast`; `crate::manner`.)
            announced.words = data::starred(&announced.words);
            announced.lines.iter_mut().for_each(|line| *line = data::starred(line));
            self.announce(announced);
        }
        true
    }

    /// An announcement is said in the chat in its colour, and put across the top of the screen: three stay
    /// there at most, the oldest making room (`UIBroadcastBalloon::AddMessage`), each for twenty seconds.
    fn announce(&mut self, announced: Announcement) {
        let [red, green, blue] = announced.colour;
        let colour = format!("#{red:02x}{green:02x}{blue:02x}");
        self.chat_line_of(announced.kind, announced.words, &colour);
        if announced.lines.is_empty() {
            return;
        }
        self.shown_across(announced.lines, colour, announced.letters);
    }

    /// Lines are put across the top of the screen in a colour, their letters so high (the client's
    /// `UIBroadcastBalloon::AddMessage`): an announcement's, and what else the client shows there
    /// (`incoming/fight.rs`).
    pub(super) fn shown_across(&mut self, lines: Vec<String>, colour: String, letters: u16) {
        let shown = &mut self.broadcast.shown;
        shown.push(Announced { lines, colour, letters, until: js_sys::Date::now() + broadcast::SHOWN_FOR });
        if shown.len() > broadcast::SHOWN_AT_ONCE {
            shown.remove(0);
        }
        self.redraw(&[&wnd::BROADCAST]);
        // It is looked at again when its time is over (`App::watch`).
        after(broadcast::SHOWN_FOR as i32 + 50, Action::Watch);
    }

    /// The announcements whose time is over leave the screen (`UIBroadcastBalloon::OnProcess`).
    pub(in crate::web::app) fn announcements_pass(&mut self) {
        let now = js_sys::Date::now();
        let before = self.broadcast.shown.len();
        self.broadcast.shown.retain(|announced| announced.until > now);
        if self.broadcast.shown.len() != before {
            self.redraw(&[&wnd::BROADCAST]);
        }
    }
}
