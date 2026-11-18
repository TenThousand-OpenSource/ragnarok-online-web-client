//! The presses of a talk with an NPC: "next" and "close" under what it says, a line of the menu it offers, and
//! the OK and Cancel of that menu and of the line to type into (`UISayDialogWnd`, `UIChooseWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// "Next" under what the NPC said: it goes on.
        Next = "dialog-next",
        /// "Close" under what the NPC said: the talk is over.
        Close = "dialog-close",
        /// OK: the picked line of the menu, or what was typed, is sent.
        Ok = "dialog-ok",
        /// Cancel: the menu is turned down.
        Cancel = "dialog-cancel",
        /// A line of the menu is picked: which, counted from 0.
        Pick(usize) = "dialog-pick",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Next, Press::Close, Press::Ok, Press::Cancel, Press::Pick(3)] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
