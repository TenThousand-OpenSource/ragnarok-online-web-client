//! What the client asks of the map server about a pet: a command of its window's menu, an egg to hatch, a
//! monster to throw the bait to, a name for it, and what it shows or says.

use crate::net::bytes::Writer;
use crate::net::companions;

/// CZ_COMMAND_PET (0x01a1): `u8 order`.
pub fn command(order: u8) -> Vec<u8> {
    Writer::new(0x01a1).u8(order).done()
}

/// CZ_SELECT_PETEGG (0x01a7): `u16 index`. The egg at that place of the inventory is hatched.
pub fn hatch(index: u16) -> Vec<u8> {
    Writer::new(0x01a7).u16(index).done()
}

/// CZ_TRYCAPTURE_MONSTER (0x019f): `u32 target`. The bait is thrown to that monster.
pub fn capture(target: u32) -> Vec<u8> {
    Writer::new(0x019f).u32(target).done()
}

/// CZ_RENAME_PET (0x01a5): `char name[24]`.
pub fn rename(name: &str) -> Vec<u8> {
    companions::rename(0x01a5, name)
}

/// CZ_PET_ACT (0x01a9): `i32 number`: the pet's owner tells what its pet shows or says.
pub fn act(number: i32) -> Vec<u8> {
    Writer::new(0x01a9).i32(number).done()
}
