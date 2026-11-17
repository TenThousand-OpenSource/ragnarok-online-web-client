//! Basic Information and the Status window.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn status_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "basic-size" => {
                self.hero.basic_small = !self.hero.basic_small;
                self.render();
            }
            "status" => {
                self.hero.status_open = !self.hero.status_open;
                self.render();
            }
            "raise" => {
                if let (Ok(stat), Some(socket)) = (arg.parse::<u16>(), &self.socket) {
                    socket.send(&packets::raise_stat(packets::parameter::STR + stat.min(5), 1));
                }
            }
            _ => return false,
        }
        true
    }
}
