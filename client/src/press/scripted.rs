//! The presses of what an NPC's script shows: the cross of a picture that has one (the client's
//! `UIIllustWnd` of the kinds 3 and 4).

presses! {
    /// What can be pressed here.
    Press {
        /// The cross of the picture: it goes away.
        CutinClose = "cutin-close",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let (what, arg) = Press::CutinClose.name();
        assert_eq!(Press::parse(what, &arg), Some(Press::CutinClose));
    }
}
