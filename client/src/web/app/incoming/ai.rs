//! What the scripts of the homunculus and the mercenary need of the server's packets and the world does not
//! keep: whom everyone last struck. The packets belong to other systems, which read them after this has
//! overheard them.

use super::*;

impl App {
    /// Overhears the packets that tell of a blow, a skill or someone gone; never the reader of any.
    pub(super) fn ai_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        // Nobody listens while the player has neither a homunculus nor a mercenary.
        if self.homunculus.unit == 0 && self.mercenary.info.is_none() {
            return false;
        }
        match id {
            // A blow, of any kind but picking up, sitting down and standing up (the client's
            // `ProcessHitMessage` notes the one struck as the striker's `hitTargetAid`).
            net::actors::ZC_NOTIFY_ACT => {
                if let Some(act) = net::actors::act(packet).filter(|act| !matches!(act.kind, 1..=3)) {
                    self.ai.targets.insert(act.source, act.target);
                }
            }
            skills::ZC_NOTIFY_SKILL => {
                if let Some(landing) = skills::landing(packet) {
                    self.ai.targets.insert(landing.source, landing.target);
                }
            }
            skills::ZC_USESKILL_ACK => {
                if let Some(casting) = skills::casting(packet).filter(|casting| casting.target != 0) {
                    self.ai.targets.insert(casting.source, casting.target);
                }
            }
            net::actors::ZC_NOTIFY_VANISH => {
                if let Some((gone, _)) = net::actors::vanished(packet) {
                    self.ai.targets.remove(&gone);
                }
            }
            // On another map no one of the last is in sight.
            packets::ZC_NPCACK_MAPMOVE | packets::ZC_ACCEPT_ENTER => self.ai.targets.clear(),
            _ => {}
        }
        false
    }
}
