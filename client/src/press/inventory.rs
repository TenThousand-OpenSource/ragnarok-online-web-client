//! The presses of what is carried: the inventory and its tabs, the Equipment window, an item's description, the
//! list of items a magnifier can identify, and the box that asks how many to drop.

presses! {
    /// What can be pressed here.
    Press {
        /// The inventory is opened or closed: its icon in the menu, and the cross in its corner.
        Window = "inventory",
        /// A tab of the inventory, counted from the top: what is used, what is worn, the rest.
        Tab(usize) = "inventory-tab",
        /// A row of the list of items to identify, counted from the top.
        IdentifyPick(usize) = "identify-pick",
        /// The list of items to identify: OK, for the row picked.
        IdentifyOk = "identify-ok",
        /// The list of items to identify: Cancel.
        IdentifyCancel = "identify-cancel",
        /// The box that asks how many to drop: OK. The box is a form and its OK sends it, so the press does nothing.
        ThrowOk = "throw-ok",
        /// The box that asks how many to drop is put away.
        ThrowCancel = "throw-cancel",
        /// The Equipment window's "off" button: off the mount, out of the machine, the falcon let go.
        MountOff = "mount-off",
        /// The Equipment window is opened or closed: its icon in the menu, and the cross in its corner.
        Equipment = "equipment",
        /// The cross of an item's description.
        InfoClose = "item-info-close",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [
            Press::Window,
            Press::Tab(2),
            Press::IdentifyPick(5),
            Press::IdentifyOk,
            Press::IdentifyCancel,
            Press::ThrowOk,
            Press::ThrowCancel,
            Press::MountOff,
            Press::Equipment,
            Press::InfoClose,
        ];
        assert_eq!(all.len(), Press::NAMES.len());
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
