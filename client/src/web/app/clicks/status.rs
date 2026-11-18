//! Basic Information and the Status window.

use super::*;

impl App {
    /// A press of this system (`crate::press::status`).
    pub(super) fn status_pressed(&mut self, press: crate::press::status::Press) {
        use crate::press::status::Press;
        match press {
            Press::BasicSize => {
                self.hero.basic_small = !self.hero.basic_small;
                self.render();
            }
            Press::Window => {
                self.hero.status_open = !self.hero.status_open;
                self.render();
            }
            Press::Raise(stat) => {
                if let Some(socket) = &self.socket {
                    socket.send(&packets::raise_stat(packets::parameter::STR + stat.min(5), 1));
                }
            }
        }
    }
}
