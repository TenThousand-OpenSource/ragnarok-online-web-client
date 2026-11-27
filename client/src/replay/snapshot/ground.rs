//! The sections of a replay file about what is on the ground. Section 16: the things that lie there
//! (`CGameMode::SaveGroundItems`, read by `ReadReplayGroundItems`, `GameMode.cpp`): between 15000 and 15001 a
//! chunk 0xCF a thing, the packet ZC_ITEM_ENTRY (0x009D, 0x13 bytes) whole. Section 17: the skills' units
//! (`CGameMode::WriteReplaySkillState`, `ReadReplayGroundSkills`): between 17000 and 17001 a chunk 0xCD a
//! unit, the packet ZC_SKILL_ENTRY (0x011F, 0x10 bytes) whole; the reader also takes 0xCE, a unit that
//! carries words (ZC_SKILL_ENTRY2, 0x01C9, 0x61 bytes), which this version of the client never writes.

use super::*;

/// The numbers of the two packets (`ID_ZC_ITEM_ENTRY`, `ID_ZC_SKILL_ENTRY`, `Network/Packets/PacketId.h`) and
/// how long each is.
pub const ZC_ITEM_ENTRY: u16 = 0x009d;
pub const ZC_SKILL_ENTRY: u16 = 0x011f;
pub const ITEM_BYTES: usize = 0x13;
pub const SKILL_BYTES: usize = 0x10;

/// A thing that lies on the ground (`ItemEntryPacket`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroundItem {
    /// Its own number on the ground.
    pub id: u32,
    pub item: u32,
    pub identified: bool,
    pub x: i16,
    pub y: i16,
    /// How many.
    pub count: i16,
    /// Where within the cell (`subX`, `subY`).
    pub within: (u8, u8),
}

/// The packet of one, as its chunk holds it: the count before the place within the cell.
pub fn item_entry(item: &GroundItem) -> Vec<u8> {
    let cell = [item.x.to_le_bytes(), item.y.to_le_bytes(), item.count.to_le_bytes()].concat();
    let ids = [item.id.to_le_bytes(), item.item.to_le_bytes()].concat();
    [&ZC_ITEM_ENTRY.to_le_bytes()[..], &ids, &[item.identified.into()], &cell, &[item.within.0, item.within.1]].concat()
}

/// Section 16.
pub fn items(items: &[GroundItem]) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 15000, &[]);
    items.iter().for_each(|item| tagged(&mut section, 0xcf, &item_entry(item)));
    tagged(&mut section, 15001, &[]);
    section
}

/// Section 16 read back; a chunk too short for the packet is passed by.
pub fn items_read(section: &[u8]) -> Option<Vec<GroundItem>> {
    let chunks = opened(section, 15000)?.into_iter().filter(|(kind, bytes)| *kind == 0xcf && bytes.len() >= ITEM_BYTES);
    let item = |bytes: &[u8]| GroundItem {
        id: u32_of(&bytes[2..]),
        item: u32_of(&bytes[6..]),
        identified: bytes[10] != 0,
        x: i16_of(&bytes[11..]),
        y: i16_of(&bytes[13..]),
        count: i16_of(&bytes[15..]),
        within: (bytes[17], bytes[18]),
    };
    Some(chunks.map(|(_, bytes)| item(bytes)).collect())
}

/// A skill's unit on the ground (`SkillEntryPacket`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroundSkill {
    pub id: u32,
    /// Who made it.
    pub creator: u32,
    pub x: i16,
    pub y: i16,
    /// What kind of unit it is (the packet's `job`).
    pub kind: u8,
    /// The client writes 1 for every unit it kept.
    pub visible: bool,
}

/// The units of section 17: those of the packet of 16 bytes, and, as their bytes, those with words (0xCE)
/// of a file that has them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroundSkills {
    pub units: Vec<GroundSkill>,
    pub worded: Vec<Vec<u8>>,
}

/// The packet of one, as its chunk holds it.
pub fn skill_entry(unit: &GroundSkill) -> Vec<u8> {
    let ids = [unit.id.to_le_bytes(), unit.creator.to_le_bytes()].concat();
    let cell = [unit.x.to_le_bytes(), unit.y.to_le_bytes()].concat();
    [&ZC_SKILL_ENTRY.to_le_bytes()[..], &ids, &cell, &[unit.kind, unit.visible.into()]].concat()
}

/// Section 17: the units, then those with words when there are any.
pub fn skills(skills: &GroundSkills) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 17000, &[]);
    skills.units.iter().for_each(|unit| tagged(&mut section, 0xcd, &skill_entry(unit)));
    skills.worded.iter().for_each(|unit| tagged(&mut section, 0xce, unit));
    tagged(&mut section, 17001, &[]);
    section
}

/// Section 17 read back.
pub fn skills_read(section: &[u8]) -> Option<GroundSkills> {
    let mut skills = GroundSkills::default();
    for (kind, bytes) in opened(section, 17000)? {
        match kind {
            0xcd if bytes.len() >= SKILL_BYTES => skills.units.push(GroundSkill {
                id: u32_of(&bytes[2..]),
                creator: u32_of(&bytes[6..]),
                x: i16_of(&bytes[10..]),
                y: i16_of(&bytes[12..]),
                kind: bytes[14],
                visible: bytes[15] != 0,
            }),
            0xce => skills.worded.push(bytes.to_vec()),
            _ => {}
        }
    }
    Some(skills)
}
