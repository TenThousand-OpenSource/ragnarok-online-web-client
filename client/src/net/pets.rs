//! Pets: what the map server tells of the player's own (everything about it, one thing that changed, how
//! feeding it went), what any pet in sight shows and says, the eggs to hatch and the catch of a taming, and what
//! the client asks (`pets/requests.rs`). The words of the pet's window, what a pet feels and says at a moment
//! and the lines of `PetTalkTable.xml` are rules and no packets: they are in `crate::pets`, and named from here
//! as before.

use super::bytes::{i32_at, text, u16_at, u32_at, words_from};
use super::companions;

mod requests;
#[cfg(test)]
mod tests;

pub use crate::pets::{feeling, hunger_state, hunger_tag, hunger_word, intimacy_state, intimacy_word, moment, moment_tag, speaks, speech};
pub use requests::*;

/// ZC_START_CAPTURE (2 bytes): a taming item was used, and the monster to tame is to be picked.
pub const ZC_START_CAPTURE: u16 = 0x019e;
/// ZC_TRYCAPTURE_MONSTER: whether the monster was caught.
pub const ZC_TRYCAPTURE_MONSTER: u16 = 0x01a0;
/// ZC_PROPERTY_PET: everything about the pet at once.
pub const ZC_PROPERTY_PET: u16 = 0x01a2;
/// ZC_FEED_PET: whether the pet was fed.
pub const ZC_FEED_PET: u16 = 0x01a3;
/// ZC_CHANGESTATE_PET: one thing about a pet changed.
pub const ZC_CHANGESTATE_PET: u16 = 0x01a4;
/// ZC_PETEGG_LIST: the eggs that can be hatched.
pub const ZC_PETEGG_LIST: u16 = 0x01a6;
/// ZC_PET_ACT: a pet shows what it feels, or says something.
pub const ZC_PET_ACT: u16 = 0x01aa;

/// What ZC_CHANGESTATE_PET tells of (its `type`).
pub mod change {
    /// `OWN`: this unit is the player's pet.
    pub use crate::net::companions::change::{HUNGER, INTIMACY, OWN};
    pub const ACCESSORY: u8 = 3;
    /// The pet does a trick: the value is which (1 to 3, and 4 for its special one).
    pub const PERFORMANCE: u8 = 4;
    /// The pet went back into its egg when the value is 0.
    pub const EGG: u8 = 6;
}

/// The commands of CZ_COMMAND_PET, as the pet window's menu gives them.
pub mod order {
    pub const FEED: u8 = 1;
    pub const PERFORM: u8 = 2;
    pub const TO_EGG: u8 = 3;
    pub const UNEQUIP: u8 = 4;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pet {
    pub name: String,
    /// It was given a name already; that can be done once.
    pub renamed: bool,
    pub level: u16,
    /// 0 to 100.
    pub hunger: i32,
    /// 0 to 1000.
    pub intimacy: i32,
    /// The item it wears, 0 for none.
    pub accessory: u16,
    /// The monster it is.
    pub job: u16,
}

/// ZC_PROPERTY_PET (0x01a2, 37 bytes): `char name[24]`, `u8 renamed`, `u16 level`, `u16 hunger`,
/// `u16 intimacy`, `u16 accessory`, `u16 job`.
pub fn property(packet: &[u8]) -> Option<Pet> {
    let packet = packet.get(..37)?;
    Some(Pet {
        name: text(&packet[2..26]),
        renamed: packet[26] != 0,
        level: u16_at(packet, 27),
        hunger: i32::from(u16_at(packet, 29)),
        intimacy: i32::from(u16_at(packet, 31)),
        accessory: u16_at(packet, 33),
        job: u16_at(packet, 35),
    })
}

/// ZC_CHANGESTATE_PET (0x01a4, 11 bytes): `u8 type`, `u32 unit`, `i32 value`.
pub fn changed(packet: &[u8]) -> Option<(u8, u32, i32)> {
    companions::changed(packet, 2)
}

/// ZC_FEED_PET (0x01a3, 7 bytes): `u8 fed`, `u32 item`: whether the pet ate, and the food it eats.
pub use companions::fed;

/// ZC_PETEGG_LIST (0x01a6): `u16 length`, then the place in the inventory of each egg that can be hatched.
pub fn eggs(packet: &[u8]) -> Vec<u16> {
    words_from(packet, 4)
}

/// ZC_TRYCAPTURE_MONSTER (0x01a0, 3 bytes): `u8 caught`: whether the bait took.
pub fn caught(packet: &[u8]) -> Option<bool> {
    packet.get(2).map(|caught| *caught != 0)
}

/// What a pet does, as ZC_PET_ACT tells it.
#[derive(Debug, PartialEq)]
pub enum Act {
    /// It shows that emotion.
    Emotion(u16),
    /// It says a line: for that monster, at that moment, in that state of hunger.
    Speech { job: u16, moment: i32, hunger: usize },
}

/// ZC_PET_ACT (0x01aa, 10 bytes): `u32 unit`, `i32 number`. A number ending in 2 is an emotion (the rest of
/// it); any other is something said (the client's `ReceivePetAct`).
pub fn acted(packet: &[u8]) -> Option<(u32, Act)> {
    let packet = packet.get(..10)?;
    let (unit, number) = (u32_at(packet, 2), i32_at(packet, 6));
    let rest = number / 10;
    let act = if number % 10 == 2 { Act::Emotion(u16::try_from(rest).ok()?) } else { Act::Speech { job: u16::try_from(rest / 100).ok()?, moment: rest / 10 % 10, hunger: usize::try_from(rest % 10).ok()? } };
    Some((unit, act))
}
