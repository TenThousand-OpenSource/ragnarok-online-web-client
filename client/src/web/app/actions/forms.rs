//! What the page's forms do when sent.

use super::*;

impl App {
    /// A line typed that is one of the client's own commands, or a system's, is done: `false` for words that
    /// are none, which are said. The chat's line goes by this, and the line of a talk with one player.
    pub(in crate::web::app) fn typed_command(&mut self, words: &str) -> bool {
        self.party_typed(words) || self.guild_typed(words) || self.exchange_typed(words) || self.command(words)
    }

    /// A form of the page was sent: logging in and the account's forms, how many to drop, a line of chat.
    pub(super) fn submitted(&mut self, form: Form) {
        match form {
            Form::Login => {
                self.chime();
                self.login()
            }
            Form::Register => self.register(),
            Form::Recover => self.recover(),
            Form::Reset => self.reset(),
            Form::Create => self.create_character(),
            Form::Throw => {
                if let (Some((index, most)), Some(socket)) = (self.bag.throwing.take(), &self.socket) {
                    if let Some(count) = Self::counted(&wnd::THROW, most) {
                        socket.send(&items::throw(index, count));
                    }
                }
                self.redraw(&[&wnd::THROW]);
            }
            Form::Offer => self.exchange_counted(),
            Form::Attach => self.mail_counted(),
            Form::CartCount => self.cart_counted(),
            Form::VendingCount => self.vending_counted(),
            Form::Stall => self.stall_sent(),
            Form::MailSearch => self.mail_searched(),
            Form::ShopCount => self.shop_counted(),
            Form::StorageCount => self.storage_counted(),
            Form::StorageSearch => self.storage_searched(),
            Form::Whisper(place) => self.whisper_typed(place),
            Form::Room => self.room_typed(),
            Form::RoomMake => self.room_made(),
            Form::RoomPassword => self.room_password_typed(),
            Form::Shortcuts => self.shortcuts_kept(),
            Form::TipSearch => self.tip_searched(),
            Form::Chat => {
                let words = field("chat-input");
                let words = words.trim();
                // The client's own commands come first; everything else is said (the server reads its
                // @commands).
                if !words.is_empty() && self.typed_command(words) {
                    set_field("chat-input", "");
                    return;
                }
                // A name in the small box at the left sends the words to that one alone: the line shows once
                // the server has answered (`clicks/whispers.rs`).
                let to = field("chat-to");
                let to = to.trim();
                if !words.is_empty() && !to.is_empty() && self.game.is_some() {
                    self.whisper_send(to, words);
                    return set_field("chat-input", "");
                }
                if self.say_aloud(words) {
                    set_field("chat-input", "");
                }
            }
        }
    }
}
