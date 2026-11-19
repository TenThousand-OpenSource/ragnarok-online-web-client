//! The presses of what a fight opens: the OK of the window Sense shows a monster in (the client's
//! `UIMonsterInfoWnd`), and the two buttons of the box that asks a Star Gladiator whether the map is to be a
//! place of the sun, the moon or the stars (`UIStarPlaceAcceptWnd`).

presses! {
    /// What can be pressed here.
    Press {
        /// The OK of the Monster Info window.
        SenseOk = "fight-sense-ok",
        /// The map is to be the place asked about.
        StarYes = "fight-star-yes",
        /// It is not.
        StarNo = "fight-star-no",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::SenseOk, Press::StarYes, Press::StarNo] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
