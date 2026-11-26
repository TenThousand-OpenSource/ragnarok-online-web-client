//! The presses of the reputation's window (the client's `UIReputeWnd`).

use super::Step;

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon, the window's key, its cross and Close: the window opens, or closes if it was open.
        Window = "reputation",
        /// The drop-down of the groups opens, or closes.
        Groups = "reputation-groups",
        /// A line of the drop-down: 0 all of them, then the groups in their order.
        Group(usize) = "reputation-group",
        /// The arrows: the page after and the page before.
        Page(Step) = "reputation-page",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back([Press::Window, Press::Groups, Press::Group(3), Press::Page(Step::More), Press::Page(Step::Less)]);
    }
}
