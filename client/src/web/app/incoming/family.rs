//! What the server says of the family (`crate::net::family`): a wedding's confetti, the partner's name, a
//! divorce, someone asking to adopt the character and why an adoption cannot be.

use super::*;
use crate::net::family;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn family_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // `ReceiveCongratulation`: the confetti falls on that one (`EF_COLORPAPER`).
            family::ZC_CONGRATULATION => {
                if let Some(who) = family::congratulated(packet) {
                    world::burst(family::CONFETTI, (who != self.own_account()).then_some(who), None, 0.0);
                }
            }
            // `ReceiveCoupleName`: kept, as the client keeps it for the words of its partner's skills.
            family::ZC_COUPLENAME => self.family.partner = family::partner(packet).unwrap_or_default(),
            // `ReceiveDivorceNotice`: the client's line 836 with both names, in its `0xFFFF00` (blue first).
            family::ZC_DIVORCE => {
                if let (Some(other), Some(game)) = (family::partner(packet), &self.game) {
                    let line = crate::family::divorce_line(&Self::words(836, "%s and %s have divorced from each other."), &game.name, &other);
                    self.family.partner.clear();
                    self.chat_line(line, CYAN);
                }
            }
            // `ReceiveBabyRequest`: the question opens.
            family::ZC_REQ_BABY => {
                self.family.asks = family::adoption_asked(packet).map(|parent| (parent, false));
                self.redraw(&[&wnd::FAMILY_ASKS]);
            }
            // `ReceiveBabyMessage`: why not, in the client's `0x9B9BFF` (blue first).
            family::ZC_BABYMSG => {
                if let Some((line, otherwise)) = family::adoption_refused(packet) {
                    self.chat_line(Self::words(line, otherwise), "#ff9b9b");
                }
            }
            _ => return false,
        }
        true
    }
}
