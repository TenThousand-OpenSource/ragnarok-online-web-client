//! What the buttons of the windows a fight opens do: the OK of the Monster Info window, and the answer to the
//! server's question about a Star Gladiator's place (the client's `UIMonsterInfoWnd::SendMsg`,
//! `UIStarPlaceAcceptWnd::SendMsg` and `MM_STARPLACE_ACCEPT`).

use super::*;

impl App {
    /// A press of this system (`crate::press::fight`).
    pub(super) fn fight_pressed(&mut self, press: crate::press::fight::Press) {
        use crate::press::fight::Press;
        match press {
            Press::SenseOk => {
                self.fight.sensed = None;
                self.redraw(&[&wnd::SENSE_WINDOW]);
            }
            // The map is to be that place: the server is told which (CZ_AGREE_STARPLACE).
            Press::StarYes => {
                if let Some(which) = self.fight.star.take() {
                    self.send_on_map(&crate::net::fight::agree_star_place(which));
                }
                self.redraw(&[&wnd::STAR_ASKS]);
            }
            Press::StarNo => {
                self.fight.star = None;
                self.redraw(&[&wnd::STAR_ASKS]);
            }
        }
    }
}
