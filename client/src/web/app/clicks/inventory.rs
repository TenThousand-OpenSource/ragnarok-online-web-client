//! The windows of what is carried: the inventory, the equipment, an item's description.

use super::*;

impl App {
    /// What a press on one of these does; `false` when the thing pressed is not of this kind.
    pub(super) fn item_click(&mut self, what: &str, arg: &str) -> bool {
        match what {
            "inventory" => {
                self.bag.open = !self.bag.open;
                self.render();
            }
            "inventory-tab" => {
                self.bag.tab = [Tab::Usable, Tab::Equipment, Tab::Other][arg.parse::<usize>().unwrap_or(0).min(2)];
                self.refresh_items();
            }
            "identify-pick" => {
                self.bag.identify_picked = arg.parse().ok();
                self.render();
            }
            // CZ_REQ_ITEMIDENTIFY (0x0178): `u16 index`. OK asks for the picked item (and does nothing while none
            // is picked); Cancel tells the server -1, so it lets go of the magnifier's use.
            "identify-ok" | "identify-cancel" => {
                let picked = self.bag.identify.as_ref().zip(self.bag.identify_picked).and_then(|(items, picked)| items.get(picked).copied());
                let index = match (what, picked) {
                    ("identify-ok", Some(index)) => index,
                    ("identify-ok", None) => return true,
                    _ => 0xffff,
                };
                self.send(&items::identify(index));
                self.bag.identify = None;
                self.render();
            }
            "throw-cancel" => {
                self.bag.throwing = None;
                self.refresh_throw();
            }
            // CZ_UNINSTALLATION (0x0b35): `u8 what`. The Equipment window's "off" button.
            "mount-off" => self.send_on_map(&[0x35, 0x0b, crate::mounts::off(self.hero.state).0]),
            "equipment" => {
                self.bag.equipment_open = !self.bag.equipment_open;
                self.render();
            }
            "item-info-close" => {
                self.bag.info = None;
                self.refresh_items();
            }
            _ => return false,
        }
        true
    }
}
