//! The spirit spheres about a monk and the coins about a gunslinger: how many someone has, as the server tells
//! it (`crate::spirits`, shown by `world/spirits.rs`).

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. One packet of another system
    /// is overheard and left to its own reader: someone going out of sight, whose spheres go too.
    pub(super) fn spirits_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // (One struck down stays in sight; the server tells its spheres gone by itself.)
            units::ZC_NOTIFY_VANISH => {
                if let Some((who, false)) = units::vanished(packet) {
                    world::spheres_gone(who);
                }
                return false;
            }
            // Both name nobody with an account of nothing (`CGameMode::UpdateActorSpiritCount`,
            // `CGameMode::Zc_Spirits2`).
            units::ZC_SPIRITS | units::ZC_SPIRITS2 => {
                if let Some((who, count)) = units::spirits(packet).filter(|(who, _)| *who != 0) {
                    let on = (who != self.own_account()).then_some(who);
                    if id == units::ZC_SPIRITS { world::spheres_told(on, count) } else { world::spheres_seen(on, count) }
                }
            }
            _ => return false,
        }
        true
    }
}
