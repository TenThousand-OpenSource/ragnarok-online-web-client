//! The skills a character knows: the list, one added, one raised; raising and using one.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub id: u16,
    /// How it is used: 0 is passive, 1 on an enemy, 2 on a place, 4 on oneself, 16 on a friend, 32 a trap.
    pub target: u32,
    pub level: u16,
    pub sp: u16,
    pub range: u16,
    /// Whether a skill point can raise it now.
    pub raisable: bool,
}

impl Skill {
    pub fn passive(&self) -> bool {
        self.target == 0
    }

    /// Used on the character itself, with no target to choose.
    pub fn on_self(&self) -> bool {
        self.target == 4
    }
}

/// ZC_SKILLINFO_LIST (0x010f): `u16 length`, then 37 bytes a skill: `u16 id, i32 target, u16 level, u16 sp,
/// u16 range, char[24] name, u8 raisable`.
pub fn skills(packet: &[u8]) -> Vec<Skill> {
    packet.get(4..).unwrap_or_default().chunks_exact(37).map(|skill| skill_at(skill, true)).collect()
}

fn skill_at(skill: &[u8], full: bool) -> Skill {
    Skill { id: u16_at(skill, 0), target: u32_at(skill, 2), level: u16_at(skill, 6), sp: u16_at(skill, 8), range: u16_at(skill, 10), raisable: full && skill[36] != 0 }
}

/// ZC_ADD_SKILL (0x0111): one skill, laid out as in the list.
pub fn skill_added(packet: &[u8]) -> Option<Skill> {
    packet.get(2..39).map(|skill| skill_at(skill, true))
}

/// ZC_SKILLINFO_UPDATE (0x010e): `u16 id, u16 level, u16 sp, u16 range, u8 raisable`: a skill went up a level.
pub fn skill_raised(packet: &[u8]) -> Option<(u16, u16, u16, u16, bool)> {
    (packet.len() >= 11).then(|| (u16_at(packet, 2), u16_at(packet, 4), u16_at(packet, 6), u16_at(packet, 8), packet[10] != 0))
}

/// CZ_UPGRADE_SKILLLEVEL (0x0112): `u16 id`: spends a skill point.
pub fn raise_skill(id: u16) -> Vec<u8> {
    [0x0112u16.to_le_bytes(), id.to_le_bytes()].concat()
}

/// CZ_USE_SKILL (0x0438): `u16 level, u16 id, u32 target`.
pub fn use_skill(level: u16, id: u16, target: u32) -> Vec<u8> {
    [&0x0438u16.to_le_bytes()[..], &level.to_le_bytes(), &id.to_le_bytes(), &target.to_le_bytes()].concat()
}
