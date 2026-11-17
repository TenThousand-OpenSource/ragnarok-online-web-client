//! The chat window.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn chat_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "chat-tab" => {
                self.chat.tab = arg.parse().unwrap_or(0);
                self.render();
            }
            "chat-lock" => {
                self.chat.locked = !self.chat.locked;
                self.render();
            }
            "chat-size" => {
                self.chat_resize();
                self.render();
            }
            // Hides the log, leaving the input line, or brings it back as tall as it was.
            "chat-mini" => {
                if self.chat.rows > 0 {
                    self.chat.rows_kept = self.chat.rows;
                    self.chat.rows = 0;
                } else {
                    self.chat.rows = self.chat.rows_kept.max(1);
                }
                self.render();
            }
            _ => return false,
        }
        true
    }
}
