//! The probabilities: the menu's icon asks, and a yes opens the page.

use super::*;
use crate::press::probability::Press;

impl App {
    /// A press of this system (`crate::press::probability`).
    pub(super) fn probability_pressed(&mut self, press: Press) {
        match press {
            // `UIMenuIconWnd::SendMsg`, `MENUICON_PROBABILITY`: the client's message box asks.
            Press::Ask => self.probability.asking = true,
            // …and on a yes the client hands the address of its line `MSI_ETC_PROBABILITY_URL` to Windows
            // (`ShellExecuteA`, "open"): the browser opens it beside the game. A line that is no address of
            // the web opens nothing.
            Press::Yes => {
                self.probability.asking = false;
                if let (Some(address), Some(window)) = (crate::probability::address(&data::probability_address()), web_sys::window()) {
                    let _ = window.open_with_url_and_target_and_features(address, "_blank", "noopener");
                }
            }
            Press::No => self.probability.asking = false,
        }
        self.render();
    }
}
