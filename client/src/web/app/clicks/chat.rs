//! The chat window.

use super::*;

impl App {
    /// A press of this system (`crate::press::chat`).
    pub(super) fn chat_pressed(&mut self, press: crate::press::chat::Press) {
        use crate::press::chat::Press;
        match press {
            Press::Tab(tab) => {
                self.chat.tab = tab;
                self.render();
            }
            Press::Lock => {
                self.chat.locked = !self.chat.locked;
                self.render();
            }
            Press::Size => {
                self.chat_resize();
                self.render();
            }
            // Hides the log, leaving the input line, or brings it back as tall as it was.
            Press::Mini => {
                if self.chat.rows > 0 {
                    self.chat.rows_kept = self.chat.rows;
                    self.chat.rows = 0;
                } else {
                    self.chat.rows = self.chat.rows_kept.max(1);
                }
                self.render();
            }
        }
    }
}
