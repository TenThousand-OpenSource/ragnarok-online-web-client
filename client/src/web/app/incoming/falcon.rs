//! The hunter's falcon: the skills that send it at something. The falcon itself needs no packet of its own
//! (the world draws one for everyone whose effect state has its bit); this overhears the three packets by
//! which the client sends it flying: a hit of Blitz Beat or Falcon Assault (`CGameActor::ProcessSkillMessage`),
//! Detect used on a place (`CGameActor::PlayGroundSkillEffect`) and Spring Trap used on a trap
//! (`CGameMode::ReceiveSkillUse`).

use super::*;
use crate::falcon;
use world::Prey;

impl App {
    /// Overhears the packets of skills that send a falcon; never the reader of any.
    pub(super) fn falcon_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        let own = self.own_account();
        let other = |who: u32| (who != own).then_some(who);
        match id {
            skills::ZC_NOTIFY_SKILL => {
                if let Some(landing) = skills::landing(packet).filter(|landing| falcon::strikes(landing.skill)) {
                    world::falcon_sent(other(landing.source), Prey::Someone(other(landing.target)));
                }
            }
            skills::ZC_NOTIFY_GROUNDSKILL => {
                if let Some((_, who, _, cell)) = skills::on_ground(packet).filter(|used| used.0 == falcon::DETECT) {
                    world::falcon_sent(other(who), Prey::Cell(cell));
                }
            }
            // The one a Spring Trap is used on is the trap, a unit on the ground.
            skills::ZC_USE_SKILL => {
                if let Some((_, _, trap, who, _)) = skills::used(packet).filter(|used| used.0 == falcon::SPRING_TRAP) {
                    world::falcon_sent(other(who), Prey::Someone(Some(trap)));
                }
            }
            _ => {}
        }
        false
    }
}
