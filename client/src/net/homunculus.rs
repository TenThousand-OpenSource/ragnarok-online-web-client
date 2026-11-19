//! The alchemist's homunculus: what the map server tells of it, and what its owner's client asks for it.
//!
//! Layouts from rAthena (`clif_hominfo`, `clif_send_homdata`, `clif_homunculus_updatestatus`, `clif_hom_food`,
//! `clif_parse_HomMenu`, `clif_parse_HomMoveTo`, `clif_parse_HomAttack`, `clif_parse_HomMoveToMaster`,
//! `clif_parse_ChangeHomunculusName`) at PACKETVER 20260219. The skills come laid out as the character's own
//! (`net::inventory::skills`, `skill_raised`).
//!
//! What its window words and reckons (`intimacy_word`, `hungry_now`, `attack_speed`) is a rule and no packet: it
//! is in `crate::homunculus`. What its packets share with a pet's is in
//! `net/companions.rs`.

use super::bytes::{i64_at, text, u16_at, u32_at, Writer};
use super::companions;
use super::packets::parameter as character;

#[cfg(test)]
mod tests;


/// ZC_PROPERTY_HOMUN: everything about the homunculus at once. The server of this PACKETVER sends the last;
/// the two older layouts are read too, as the client reads them.
pub const ZC_PROPERTY_HOMUN: u16 = 0x022e;
pub const ZC_PROPERTY_HOMUN_2: u16 = 0x09f7;
pub const ZC_PROPERTY_HOMUN_4: u16 = 0x0ba4;
/// ZC_FEED_MER: whether the homunculus was fed.
pub const ZC_FEED_MER: u16 = 0x022f;
/// ZC_CHANGESTATE_MER: one thing about the homunculus changed.
pub const ZC_CHANGESTATE_MER: u16 = 0x0230;
/// ZC_HOSKILLINFO_LIST and ZC_HOSKILLINFO_UPDATE: its skills, and one of them raised.
pub const ZC_HOSKILLINFO_LIST: u16 = 0x0235;
pub const ZC_HOSKILLINFO_UPDATE: u16 = 0x0239;
/// ZC_HO_PAR_CHANGE: its life, its SP or its experience changed; the older packet carries 32 bits.
pub const ZC_HO_PAR_CHANGE: u16 = 0x07db;
pub const ZC_HO_PAR_CHANGE2: u16 = 0x0ba5;

/// The setting of CZ_CONFIG and ZC_CONFIG that has the homunculus fed by itself (the client's
/// `CSession::SendHomunculusAutoFeedOption`, rAthena's `CONFIG_HOMUNCULUS_AUTOFEED`).
pub const AUTOFEED: u32 = 3;

/// What ZC_CHANGESTATE_MER tells of (its `state`; the client's `HO_*`).
pub mod change {
    /// `OWN`: this unit is the player's homunculus.
    pub use crate::net::companions::change::{HUNGER, INTIMACY, OWN};
}

/// The commands of CZ_COMMAND_MER (the client's `MM_COMMAND_HOMUN`).
pub mod order {
    pub const INFO: u8 = 0;
    pub const FEED: u8 = 1;
    pub const DELETE: u8 = 2;
}

/// What ZC_HO_PAR_CHANGE names (the server's `SP_*`, as `net::packets::parameter` has them for the character).
pub mod parameter {
    use super::character;
    pub const EXP: u16 = character::BASE_EXP;
    pub const HP: u16 = character::HP;
    pub const SP: u16 = character::SP;
}

/// The first and the last skill of a homunculus (the client's `CSession::IsHomunculusSkill`).
pub fn is_skill(id: u32) -> bool {
    (8000..=8060).contains(&id)
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Homunculus {
    pub name: String,
    /// Bit 0: it was given a name already; bit 1: it rests (vaporized); bit 2: it lives.
    pub flags: u8,
    pub level: u16,
    /// 0 to 100.
    pub hunger: i32,
    /// 0 to 1000.
    pub intimacy: i32,
    /// Attack, magic attack, hit, critical, defence, magic defence, flee, and the delay between its blows in
    /// milliseconds: the eight numbers down the left of its window.
    pub stats: [i32; 8],
    pub hp: i64,
    pub max_hp: i64,
    pub sp: i64,
    pub max_sp: i64,
    pub exp: i64,
    pub next_exp: i64,
    pub skill_points: u16,
    /// How far its blows reach, in cells.
    pub range: u16,
}

/// ZC_PROPERTY_HOMUN in its three layouts. All begin `char name[24]`, `u8 flags`, `u16 level`, `u16 hunger`,
/// `u16 intimacy`; the two older ones have a `u16 accessory` next; then eight `u16`: attack, magic attack, hit,
/// critical, defence, magic defence, flee, attack delay. After them:
/// - 0x022e (71 bytes): `u16 hp, u16 max_hp, u16 sp, u16 max_sp, u32 exp, u32 next_exp`;
/// - 0x09f7 (75 bytes): `u32 hp, u32 max_hp, u16 sp, u16 max_sp, u32 exp, u32 next_exp`;
/// - 0x0ba4 (85 bytes): `u32 hp, u32 max_hp, u32 sp, u32 max_sp, i64 exp, i64 next_exp`;
/// and last `u16 skill_points, u16 range`.
pub fn property(id: u16, packet: &[u8]) -> Option<Homunculus> {
    let (length, stats_at) = match id {
        ZC_PROPERTY_HOMUN => (71, 35),
        ZC_PROPERTY_HOMUN_2 => (75, 35),
        ZC_PROPERTY_HOMUN_4 => (85, 33),
        _ => return None,
    };
    let packet = packet.get(..length)?;
    let word = |at: usize| i64::from(u16_at(packet, at));
    let long = |at: usize| i64::from(u32_at(packet, at));
    let after = stats_at + 16;
    let (hp, max_hp, sp, max_sp, exp, next_exp, rest) = match id {
        ZC_PROPERTY_HOMUN => (word(after), word(after + 2), word(after + 4), word(after + 6), long(after + 8), long(after + 12), after + 16),
        ZC_PROPERTY_HOMUN_2 => (long(after), long(after + 4), word(after + 8), word(after + 10), long(after + 12), long(after + 16), after + 20),
        _ => (long(after), long(after + 4), long(after + 8), long(after + 12), i64_at(packet, after + 16), i64_at(packet, after + 24), after + 32),
    };
    Some(Homunculus {
        name: text(&packet[2..26]),
        flags: packet[26],
        level: u16_at(packet, 27),
        hunger: i32::from(u16_at(packet, 29)),
        intimacy: i32::from(u16_at(packet, 31)),
        stats: std::array::from_fn(|stat| i32::from(u16_at(packet, stats_at + stat * 2))),
        hp,
        max_hp,
        sp,
        max_sp,
        exp,
        next_exp,
        skill_points: u16_at(packet, rest),
        range: u16_at(packet, rest + 2),
    })
}

/// ZC_CHANGESTATE_MER (0x0230, 12 bytes): `u8 type` (unused), `u8 state`, `u32 unit`, `i32 value`.
pub fn changed(packet: &[u8]) -> Option<(u8, u32, i32)> {
    companions::changed(packet, 3)
}

/// ZC_HO_PAR_CHANGE (0x07db, 8 bytes: `u16 parameter, i32 value`) and ZC_HO_PAR_CHANGE2 (0x0ba5, 12 bytes:
/// `u16 parameter, u64 value`).
pub fn parameter_changed(id: u16, packet: &[u8]) -> Option<(u16, i64)> {
    match id {
        ZC_HO_PAR_CHANGE => super::packets::number_and_value(packet).map(|(parameter, value)| (parameter, i64::from(value))),
        ZC_HO_PAR_CHANGE2 => packet.get(..12).map(|packet| (u16_at(packet, 2), i64_at(packet, 4))),
        _ => None,
    }
}

/// ZC_FEED_MER (0x022f, 7 bytes): `u8 fed`, `u32 item`: whether it ate, and the food it eats.
pub use companions::fed;

/// CZ_COMMAND_MER (0x022d, 5 bytes): `u16 type` (always 0), `u8 order`.
pub fn command(order: u8) -> Vec<u8> {
    Writer::new(0x022d).u16(0).u8(order).done()
}

/// CZ_RENAME_MER (0x0231, 26 bytes): `char name[24]`.
pub fn rename(name: &str) -> Vec<u8> {
    companions::rename(0x0231, name)
}

/// CZ_REQUEST_MOVENPC (0x0232, 9 bytes): `u32 unit`, then the cell in three bytes, ten bits each way (the
/// client's `AiMove`). The homunculus, or the mercenary, is asked to walk there.
pub fn move_to(unit: u32, cell: (u16, u16)) -> Vec<u8> {
    Writer::new(0x0232).u32(unit).bytes(&super::packets::cell_packed(cell)).done()
}

/// CZ_REQUEST_ACTNPC (0x0233, 11 bytes): `u32 unit, u32 target, u8 action` (always 0: one blow; the client's
/// `AiHomunAttack`).
pub fn attack(unit: u32, target: u32) -> Vec<u8> {
    Writer::new(0x0233).u32(unit).u32(target).u8(0).done()
}

/// CZ_REQUEST_MOVETOOWNER (0x0234, 6 bytes): `u32 unit`: it is asked to come to its owner (`AiMoveToOwner`).
pub fn to_owner(unit: u32) -> Vec<u8> {
    Writer::new(0x0234).u32(unit).done()
}
