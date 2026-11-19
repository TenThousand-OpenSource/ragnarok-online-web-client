//! The presses of the list of packets this client does not read (a tool of this project's own, `/packets`).

presses! {
    /// What can be pressed here.
    Press {
        /// The cross of the window.
        Close = "unread-close",
        /// Everything counted so far is forgotten.
        Clear = "unread-clear",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::Close, Press::Clear] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
