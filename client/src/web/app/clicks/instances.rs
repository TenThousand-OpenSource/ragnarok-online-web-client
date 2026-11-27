//! The instances' window: shown and put away, and the leader's button that destroys the instance
//! (`crate::instances`, the client's `UIMemorialDunWnd::SendMsg`).

use super::*;

impl App {
    /// A press of this system (`crate::press::instances`).
    pub(super) fn instances_pressed(&mut self, press: crate::press::instances::Press) {
        use crate::press::instances::Press;
        match press {
            // `HK_MEMORIALWND_ONOFF`: the window that there is shows or hides; without an instance there is
            // none (`EnsureWindow` makes none for an empty name).
            Press::Window => {
                if self.instances.known.is_some() {
                    self.instances.shown = !self.instances.shown;
                    self.instances.wanted |= self.instances.shown;
                }
            }
            // `ID_CLOSE`: put away; and while the instance runs, it stays away when the server tells of the
            // instance again.
            Press::Close => {
                self.instances.shown = false;
                if self.instances.known.as_ref().is_some_and(|instance| instance.state == crate::instances::State::Running) {
                    self.instances.wanted = false;
                }
            }
            // `ID_DELETE`, `MM_MEMORIALDUNGEON_COMMAND`: the leader's alone, and asked of nobody twice.
            Press::Destroy => {
                if self.leads_party() {
                    self.send_on_map(&net::instances::instance_command(crate::instances::DESTROY));
                }
            }
        }
        self.redraw(&[&wnd::INSTANCE]);
    }
}
