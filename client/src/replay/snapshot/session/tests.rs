//! Tests of what goes between this client's session and the Windows client's sections of a replay file.

use super::*;
use crate::replay::Header;

fn sword() -> Item {
    Item { index: 4, id: 1101, kind: 5, amount: 1, wearable: 2, worn: 2, refine: 7, identified: true, cards: [4001, 0, 0, 0] }
}

#[test]
fn an_item_goes_into_the_clients_record_and_comes_back() {
    let kept = item(&sword(), b"Sword");
    assert_eq!((kept.kind, kept.index, kept.location, kept.worn, kept.quantity, kept.id, kept.refine), (5, 4, 2, 2, 1, 1101, 7));
    assert_eq!((kept.name.as_slice(), kept.cards, kept.identified), (&b"Sword"[..], [4001, 0, 0, 0], true));
    assert_eq!(item_back(&kept), Some(sword()));
    // An empty place of a row of ten is no item.
    assert_eq!(item_back(&items::Item::default()), None);
}

#[test]
fn what_is_worn_stands_at_the_clients_place_of_the_ten() {
    let places = [(1, 0), (2, 1), (34, 1), (4, 2), (8, 3), (16, 4), (32, 5), (64, 6), (128, 7), (256, 8), (512, 9), (513, 9), (769, 8)];
    for (worn, place) in places {
        assert_eq!(place_worn(worn), Some(place), "worn at {worn}");
    }
    // Not worn, ammunition, a costume: none of the ten.
    assert_eq!([0, 0x8000, 1024, 4096].map(place_worn), [None; 4]);
}

#[test]
fn the_lists_keep_what_is_worn_apart_and_carried_puts_it_back() {
    let potion = Item { index: 2, id: 501, kind: 0, amount: 30, wearable: 0, worn: 0, refine: 0, identified: true, cards: [0; 4] };
    let arrows = Item { index: 3, id: 1750, kind: 10, amount: 500, wearable: 0x8000, worn: 0x8000, ..potion.clone() };
    let given = [potion.clone(), arrows.clone(), sword()];
    let lists = lists(given.iter().map(|one| item(one, b"")).collect(), vec![item(&potion, b"")], Vec::new());
    assert_eq!(lists.inventory.iter().map(|item| item.id).collect::<Vec<_>>(), [501, 1750]);
    assert_eq!(lists.equipped[1].id, 1101);
    assert_eq!(lists.equipped.iter().filter(|item| item.id != 0).count(), 1);
    assert_eq!(lists.cart.len(), 1);
    // Through the section and back: the same three, what is worn still worn.
    let read = items::read(&items::written(&lists)).unwrap();
    let mut carried = carried(&read);
    carried.sort_by_key(|item| item.index);
    assert_eq!(carried, given);
}

#[test]
fn the_bar_is_the_clients_36_places() {
    let bar = [Hotkey { skill: true, id: 5, count: 10 }, Hotkey::default(), Hotkey { skill: false, id: 501, count: 0 }];
    let slots = slots(&bar);
    assert_eq!(slots[0], rest::Slot { kind: 1, id: 5, value: 10 });
    assert_eq!(slots[2], rest::Slot { kind: 0, id: 501, value: 0 });
    let back = hotkeys(&slots);
    assert_eq!(back.len(), 36);
    assert_eq!(&back[..3], &bar);
    assert!(back[3..].iter().all(|hotkey| *hotkey == Hotkey::default()));
    // More places than the client keeps are left out.
    assert_eq!(super::slots(&[Hotkey { skill: true, id: 1, count: 1 }; 38])[35].id, 1);
}

#[test]
fn a_quest_keeps_its_time_as_a_date_and_comes_back() {
    let hunt = told::Hunt { id: 7_122_001, condition: 0, mob: 1002, levels: (0, 0), killed: 3, goal: 10, name: "Poring".to_owned() };
    // 2026-11-27 14:05:00 UTC.
    let given = told::Quest { id: 7122, active: true, deadline: 1_795_788_300, hunts: vec![hunt] };
    let kept = quest(&given);
    assert_eq!((kept.id, kept.state, kept.when), (7122, 1, [2026, 11, 27, 14, 5]));
    assert_eq!((kept.hunts[0].monster, kept.hunts[0].killed, kept.hunts[0].goal, kept.hunts[0].name.as_slice()), (1002, 3, 10, &b"Poring"[..]));
    assert_eq!(quest_back(&kept), given);
    // One that does not run out, and a leap day.
    let endless = told::Quest { id: 1, active: false, deadline: 0, hunts: Vec::new() };
    assert_eq!(quest(&endless).when, [0; 5]);
    assert_eq!(quest_back(&quest(&endless)), endless);
    assert_eq!(quest(&told::Quest { deadline: 1_709_164_800, ..endless.clone() }).when, [2024, 2, 29, 0, 0]);
    assert_eq!(quest_back(&quests::Quest { when: [1970, 1, 1, 0, 1], ..quests::Quest::default() }).deadline, 60);
}

#[test]
fn those_of_the_party_and_friends_come_back_without_what_the_client_does_not_keep() {
    let (name, map) = ("Lead".to_owned(), "prontera.gat".to_owned());
    let given = party::Member { account: 2_000_001, character: 150_001, name, map, leader: true, offline: false, job: 7, level: 99 };
    let kept = member(&given);
    assert_eq!((kept.valid, kept.leader, kept.offline, kept.name.as_slice()), (1, 1, 0, &b"Lead"[..]));
    assert_eq!(member_back(&kept), party::Member { job: 0, level: 0, ..given });
    let pal = friends::Friend { account: 2_000_002, character: 150_002, name: "Pal".to_owned(), offline: true };
    assert_eq!(friend_back(&friend(&pal)), pal);
}

fn file(client: Vec<(usize, Vec<u8>)>) -> File {
    let header = Header { name: "PlaySix".to_owned(), map: "prt_fild08.gat".to_owned(), account: 2_000_006, x: 150, y: 198, direction: 4, ..Header::default() };
    File { header, client, ..File::default() }
}

#[test]
fn the_character_of_a_file_is_read_out_of_the_clients_sections() {
    let looks = sections::Looks {
        account: 2_000_006,
        character: 150_006,
        job: 7,
        head: 5,
        weapon: 1101,
        shield: 2101,
        head_colour: 3,
        worn: [1, 224, 5],
        clothes_colour: 2,
        body_state: 0,
        effect_state: 32,
    };
    let values: HashMap<u16, i64> = [(11, 99), (55, 50), (20, 4_956_300), (13, 99), (18, 7), (5, 7000)].into();
    let sections = vec![(3, sections::character(&looks, &values)), (4, sections::stats(&[1, 2, 3, 4, 5, 6], &values)), (OWN, own(0))];
    let read = character(&file(sections.clone())).unwrap();
    let who = Who { character: 150_006, job: 7, level: 99, job_level: 50, zeny: 4_956_300, look: [5, 3, 2, 1, 224, 5, 0], stats: [99, 0, 0, 0, 0, 7] };
    assert_eq!((read.who, read.looks, read.bonus), (who, looks, [1, 2, 3, 4, 5, 6]));
    assert_eq!(read.values.get(&5), Some(&7000));
    // A file with no record of the sex (the Windows client's) has a man; one without the third section, nobody.
    assert_eq!(character(&file(sections[..2].to_vec())).unwrap().who.look[6], 1);
    assert_eq!(character(&file(sections[1..].to_vec())), None);
    // The section of this client's own is three records, the sex a byte.
    assert_eq!(own(1), [&[0x28, 0x23, 0, 0, 0, 0][..], &[0x36, 0x23, 1, 0, 0, 0, 1], &[0x29, 0x23, 0, 0, 0, 0]].concat());
}

#[test]
fn the_packet_that_enters_the_game_is_read_by_this_clients_reader() {
    // The server's time is what stands at the head of the packets, if anything does.
    let mut given = file(Vec::new());
    assert_eq!(server_time(&given), 0);
    given.packets = vec![(0, vec![0x0f, 0x01, 4, 0]), (0, vec![0x7f, 0x00, 0x78, 0x56, 0x34, 0x92]), (40, vec![0x7f, 0x00, 1, 0, 0, 0])];
    assert_eq!(server_time(&given), 0x9234_5678);
    given.packets.remove(1);
    assert_eq!(server_time(&given), 0);
    let packet = entering(&given.header, 0);
    assert_eq!(entering(&given.header, 0x9234_5678)[2..6], [0x78, 0x56, 0x34, 0x92]);
    assert_eq!(packet.len(), 13);
    assert_eq!(crate::net::packet_length(crate::net::packets::ZC_ACCEPT_ENTER), Some(13));
    let accepted = crate::net::packets::map_accepted(&packet).unwrap();
    assert_eq!((accepted.x, accepted.y, accepted.direction, accepted.server_tick), (150, 198, 4, 0));
    // A header that names a cell off any map does not break the packet.
    let off = Header { x: -5, y: 70_000, direction: -1, ..Header::default() };
    let accepted = crate::net::packets::map_accepted(&entering(&off, 0)).unwrap();
    assert_eq!((accepted.x, accepted.y, accepted.direction), (0, 1023, 7));
}

#[test]
fn a_status_is_the_packet_this_clients_reader_reads() {
    use crate::net::statuses::{changed, Change, NO_TIME};
    let timed = screen::StatusIcon { id: 37, expires: 1_700_000, values: [1, 2, 3], total: 1_800_000, remaining: 0 };
    let packet = status(2_000_006, &timed);
    assert_eq!(packet.len(), 29);
    let change = Change { state: 37, who: 2_000_006, on: true, total: 1_800_000, remaining: 1_700_000, values: [1, 2, 3] };
    assert_eq!(changed(&packet), Some(change));
    assert!(change.timed());
    // One that does not end, as the client keeps it either way.
    for expires in [0, 999_999] {
        let change = changed(&status(1, &screen::StatusIcon { id: 12, expires, ..timed.clone() })).unwrap();
        assert_eq!((change.remaining, change.timed()), (NO_TIME, false));
    }
}
