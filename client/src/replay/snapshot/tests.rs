//! Tests of the session's sections of a replay file (6 to 14): each written with the client's tags in the
//! client's order and at the client's sizes, and read back as it was given.

use super::*;

/// The tags of a section's records in their order, and of each how many bytes it holds.
pub(super) fn laid(section: &[u8]) -> Vec<(u16, usize)> {
    let records = tags(section);
    assert_eq!(records.iter().map(|(_, bytes)| 6 + bytes.len()).sum::<usize>(), section.len(), "the records fill the section");
    records.into_iter().map(|(tag, bytes)| (tag, bytes.len())).collect()
}

pub(super) fn quests() -> Vec<quests::Quest> {
    let hunt = quests::Hunt { id: 7, condition: 1, monster: 1002, min_level: 1, max_level: 99, killed: 3, goal: 10, name: b"Poring".to_vec() };
    vec![
        quests::Quest { id: 7122, state: 1, when: [2026, 11, 27, 14, 5], hunts: vec![hunt.clone(), quests::Hunt { monster: 1113, ..hunt }] },
        quests::Quest { id: 8000, state: 2, when: [0; 5], hunts: Vec::new() },
    ]
}

#[test]
fn the_quests_are_one_piece_of_records_of_the_clients_sizes() {
    let section = quests::written(&quests());
    let sizes = 4 + 2 * quests::QUEST_BYTES + 2 * quests::HUNT_BYTES;
    assert_eq!((quests::QUEST_BYTES, quests::HUNT_BYTES), (0x18, 0x47));
    assert_eq!(laid(&section), [(3500, 0), (3502, 4), (3510, sizes), (3501, 0)]);
    let blob = tags(&section)[2].1;
    assert_eq!(&blob[..4], [2, 0, 0, 0]);
    // The first quest's record: its number, its state, a byte left, two hunts, the five of the time.
    assert_eq!(&blob[4..12], [0xd2, 0x1b, 0, 0, 1, 0, 2, 0]);
    assert_eq!(i16_of(&blob[12..]), 2026);
    assert_eq!(u16_of(&blob[20..]), 5);
    // Its first hunt: the name at 0x14 in 0x33 bytes.
    let hunt = &blob[4 + 0x18..4 + 0x18 + 0x47];
    assert_eq!((i32_of(&hunt[8..]), i16_of(&hunt[18..])), (1002, 10));
    assert_eq!(&hunt[0x14..0x1b], b"Poring\0");
    assert_eq!(quests::read(&section), Some(quests()));
}

#[test]
fn quests_are_written_by_their_numbers_and_a_long_name_is_cut_to_fifty_bytes() {
    let mut given = quests();
    given.reverse();
    given[1].hunts[0].name = vec![b'a'; 80];
    let read = quests::read(&quests::written(&given)).unwrap();
    assert_eq!(read.iter().map(|quest| quest.id).collect::<Vec<_>>(), [7122, 8000]);
    assert_eq!(read[0].hunts[0].name, vec![b'a'; 50]);
}

#[test]
fn a_short_hunt_record_is_read_when_the_kind_is_not_51() {
    let mut blob = 1u32.to_le_bytes().to_vec();
    blob.extend([&9001u32.to_le_bytes()[..], &[1, 0, 1, 0], &[0; 16]].concat());
    let mut hunt = [&5i32.to_le_bytes()[..], &0i32.to_le_bytes(), &1031i32.to_le_bytes(), &[0, 0, 0, 0, 2, 0, 20, 0]].concat();
    hunt.extend(padded(b"Poporing", 0x18));
    assert_eq!(hunt.len(), quests::SHORT_HUNT_BYTES);
    blob.extend(hunt);
    let mut section = Vec::new();
    tagged(&mut section, 3500, &[]);
    tagged(&mut section, 3502, &0x18i32.to_le_bytes());
    tagged(&mut section, 3510, &blob);
    tagged(&mut section, 3501, &[]);
    let read = quests::read(&section).unwrap();
    assert_eq!(read[0].hunts, [quests::Hunt { id: 5, monster: 1031, killed: 2, goal: 20, name: b"Poporing".to_vec(), ..Default::default() }]);
    assert_eq!(quests::read(&section[6..]), None);
}

pub(super) fn messenger() -> people::Messenger {
    let member = |account: u32, name: &[u8], leader: i32| people::Member {
        valid: 1,
        account,
        character: account - 1_850_000,
        leader,
        offline: 0,
        colour: 0x00ff_ffff,
        name: name.to_vec(),
        map: b"prontera.gat".to_vec(),
    };
    people::Messenger {
        leader: 1,
        party_name: b"Six".to_vec(),
        party: vec![member(2_000_006, b"PlaySix", 1), member(2_000_007, b"Other", 0)],
        friends: vec![member(2_000_009, b"Friend", 0)],
    }
}

#[test]
fn the_messenger_keeps_its_lists_as_members_one_after_another() {
    let given = messenger();
    let section = people::messenger(&given);
    let party = 2 * people::MEMBER_BYTES + 7 + 5 + 2 * 12;
    let friends = people::MEMBER_BYTES + 6 + 12;
    assert_eq!(people::MEMBER_BYTES, 84);
    assert_eq!(laid(&section), [(4000, 0), (4010, 1), (4011, 4), (4012, 4), (4013, 0), (4014, 3), (4015, party), (4016, friends), (4001, 0)]);
    let list = tags(&section)[7].1;
    let member = [(251, 0), (253, 4), (254, 4), (255, 4), (258, 4), (259, 4), (260, 4), (256, 6), (257, 12), (252, 0)];
    assert_eq!(laid(list), member);
    assert_eq!(people::messenger_read(&section), Some(given));
    // Nobody in a list: a record of no bytes.
    let empty = people::messenger(&people::Messenger::default());
    assert_eq!(laid(&empty)[6..8], [(4015, 0), (4016, 0)]);
    assert_eq!(people::messenger_read(&empty), Some(people::Messenger::default()));
}

pub(super) fn item() -> items::Item {
    items::Item {
        kind: 4,
        index: 5,
        location: 2,
        worn: 2,
        quantity: 1,
        prices: [100, 50],
        cards: [4001, 0, 0, 4002],
        id: 1201,
        name: b"+7 Knife".to_vec(),
        identified: true,
        damaged: false,
        refine: 7,
        bind: 1,
        expires: 1_800_000_000,
        sprite: 1,
        grade: 2,
        options: vec![(1, 50, 0), (2, -3, 1)],
    }
}

pub(super) fn items() -> items::Items {
    let mut given = items::Items {
        inventory: vec![item(), items::Item { id: 501, quantity: 30, index: 2, identified: true, name: b"Red Potion".to_vec(), ..Default::default() }],
        cart: vec![items::Item { id: 909, quantity: 5, ..Default::default() }],
        storage: vec![items::Item { id: 910, quantity: 9, ..Default::default() }],
        other: vec![(4522, vec![item()])],
        rows: vec![(4603, vec![items::Item::default(); 10])],
        ..Default::default()
    };
    given.equipped[1] = item();
    given.rows[0].1[8] = items::Item { id: 5000, worn: 1024, ..Default::default() };
    given
}

#[test]
fn an_item_is_its_21_records_in_the_clients_order() {
    let empty = items::written(&items::Items::default());
    let lists: Vec<(u16, usize)> = (4510..=4522).map(|tag| (tag, 0)).collect();
    let rows: Vec<(u16, usize)> = (4601..=4606).map(|tag| (tag, 10 * items::ITEM_BYTES)).collect();
    assert_eq!(items::ITEM_BYTES, 21 * 6 + 95);
    assert_eq!(laid(&empty), [vec![(4500, 0), (4701, 4), (4702, 4)], lists, rows, vec![(4501, 0)]].concat());
    let one = &tags(&empty)[16].1[..items::ITEM_BYTES];
    let sizes = [0, 4, 4, 4, 4, 4, 4, 4, 16, 4, 0, 1, 1, 4, 2, 4, 4, 2, 4, 25, 0];
    let order = [281, 283, 285, 284, 286, 287, 288, 289, 290, 291, 292, 293, 294, 295, 296, 297, 298, 299, 300, 301, 282];
    assert_eq!(laid(one), order.into_iter().zip(sizes).collect::<Vec<_>>());
}

#[test]
fn the_items_are_read_back_as_they_were_written() {
    let given = items();
    let section = items::written(&given);
    let inventory = tags(&section)[3].1;
    assert_eq!(inventory.len(), 2 * items::ITEM_BYTES + 8 + 10);
    let records = tags(inventory);
    assert_eq!(records[9], (291, &1201i32.to_le_bytes()[..]));
    assert_eq!(records[18], (300, &2i32.to_le_bytes()[..]));
    assert_eq!(&records[19].1[..10], [1, 0, 50, 0, 0, 2, 0, 0xfd, 0xff, 1]);
    assert_eq!(items::read(&section), Some(given));
    assert_eq!(items::read(&items::written(&items::Items::default())), Some(items::Items::default()));
    assert_eq!(items::read(&[]), None);
}

pub(super) fn companions() -> companions::Companions {
    companions::Companions {
        homunculus: companions::Homunculus {
            id: 110_000_001,
            name: b"Lif".to_vec(),
            stats: [1, 2, 3, 4, 5, 6, 7, 8],
            job: 6001,
            level: 40,
            hp: 900,
            max_hp: 1000,
            sp: 50,
            max_sp: 60,
            friendly: 500,
            accessory: 0,
            exp: 5_000_000_000,
            max_exp: 6_000_000_000,
            hunger: 70,
            max_hunger: 100,
            renamed: 1,
            alive: 1,
            range: 1,
        },
        mercenary: companions::Mercenary { id: 7, name: b"Archer".to_vec(), job: 6017, expires: 1_800_000_000, alive: 1, ..Default::default() },
        pet: companions::Pet { id: 9, renamed: 1, name: b"Poring".to_vec(), job: 1002, level: 3, hunger: 80, friendly: 900, egg: 4, ..Default::default() },
        elemental: companions::Elemental { id: 11, hp: 100, sp: 20, max_hp: 200, max_sp: 40 },
    }
}

#[test]
fn the_companions_are_every_member_at_its_size() {
    let given = companions();
    let section = companions::written(&given);
    let laid = laid(&section);
    let tags: Vec<u16> = [vec![5000], (5101..=5134).collect(), (5201..=5229).collect(), (5301..=5310).collect(), (5401..=5405).collect(), vec![5001]].concat();
    assert_eq!(laid.iter().map(|(tag, _)| *tag).collect::<Vec<_>>(), tags);
    for (tag, size) in laid {
        let client = match tag {
            5000 | 5001 => 0,
            5102 | 5202 | 5303 => 32,
            5119 | 5120 => 8,
            _ => 4,
        };
        assert_eq!(size, client, "{tag}");
    }
    assert_eq!(companions::read(&section), Some(given));
}

pub(super) fn shortcuts() -> rest::Shortcuts {
    let mut shortcuts = rest::Shortcuts::default();
    shortcuts.slots[0] = rest::Slot { kind: 0, id: 501, value: 30 };
    shortcuts.slots[35] = rest::Slot { kind: 1, id: 28, value: 10 };
    shortcuts.quick[1] = rest::Slot { kind: 0, id: 502, value: 1 };
    shortcuts
}

#[test]
fn the_shortcuts_are_seven_bytes_a_place() {
    let section = rest::shortcuts(&shortcuts());
    assert_eq!(laid(&section), [(7000, 0), (7100, 252), (7101, 14), (7001, 0)]);
    let slots = tags(&section)[1].1;
    assert_eq!(&slots[..7], [0, 0xf5, 1, 0, 0, 30, 0]);
    assert_eq!(&slots[245..], [1, 28, 0, 0, 0, 10, 0]);
    assert_eq!(rest::shortcuts_read(&section), Some(shortcuts()));
}

pub(super) fn memorial() -> rest::Memorial {
    rest::Memorial { open: 1, x: 100, y: 200, state: 2, value: 3, name: b"Endless Tower".to_vec(), position: 4, created: 1_700_000_000, expires: 1_700_003_600 }
}

pub(super) fn queue() -> rest::Queue {
    rest::Queue {
        entry: Some(3),
        ready: 1,
        queue_name: b"Tierra Gorge".to_vec(),
        character_name: b"PlaySix".to_vec(),
        selected: Some(2),
        standby: Some(rest::Standby { waiting: 12, ranking: 5, name: b"Tierra Gorge".to_vec() }),
    }
}

pub(super) fn shops() -> rest::Shops {
    let item = |index: u32, item: i32| rest::ShopItem { index, item, first: 10, second: 20 };
    rest::Shops {
        names: vec![(b"Tool Dealer".to_vec(), 1), (b"sixteen bytes it".to_vec(), 2)],
        shops: vec![rest::Shop { id: 1, items: vec![item(0, 501), item(1, 502)], flag: 1 }, rest::Shop { id: 2, items: Vec::new(), flag: 0 }],
    }
}

#[test]
fn the_memorial_the_queue_and_the_shops_are_laid_out_as_the_clients() {
    let section = rest::memorial(&memorial());
    let fours = [5510, 5511, 5512, 5513, 5514].map(|tag| (tag, 4)).to_vec();
    assert_eq!(laid(&section), [vec![(5500, 0)], fours, vec![(5515, 13), (5516, 4), (5517, 4), (5518, 4), (5501, 0)]].concat());
    assert_eq!(rest::memorial_read(&section), Some(memorial()));

    // A queue nobody waits in is the three records; one waited in, all of them in the client's order.
    assert_eq!(laid(&rest::queue(&rest::Queue::default())), [(6500, 0), (6512, 1), (6501, 0)]);
    let section = rest::queue(&queue());
    assert_eq!(laid(&section), [(6500, 0), (6511, 4), (6512, 1), (6513, 12), (6514, 7), (6515, 4), (6517, 4), (6518, 4), (6516, 12), (6501, 0)]);
    assert_eq!(rest::queue_read(&section), Some(queue()));

    assert_eq!(laid(&rest::shops(&rest::Shops::default())), [(7500, 0), (7501, 0)]);
    let section = rest::shops(&shops());
    assert_eq!(laid(&section), [(7500, 0), (7502, 22), (7502, 22), (7504, 20), (7504, 20), (7505, 8), (7505, 8), (7501, 0)]);
    let name = tags(&section)[2].1;
    assert_eq!((&name[..2], &name[2..18], i32_of(&name[18..])), (&[0, 0][..], &b"sixteen bytes it"[..], 2));
    assert_eq!(rest::shops_read(&section), Some(shops()));
}
