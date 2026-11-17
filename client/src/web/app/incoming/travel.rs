//! Arriving on a map, the character's own steps, and being moved or sent back to the characters.

use super::*;

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn travel_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_ACCEPT_ENTER => self.entered_map(packet),
            packets::ZC_REFUSE_ENTER => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            packets::ZC_NOTIFY_PLAYERMOVE => {
                if let Some((from, to)) = packets::player_move(packet) {
                    world::walk(from, to);
                }
            }
            packets::ZC_STOPMOVE => {
                let own = self.account.as_ref().map(|account| account.account_id);
                if let Some((id, x, y)) = packets::stop_move(packet) {
                    if Some(id) == own { world::stand((x, y)) } else { world::unit_stand(id, (x, y)) }
                }
            }
            // 0x00b3: the server lets the character out of the game, back to the characters (type 1).
            0x00b3 => {
                if packet.get(2) == Some(&1) {
                    self.back_to_characters();
                }
            }
            // 0x018b: leaving was refused (a fight just now, for one).
            0x018b => {
                if packet.get(2..4) != Some(&[0, 0]) {
                    self.chat_line(crate::i18n::tr("You cannot leave the game right now.").to_owned(), "#ff8484");
                }
            }
            // Moved to another place, on this map or another: the map is loaded and the server told so.
            packets::ZC_NPCACK_MAPMOVE => {
                if let Some((map, x, y)) = packets::map_move(packet) {
                    // The screen goes dark, the character is moved, and the screen comes back, as in the
                    // original: no window is shown for it.
                    fade(true);
                    after_fade(Action::Arrive(map, x, y));
                }
            }
            _ => return false,
        }
        true
    }
}
