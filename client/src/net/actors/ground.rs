//! The things that lie on the ground: one falling, one coming into sight, one gone, and picking one up.

use super::*;

/// ZC_ITEM_FALL_ENTRY (0x0add): `u32 id, u32 item, u16 type, u8 identified, i16 x, i16 y, u8 sub_x, u8 sub_y,
/// i16 count, …`: a thing falls to the ground. ZC_ITEM_ENTRY (0x009d): `u32 id, u32 item, u8 identified, u16 x,
/// u16 y, u16 count, …`: one that lay there comes into sight. ZC_ITEM_DISAPPEAR (0x00a1): `u32 id`.
pub const ZC_ITEM_FALL_ENTRY: u16 = 0x0add;
pub const ZC_ITEM_ENTRY: u16 = 0x009d;
pub const ZC_ITEM_DISAPPEAR: u16 = 0x00a1;

/// A thing on the ground, as ZC_ITEM_FALL_ENTRY and ZC_ITEM_ENTRY tell of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dropped {
    /// The thing's own number on the ground.
    pub id: u32,
    pub item: u32,
    pub cell: (u16, u16),
    /// Where within the cell, in sixteenths (`sub_x`, `sub_y`).
    pub within: (u8, u8),
    pub identified: bool,
    /// How many.
    pub count: u16,
}

/// A thing on the ground, from either packet (the count stands after the place within the cell in the one,
/// before it in the other).
pub fn dropped(id: u16, packet: &[u8]) -> Option<Dropped> {
    let (at, within, identified, count) = if id == ZC_ITEM_FALL_ENTRY { (13, 17, 12, 19) } else { (11, 17, 10, 15) };
    Some(Dropped {
        id: long(packet, 2)?,
        item: long(packet, 6)?,
        cell: (word(packet, at)?, word(packet, at + 2)?),
        within: (*packet.get(within)?, *packet.get(within + 1)?),
        identified: *packet.get(identified)? != 0,
        count: word(packet, count)?,
    })
}

/// CZ_ITEM_PICKUP (0x0362): `u32 id`.
pub fn pick_up(id: u32) -> Vec<u8> {
    Writer::new(0x0362).u32(id).done()
}
