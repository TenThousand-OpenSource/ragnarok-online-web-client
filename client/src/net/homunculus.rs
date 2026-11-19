//! The alchemist's homunculus: what the map server tells of it, and what its owner's client asks for it.
//!
//! Layouts from rAthena (`clif_hominfo`, `clif_send_homdata`, `clif_homunculus_updatestatus`, `clif_hom_food`,
//! `clif_parse_HomMenu`, `clif_parse_HomMoveTo`, `clif_parse_HomAttack`, `clif_parse_HomMoveToMaster`,
//! `clif_parse_ChangeHomunculusName`) at PACKETVER 20260219. The skills come laid out as the character's own
//! (`net::items::skills`, `skill_raised`).

use super::bytes::{name24, text, u16_at, u32_at, u64_at};

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
    /// This unit is the player's homunculus.
    pub const OWN: u8 = 0;
    pub const INTIMACY: u8 = 1;
    pub const HUNGER: u8 = 2;
}

/// The commands of CZ_COMMAND_MER (the client's `MM_COMMAND_HOMUN`).
pub mod order {
    pub const INFO: u8 = 0;
    pub const FEED: u8 = 1;
    pub const DELETE: u8 = 2;
}

/// What ZC_HO_PAR_CHANGE names (the server's `SP_*`).
pub mod parameter {
    pub const EXP: u16 = 1;
    pub const HP: u16 = 5;
    pub const SP: u16 = 7;
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
        _ => (long(after), long(after + 4), long(after + 8), long(after + 12), u64_at(packet, after + 16) as i64, u64_at(packet, after + 24) as i64, after + 32),
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
    let packet = packet.get(..12)?;
    Some((packet[3], u32_at(packet, 4), u32_at(packet, 8) as i32))
}

/// ZC_HO_PAR_CHANGE (0x07db, 8 bytes: `u16 parameter, i32 value`) and ZC_HO_PAR_CHANGE2 (0x0ba5, 12 bytes:
/// `u16 parameter, u64 value`).
pub fn parameter_changed(id: u16, packet: &[u8]) -> Option<(u16, i64)> {
    match id {
        ZC_HO_PAR_CHANGE => packet.get(..8).map(|packet| (u16_at(packet, 2), i64::from(u32_at(packet, 4) as i32))),
        ZC_HO_PAR_CHANGE2 => packet.get(..12).map(|packet| (u16_at(packet, 2), u64_at(packet, 4) as i64)),
        _ => None,
    }
}

/// ZC_FEED_MER (0x022f, 7 bytes): `u8 fed`, `u32 item`: whether it ate, and the food it eats.
pub fn fed(packet: &[u8]) -> Option<(bool, u32)> {
    let packet = packet.get(..7)?;
    Some((packet[2] != 0, u32_at(packet, 3)))
}

/// CZ_COMMAND_MER (0x022d, 5 bytes): `u16 type` (always 0), `u8 order`.
pub fn command(order: u8) -> Vec<u8> {
    vec![0x2d, 0x02, 0, 0, order]
}

/// CZ_RENAME_MER (0x0231, 26 bytes): `char name[24]`.
pub fn rename(name: &str) -> Vec<u8> {
    [&0x0231u16.to_le_bytes()[..], &name24(name)].concat()
}

/// CZ_REQUEST_MOVENPC (0x0232, 9 bytes): `u32 unit`, then the cell in three bytes, ten bits each way (the
/// client's `AiMove`). The homunculus, or the mercenary, is asked to walk there.
pub fn move_to(unit: u32, (x, y): (u16, u16)) -> Vec<u8> {
    let cell = [(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, (y << 4) as u8];
    [&0x0232u16.to_le_bytes()[..], &unit.to_le_bytes(), &cell].concat()
}

/// CZ_REQUEST_ACTNPC (0x0233, 11 bytes): `u32 unit, u32 target, u8 action` (always 0: one blow; the client's
/// `AiHomunAttack`).
pub fn attack(unit: u32, target: u32) -> Vec<u8> {
    [&0x0233u16.to_le_bytes()[..], &unit.to_le_bytes(), &target.to_le_bytes(), &[0]].concat()
}

/// CZ_REQUEST_MOVETOOWNER (0x0234, 6 bytes): `u32 unit`: it is asked to come to its owner (`AiMoveToOwner`).
pub fn to_owner(unit: u32) -> Vec<u8> {
    [&0x0234u16.to_le_bytes()[..], &unit.to_le_bytes()].concat()
}

/// How close a homunculus is to its owner, as the line of the client's message table that says it and the
/// English of it (`UIHomunInfoWnd::Paint`: seven stages, and "Unknown" outside them).
pub fn intimacy_word(intimacy: i32) -> (usize, &'static str) {
    match intimacy {
        0..=3 => (1022, "Hate with a Passion"),
        4..=10 => (1021, "Hate"),
        11..=100 => (672, "Awkward"),
        101..=250 => (673, "Shy"),
        251..=750 => (669, "Neutral"),
        751..=910 => (674, "Cordial"),
        911..=1000 => (675, "Loyal"),
        _ => (676, "Unknown"),
    }
}

/// Whether the chat is to say that the homunculus is hungry: at a hunger of 10, 5 and 1, once for each as it
/// is reached (the client's `ReceiveHomunculusInfo3` and its `m_homunHungerNotified`). Gives the hunger to
/// remember as told of.
pub fn hungry_now(hunger: i32, told: i32) -> Option<i32> {
    (matches!(hunger, 1 | 5 | 10) && told != hunger).then_some(hunger)
}

/// The number its window shows for the speed of its blows (`UIHomunInfoWnd::Paint`: `(2000 - delay) / 10`).
pub fn attack_speed(delay: i32) -> i32 {
    (2000 - delay) / 10
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ZC_PROPERTY_HOMUN of the newest layout, field by field.
    fn newest() -> Vec<u8> {
        let mut packet = vec![0xa4, 0x0b];
        packet.extend_from_slice(&name24("Lif"));
        packet.push(0b101);
        for word in [12u16, 55, 430] {
            packet.extend_from_slice(&word.to_le_bytes());
        }
        for stat in [31u16, 22, 40, 2, 15, 9, 33, 1400] {
            packet.extend_from_slice(&stat.to_le_bytes());
        }
        for long in [320u32, 410, 60, 75] {
            packet.extend_from_slice(&long.to_le_bytes());
        }
        packet.extend_from_slice(&5_000_000_000i64.to_le_bytes());
        packet.extend_from_slice(&9_000_000_000i64.to_le_bytes());
        packet.extend_from_slice(&3u16.to_le_bytes());
        packet.extend_from_slice(&1u16.to_le_bytes());
        packet
    }

    #[test]
    fn reads_everything_about_a_homunculus() {
        let packet = newest();
        assert_eq!(packet.len(), 85);
        let read = property(ZC_PROPERTY_HOMUN_4, &packet).unwrap();
        assert_eq!((read.name.as_str(), read.flags, read.level, read.hunger, read.intimacy), ("Lif", 5, 12, 55, 430));
        assert_eq!(read.stats, [31, 22, 40, 2, 15, 9, 33, 1400]);
        assert_eq!((read.hp, read.max_hp, read.sp, read.max_sp), (320, 410, 60, 75));
        assert_eq!((read.exp, read.next_exp, read.skill_points, read.range), (5_000_000_000, 9_000_000_000, 3, 1));
        assert_eq!(property(ZC_PROPERTY_HOMUN_4, &packet[..84]), None);
    }

    #[test]
    fn reads_the_older_layouts_too() {
        // 0x022e: an accessory after the intimacy, and everything after the stats in 16 and 32 bits.
        let mut packet = vec![0x2e, 0x02];
        packet.extend_from_slice(&name24("Amistr"));
        packet.push(1);
        for word in [3u16, 20, 900, 0, 1, 2, 3, 4, 5, 6, 7, 8, 100, 200, 30, 40] {
            packet.extend_from_slice(&word.to_le_bytes());
        }
        packet.extend_from_slice(&7u32.to_le_bytes());
        packet.extend_from_slice(&50u32.to_le_bytes());
        packet.extend_from_slice(&[2, 0, 1, 0]);
        assert_eq!(packet.len(), 71);
        let read = property(ZC_PROPERTY_HOMUN, &packet).unwrap();
        assert_eq!((read.level, read.intimacy, read.stats, read.hp, read.max_sp, read.exp, read.next_exp, read.skill_points, read.range), (3, 900, [1, 2, 3, 4, 5, 6, 7, 8], 100, 40, 7, 50, 2, 1));
        // 0x09f7: the same with the life in 32 bits.
        let mut packet = packet[..51].to_vec();
        packet[..2].copy_from_slice(&[0xf7, 0x09]);
        packet.extend_from_slice(&70_000u32.to_le_bytes());
        packet.extend_from_slice(&80_000u32.to_le_bytes());
        packet.extend_from_slice(&[30, 0, 40, 0, 7, 0, 0, 0, 50, 0, 0, 0, 2, 0, 1, 0]);
        assert_eq!(packet.len(), 75);
        let read = property(ZC_PROPERTY_HOMUN_2, &packet).unwrap();
        assert_eq!((read.hp, read.max_hp, read.sp, read.max_sp, read.next_exp, read.range), (70_000, 80_000, 30, 40, 50, 1));
    }

    #[test]
    fn reads_what_changed() {
        assert_eq!(changed(&[0x30, 0x02, 0, 1, 0x10, 0x27, 0, 0, 0xb0, 0x01, 0, 0]), Some((change::INTIMACY, 10000, 432)));
        assert_eq!(changed(&[0x30, 0x02, 0, 0]), None);
        assert_eq!(parameter_changed(ZC_HO_PAR_CHANGE2, &[0xa5, 0x0b, 5, 0, 0x2c, 1, 0, 0, 0, 0, 0, 0]), Some((parameter::HP, 300)));
        assert_eq!(parameter_changed(ZC_HO_PAR_CHANGE, &[0xdb, 0x07, 7, 0, 40, 0, 0, 0]), Some((parameter::SP, 40)));
        assert_eq!(fed(&[0x2f, 0x02, 1, 0x19, 0x02, 0, 0]), Some((true, 537)));
    }

    #[test]
    fn asks_for_it_as_the_client_does() {
        assert_eq!(command(order::FEED), [0x2d, 0x02, 0, 0, 1]);
        assert_eq!(rename("Mimi").len(), 26);
        // The cell packed as rAthena's `RBUFPOS` unpacks it: x in the first ten bits, y in the next ten.
        let packet = move_to(0x01020304, (155, 187));
        assert_eq!(&packet[..6], &[0x32, 0x02, 4, 3, 2, 1]);
        let (a, b, c) = (u16::from(packet[6]), u16::from(packet[7]), u16::from(packet[8]));
        assert_eq!(((a << 2) | (b >> 6), ((b & 0x3f) << 4) | (c >> 4)), (155, 187));
        assert_eq!(attack(7, 9), [0x33, 0x02, 7, 0, 0, 0, 9, 0, 0, 0, 0]);
        assert_eq!(to_owner(7), [0x34, 0x02, 7, 0, 0, 0]);
    }

    #[test]
    fn words_it_as_its_window_does() {
        assert_eq!((intimacy_word(0).0, intimacy_word(10).0, intimacy_word(100).0, intimacy_word(250).0, intimacy_word(750).0, intimacy_word(910).0, intimacy_word(1000).0, intimacy_word(1001).0), (1022, 1021, 672, 673, 669, 674, 675, 676));
        assert_eq!((hungry_now(10, 0), hungry_now(10, 10), hungry_now(5, 10), hungry_now(11, 10)), (Some(10), None, Some(5), None));
        assert_eq!(attack_speed(1400), 60);
        assert!(is_skill(8001) && is_skill(8060) && !is_skill(8061) && !is_skill(7999));
    }
}
