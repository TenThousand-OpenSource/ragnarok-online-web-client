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
            // Opened, the window puts away the button that tells of a level gained and takes the mark off
            // its icon (`UIWindowMgr::EnsureWindow`, `WID_STATUSWND`: `CloseWindow(WID_NOTIFYLEVELUPWND)` and
            // `ID_NOTICE` off).
            Press::Window => {
                self.hero.status_open = !self.hero.status_open;
                self.hero.level_told &= !self.hero.status_open;
                self.render();
            }
            // The button that tells of a level gained (`UINotifyLevelUpWnd::SendMsg`): the Status window is
            // shown, which puts the button away, and comes to the front.
            Press::LevelGained => {
                (self.hero.status_open, self.hero.level_told) = (true, false);
                self.render();
                self.window_pressed(wnd::STATUS.id);
            }
            Press::Raise(stat) => {
                if let Some(socket) = &self.socket {
                    socket.send(&packets::raise_stat(packets::parameter::STR + stat.min(5), 1));
                }
            }
        }
    }
}
