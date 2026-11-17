//! Taming a monster and hatching its egg.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn pet_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // ZC_PETEGG_LIST (0x01a6): `u16 length`, then the place in the inventory of each egg. An incubator was
            // used: the eggs are listed to pick the one to hatch (the client's `UIPetEggListWnd`).
            0x01a6 => {
                self.eggs = Some(packet.get(4..).unwrap_or_default().chunks_exact(2).map(|index| u16::from_le_bytes([index[0], index[1]])).collect());
                self.egg_picked = 0;
                self.render();
            }
            // ZC_START_CAPTURE (0x019e): a taming item was used. The monster to tame is picked with the pointer, as
            // a skill's target is (the client's `Zc_Start_Capture`: reach 5).
            0x019e => {
                self.aimed = Some((TAME, 0));
                world::aim(Some(false), 5);
                // Taming and fighting do not go together: the fight stops as soon as the bait is in hand.
                world::stand_down();
                if let Some(socket) = &self.socket {
                    socket.send(&0x0118u16.to_le_bytes());
                }
            }
            // ZC_TRYCAPTURE_MONSTER (0x01a0): `u8 caught`. The reels slow down and stop on the picture of the
            // catch (the eighth) or of the miss (the fourth).
            0x01a0 => {
                if let (Some(taming), Some(caught)) = (&mut self.taming, packet.get(2)) {
                    (taming.wanted, taming.delay, taming.longest) = (if *caught != 0 { 7 } else { 3 }, 7, 7);
                }
            }
            _ => return false,
        }
        true
    }
}
