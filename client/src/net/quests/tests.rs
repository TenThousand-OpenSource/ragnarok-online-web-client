//! The packets of the quests, read and written.

use super::*;
use crate::net::bytes::testing::field;

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
    bytes.extend(field(name, 24));
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
    assert_eq!(
        hunted(&packet),
        [Progress { quest: 7122, hunt: 7_122_000, goal: 10, killed: 4 }, Progress { quest: 7122, hunt: 7_122_001, goal: 10, killed: 10 }]
    );
    assert!(hunted(&packet[..5]).is_empty());
}

#[test]
fn reads_what_a_new_quest_hunts() {
    let mut packet = vec![0xfe, 0x08, 16, 0];
    packet.extend(7_122_000u32.to_le_bytes());
    packet.extend(1031u32.to_le_bytes());
    packet.extend(10u16.to_le_bytes());
    packet.extend(0u16.to_le_bytes());
    assert_eq!(hunting_info(&packet), [HuntInfo { quest: 7_122_000, mob: 1031, goal: 10, killed: 0 }]);
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
