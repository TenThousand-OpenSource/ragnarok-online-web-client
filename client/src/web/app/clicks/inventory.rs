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
            "throw-cancel" => {
                self.bag.throwing = None;
                self.refresh_throw();
            }
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
