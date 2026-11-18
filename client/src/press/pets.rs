//! The presses of the pet: the slot machine of taming, the list of eggs to hatch, the pet window with its menu
//! of commands and its questions, and the menu on the pet itself.

/// A row of the pet's menus: what the pet is told to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    /// The pet window is opened (only the menu on the pet itself has this row).
    Info,
    Feed,
    Perform,
    /// Its accessory is taken off.
    Unequip,
    /// Back into its egg.
    Egg,
}

impl Order {
    /// Every order, as the menu on the pet itself lists them.
    pub const ALL: [Order; 5] = [Order::Info, Order::Feed, Order::Perform, Order::Unequip, Order::Egg];

    fn text(self) -> &'static str {
        match self {
            Order::Info => "info",
            Order::Feed => "feed",
            Order::Perform => "perform",
            Order::Unequip => "unequip",
            Order::Egg => "egg",
        }
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, to: &mut std::fmt::Formatter) -> std::fmt::Result {
        to.write_str(self.text())
    }
}

impl std::str::FromStr for Order {
    type Err = ();

    fn from_str(text: &str) -> Result<Self, ()> {
        Order::ALL.into_iter().find(|order| order.text() == text).ok_or(())
    }
}

presses! {
    /// What can be pressed here.
    Press {
        /// The slot machine of taming: its reels are started.
        TamingShoot = "taming-shoot",
        /// A row of the list of eggs to hatch, counted from the top.
        EggPick(usize) = "egg-pick",
        /// The list of eggs: OK, the picked egg is hatched.
        EggOk = "egg-ok",
        /// The list of eggs: Cancel.
        EggCancel = "egg-cancel",
        /// The pet window is opened or closed: the cross in its corner.
        Window = "pet",
        /// The pet window's `< Command >` box: its menu is opened or closed.
        Menu = "pet-menu",
        /// A row of the pet window's menu, or of the menu on the pet itself.
        Do(Order) = "pet-do",
        /// The button by the pet's name: the name typed is to be given to it.
        Rename = "pet-rename",
        /// What the pet window asks (to feed, to rename): OK.
        Yes = "pet-yes",
        /// What the pet window asks: Cancel.
        No = "pet-no",
    }
}

#[cfg(test)]
mod tests {
    use super::{Order, Press};
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let mut all = vec![Press::TamingShoot, Press::EggPick(3), Press::EggOk, Press::EggCancel, Press::Window, Press::Menu, Press::Rename, Press::Yes, Press::No];
        all.extend(Order::ALL.map(Press::Do));
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }

    #[test]
    fn an_order_nobody_gives_is_no_press() {
        assert_eq!(Press::parse("pet-do", "sit"), None);
    }
}
