//! The probabilities: the question the menu's icon of them asks (the client's `UIMenuIconWnd::SendMsg`,
//! `MENUICON_PROBABILITY`). The client has no window of its own for them.

use super::*;
use crate::press::probability::Press;

impl App {
    /// Whether to visit the page of the probabilities, in the client's message box (`UIWindowMgr::Confirm`
    /// with its line `MSI_ETC_PROBABILITY_WEB_CONNECT`, OK and Cancel).
    pub(in crate::web) fn probability_question(&self) -> String {
        let asks = data::probability_asks();
        confirm_box(if asks.is_empty() { "Visit the probability list webpage?" } else { &asks }, Press::Yes, Press::No)
    }
}
