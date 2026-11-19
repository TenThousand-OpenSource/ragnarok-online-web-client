//! The presses of the merchant's cart: its window and the looks it may be given.

presses! {
    /// What can be pressed here.
    Press {
        /// The cart's window is opened or closed: its key, the Equipment window's button, the cross.
        Window = "cart",
        /// A look offered for the cart, by its kind: the cart is to look so.
        Look(u8) = "cart-look",
        /// The window of the looks is closed.
        LooksClose = "cart-looks-close",
        /// How many of a thing go into the cart or out of it: OK.
        CountOk = "cart-count-ok",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Window, Press::Look(13), Press::LooksClose, Press::CountOk];
        for name in Press::NAMES {
            assert!(all.iter().any(|press| press.name().0 == *name), "{name} is not tried");
        }
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
