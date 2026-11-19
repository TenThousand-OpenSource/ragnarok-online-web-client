//! The skills a character knows: the list, one added, one raised; raising and using one.

use super::*;
use crate::net::bytes::{long, records, word, Writer};

/// ZC_SKILLINFO_UPDATE2 (0x07e1): a skill of the character's changed (`skill_changed`).
pub const ZC_SKILLINFO_UPDATE2: u16 = 0x07e1;
/// ZC_SKILLINFO_DELETE (0x0441): `u16 skill`: a skill is taken from the character (one copied by Plagiarism).
pub const ZC_SKILLINFO_DELETE: u16 = 0x0441;

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
    records(packet, 4, 37).filter_map(|skill| skill_told(skill, 36)).collect()
}

/// A skill as every packet that tells one whole lays it out, `u16 id, i32 target, u16 level, u16 sp, u16 range`,
/// with the byte that says whether it can be raised at `raisable` of the record (after a name in some, at once
/// in others); none where the record ends before that.
pub(in crate::net) fn skill_told(record: &[u8], raisable: usize) -> Option<Skill> {
    Some(Skill {
        id: word(record, 0)?,
        target: long(record, 2)?,
        level: word(record, 6)?,
        sp: word(record, 8)?,
        range: word(record, 10)?,
        raisable: *record.get(raisable)? != 0,
    })
}

/// ZC_ADD_SKILL (0x0111): one skill, laid out as in the list.
pub fn skill_added(packet: &[u8]) -> Option<Skill> {
    packet.get(2..39).and_then(|skill| skill_told(skill, 36))
}

/// A skill that went up a level, as ZC_SKILLINFO_UPDATE tells it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkillRaised {
    pub id: u16,
    pub level: u16,
    pub sp: u16,
    pub range: u16,
    pub raisable: bool,
}

/// ZC_SKILLINFO_UPDATE (0x010e): `u16 id, u16 level, u16 sp, u16 range, u8 raisable`: a skill went up a level.
pub fn skill_raised(packet: &[u8]) -> Option<SkillRaised> {
    (packet.len() >= 11).then(|| SkillRaised {
        id: u16_at(packet, 2),
        level: u16_at(packet, 4),
        sp: u16_at(packet, 6),
        range: u16_at(packet, 8),
        raisable: packet[10] != 0,
    })
}

/// ZC_SKILLINFO_UPDATE2: `u16 id, i32 target, u16 level, u16 sp, u16 range, u8 raisable`: a skill as it is now
/// (the fields of `Skill`).
pub fn skill_changed(packet: &[u8]) -> Option<Skill> {
    skill_told(packet.get(2..)?, 12)
}

/// The skill ZC_SKILLINFO_DELETE takes away.
pub fn skill_taken(packet: &[u8]) -> Option<u16> {
    word(packet, 2)
}

/// CZ_UPGRADE_SKILLLEVEL (0x0112): `u16 id`: spends a skill point.
pub fn raise_skill(id: u16) -> Vec<u8> {
    Writer::new(0x0112).u16(id).done()
}

/// CZ_USE_SKILL (0x0438): `u16 level, u16 id, u32 target`.
pub fn use_skill(level: u16, id: u16, target: u32) -> Vec<u8> {
    Writer::new(0x0438).u16(level).u16(id).u32(target).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::packet;

    #[test]
    fn a_skill_changed_and_one_taken_away() {
        let told = packet(
            ZC_SKILLINFO_UPDATE2,
            &[&14u16.to_le_bytes(), &1u32.to_le_bytes(), &5u16.to_le_bytes(), &20u16.to_le_bytes(), &9u16.to_le_bytes(), &[0]]
        );
        assert_eq!(told.len(), 15);
        assert_eq!(skill_changed(&told), Some(crate::net::inventory::Skill { id: 14, target: 1, level: 5, sp: 20, range: 9, raisable: false }));
        assert_eq!(skill_taken(&packet(ZC_SKILLINFO_DELETE, &[&14u16.to_le_bytes()])), Some(14));
    }
}
