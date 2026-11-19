//! A hired mercenary: what the map server tells of it, and the command its owner gives it.
//!
//! Layouts from rAthena (`clif_mercenary_info`, `clif_mercenary_updatestatus`, `clif_mercenary_skillblock`,
//! `clif_parse_mercenary_action`) at PACKETVER 20260219. Its skills come laid out as the character's own
//! (`net::items::skills`), and it is moved and made to strike with the homunculus's requests
//! (`net::homunculus::{move_to, attack, to_owner}`).

use super::bytes::{text, u16_at, u32_at};

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
pub mod parameter {
    pub const HP: u16 = 5;
    pub const MAX_HP: u16 = 6;
    pub const SP: u16 = 7;
    pub const MAX_SP: u16 = 8;
    pub const ATTACK: u16 = 41;
    pub const HIT: u16 = 49;
    pub const ATTACK_DELAY: u16 = 53;
    pub const FLEE: u16 = 165;
    pub const KILLS: u16 = 189;
    pub const FAITH: u16 = 190;
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
    let long = |at: usize| u32_at(packet, at) as i32;
    Some(Mercenary {
        unit: u32_at(packet, 2),
        stats: std::array::from_fn(|stat| i32::from(u16_at(packet, 6 + stat * 2))),
        name: text(&packet[22..46]),
        level: u16_at(packet, 46),
        hp: long(48),
        max_hp: long(52),
        sp: long(56),
        max_sp: long(60),
        expires: u32_at(packet, 64),
        faith: i32::from(u16_at(packet, 68) as i16),
        calls: long(70),
        kills: long(74),
        range: u16_at(packet, 78),
    })
}

/// ZC_MER_PAR_CHANGE (0x02a2, 8 bytes): `u16 parameter, i32 value`.
pub fn parameter_changed(packet: &[u8]) -> Option<(u16, i32)> {
    let packet = packet.get(..8)?;
    Some((u16_at(packet, 2), u32_at(packet, 4) as i32))
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
    vec![0x9f, 0x02, order]
}

/// The title of its window, as the line of the client's message table and the English of it, by the job it is
/// (`UIMerInfoWnd::Paint`: ten archers from 6017, ten spearmen from 6027, ten swordsmen from 6037, and a
/// monster otherwise).
pub fn title(job: u16) -> (usize, &'static str) {
    match job {
        6017..=6026 => (1181, "Mercenary: Archer"),
        6037..=6046 => (1182, "Mercenary: Swordman"),
        6027..=6036 => (1183, "Mercenary: Spearman"),
        _ => (1367, "Mercenary Info - Monster Type"),
    }
}

/// Which mercenary a job is to its script (`AiMercenaryGetV`, `V_MERTYPE`): the hirelings count from 1 (the
/// client takes the job modulo 6017 and adds one, for the jobs its `IsHirelingJob` holds for: 6018 to 6052),
/// anything else is 1.
pub fn kind(job: u16) -> i32 {
    if (6018..=6052).contains(&job) { i32::from(job % 6017) + 1 } else { 1 }
}

/// The time its contract ends, written as the client writes it (its line 1260, `%m/%d %H:%M`, through
/// `strftime`): `month`, `day`, `hour` and `minute` are the local ones of that moment, which the page knows.
pub fn expiry(format: &str, (month, day, hour, minute): (u32, u32, u32, u32)) -> String {
    format.replace("%m", &format!("{month:02}")).replace("%d", &format!("{day:02}")).replace("%H", &format!("{hour:02}")).replace("%M", &format!("{minute:02}"))
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
        assert_eq!((read.hp, read.max_hp, read.sp, read.max_sp, read.expires, read.faith, read.calls, read.kills, read.range), (256, 256, 50, 50, 1_795_000_000, -3, 2, 41, 9));
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
        assert_eq!((title(6017).0, title(6026).0, title(6027).0, title(6037).0, title(6046).0, title(1002).0), (1181, 1181, 1183, 1182, 1182, 1367));
        assert_eq!((kind(6017), kind(6018), kind(6046), kind(1002)), (1, 2, 30, 1));
        assert!(is_skill(8201) && is_skill(8400) && !is_skill(8001) && !is_skill(8242));
        assert_eq!(expiry("%m/%d %H:%M", (3, 9, 7, 5)), "03/09 07:05");
    }
}
