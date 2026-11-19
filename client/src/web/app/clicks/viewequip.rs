//! Looking at another player's equipment: asking for it, closing its window, and the box of the player's own
//! Equipment window that lets others look (the client's `MENUID_VIEW_OTHERUSER_EQUIPWND`,
//! `UIEquipWnd::SendMsg` with `ID_CHECK`, and `MM_OPEN_EQUIPEDITEM_CHANGE`).

use super::*;
use crate::net::viewequip;

impl App {
    /// A press of this system (`crate::press::viewequip`).
    pub(super) fn viewequip_pressed(&mut self, press: crate::press::viewequip::Press) {
        use crate::press::viewequip::Press;
        match press {
            Press::Close => {
                self.viewequip.seen = None;
                self.redraw(&[&wnd::VIEW_EQUIP]);
            }
            Press::Allow => self.equipment_open_switched(),
        }
    }

    /// The server is asked what that player wears; its answer opens the window, or says in the chat that the
    /// other does not let it be looked at (`incoming/viewequip.rs`).
    pub(in crate::web::app) fn viewequip_asked(&mut self, account: u32) {
        self.send_on_map(&viewequip::ask(account));
    }

    /// The server is asked to let others look at the player's equipment, or to stop letting them: the box
    /// of the Equipment window, and the client's `/eqopen` (`TT_REQ_OPEN_EQUIPEDITEM_ONOFF`). The box turns
    /// when the server has answered.
    pub(in crate::web::app) fn equipment_open_switched(&mut self) {
        self.send_on_map(&crate::net::config::set(viewequip::OPEN_EQUIPMENT, !self.viewequip.allowed));
    }
}
