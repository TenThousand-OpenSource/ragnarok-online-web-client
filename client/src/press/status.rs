//! The presses of Basic Information and the Status window: the button that makes Basic Information small, the
//! one that opens and closes the Status window, the arrows that raise a stat, and the button that tells of a level
//! gained.

presses! {
    /// What can be pressed here.
    Press {
        /// Basic Information's button that makes it small and large again.
        BasicSize = "basic-size",
        /// The Status window is opened or closed.
        Window = "status",
        /// The arrow beside a stat raises it by one: which stat, counted from 0 at STR.
        Raise(u16) = "raise",
        /// The button that tells of a base level gained (the client's `UINotifyLevelUpWnd`): the Status window
        /// opens.
        LevelGained = "level-gained",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::BasicSize, Press::Window, Press::Raise(4), Press::LevelGained] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
