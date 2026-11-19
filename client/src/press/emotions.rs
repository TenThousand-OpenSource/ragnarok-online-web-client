//! The presses of the window of emotions (the client's `UICashEmotionListWnd`, Alt+L) and of the Shortcut List
//! (`UIEmotionWnd`, Alt+M): ten lines, each said with Alt and a digit.

presses! {
    /// What can be pressed here.
    Press {
        /// The window of emotions opens, or closes if it was open.
        Window = "emotions",
        /// An emotion of the grid, by its number: its name is written where the player types
        /// (`UM_CASH_EMOTION_ACTIVATE`). A double click shows it (`press::things::Use::Emotion`).
        Pick(u16) = "emotion-pick",
        /// The Shortcut List opens, or closes if it was open: from the window of emotions, and from its own key.
        Shortcuts = "shortcuts",
        /// The Shortcut List's button that shows the window of emotions.
        View = "shortcuts-view",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Window, Press::Pick(18), Press::Shortcuts, Press::View];
        assert_eq!(all.len(), Press::NAMES.len(), "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
