//! The presses of the Tipbox (the client's `UITipboxWnd`, Alt+D, `/tip`).

use super::Step;

presses! {
    /// What can be pressed here.
    Press {
        /// The window opens on a tip picked by lot, or closes if it was open.
        Window = "tips",
        /// The arrows: the next page of the tip and the one before.
        Page(Step) = "tips-page",
        /// A link to another tip in the text, or a row of what a search found: that tip is shown.
        Tip(u32) = "tips-tip",
        /// The box "Open on Startup".
        Startup = "tips-startup",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        let all = [Press::Window, Press::Page(Step::More), Press::Page(Step::Less), Press::Tip(1000), Press::Startup];
        assert_eq!(all.len(), Press::NAMES.len() + 1, "a press is not tested");
        for press in all {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
    }
}
