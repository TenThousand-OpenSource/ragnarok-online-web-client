//! Tests of the map's and the windows' sections of a replay file (15 to 24): their chunks at the client's
//! sizes, the packets in them read by this client's own readers of those packets, and all the sections
//! together through a whole file.

use super::tests::*;
use super::*;
use crate::net::actors::{dropped, unit, Kind, Posture};

fn standing() -> actors::Standing {
    actors::Standing {
        kind: 5,
        id: 110_012_345,
        character: 0,
        speed: 200,
        body_state: 1,
        health_state: 2,
        effect_state: 0x10,
        job: 1002,
        hair: 3,
        weapon: 1201,
        shield: 2101,
        headgear: [10, 224, 5],
        hair_colour: 4,
        clothes_colour: 2,
        head_direction: 1,
        robe: 7,
        guild: 77,
        emblem: 3,
        honor: 9,
        virtue: 0x40,
        pk: 0,
        sex: 1,
        x: 150,
        y: 198,
        direction: 4,
        size: [5, 5],
        state: 2,
        level: 99,
        font: 0,
        max_hp: 9000,
        hp: 7000,
        boss: 1,
        body: 0,
        name: b"Poring".to_vec(),
    }
}

#[test]
fn one_who_stands_is_the_packet_of_0x54_bytes_and_the_name() {
    let given = standing();
    let packet = actors::stand_entry(&given);
    assert_eq!(packet.len(), actors::STAND_BYTES + 6);
    assert_eq!((u16_of(&packet), u16_of(&packet[2..])), (0x09ff, 0x54 + 6));
    // The client's places (`ActorStandEntryPacket`): the job, the guild, the cell and the way faced, the state,
    // the life, the name.
    assert_eq!((u16_of(&packet[0x17..]), u32_of(&packet[0x31..]), packet[0x44]), (1002, 77, 2));
    assert_eq!(&packet[0x3f..0x42], [150 >> 2, (150u16 << 6) as u8 | (198 >> 4), (198u16 << 4) as u8 | 4]);
    assert_eq!((i32_of(&packet[0x49..]), i32_of(&packet[0x4d..]), &packet[0x54..]), (9000, 7000, &b"Poring"[..]));
}

#[test]
fn one_who_stands_is_read_by_our_reader_of_the_packet() {
    let given = standing();
    let read = unit(0x09ff, &actors::stand_entry(&given)).expect("the packet is read");
    assert_eq!((read.id, read.kind, read.job, read.speed), (given.id, Kind::Monster, given.job, given.speed));
    assert_eq!((read.body, read.health, read.state), (given.body_state, given.health_state, given.effect_state));
    assert_eq!((read.hair_style, read.hair_colour, read.body_palette), (given.hair, given.hair_colour, given.clothes_colour));
    assert_eq!((read.weapon, read.shield, read.headgear), (given.weapon, given.shield, given.headgear));
    assert_eq!((read.head, read.guild, read.emblem, read.third), (1, given.guild, given.emblem, given.virtue));
    assert_eq!((read.sex, read.cell, read.direction, read.posture), (given.sex, (given.x, given.y), given.direction, Posture::Sits));
    assert_eq!((read.level, read.boss, read.life), (given.level, given.boss, Some((given.hp, given.max_hp))));
    assert_eq!((read.name.as_str(), read.walks), ("Poring", None));
    // One with no name, lying dead on the farthest cell.
    let far = actors::Standing { x: 1023, y: 1000, direction: 7, state: 1, name: Vec::new(), max_hp: -1, hp: -1, ..given };
    let read = unit(0x09ff, &actors::stand_entry(&far)).expect("the packet is read");
    assert_eq!((read.cell, read.direction, read.posture, read.life, read.name.as_str()), ((1023, 1000), 7, Posture::Lies, None, ""));
}

fn chunks() -> Vec<actors::ActorChunk> {
    vec![
        actors::stands(&standing()),
        actors::room(110_012_345, 4, 20, 1, 1, b"hello"),
        actors::sign(false, 110_012_345, b"cheap potions"),
        actors::sign(true, 110_012_345, b"buying herbs"),
        actors::ActorChunk { kind: actors::kind::MADOGEAR, bytes: vec![0; 0x1c] },
    ]
}

#[test]
fn the_actors_are_chunks_between_10000_and_10001() {
    let given = chunks();
    let section = actors::written(&given);
    assert_eq!(laid(&section), [(10000, 0), (0xd9, 0x54 + 6), (0xc9, 0x11 + 5), (0xca, 0x56), (0xcc, 0x56), (0x191, 0x1c), (10001, 0)]);
    // The chat room's length is 17 and its title's; a sign begins with the number of its packet.
    assert_eq!((u16_of(&given[1].bytes[2..]), u32_of(&given[1].bytes[4..]), given[1].bytes[16]), (22, 110_012_345, 1));
    assert_eq!((u16_of(&given[2].bytes), u16_of(&given[3].bytes), &given[2].bytes[6..19]), (0x0131, 0x0814, &b"cheap potions"[..]));
    assert_eq!(actors::read(&section), Some(given));
    assert_eq!(laid(&actors::written(&[])), [(10000, 0), (10001, 0)]);
    assert_eq!(actors::read(&[]), None);
}

fn ground_items() -> Vec<ground::GroundItem> {
    vec![ground::GroundItem { id: 31, item: 909, identified: true, x: 151, y: 199, count: 3, within: (6, 9) }]
}

fn ground_skills() -> ground::GroundSkills {
    ground::GroundSkills {
        units: vec![ground::GroundSkill { id: 41, creator: 2_000_006, x: 152, y: 200, kind: 0x7f, visible: true }],
        worded: vec![[&[0xc9, 0x01][..], &[7; 14], &[1], &padded(b"hi", 80)].concat()],
    }
}

#[test]
fn what_lies_on_the_ground_is_the_packet_our_reader_reads() {
    let given = ground_items();
    let section = ground::items(&given);
    assert_eq!(laid(&section), [(15000, 0), (0xcf, 0x13), (15001, 0)]);
    let packet = tags(&section)[1].1;
    let read = dropped(0x009d, packet).expect("the packet is read");
    assert_eq!((read.id, read.item, read.cell, read.within, read.identified, read.count), (31, 909, (151, 199), (6, 9), true, 3));
    assert_eq!(crate::net::packet_length(0x009d), Some(0x13));
    assert_eq!(ground::items_read(&section), Some(given));
}

#[test]
fn a_skills_unit_is_the_packet_of_16_bytes() {
    let given = ground_skills();
    let section = ground::skills(&given);
    assert_eq!(laid(&section), [(17000, 0), (0xcd, 0x10), (0xce, 0x61), (17001, 0)]);
    let packet = tags(&section)[1].1;
    assert_eq!(packet, [0x1f, 0x01, 41, 0, 0, 0, 0x86, 0x84, 0x1e, 0, 152, 0, 200, 0, 0x7f, 1]);
    // Our reader of the packet with words (0x01c9) reads the same sixteen bytes so, a unit with no words.
    let read = crate::net::skill_units::written(&[packet, &[0]].concat()).expect("the packet is read");
    assert_eq!((read.id, read.maker, read.cell, read.kind, read.seen), (41, 2_000_006, (152, 200), 0x7f, true));
    assert_eq!(crate::net::skill_units::written(&given.worded[0]).map(|unit| unit.words), Some("hi".to_owned()));
    assert_eq!(ground::skills_read(&section), Some(given));
}

fn icons() -> Vec<screen::StatusIcon> {
    vec![screen::StatusIcon { id: 12, expires: 55_000, values: [10, 0, 0], total: 240_000, remaining: 55_000 }]
}

fn windows() -> Vec<(u32, Vec<u8>)> {
    let mut said = Vec::new();
    tagged(&mut said, 22500, &[]);
    tagged(&mut said, 22503, b"Welcome.");
    tagged(&mut said, 22501, &[]);
    vec![(16, said), (321, Vec::new()), (10011, Vec::new())]
}

#[test]
fn the_icons_and_the_windows_are_laid_out_as_the_clients() {
    let section = screen::icons(&icons());
    assert_eq!(laid(&section), [(19000, 0), (19011, screen::ICON_BYTES), (19001, 0)]);
    assert_eq!(screen::ICON_BYTES, 28);
    let icon = tags(&section)[1].1;
    assert_eq!((u32_of(icon), u32_of(&icon[4..]), u32_of(&icon[8..]), u32_of(&icon[20..]), u32_of(&icon[24..])), (12, 55_000, 10, 240_000, 55_000));
    assert_eq!(screen::icons_read(&section), Some(icons()));

    let section = screen::windows(&windows());
    assert_eq!(laid(&section), [(20000, 0), (20116, 26), (20421, 0), (30111, 0), (20001, 0)]);
    assert_eq!(screen::windows_read(&section), Some(windows()));
    assert_eq!(laid(&screen::windows(&[])), [(20000, 0), (20001, 0)]);
    // The exchange's window is written and read here; the client's own reader does not know it again.
    assert!(screen::window_known(321) && !screen::window_known(10011));
    // A number no tag has room for is left out.
    assert_eq!(screen::windows(&[(70_000, Vec::new())]).len(), 12);
}

#[test]
fn the_small_sections_are_the_clients_bytes() {
    assert_eq!(laid(&screen::settings(1)), [(25000, 0), (25010, 4), (25001, 0)]);
    assert_eq!((screen::settings(1).len(), screen::settings_read(&screen::settings(1))), (22, Some(1)));
    assert_eq!(laid(&screen::reserved()), [(26000, 0), (26001, 0)]);
    assert_eq!(laid(&screen::on_quest(1)), [(14000, 0), (14011, 4), (14001, 0)]);
    assert_eq!((screen::on_quest_read(&screen::on_quest(1)), screen::on_quest_read(&screen::settings(1))), (Some(1), None));
    let navigation = screen::navigation();
    let order = [8000, 8002, 8003, 8004, 8005, 8009, 8010, 8011, 8013, 8014, 8016, 8017, 8018, 8019, 8020, 8023, 8012, 8001];
    assert_eq!(navigation.len(), 151);
    assert_eq!(laid(&navigation).iter().map(|(tag, _)| *tag).collect::<Vec<_>>(), order);
    assert!(screen::navigation_read(&navigation) && !screen::navigation_read(&navigation[..150]) && !screen::navigation_read(&screen::reserved()));
}

fn guild() -> people::Guild {
    let card = |id: u32, name: &[u8], guild: &[u8]| people::NameCard { id, name: name.to_vec(), party: Vec::new(), guild: guild.to_vec(), clan: Vec::new() };
    people::Guild {
        cards: vec![card(2_000_006, b"PlaySix", b"Sixes"), card(2_000_007, b"Other", b"")],
        emblems: vec![people::emblem(b"77_3.ebm", &[1, 2, 3]), people::emblem(b"78_1.ebm", &[4, 5])],
        guild: 77,
        emblem_version: 3,
    }
}

#[test]
fn the_guild_keeps_the_names_each_text_after_its_length() {
    let given = guild();
    let section = people::guild(&given);
    let cards = 4 + (4 + 16 + 7 + 5) + (4 + 16 + 5);
    assert_eq!(laid(&section), [(13000, 0), (13003, cards), (13002, 15), (13004, 4), (13005, 4), (13002, 14), (13001, 0)]);
    let blob = tags(&section)[1].1;
    assert_eq!((u32_of(blob), u32_of(&blob[4..]), i32_of(&blob[8..]), &blob[12..19], i32_of(&blob[19..])), (2, 2_000_006, 7, &b"PlaySix"[..], 0));
    assert_eq!(&tags(&section)[2].1[..12], [8, 0, 0, 0, b'7', b'7', b'_', b'3', b'.', b'e', b'b', b'm']);
    assert_eq!(people::guild_read(&section), Some(given));
    // One in no guild, with nobody's name: the guild and the emblem's version alone.
    assert_eq!(laid(&people::guild(&people::Guild::default())), [(13000, 0), (13004, 4), (13005, 4), (13001, 0)]);
}

fn snapshot() -> Snapshot {
    Snapshot {
        quests: quests(),
        messenger: messenger(),
        items: items(),
        companions: companions(),
        memorial: memorial(),
        queue: queue(),
        shortcuts: shortcuts(),
        shops: shops(),
        actors: chunks(),
        ground_items: ground_items(),
        ground_skills: ground_skills(),
        icons: icons(),
        windows: windows(),
        guild: guild(),
        on_quest: 1,
    }
}

#[test]
fn all_the_sections_go_through_a_whole_file() {
    let given = snapshot();
    let written = sections(&given);
    assert_eq!(written.iter().map(|(number, _)| *number).collect::<Vec<_>>(), [6, 7, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24]);
    let header = crate::replay::Header { when: [2026, 11, 27, 9, 30, 1], name: "PlaySix".to_owned(), map: "prontera.gat".to_owned(), ..Default::default() };
    let file = crate::replay::File { header, client: written.clone(), ..Default::default() };
    let read = crate::replay::read(&crate::replay::written(&file)).expect("the file is read");
    assert_eq!(read.client, written);
    assert_eq!(of_file(&read), given);
    assert_eq!(screen::settings_read(read.section(number::SETTINGS)), Some(0));
    assert!(screen::navigation_read(read.section(number::NAVIGATION)));
    // A file with none of them, and an empty snapshot written and read, are both as empty as a new one.
    assert_eq!(of_file(&crate::replay::File::default()), Snapshot::default());
    let empty = crate::replay::File { client: sections(&Snapshot::default()), ..Default::default() };
    assert_eq!(of_file(&empty), Snapshot::default());
}

#[test]
fn a_chunk_is_the_packet_this_clients_readers_take() {
    use crate::net::{actors as told, chat_rooms, vending};
    // One who stands: the packet as it is, its number and its length written anew whatever the file says.
    let mut stood = actors::stands(&standing());
    stood.bytes[..4].fill(0);
    let packet = actors::packet(&stood).unwrap();
    assert_eq!(packet, actors::stand_entry(&standing()));
    assert_eq!(told::unit(told::ZC_NOTIFY_STANDENTRY, &packet).map(|unit| unit.id), Some(standing().id));
    // A chat room's sign and a stall's.
    let room = chat_rooms::room(&actors::packet(&actors::room(7, 9, 20, 3, 1, b"come in")).unwrap()).unwrap();
    assert_eq!((room.owner, room.id, room.limit, room.users, room.kind, room.title.as_str()), (7, 9, 20, 3, 1, "come in"));
    assert_eq!(vending::sign(&actors::packet(&actors::sign(false, 7, b"cheap")).unwrap()), Some((7, "cheap".to_owned())));
    // One made by its looks alone is shown as one who stands.
    let mut record = vec![0u8; 0x3f];
    record[2..6].copy_from_slice(&2_000_009u32.to_le_bytes());
    record[6..10].copy_from_slice(&4012u32.to_le_bytes());
    record[0x0a..0x0e].copy_from_slice(&[150, 0, 160, 0]);
    record[0x0e] = 1;
    record[0x25..0x2a].copy_from_slice(b"Clone");
    let unit = told::unit(told::ZC_NOTIFY_STANDENTRY, &actors::packet(&actors::ActorChunk { kind: actors::kind::LOOKS, bytes: record }).unwrap()).unwrap();
    assert_eq!((unit.id, unit.job, unit.cell, unit.sex, unit.name.as_str(), unit.kind), (2_000_009, 4012, (150, 160), 1, "Clone", told::Kind::Player));
    // What is too short, or of a kind this client shows nothing of, is passed by.
    for (kind, bytes) in [(actors::kind::STANDS, vec![0; 0x53]), (actors::kind::WALKS, vec![0; 0x59]), (actors::kind::ROOM, vec![0; 16])] {
        assert_eq!(actors::packet(&actors::ActorChunk { kind, bytes }), None);
    }
    for kind in [actors::kind::STALL, actors::kind::LOOKS, actors::kind::BAR, actors::kind::MADOGEAR, actors::kind::BUYING, 0xd0, 0xd8] {
        assert_eq!(actors::packet(&actors::ActorChunk { kind, bytes: vec![0; 8] }), None, "kind {kind:#x}");
    }
}

#[test]
fn one_that_walks_is_the_packet_of_0x5a_bytes_and_the_name_and_our_reader_reads_it() {
    use crate::net::actors::{unit, Walked, ZC_NOTIFY_MOVEENTRY};
    let one = standing();
    let packet = actors::move_entry(&one, (640, 277), 0x8123_4567);
    assert_eq!(packet.len(), actors::MOVE_BYTES + one.name.len());
    assert_eq!(&packet[..4], [&[0xfd, 0x09][..], &(packet.len() as u16).to_le_bytes()].concat());
    // The time it set out at stands after the lower headgear, and everything behind is four bytes on.
    assert_eq!(packet[4..0x25], actors::stand_entry(&one)[4..0x25]);
    assert_eq!(packet[0x25..0x29], 0x8123_4567u32.to_le_bytes());
    assert_eq!(packet[0x29..0x43], actors::stand_entry(&one)[0x25..0x3f]);
    let read = unit(ZC_NOTIFY_MOVEENTRY, &packet).unwrap();
    assert_eq!(read.walks, Some(Walked { from: (one.x, one.y), to: (640, 277), within: (8, 8), started: 0x8123_4567 }));
    let stood = unit(crate::net::actors::ZC_NOTIFY_STANDENTRY, &actors::stand_entry(&one)).unwrap();
    // But for the walk and how it stands, it is told as the one that stands is.
    let alike = crate::net::actors::Unit { walks: None, direction: stood.direction, posture: stood.posture, ..read };
    assert_eq!(alike, stood);
    // Its chunk is handed on as the packet.
    assert_eq!(actors::packet(&actors::walks(&one, (640, 277), 7)), Some(actors::move_entry(&one, (640, 277), 7)));
}
