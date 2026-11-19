//! Skills in use: what the map server tells of a skill being cast, of its landing and of one that could not be
//! used, of an effect it has shown on someone, of a skill an item casts, of life recovered, a cast broken off
//! and a blow out of reach; and how the client asks for one aimed at a place. What a failure says and which
//! effect a notice shows are rules and no packets: `crate::skills`, still named from here.

use super::bytes::{i32_at, id, int, long, short, u16_at, word, Writer};


/// ZC_USESKILL_ACK (0x0b1a): `u32 who, u32 on whom, u16 x, u16 y, u16 skill, u32 element, u32 milliseconds,
/// u8 disposable, u32 attack motion`. Someone starts to cast.
pub const ZC_USESKILL_ACK: u16 = 0x0b1a;
/// ZC_NOTIFY_SKILL (0x01de): `u16 skill, u32 who, u32 on whom, u32 tick, i32 attack motion, i32 attacked motion,
/// i32 damage, i16 level, i16 hits, i8 action`. A skill lands.
pub const ZC_NOTIFY_SKILL: u16 = 0x01de;
/// ZC_USE_SKILL (0x09cb): `u16 skill, i32 level, u32 on whom, u32 who, i8 result`. A skill that does no harm
/// is used.
pub const ZC_USE_SKILL: u16 = 0x09cb;
/// ZC_NOTIFY_GROUNDSKILL (0x0117): `u16 skill, u32 who, i16 level, i16 x, i16 y, u32 tick`.
pub const ZC_NOTIFY_GROUNDSKILL: u16 = 0x0117;
/// ZC_ACK_TOUSESKILL (0x0110): `u16 skill, i32 kind, u32 item, u8 flag, u8 cause`. A skill could not be used.
pub const ZC_ACK_TOUSESKILL: u16 = 0x0110;
/// ZC_NOTIFY_EFFECT2 (0x01f3: `u32 who, u32 effect`) and ZC_NOTIFY_EFFECT3 (0x0284: the same and a
/// `u32 value`): the server has one of the client's effects shown on someone.
pub const ZC_NOTIFY_EFFECT2: u16 = 0x01f3;
pub const ZC_NOTIFY_EFFECT3: u16 = 0x0284;
/// ZC_AUTORUN_SKILL (0x0147): an item used has the character cast a skill.
pub const ZC_AUTORUN_SKILL: u16 = 0x0147;
/// ZC_RECOVERY (0x013d) and ZC_RECOVERY2 (0x0a27): the character recovers life or spirit.
pub const ZC_RECOVERY: u16 = 0x013d;
pub const ZC_RECOVERY2: u16 = 0x0a27;
/// ZC_DISPEL (0x01b9): a cast is broken off.
pub const ZC_DISPEL: u16 = 0x01b9;
/// ZC_ATTACK_RANGE (0x013a) and ZC_ATTACK_FAILURE_FOR_DISTANCE (0x0139).
pub const ZC_ATTACK_RANGE: u16 = 0x013a;
pub const ZC_ATTACK_FAILURE_FOR_DISTANCE: u16 = 0x0139;

/// Someone starts to cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Casting {
    pub source: u32,
    /// 0 for a skill aimed at a place.
    pub target: u32,
    pub cell: (u16, u16),
    pub skill: u16,
    /// The element the cast shows as.
    pub element: u32,
    pub milliseconds: u32,
}

pub fn casting(packet: &[u8]) -> Option<Casting> {
    Some(Casting {
        source: long(packet, 2)?,
        target: long(packet, 6)?,
        cell: (word(packet, 10)?, word(packet, 12)?),
        skill: word(packet, 14)?,
        element: long(packet, 16)?,
        milliseconds: long(packet, 20)?
    })
}

/// A skill lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Landing {
    pub skill: u16,
    pub source: u32,
    pub target: u32,
    /// How long the caster's motion and the struck one's flinch take, in milliseconds.
    pub attack_motion: i32,
    pub attacked_motion: i32,
    pub damage: i32,
    pub level: i16,
    pub hits: i16,
    pub action: u8,
}

pub fn landing(packet: &[u8]) -> Option<Landing> {
    Some(Landing {
        skill: word(packet, 2)?,
        source: long(packet, 4)?,
        target: long(packet, 8)?,
        attack_motion: int(packet, 16)?,
        attacked_motion: int(packet, 20)?,
        damage: int(packet, 24)?,
        level: short(packet, 28)?,
        hits: short(packet, 30)?,
        action: *packet.get(32)?,
    })
}

/// A harmless skill used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Used {
    pub skill: u16,
    pub level: i32,
    /// On whom.
    pub target: u32,
    pub who: u32,
    /// Whether it worked.
    pub worked: bool,
}

/// A harmless skill used: `u16 skill, i32 level, u32 target, u32 who, u8 worked`.
pub fn used(packet: &[u8]) -> Option<Used> {
    Some(Used {
        skill: word(packet, 2)?,
        level: int(packet, 4)?,
        target: long(packet, 8)?,
        who: long(packet, 12)?,
        worked: *packet.get(16)? != 0,
    })
}

/// A skill used on a place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OnGround {
    pub skill: u16,
    pub who: u32,
    pub level: i16,
    pub cell: (u16, u16),
}

/// A skill used on a place: `u16 skill, u32 who, i16 level, u16 x, u16 y`.
pub fn on_ground(packet: &[u8]) -> Option<OnGround> {
    Some(OnGround {
        skill: word(packet, 2)?,
        who: long(packet, 4)?,
        level: short(packet, 8)?,
        cell: (word(packet, 10)?, word(packet, 12)?),
    })
}

/// CZ_USE_SKILL_TOGROUND (0x0366): `u16 level, u16 skill, u16 x, u16 y`.
pub fn use_on_ground(level: u16, skill: u16, cell: (u16, u16)) -> Vec<u8> {
    Writer::new(0x0366).u16(level).u16(skill).u16(cell.0).u16(cell.1).done()
}

/// What ZC_ACK_TOUSESKILL tells of a skill that was not used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Failed {
    pub skill: u16,
    /// A value that goes with the cause.
    pub value: i32,
    /// It did work after all: then nothing is said.
    pub worked: bool,
    pub cause: u8,
}

/// ZC_ACK_TOUSESKILL (0x0110, 14 bytes): the skill, a value that goes with the cause, an item, whether it did
/// work after all (then nothing is said), and the cause.
pub fn failed(packet: &[u8]) -> Option<Failed> {
    let packet = packet.get(..14)?;
    Some(Failed { skill: u16_at(packet, 2), value: i32_at(packet, 4), worked: packet[12] != 0, cause: packet[13] })
}

/// An effect shown on someone (either form): `(who, effect)`. The value of the second form is not read.
pub fn effect_shown(packet: &[u8]) -> Option<(u32, u32)> {
    Some((long(packet, 2)?, long(packet, 6)?))
}

/// The number ZC_NOTIFY_EFFECT3 sends with its effect (none in ZC_NOTIFY_EFFECT2, which is shorter).
pub fn effect_number(packet: &[u8]) -> Option<i32> {
    int(packet, 10)
}

/// The skill an item used has the character cast (`u16 skill, u32 kind, u16 level, …`): `(skill, level)`.
pub fn item_skill(packet: &[u8]) -> Option<(u16, u16)> {
    Some((word(packet, 2)?, word(packet, 8)?))
}

/// What the character recovers (`u16 which, amount`: the amount an `i16` in the older form, an `i32` in the
/// newer): `(which of its values, how much)`.
pub fn recovered(packet: &[u8]) -> Option<(u16, i32)> {
    let amount = if id(packet)? == ZC_RECOVERY { i32::from(short(packet, 4)?) } else { int(packet, 4)? };
    Some((word(packet, 2)?, amount))
}

/// Whose cast is broken off (`u32 who`).
pub fn dispelled(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// How far the character's weapon reaches (`u16 range`).
pub fn attack_range(packet: &[u8]) -> Option<u16> {
    word(packet, 2)
}

/// A blow asked for from too far (`u32 target, i16 x, i16 y, i16 own x, i16 own y, i16 range`):
/// `(target, how far the weapon reaches)`.
pub fn out_of_reach(packet: &[u8]) -> Option<(u32, i16)> {
    Some((long(packet, 2)?, short(packet, 14)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::failure;

    #[test]
    fn reads_what_else_the_server_says_of_blows_and_effects() {
        let mut shown = vec![0u8; 14];
        shown[2..6].copy_from_slice(&2_000_001u32.to_le_bytes());
        shown[6..10].copy_from_slice(&315u32.to_le_bytes());
        assert_eq!((effect_shown(&shown), effect_shown(&shown[..10]), effect_shown(&shown[..9])), (Some((2_000_001, 315)), Some((2_000_001, 315)), None));
        // An item's skill: Teleport at its first level.
        let cast = [0x47, 0x01, 26, 0, 4, 0, 0, 0, 1, 0];
        assert_eq!(item_skill(&cast), Some((26, 1)));
        // Life recovered, in the older form (a short) and the newer (a long).
        assert_eq!(recovered(&[0x3d, 0x01, 5, 0, 0x2c, 0x01]), Some((5, 300)));
        assert_eq!(recovered(&[0x27, 0x0a, 7, 0, 0xa0, 0x86, 0x01, 0]), Some((7, 100_000)));
        assert_eq!(recovered(&[0x27, 0x0a, 7, 0, 0xa0]), None);
        assert_eq!(dispelled(&[0xb9, 0x01, 0x40, 0x42, 0x0f, 0]), Some(1_000_000));
        assert_eq!(attack_range(&[0x3a, 0x01, 9, 0]), Some(9));
        let mut far = vec![0u8; 16];
        far[2..6].copy_from_slice(&110_005u32.to_le_bytes());
        far[14..16].copy_from_slice(&3i16.to_le_bytes());
        assert_eq!((out_of_reach(&far), out_of_reach(&far[..15])), (Some((110_005, 3)), None));
    }

    #[test]
    fn reads_a_cast_and_its_landing() {
        let mut cast = vec![0u8; 29];
        cast[..2].copy_from_slice(&ZC_USESKILL_ACK.to_le_bytes());
        cast[2..6].copy_from_slice(&2_000_000u32.to_le_bytes());
        cast[6..10].copy_from_slice(&110_000u32.to_le_bytes());
        cast[14..16].copy_from_slice(&19u16.to_le_bytes());
        cast[16..20].copy_from_slice(&3u32.to_le_bytes());
        cast[20..24].copy_from_slice(&2800u32.to_le_bytes());
        assert_eq!(casting(&cast), Some(Casting { source: 2_000_000, target: 110_000, cell: (0, 0), skill: 19, element: 3, milliseconds: 2800 }));

        let mut hit = vec![0u8; 33];
        hit[2..4].copy_from_slice(&19u16.to_le_bytes());
        hit[4..8].copy_from_slice(&2_000_000u32.to_le_bytes());
        hit[8..12].copy_from_slice(&110_000u32.to_le_bytes());
        hit[16..20].copy_from_slice(&500i32.to_le_bytes());
        hit[20..24].copy_from_slice(&300i32.to_le_bytes());
        hit[24..28].copy_from_slice(&450i32.to_le_bytes());
        hit[28..30].copy_from_slice(&5i16.to_le_bytes());
        hit[30..32].copy_from_slice(&5i16.to_le_bytes());
        hit[32] = 8;
        let landed = landing(&hit).unwrap();
        assert_eq!(
            (landed.skill, landed.source, landed.target, landed.damage, landed.level, landed.hits, landed.action),
            (19, 2_000_000, 110_000, 450, 5, 5, 8)
        );
        assert_eq!((landed.attack_motion, landed.attacked_motion), (500, 300));
        assert_eq!(use_on_ground(3, 47, (150, 160)).len(), 10);
    }

    #[test]
    fn a_skill_used_is_read() {
        let mut packet = vec![0u8, 0, 28, 0];
        packet.extend(300i32.to_le_bytes());
        packet.extend(9u32.to_le_bytes());
        packet.extend(7u32.to_le_bytes());
        packet.push(1);
        assert_eq!(used(&packet), Some(Used { skill: 28, level: 300, target: 9, who: 7, worked: true }));
        assert_eq!(used(&packet[..16]), None);
        let mut packet = vec![0u8, 0, 12, 0, 7, 0, 0, 0, 3, 0];
        packet.extend([150, 0, 160, 0]);
        assert_eq!(on_ground(&packet), Some(OnGround { skill: 12, who: 7, level: 3, cell: (150, 160) }));
        assert_eq!(on_ground(&packet[..13]), None);
    }

    #[test]
    fn says_why_a_skill_failed() {
        let packet = [0x10, 0x01, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        assert_eq!(failed(&packet), Some(Failed { skill: 46, value: 0, worked: false, cause: 1 }));
        // Not enough SP; an emotion without the Basic Skill for it; a skill still cooling down.
        assert_eq!((failure(46, 0, 1), failure(1, 1, 0), failure(46, 0, 4), failure(46, 0, 200)), (Some(202), Some(160), Some(1460), None));
    }
}
