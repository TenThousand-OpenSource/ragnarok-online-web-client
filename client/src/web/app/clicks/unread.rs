//! The list of packets this client does not read: opening it with the command `/packets`, and its buttons.

use super::*;

impl App {
    /// A press of this system (`crate::press::unread`).
    pub(super) fn unread_pressed(&mut self, press: crate::press::unread::Press) {
        use crate::press::unread::Press;
        match press {
            Press::Close => self.unread.open = false,
            Press::Clear => self.unread.tally.clear(),
        }
        self.redraw(&[&wnd::UNREAD]);
    }

    /// `/packets` shows the list, or puts it away. The command is this project's own: the client has none
    /// of that name.
    pub(in crate::web::app) fn unread_switched(&mut self) {
        self.unread.open = !self.unread.open;
        self.redraw(&[&wnd::UNREAD]);
    }
}
