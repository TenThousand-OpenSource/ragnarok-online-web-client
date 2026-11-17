//! Skills in use: what the map server tells of a skill being cast and of its landing, and how the client asks
//! for one aimed at a place.

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

fn short(packet: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(packet.get(at..at + 2)?.try_into().ok()?))
}

fn long(packet: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(packet.get(at..at + 4)?.try_into().ok()?))
}

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
    Some(Casting { source: long(packet, 2)?, target: long(packet, 6)?, cell: (short(packet, 10)?, short(packet, 12)?), skill: short(packet, 14)?, element: long(packet, 16)?, milliseconds: long(packet, 20)? })
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
        skill: short(packet, 2)?,
        source: long(packet, 4)?,
        target: long(packet, 8)?,
        attack_motion: long(packet, 16)? as i32,
        attacked_motion: long(packet, 20)? as i32,
        damage: long(packet, 24)? as i32,
        level: short(packet, 28)? as i16,
        hits: short(packet, 30)? as i16,
        action: *packet.get(32)?,
    })
}

/// A harmless skill used: `(skill, level, on whom, who, whether it worked)`.
pub fn used(packet: &[u8]) -> Option<(u16, i32, u32, u32, bool)> {
    Some((short(packet, 2)?, long(packet, 4)? as i32, long(packet, 8)?, long(packet, 12)?, *packet.get(16)? != 0))
}

/// A skill used on a place: `(skill, who, level, cell)`.
pub fn on_ground(packet: &[u8]) -> Option<(u16, u32, i16, (u16, u16))> {
    Some((short(packet, 2)?, long(packet, 4)?, short(packet, 8)? as i16, (short(packet, 10)?, short(packet, 12)?)))
}

/// CZ_USE_SKILL_TOGROUND (0x0366): `u16 level, u16 skill, u16 x, u16 y`.
pub fn use_on_ground(level: u16, skill: u16, cell: (u16, u16)) -> Vec<u8> {
    [0x0366u16.to_le_bytes(), level.to_le_bytes(), skill.to_le_bytes(), cell.0.to_le_bytes(), cell.1.to_le_bytes()].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!((landed.skill, landed.source, landed.target, landed.damage, landed.level, landed.hits, landed.action), (19, 2_000_000, 110_000, 450, 5, 5, 8));
        assert_eq!((landed.attack_motion, landed.attacked_motion), (500, 300));
        assert_eq!(use_on_ground(3, 47, (150, 160)).len(), 10);
    }
}
