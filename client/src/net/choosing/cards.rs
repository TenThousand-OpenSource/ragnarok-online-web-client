//! A card set into equipment: what the card can go into, the thing picked, and whether it is in.

use super::*;

/// CZ_REQ_ITEMCOMPOSITION_LIST (0x017a, 4 bytes): `u16 index`: asks what a card carried there can be set
/// into.
pub fn ask_compound(card: u16) -> Vec<u8> {
    Writer::new(0x017a).u16(card).done()
}

/// ZC_ITEMCOMPOSITION_LIST (0x017b): `u16 length`, then where each thing is carried that the card can be
/// set into.
pub fn compoundable(packet: &[u8]) -> Vec<u16> {
    words_from(packet, 4)
}

/// CZ_REQ_ITEMCOMPOSITION (0x017c, 6 bytes): `u16 card, u16 equipment`: where each is carried.
pub fn compound(card: u16, equipment: u16) -> Vec<u8> {
    Writer::new(0x017c).u16(card).u16(equipment).done()
}

/// ZC_ACK_ITEMCOMPOSITION (0x017d, 7 bytes): `u16 equipment, u16 card, u8 result` (0 the card is in).
pub fn compounded(packet: &[u8]) -> Option<(u16, u16, bool)> {
    Some((word(packet, 2)?, word(packet, 4)?, *packet.get(6)? == 0))
}
