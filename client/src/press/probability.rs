//! The presses of the probabilities: the menu's icon and the two answers to what it asks (the client's
//! `UIMenuIconWnd::SendMsg`, `MENUICON_PROBABILITY`).

presses! {
    /// What can be pressed here.
    Press {
        /// The menu's icon: whether to visit the page of the probabilities is asked.
        Ask = "probability",
        /// The page is opened.
        Yes = "probability-yes",
        /// It is not.
        No = "probability-no",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        crate::press::testing::read_back([Press::Ask, Press::Yes, Press::No]);
    }
}
