//! A hired mercenary: what the map server tells of it, and the command its owner gives it.
//!
//! Layouts from rAthena (`clif_mercenary_info`, `clif_mercenary_updatestatus`, `clif_mercenary_skillblock`,
//! `clif_parse_mercenary_action`) at PACKETVER 20260219. Its skills come laid out as the character's own
//! (`net::inventory::skills`), and it is moved and made to strike with the homunculus's requests
//! (`net::homunculus::{move_to, attack, to_owner}`).
//!
//! The title of its window, its kind to its script and the time its contract ends (`title`, `kind`, `expiry`)
//! are rules and no packets: they are in `crate::mercenary`.

use super::bytes::{i16_at, i32_at, text, u16_at, u32_at, Writer};
use super::packets::parameter as character;


/// ZC_MER_INIT: everything about the mercenary at once.
pub const ZC_MER_INIT: u16 = 0x029b;
/// ZC_MER_SKILLINFO_LIST and ZC_MER_SKILLINFO_UPDATE: its skills, and one of them changed.
pub const ZC_MER_SKILLINFO_LIST: u16 = 0x029d;
pub const ZC_MER_SKILLINFO_UPDATE: u16 = 0x029e;
/// ZC_MER_PAR_CHANGE: one number of it changed.
pub const ZC_MER_PAR_CHANGE: u16 = 0x02a2;

/// The commands of CZ_MER_COMMAND (the client's `MM_COMMAND_MER`).
pub mod order {
    pub const INFO: u8 = 1;
    pub const DELETE: u8 = 2;
}

/// What ZC_MER_PAR_CHANGE names (the server's `SP_*`), those the client reads (`CGameMode::Zc_Mer_Par_Change`).
/// The first seven are the character's own (`net::packets::parameter`); the last three it does not have.
pub mod parameter {
    use super::character;
    pub const HP: u16 = character::HP;
    pub const MAX_HP: u16 = character::MAX_HP;
    pub const SP: u16 = character::SP;
    pub const MAX_SP: u16 = character::MAX_SP;
    pub const ATTACK: u16 = character::ATK;
    pub const HIT: u16 = character::HIT;
    pub const ATTACK_DELAY: u16 = character::ATTACK_DELAY;
    pub const FLEE: u16 = character::MERCENARY_FLEE;
    pub const KILLS: u16 = character::MERCENARY_KILLS;
    pub const FAITH: u16 = character::MERCENARY_FAITH;
}

/// The skills of mercenaries (the client's `CSession::IsMercenarySkill`).
pub fn is_skill(id: u32) -> bool {
    (8200..=8241).contains(&id) || (8400..=8457).contains(&id)
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mercenary {
    /// The unit on the map that is it.
    pub unit: u32,
    /// Attack, magic attack, hit, critical, defence, magic defence, flee, and the delay between its blows in
    /// milliseconds: the eight numbers down the left of its window.
    pub stats: [i32; 8],
    pub name: String,
    pub level: u16,
    pub hp: i32,
    pub max_hp: i32,
    pub sp: i32,
    pub max_sp: i32,
    /// When its contract ends: seconds since 1970.
    pub expires: u32,
    /// The faith its guild has in the player.
    pub faith: i32,
    /// How many mercenaries of its guild the player has called.
    pub calls: i32,
    pub kills: i32,
    /// How far its blows reach, in cells.
    pub range: u16,
}

/// ZC_MER_INIT (0x029b, 80 bytes): `u32 unit`, eight `u16` (attack, magic attack, hit, critical, defence, magic
/// defence, flee, attack delay), `char name[24]`, `u16 level`, `i32 hp, max_hp, sp, max_sp`, `u32 expires`,
/// `i16 faith`, `i32 calls`, `i32 kills`, `u16 range`.
pub fn init(packet: &[u8]) -> Option<Mercenary> {
    let packet = packet.get(..80)?;
    Some(Mercenary {
        unit: u32_at(packet, 2),
        stats: std::array::from_fn(|stat| i32::from(u16_at(packet, 6 + stat * 2))),
        name: text(&packet[22..46]),
        level: u16_at(packet, 46),
        hp: i32_at(packet, 48),
        max_hp: i32_at(packet, 52),
        sp: i32_at(packet, 56),
        max_sp: i32_at(packet, 60),
        expires: u32_at(packet, 64),
        faith: i32::from(i16_at(packet, 68)),
        calls: i32_at(packet, 70),
        kills: i32_at(packet, 74),
        range: u16_at(packet, 78),
    })
}

/// ZC_MER_PAR_CHANGE (0x02a2, 8 bytes): `u16 parameter, i32 value`.
pub fn parameter_changed(packet: &[u8]) -> Option<(u16, i32)> {
    super::packets::number_and_value(packet)
}

impl Mercenary {
    /// Takes in one number that changed (`CGameMode::Zc_Mer_Par_Change`: what the client reads of it, and
    /// nothing else).
    pub fn change(&mut self, parameter: u16, value: i32) {
        match parameter {
            parameter::HP => self.hp = value,
            parameter::MAX_HP => self.max_hp = value,
            parameter::SP => self.sp = value,
            parameter::MAX_SP => self.max_sp = value,
            parameter::ATTACK => self.stats[0] = value,
            parameter::HIT => self.stats[2] = value,
            parameter::FLEE => self.stats[6] = value,
            parameter::ATTACK_DELAY => self.stats[7] = value,
            parameter::KILLS => self.kills = value,
            parameter::FAITH => self.faith = value,
            _ => {}
        }
    }
}

/// CZ_MER_COMMAND (0x029f, 3 bytes): `u8 order`.
pub fn command(order: u8) -> Vec<u8> {
    Writer::new(0x029f).u8(order).done()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_everything_about_a_mercenary() {
        let mut packet = vec![0x9b, 0x02];
        packet.extend_from_slice(&0x0a0b0c0du32.to_le_bytes());
        for stat in [170u16, 9, 60, 3, 28, 4, 51, 950] {
            packet.extend_from_slice(&stat.to_le_bytes());
        }
        packet.extend_from_slice(&crate::net::bytes::name24("Mina"));
        packet.extend_from_slice(&15u16.to_le_bytes());
        for long in [256i32, 256, 50, 50] {
            packet.extend_from_slice(&long.to_le_bytes());
        }
        packet.extend_from_slice(&1_795_000_000u32.to_le_bytes());
        packet.extend_from_slice(&(-3i16).to_le_bytes());
        packet.extend_from_slice(&2i32.to_le_bytes());
        packet.extend_from_slice(&41i32.to_le_bytes());
        packet.extend_from_slice(&9u16.to_le_bytes());
        assert_eq!(packet.len(), 80);
        let read = init(&packet).unwrap();
        assert_eq!((read.unit, read.name.as_str(), read.level, read.stats), (0x0a0b0c0d, "Mina", 15, [170, 9, 60, 3, 28, 4, 51, 950]));
        assert_eq!(
            (read.hp, read.max_hp, read.sp, read.max_sp, read.expires, read.faith, read.calls, read.kills, read.range),
            (256, 256, 50, 50, 1_795_000_000, -3, 2, 41, 9)
        );
        assert_eq!(init(&packet[..79]), None);
    }

    #[test]
    fn takes_in_what_changes() {
        let mut mercenary = Mercenary { hp: 100, kills: 1, ..Mercenary::default() };
        let (parameter, value) = parameter_changed(&[0xa2, 0x02, 5, 0, 77, 0, 0, 0]).unwrap();
        mercenary.change(parameter, value);
        mercenary.change(parameter::KILLS, 2);
        mercenary.change(parameter::ATTACK_DELAY, 800);
        // The client does not read the magic attack, the critical or the defences: they stay.
        mercenary.change(43, 99);
        assert_eq!((mercenary.hp, mercenary.kills, mercenary.stats), (77, 2, [0, 0, 0, 0, 0, 0, 0, 800]));
        assert_eq!(command(order::DELETE), [0x9f, 0x02, 2]);
    }

    #[test]
    fn names_it_by_its_job() {
        assert!(is_skill(8201) && is_skill(8400) && !is_skill(8001) && !is_skill(8242));
    }
}
