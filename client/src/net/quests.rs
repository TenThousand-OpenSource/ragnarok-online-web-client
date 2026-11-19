//! The quests: the journal the map server sends on entering, what it says as a quest is taken, advanced and
//! dropped, and the mark it puts over an NPC.
//!
//! The layouts are rAthena's `clif_quest_*` (`src/map/clif.cpp`) for PACKETVER 20260219, where the list, the new
//! quest and the count of monsters are the versions with a hunt's own number and its levels (`0x09f8`, `0x09f9`,
//! `0x09fa`); the client reads them in `CGameMode::ReceiveOngoingQuestList`, `ReceiveAddQuest` and
//! `ReceiveQuestHuntUpdate`.

use super::bytes::{long, text, word};

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
        levels: (word(bytes, 12)? as i16, word(bytes, 14)? as i16),
        killed: word(bytes, 16)? as i16,
        goal: if with_goal { word(bytes, 18)? as i16 } else { 0 },
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
        let (Some(id), Some(active), Some(deadline), Some(hunts)) = (long(packet, at), packet.get(at + 4), long(packet, at + 9), word(packet, at + 13)) else { break };
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
    packet
        .get(6..)
        .unwrap_or_default()
        .chunks_exact(12)
        .take(count)
        .filter_map(|entry| Some(Progress { quest: long(entry, 0)?, hunt: long(entry, 4)?, goal: word(entry, 8)? as i16, killed: word(entry, 10)? as i16 }))
        .collect()
}

/// ZC_HUNTING_QUEST_INFO (0x08fe): `u16 length`, then entries of `u32 quest`, `u32 mob`, `u16 goal`,
/// `u16 killed`, as the client reads them (its `questUpdateType2`): each `(quest, mob, goal, killed)`. (rAthena
/// writes the hunt's own number where the client reads the quest's, so from that server no entry is of a quest
/// the journal has, and the counts come with ZC_UPDATE_MISSION_HUNT_EX.)
pub fn hunting_info(packet: &[u8]) -> Vec<(u32, u32, i16, i16)> {
    packet.get(4..).unwrap_or_default().chunks_exact(12).filter_map(|entry| Some((long(entry, 0)?, long(entry, 4)?, word(entry, 8)? as i16, word(entry, 10)? as i16))).collect()
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
    let mut packet = vec![0xb6, 0x02];
    packet.extend(quest.to_le_bytes());
    packet.push(u8::from(active));
    packet
}

/// ZC_QUEST_NOTIFY_EFFECT (0x0446, 14 bytes): `u32 npc`, `u16 x`, `u16 y`, `u16 effect`, `u16 colour`.
pub fn notified(packet: &[u8]) -> Option<Mark> {
    Some(Mark { npc: long(packet, 2)?, cell: (word(packet, 6)?, word(packet, 8)?), effect: word(packet, 10)?, colour: word(packet, 12)? })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hunt as the server writes it.
    fn hunt_bytes(id: u32, mob: u32, levels: (u16, u16), killed: u16, goal: Option<u16>, name: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(id.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        bytes.extend(mob.to_le_bytes());
        bytes.extend(levels.0.to_le_bytes());
        bytes.extend(levels.1.to_le_bytes());
        bytes.extend(killed.to_le_bytes());
        if let Some(goal) = goal {
            bytes.extend(goal.to_le_bytes());
        }
        let mut named = [0u8; 24];
        named[..name.len()].copy_from_slice(name.as_bytes());
        bytes.extend(named);
        bytes
    }

    #[test]
    fn reads_the_list_of_quests() {
        let mut packet = vec![0xf8, 0x09, 0, 0];
        packet.extend(2u32.to_le_bytes());
        // A quest with nothing to hunt, switched off.
        packet.extend(1001u32.to_le_bytes());
        packet.push(0);
        packet.extend(0u32.to_le_bytes());
        packet.extend(0u32.to_le_bytes());
        packet.extend(0u16.to_le_bytes());
        // One with two hunts and a deadline.
        packet.extend(60104u32.to_le_bytes());
        packet.push(1);
        packet.extend(3600u32.to_le_bytes());
        packet.extend(1_800_000_000u32.to_le_bytes());
        packet.extend(2u16.to_le_bytes());
        packet.extend(hunt_bytes(60_104_000, 1002, (0, 0), 3, Some(10), "Poring"));
        packet.extend(hunt_bytes(60_104_001, 1113, (10, 20), 0, Some(5), "Drops"));
        let length = packet.len() as u16;
        packet[2..4].copy_from_slice(&length.to_le_bytes());
        assert_eq!(length, 8 + 15 + 15 + 2 * 44);
        let quests = list(&packet);
        assert_eq!(quests.len(), 2);
        assert_eq!(quests[0], Quest { id: 1001, active: false, deadline: 0, hunts: Vec::new() });
        assert_eq!((quests[1].id, quests[1].active, quests[1].deadline), (60104, true, 1_800_000_000));
        assert_eq!(quests[1].hunts[0], Hunt { id: 60_104_000, condition: 0, mob: 1002, levels: (0, 0), killed: 3, goal: 10, name: "Poring".to_owned() });
        assert_eq!((quests[1].hunts[1].levels, quests[1].hunts[1].goal, quests[1].hunts[1].name.as_str()), ((10, 20), 5, "Drops"));
    }

    #[test]
    fn a_list_cut_short_gives_the_whole_quests() {
        let mut packet = vec![0xf8, 0x09, 0, 0];
        packet.extend(2u32.to_le_bytes());
        packet.extend(7u32.to_le_bytes());
        packet.push(1);
        packet.extend([0; 8]);
        packet.extend(0u16.to_le_bytes());
        packet.extend(8u32.to_le_bytes());
        assert_eq!(list(&packet).iter().map(|quest| quest.id).collect::<Vec<_>>(), [7]);
        assert!(list(&[0xf8, 0x09]).is_empty());
    }

    #[test]
    fn reads_a_new_quest() {
        let mut packet = vec![0xf9, 0x09];
        packet.extend(7122u32.to_le_bytes());
        packet.push(1);
        packet.extend(0u32.to_le_bytes());
        packet.extend(0u32.to_le_bytes());
        packet.extend(1u16.to_le_bytes());
        packet.extend(hunt_bytes(7_122_000, 1031, (0, 0), 0, None, "Poporing"));
        packet.resize(143, 0);
        let quest = added(&packet).unwrap();
        assert_eq!((quest.id, quest.active, quest.deadline, quest.hunts.len()), (7122, true, 0, 1));
        assert_eq!(quest.hunts[0], Hunt { id: 7_122_000, condition: 0, mob: 1031, levels: (0, 0), killed: 0, goal: 0, name: "Poporing".to_owned() });
        assert_eq!(added(&packet[..100]), None);
    }

    #[test]
    fn reads_the_count_of_monsters_hunted() {
        let mut packet = vec![0xfa, 0x09, 30, 0];
        packet.extend(2u16.to_le_bytes());
        for (hunt, killed) in [(7_122_000u32, 4u16), (7_122_001, 10)] {
            packet.extend(7122u32.to_le_bytes());
            packet.extend(hunt.to_le_bytes());
            packet.extend(10u16.to_le_bytes());
            packet.extend(killed.to_le_bytes());
        }
        assert_eq!(hunted(&packet), [Progress { quest: 7122, hunt: 7_122_000, goal: 10, killed: 4 }, Progress { quest: 7122, hunt: 7_122_001, goal: 10, killed: 10 }]);
        assert!(hunted(&packet[..5]).is_empty());
    }

    #[test]
    fn reads_what_a_new_quest_hunts() {
        let mut packet = vec![0xfe, 0x08, 16, 0];
        packet.extend(7_122_000u32.to_le_bytes());
        packet.extend(1031u32.to_le_bytes());
        packet.extend(10u16.to_le_bytes());
        packet.extend(0u16.to_le_bytes());
        assert_eq!(hunting_info(&packet), [(7_122_000, 1031, 10, 0)]);
    }

    #[test]
    fn reads_a_quest_dropped_and_one_switched() {
        assert_eq!(deleted(&[0xb4, 0x02, 0xd2, 0x1b, 0, 0]), Some(7122));
        assert_eq!(activated(&[0xb7, 0x02, 0xd2, 0x1b, 0, 0, 1]), Some((7122, true)));
        assert_eq!(activated(&[0xb7, 0x02, 0xd2, 0x1b, 0, 0]), None);
    }

    #[test]
    fn asks_for_a_quest_to_be_switched() {
        assert_eq!(activate(7122, false), [0xb6, 0x02, 0xd2, 0x1b, 0, 0, 0]);
        assert_eq!(activate(7122, true).len(), 7);
    }

    #[test]
    fn reads_the_mark_over_an_npc() {
        let mut packet = vec![0x46, 0x04];
        packet.extend(110_000_123u32.to_le_bytes());
        packet.extend(150u16.to_le_bytes());
        packet.extend(93u16.to_le_bytes());
        packet.extend(1u16.to_le_bytes());
        packet.extend(2u16.to_le_bytes());
        assert_eq!(notified(&packet), Some(Mark { npc: 110_000_123, cell: (150, 93), effect: 1, colour: 2 }));
        assert_eq!(notified(&packet[..13]), None);
    }
}
