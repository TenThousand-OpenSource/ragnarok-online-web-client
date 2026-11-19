//! The chat window on the app's side: a line added to it, and its next size.

use super::*;

impl App {
    /// The chat's next size, as the client steps it (`UM_TOGGLEPUSH`): one row, then three more each time up to
    /// thirteen, then none.
    pub(super) fn chat_resize(&mut self) {
        let rows = if self.chat.rows == 0 { 1 } else { self.chat.rows + 3 };
        self.chat.rows = if rows > 15 { 0 } else { rows / 3 * 3 + 1 };
    }

    /// Adds a line to the chat.
    pub(super) fn chat_line(&mut self, words: String, colour: &str) {
        self.chat_line_of(said::GENERAL, words, colour);
    }

    /// A line for the chat window, of one of the client's kinds (`said`): the window's tabs list it or not by that.
    pub(super) fn chat_line_of(&mut self, kind: u8, words: String, colour: &str) {
        // The items a line links to are asked about, so that the line can name them.
        self.chat_links_wanted(&words);
        self.chat.lines.push((words, colour.to_owned(), kind));
        if self.chat.lines.len() > super::super::hud::CHAT_KEPT {
            self.chat.lines.remove(0);
        }
        self.refresh_chat();
    }

    /// A line of the client's message table in the chat, with what it asks for written into it
    /// (`figures::filled`); nothing while the table does not have the line.
    pub(super) fn say_filled(&mut self, kind: u8, line: usize, with: &[&str], colour: &str) {
        if let Some(words) = data::message(line).filter(|words| !words.is_empty()) {
            self.chat_line_of(kind, crate::figures::filled(crate::i18n::tr(&words), with), colour);
        }
    }
}
