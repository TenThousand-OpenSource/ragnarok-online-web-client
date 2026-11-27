//! Section 6 of a replay file: the quests under way (`BuildRenewQuestReplayRecords`, `Replay.cpp`, read by
//! `CSession::ReadReplayQuests`, `Contents/ClientState.cpp`). Between 3500 and 3501: 3502, which kind of
//! record a hunt has (51 as written), and 3510, the quests in one piece (`CRenewQuestMgr::WriteOngoingQuests`,
//! `ReadOngoingQuests`, `Contents/Quest/RenewQuestMgr.cpp`): how many, then each quest's record of 0x18 bytes
//! and after it its hunts, 0x47 bytes each. The titles are not kept: the client finds them by the number.

use super::*;

/// The kind of a hunt's record the client writes (3502), and the sizes: a quest's record
/// (`RenewQuestOngoingRecord`), a hunt's of that kind (`RenewQuestHuntRecord`) with its name of 0x33 bytes,
/// and a hunt's of any other kind (`RenewQuestShortHuntRecord`) with a name of 0x18.
pub const KIND: i32 = 51;
pub const QUEST_BYTES: usize = 0x18;
pub const HUNT_BYTES: usize = 0x47;
pub const SHORT_HUNT_BYTES: usize = 0x2c;
/// Where a hunt's name begins in its record.
const NAME_AT: usize = 0x14;

/// One kind of monster a quest asks to hunt.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hunt {
    /// The hunt's own number (`m_huntId`, its low four bytes).
    pub id: i32,
    /// Which monsters count (`m_huntConditionFlags`).
    pub condition: i32,
    /// The monster (`m_huntMobId`).
    pub monster: i32,
    /// The levels of the monsters that count, least and most.
    pub min_level: i16,
    pub max_level: i16,
    /// How many are killed, and how many are asked for.
    pub killed: i16,
    pub goal: i16,
    /// The monster's name, 50 bytes of it at most.
    pub name: Vec<u8>,
}

/// A quest under way.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quest {
    pub id: u32,
    /// Its state (`m_state`).
    pub state: u8,
    /// Until when: the year, the month, the day, the hour, the minute.
    pub when: [i16; 5],
    /// Its hunts, three at most.
    pub hunts: Vec<Hunt>,
}

/// Section 6. The quests are written by their numbers upwards, as the client's map of them is walked, and of
/// a quest's hunts the three the client has room for.
pub fn written(quests: &[Quest]) -> Vec<u8> {
    let mut sorted: Vec<&Quest> = quests.iter().collect();
    sorted.sort_by_key(|quest| quest.id);
    let mut blob = (sorted.len() as u32).to_le_bytes().to_vec();
    for quest in sorted {
        let hunts = &quest.hunts[..quest.hunts.len().min(3)];
        blob.extend(quest.id.to_le_bytes());
        blob.extend([quest.state, 0]);
        blob.extend((hunts.len() as i16).to_le_bytes());
        blob.extend(quest.when.iter().flat_map(|part| part.to_le_bytes()));
        blob.extend([0; 6]);
        for hunt in hunts {
            blob.extend([hunt.id, hunt.condition, hunt.monster].iter().flat_map(|part| part.to_le_bytes()));
            blob.extend([hunt.min_level, hunt.max_level, hunt.killed, hunt.goal].iter().flat_map(|part| part.to_le_bytes()));
            // (`strncpy_s(hunt.name, 0x33, …, _TRUNCATE)`: fifty bytes and a zero.)
            blob.extend(padded(&hunt.name[..hunt.name.len().min(HUNT_BYTES - NAME_AT - 1)], HUNT_BYTES - NAME_AT));
        }
    }
    let mut section = Vec::new();
    tagged(&mut section, 3500, &[]);
    tagged(&mut section, 3502, &KIND.to_le_bytes());
    tagged(&mut section, 3510, &blob);
    tagged(&mut section, 3501, &[]);
    section
}

/// Section 6 read back. A hunt's record is the long one when 3502 said 51 before the quests came, the short
/// one otherwise (the reader's kind before 3502 is 0x18). Quests the piece is too short for are left out.
pub fn read(section: &[u8]) -> Option<Vec<Quest>> {
    let (mut kind, mut quests) = (0x18, Vec::new());
    for (tag, bytes) in opened(section, 3500)? {
        match tag {
            3502 => kind = i32_of(bytes),
            3510 => quests = blob(bytes, if kind == KIND { HUNT_BYTES } else { SHORT_HUNT_BYTES }),
            _ => {}
        }
    }
    Some(quests)
}

/// The quests of 3510, a hunt's record of so many bytes.
fn blob(bytes: &[u8], hunt_bytes: usize) -> Vec<Quest> {
    let mut quests = Vec::new();
    let mut at = 4;
    for _ in 0..u32_of(bytes) {
        let Some(record) = bytes.get(at..at + QUEST_BYTES) else { break };
        at += QUEST_BYTES;
        let mut quest = Quest { id: u32_of(record), state: record[4], ..Quest::default() };
        for (place, part) in quest.when.iter_mut().enumerate() {
            *part = i16_of(&record[8 + place * 2..]);
        }
        for _ in 0..i16_of(&record[6..]).max(0) {
            let Some(hunt) = bytes.get(at..at + hunt_bytes) else { break };
            at += hunt_bytes;
            quest.hunts.push(Hunt {
                id: i32_of(hunt),
                condition: i32_of(&hunt[4..]),
                monster: i32_of(&hunt[8..]),
                min_level: i16_of(&hunt[12..]),
                max_level: i16_of(&hunt[14..]),
                killed: i16_of(&hunt[16..]),
                goal: i16_of(&hunt[18..]),
                name: unpadded(&hunt[NAME_AT..]),
            });
        }
        quests.push(quest);
    }
    quests
}
