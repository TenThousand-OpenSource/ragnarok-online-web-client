//! What the page's forms do when sent.

use super::*;

impl App {
    /// A form of the page was sent: logging in and the account's forms, how many to drop, a line of chat.
    pub(super) fn submitted(&mut self, form: &str) {
        match form {
            "login" => {
                self.chime();
                self.login()
            }
            "register" => self.register(),
            "recover" => self.recover(),
            "reset" => self.reset(),
            "create" => self.create_character(),
            "throw" => {
                if let (Some((index, most)), Some(socket)) = (self.bag.throwing.take(), &self.socket) {
                    // As the client's window does: a number that is none, or more than there is, drops
                    // nothing, and the window goes away without a word.
                    let count = field("throw-count").trim().parse::<u32>().unwrap_or(0);
                    if (1..=u32::from(most)).contains(&count) {
                        socket.send(&items::throw(index, count as u16));
                    }
                }
                self.refresh_throw();
            }
            "chat" => {
                let words = field("chat-input");
                let words = words.trim();
                // The client's own commands come first; everything else is said (the server reads its
                // @commands).
                if !words.is_empty() && self.command(words) {
                    set_field("chat-input", "");
                    return;
                }
                if let (false, Some(socket), Some(game)) = (words.is_empty(), &self.socket, &self.game) {
                    // A name in the small box at the left sends the words to that one alone.
                    let to = field("chat-to");
                    let to = to.trim();
                    if !to.is_empty() {
                        socket.send(&npc::whisper(to, words));
                        let line = format!("( To {to} ) : {words}");
                        set_field("chat-input", "");
                        return self.chat_line(line, "#ffff00");
                    }
                    socket.send(&packets::say(&game.name, words));
                    set_field("chat-input", "");
                }
            }
            _ => {}
        }
    }
}
