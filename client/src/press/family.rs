//! The presses of the family: the answers to one who asks to adopt the character (the client's
//! `UIBabyAcceptWnd` and `UIBabyAcceptWnd2`).

presses! {
    /// What can be pressed here.
    Press {
        /// OK and Cancel of the question.
        Yes = "family-yes",
        No = "family-no",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Yes, Press::No] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
