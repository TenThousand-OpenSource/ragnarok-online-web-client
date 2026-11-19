//! The quests: the journal the map server sends on entering, what it says as a quest is taken, advanced and
//! dropped, and the mark it puts over an NPC.
//!
//! The layouts are rAthena's `clif_quest_*` (`src/map/clif.cpp`) for PACKETVER 20260219, where the list, the new
//! quest and the count of monsters are the versions with a hunt's own number and its levels (`0x09f8`, `0x09f9`,
//! `0x09fa`); the client reads them in `CGameMode::ReceiveOngoingQuestList`, `ReceiveAddQuest` and
//! `ReceiveQuestHuntUpdate`.

use super::bytes::{long, records, short, text, word, Writer};

#[cfg(test)]
mod tests;

/// ZC_ALL_QUEST_LIST3: every quest of the character, on entering.
pub const ZC_ALL_QUEST_LIST: u16 = 0x09f8;
/// ZC_ADD_QUEST_EX: a quest was taken.
pub const ZC_ADD_QUEST: u16 = 0x09f9;
/// ZC_HUNTING_QUEST_INFO: what a new quest's monsters are and how many of each are wanted.
pub const ZC_HUNTING_QUEST_INFO: u16 = 0x08fe;
/// ZC_UPDATE_MISSION_HUNT_EX: how many of a quest's monsters are hunted now.
pub const ZC_UPDATE_MISSION_HUNT: u16 = 0x09fa;
/// ZC_DEL_QUEST: a quest is gone from the journal (finished or dropped).
pub const ZC_DEL_QUEST: u16 = 0x02b4;
/// ZC_ACTIVE_QUEST: a quest was switched on or off.
pub const ZC_ACTIVE_QUEST: u16 = 0x02b7;
/// ZC_QUEST_NOTIFY_EFFECT: the mark over an NPC that has something for the character.
pub const ZC_QUEST_NOTIFY_EFFECT: u16 = 0x0446;

/// The "mark" that takes an NPC's mark away (rAthena's `QTYPE_NONE`; the client's `ReceiveMinimapMarker`).
pub const NO_MARK: u16 = 9999;

/// Monsters a quest wants hunted: one of up to three kinds.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Hunt {
    /// The hunt's own number (rAthena: the quest's number times 1000, plus which of its hunts it is).
    pub id: u32,
    /// What kind of monster counts when no one monster is named (its race, size or element).
    pub condition: u32,
    /// The monster.
    pub mob: u32,
    /// The levels of the monsters that count, least and most; 0 for no bound.
    pub levels: (i16, i16),
    pub killed: i16,
    /// How many are wanted; 0 until the server has said.
    pub goal: i16,
    /// What the server calls the monster, or the kind of monster.
    pub name: String,
}

/// A quest of the journal.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Quest {
    pub id: u32,
    /// Switched on: an inactive quest lies under the journal's third tab.
    pub active: bool,
    /// When it runs out, in seconds since 1970; 0 for never.
    pub deadline: u32,
    pub hunts: Vec<Hunt>,
}

/// The count of one hunt, as ZC_UPDATE_MISSION_HUNT_EX tells it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    pub quest: u32,
    pub hunt: u32,
    pub goal: i16,
    pub killed: i16,
}

/// The mark over an NPC, as ZC_QUEST_NOTIFY_EFFECT tells it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    pub npc: u32,
    pub cell: (u16, u16),
    /// Which mark (rAthena's `QTYPE_*`: 0 and 1 the quest's, 2 and 3 the job's, 4 and 5 the event's, and on),
    /// or `NO_MARK`.
    pub effect: u16,
    /// Which colour the minimap shows it in (rAthena's `QMARK_*`); 0 for no mark on the minimap.
    pub colour: u16,
}

/// A hunt as the list and the new quest write it: `u32 hunt`, `u32 condition`, `u32 mob`, `u16 min level`,
/// `u16 max level`, `u16 killed`, then `u16 goal` in the list only, then `char name[24]`.
fn hunt(bytes: &[u8], with_goal: bool) -> Option<Hunt> {
    let named = if with_goal { 20 } else { 18 };
    Some(Hunt {
        id: long(bytes, 0)?,
        condition: long(bytes, 4)?,
        mob: long(bytes, 8)?,
        levels: (short(bytes, 12)?, short(bytes, 14)?),
        killed: short(bytes, 16)?,
        goal: if with_goal { short(bytes, 18)? } else { 0 },
        name: text(bytes.get(named..named + 24)?),
    })
}

/// ZC_ALL_QUEST_LIST3 (0x09f8): `u16 length`, `u32 count`, then for each quest `u32 quest`, `u8 active`,
/// `u32 time left`, `u32 deadline`, `u16 hunts` and that many hunts of 44 bytes. A list cut short gives the
/// quests that are whole.
pub fn list(packet: &[u8]) -> Vec<Quest> {
    let count = long(packet, 4).unwrap_or(0);
    let mut at = 8;
    let mut quests = Vec::new();
    for _ in 0..count {
        let (Some(id), Some(active), Some(deadline), Some(hunts)) =
            (long(packet, at), packet.get(at + 4), long(packet, at + 9), word(packet, at + 13)) else { break };
        at += 15;
        let Some(hunts) = (0..usize::from(hunts)).map(|index| hunt(packet.get(at + index * 44..)?, true)).collect::<Option<Vec<_>>>() else { break };
        at += hunts.len() * 44;
        quests.push(Quest { id, active: *active != 0, deadline, hunts });
    }
    quests
}

/// ZC_ADD_QUEST_EX (0x09f9, 143 bytes): `u32 quest`, `u8 active`, `u32 time left`, `u32 deadline`, `u16 hunts`,
/// then room for three hunts of 42 bytes, of which that many are filled. How many of each are wanted is not in
/// it: ZC_UPDATE_MISSION_HUNT_EX follows.
pub fn added(packet: &[u8]) -> Option<Quest> {
    let packet = packet.get(..143)?;
    let hunts = usize::from(word(packet, 15)?).min(3);
    Some(Quest {
        id: long(packet, 2)?,
        active: packet[6] != 0,
        deadline: long(packet, 11)?,
        hunts: (0..hunts).map(|index| hunt(&packet[17 + index * 42..], false)).collect::<Option<Vec<_>>>()?,
    })
}

/// ZC_UPDATE_MISSION_HUNT_EX (0x09fa): `u16 length`, `u16 count`, then for each `u32 quest`, `u32 hunt`,
/// `u16 goal`, `u16 killed`.
pub fn hunted(packet: &[u8]) -> Vec<Progress> {
    let count = usize::from(word(packet, 4).unwrap_or(0));
    records(packet, 6, 12)
        .take(count)
        .filter_map(|entry| Some(Progress { quest: long(entry, 0)?, hunt: long(entry, 4)?, goal: short(entry, 8)?, killed: short(entry, 10)? }))
        .collect()
}

/// An entry of ZC_HUNTING_QUEST_INFO: how many of a monster a quest wants and has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HuntInfo {
    pub quest: u32,
    pub mob: u32,
    pub goal: i16,
    pub killed: i16,
}

/// ZC_HUNTING_QUEST_INFO (0x08fe): `u16 length`, then entries of `u32 quest`, `u32 mob`, `u16 goal`,
/// `u16 killed`, as the client reads them (its `questUpdateType2`). (rAthena writes the hunt's own number where
/// the client reads the quest's, so from that server no entry is of a quest the journal has, and the counts
/// come with ZC_UPDATE_MISSION_HUNT_EX.)
pub fn hunting_info(packet: &[u8]) -> Vec<HuntInfo> {
    records(packet, 4, 12)
        .filter_map(|entry| {
            Some(HuntInfo {
                quest: long(entry, 0)?,
                mob: long(entry, 4)?,
                goal: short(entry, 8)?,
                killed: short(entry, 10)?,
            })
        })
        .collect()
}

/// ZC_DEL_QUEST (0x02b4, 6 bytes): `u32 quest`.
pub fn deleted(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// ZC_ACTIVE_QUEST (0x02b7, 7 bytes): `u32 quest`, `u8 active`.
pub fn activated(packet: &[u8]) -> Option<(u32, bool)> {
    Some((long(packet, 2)?, *packet.get(6)? != 0))
}

/// CZ_ACTIVE_QUEST (0x02b6, 7 bytes): `u32 quest`, `u8 active`: the lock of a quest's row was pressed
/// (the client's `CUIQuestItem::OnActiveStateChanged`).
pub fn activate(quest: u32, active: bool) -> Vec<u8> {
    Writer::new(0x02b6).u32(quest).u8(u8::from(active)).done()
}

/// ZC_QUEST_NOTIFY_EFFECT (0x0446, 14 bytes): `u32 npc`, `u16 x`, `u16 y`, `u16 effect`, `u16 colour`.
pub fn notified(packet: &[u8]) -> Option<Mark> {
    Some(Mark { npc: long(packet, 2)?, cell: (word(packet, 6)?, word(packet, 8)?), effect: word(packet, 10)?, colour: word(packet, 12)? })
}
