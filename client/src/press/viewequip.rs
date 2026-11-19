//! The presses of looking at another player's equipment: the cross of the window that shows it (the client's
//! `UIEquipWnd` with `m_viewOther`), and the box of the player's own Equipment window that lets others look.

presses! {
    /// What can be pressed here.
    Press {
        /// The cross of the window that shows another's equipment.
        Close = "viewequip-close",
        /// The box "Show Equipment": others may look at what the player wears, or may not.
        Allow = "viewequip-allow",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Close, Press::Allow] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
