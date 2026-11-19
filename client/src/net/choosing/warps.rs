//! The places of Teleport and Warp Portal: the list the skill opens, the place picked, and whether a place
//! was kept by `/memo`.

use super::*;

/// What the client answers when a list of places is closed without a choice (`UIChooseWarpWnd`: the word
/// goes to the server as a place would).
pub const CANCEL: &str = "cancel";
/// The place the server lists for Teleport that is any cell of the map.
pub const RANDOM: &str = "Random";

/// ZC_WARPLIST (0x0abe): `u16 length, u16 skill`, then the places, 16 bytes each (a map's file name, or
/// "Random"). The skill, and the places that are not empty.
pub fn warps(packet: &[u8]) -> Option<(u16, Vec<String>)> {
    let skill = word(packet, 4)?;
    Some((skill, records(packet, 6, 16).map(text).filter(|place| !place.is_empty()).collect()))
}

/// CZ_SELECT_WARPPOINT (0x011b, 20 bytes): `u16 skill, char place[16]`: the place picked, as the server
/// named it, or `CANCEL`.
pub fn warp(skill: u16, place: &str) -> Vec<u8> {
    Writer::new(0x011b).u16(skill).field::<16>(place).done()
}

/// ZC_ACK_REMEMBER_WARPPOINT (0x011e, 3 bytes): `u8 result`: 0 the place is kept, 1 the skill's level is too
/// low, 2 the skill is not known (the client knows a 3 too).
pub fn remembered(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}
