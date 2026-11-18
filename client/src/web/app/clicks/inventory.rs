//! The windows of what is carried: the inventory, the equipment, an item's description.

use super::*;

impl App {
    /// A press of this system (`crate::press::inventory`).
    pub(super) fn item_pressed(&mut self, press: crate::press::inventory::Press) {
        use crate::press::inventory::Press;
        match press {
            Press::Window => {
                self.bag.open = !self.bag.open;
                self.render();
            }
            Press::Tab(tab) => {
                self.bag.tab = [Tab::Usable, Tab::Equipment, Tab::Other][tab.min(2)];
                self.refresh_items();
            }
            Press::IdentifyPick(row) => {
                self.bag.identify_picked = Some(row);
                self.render();
            }
            // CZ_REQ_ITEMIDENTIFY (0x0178): `u16 index`. OK asks for the picked item (and does nothing while none
            // is picked); Cancel tells the server -1, so it lets go of the magnifier's use.
            Press::IdentifyOk | Press::IdentifyCancel => {
                let picked = self.bag.identify.as_ref().zip(self.bag.identify_picked).and_then(|(items, picked)| items.get(picked).copied());
                let index = match (&press, picked) {
                    (Press::IdentifyOk, Some(index)) => index,
                    (Press::IdentifyOk, None) => return,
                    _ => 0xffff,
                };
                self.send(&items::identify(index));
                self.bag.identify = None;
                self.render();
            }
            // The box that asks how many to drop is a form, and its OK sends it (`data-submit="throw"`): the
            // press itself does nothing.
            Press::ThrowOk => {}
            Press::ThrowCancel => {
                self.bag.throwing = None;
                self.redraw(&[&wnd::THROW]);
            }
            // CZ_UNINSTALLATION (0x0b35): `u8 what`. The Equipment window's "off" button.
            Press::MountOff => self.send_on_map(&[0x35, 0x0b, crate::mounts::off(self.hero.state).0]),
            Press::Equipment => {
                self.bag.equipment_open = !self.bag.equipment_open;
                self.render();
            }
            Press::InfoClose => {
                self.bag.info = None;
                self.refresh_items();
            }
        }
    }
}
