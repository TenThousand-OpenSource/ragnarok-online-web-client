//! What a skill makes: the arrows to craft, the spells of Auto Spell, what can be forged, brewed and cooked;
//! the thing picked, and how the making ended. What may be put into a forging is a rule: `crate::choosing`.

use super::*;

/// The numbers of items a packet lists from that place to its end, four bytes each.
fn ids_from(packet: &[u8], at: usize) -> Vec<u32> {
    records(packet, at, 4).map(|id| u32_at(id, 0)).collect()
}

/// ZC_MAKINGARROW_LIST (0x01ad): `u16 length`, then `u32 item` each: what arrows can be made of.
pub fn arrow_sources(packet: &[u8]) -> Vec<u32> {
    ids_from(packet, 4)
}

/// CZ_REQ_MAKINGARROW (0x01ae, 6 bytes): `u32 item`: the thing to make arrows of.
pub fn make_arrows(item: u32) -> Vec<u8> {
    Writer::new(0x01ae).u32(item).done()
}

/// ZC_AUTOSPELLLIST (0x0afb): `u16 length`, then `u32 skill` each: the spells Auto Spell may cast (noughts
/// are left out, as the client leaves them).
pub fn spells(packet: &[u8]) -> Vec<u32> {
    ids_from(packet, 4).into_iter().filter(|skill| *skill != 0).collect()
}

/// CZ_SELECTAUTOSPELL (0x01ce, 6 bytes): `u32 skill`.
pub fn pick_spell(skill: u32) -> Vec<u8> {
    Writer::new(0x01ce).u32(skill).done()
}

/// ZC_MAKABLEITEMLIST (0x018d): `u16 length`, then 16 bytes a thing: `u32 item, u32 material[3]`: what can
/// be forged or brewed now. The materials are noughts.
pub fn makable(packet: &[u8]) -> Vec<u32> {
    records(packet, 4, 16).map(|thing| u32_at(thing, 0)).collect()
}

/// CZ_REQMAKINGITEM (0x018e, 18 bytes): `u32 item, u32 material[3]`: the thing to make, and what was put in
/// beside what it needs (the stone of an element, star crumbs). Item 0 calls the making off
/// (`UIMakeTargetListWnd`, Cancel).
pub fn make(item: u32, materials: [u32; 3]) -> Vec<u8> {
    materials.iter().fold(Writer::new(0x018e).u32(item), |packet, material| packet.u32(*material)).done()
}

/// ZC_ACK_REQMAKINGITEM (0x018f, 8 bytes): `u16 result, u32 item`: how a making ended. The results the
/// client knows (`CGameMode::ReceiveItemEffectResult`): 0 forged, 1 failed, 2 brewed, 3 failed; 4 to 7 are
/// of later content.
pub fn made(packet: &[u8]) -> Option<(u16, u32)> {
    Some((word(packet, 2)?, long(packet, 4)?))
}

/// ZC_MAKINGITEM_LIST (0x025a): `u16 length, u16 kind`, then `u32 item` each: what can be cooked (kind 1)
/// or mixed by a later skill; the kind goes back with the choice.
pub fn cookable(packet: &[u8]) -> Option<(u16, Vec<u32>)> {
    Some((word(packet, 4)?, ids_from(packet, 6)))
}

/// CZ_REQ_MAKINGITEM (0x025b, 8 bytes): `u16 kind, u32 item`.
pub fn cook(kind: u16, item: u32) -> Vec<u8> {
    Writer::new(0x025b).u16(kind).u32(item).done()
}
