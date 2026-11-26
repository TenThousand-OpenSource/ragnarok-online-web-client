//! What the server says of the navigation (`crate::net::navigation`), and what the navigation overhears of
//! the character's own steps.

use super::*;
use crate::navigation::texts;
use crate::net::navigation;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn navigation_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The server has the client find a way (`CGameMode::ReceiveNavigation`; rAthena's `navigateto`).
            navigation::ZC_NAVIGATION_ACTIVE => {
                if let Some(asked) = navigation::asked(packet) {
                    self.navigate(Wanted::Start {
                        map: asked.map,
                        kind: i32::from(asked.kind),
                        mode: texts::mode_of(i32::from(asked.services)),
                        hide: asked.hide,
                        x: i32::from(asked.cell.0),
                        y: i32::from(asked.cell.1),
                        id: i32::from(asked.monster),
                    });
                }
                true
            }
            // Overheard: the server sends the character walking (`CPlayer`'s `CheckArrival` with the cell
            // walked to), and lets it out of the game (`ZC_RESTART_ACK`: the navigation is forgotten).
            packets::ZC_NOTIFY_PLAYERMOVE => {
                if let Some(walk) = packets::player_move(packet) {
                    self.navigation_walked((i32::from(walk.to.0), i32::from(walk.to.1)));
                }
                false
            }
            packets::ZC_RESTART_ACK => {
                self.navigation_forgotten();
                false
            }
            _ => false,
        }
    }
}
