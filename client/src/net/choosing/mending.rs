//! What a smith mends and betters: the broken things to repair, the weapons to refine, the thing picked of
//! each and how it went; and a thing refined at an NPC.

use super::*;
use crate::net::inventory::cards;

/// A thing in a list of what can be repaired or refined: where its owner carries it, and what it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub index: u16,
    pub id: u32,
    pub refine: u8,
    pub cards: [u32; 4],
}

/// ZC_REPAIRITEMLIST (0x0b65): `u16 length`, then 24 bytes a thing: `i16 index, u32 item, u32 card[4], u8
/// refine, u8 grade`: the broken things of the one the skill was used on.
pub fn repairable(packet: &[u8]) -> Vec<Listed> {
    records(packet, 4, 24).map(|thing| Listed { index: u16_at(thing, 0), id: u32_at(thing, 2), cards: cards(thing, 6), refine: thing[22] }).collect()
}

/// CZ_REQ_ITEMREPAIR (0x0b66, 26 bytes): the thing picked, as the list had it (`i16 index, u32 item, u32
/// card[4], u8 refine, u8 grade`). The client answers a list closed without a choice with the index -1
/// (`MM_REPAIRITEM_SELECT_FROM_LIST`).
pub fn repair(thing: Option<&Listed>) -> Vec<u8> {
    let packet = Writer::new(0x0b66);
    match thing {
        // The grade, which the list has too, goes back as nought.
        Some(thing) => thing.cards.iter().fold(packet.u16(thing.index).u32(thing.id), |packet, card| packet.u32(*card)).u8(thing.refine).u8(0),
        None => packet.u16(0xffff).bytes(&[0; 22]),
    }
    .done()
}

/// ZC_ACK_ITEMREPAIR (0x01fe, 5 bytes): `u16 index, u8 result`: whether the repair failed.
pub fn repaired(packet: &[u8]) -> Option<bool> {
    packet.get(4).map(|result| *result == 0)
}

/// ZC_NOTIFY_WEAPONITEMLIST (0x0221): `u16 length`, then 23 bytes a weapon: `i16 index, u32 item, u8 refine,
/// u32 card[4]`: the weapons carried that the smith's skill can refine.
pub fn refinable(packet: &[u8]) -> Vec<Listed> {
    records(packet, 4, 23).map(|thing| Listed { index: u16_at(thing, 0), id: u32_at(thing, 2), refine: thing[6], cards: cards(thing, 7) }).collect()
}

/// CZ_REQ_WEAPONREFINE (0x0222, 6 bytes): `i32 index`: the weapon picked, or -1 for none
/// (`MM_REFINEITEM_SELECT_FROM_LIST`).
pub fn refine(index: Option<u16>) -> Vec<u8> {
    Writer::new(0x0222).i32(index.map_or(-1, i32::from)).done()
}

/// ZC_ACK_WEAPONREFINE (0x0223, 10 bytes): `i32 result, u32 item`: 0 refined, 1 failed, 2 the skill's level
/// is too low for it, 3 the stone for it is lacking.
pub fn refined(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

/// ZC_ACK_ITEMREFINING (0x0188, 8 bytes): `u16 result, u16 index, u16 refine`: a thing carried was refined
/// at an NPC: 0 it went well, 1 it failed, 2 it lost a level (3 is another failure the client says nothing
/// of); and how far it is refined now.
pub fn item_refined(packet: &[u8]) -> Option<(u16, u16, u8)> {
    Some((word(packet, 2)?, word(packet, 4)?, word(packet, 6)?.min(255) as u8))
}
