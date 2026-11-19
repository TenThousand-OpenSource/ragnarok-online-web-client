//! The family: the answers to one who asks to adopt the character (the client's `UIBabyAcceptWnd::SendMsg`
//! and `UIBabyAcceptWnd2::SendMsg`).

use super::*;
use crate::net::family;
use crate::press::family::Press;

impl App {
    /// A press of this system (`crate::press::family`): Cancel of either question answers no; OK of the first
    /// asks the second, which warns of what being adopted takes away, and OK of that one answers yes
    /// (`MM_JOIN_BABY`).
    pub(super) fn family_pressed(&mut self, press: Press) {
        match (press, self.adoption_asked_twice()) {
            (Press::Yes, Some(false)) => {
                if let Some((_, warned)) = &mut self.family.asks {
                    *warned = true;
                }
            }
            (press, Some(_)) => {
                if let Some((parent, _)) = self.family.asks.take() {
                    self.send_on_map(&family::adoption_answer(&parent, press == Press::Yes));
                }
            }
            (_, None) => {}
        }
        self.redraw(&[&wnd::FAMILY_ASKS]);
    }

    /// Whether a question of adoption stands, and whether it is the second one.
    fn adoption_asked_twice(&self) -> Option<bool> {
        self.family.asks.as_ref().map(|(_, warned)| *warned)
    }
}
