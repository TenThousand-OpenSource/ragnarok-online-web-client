use super::*;
use crate::net::bytes::testing::assert_lengths;
use crate::net::packet_length;

fn account() -> LoginAccepted {
    LoginAccepted { login_id1: 0x1111_2222, account_id: 2_000_000, login_id2: 0x3333_4444, sex: 1, char_servers: Vec::new() }
}

/// Every request is as long as the server's table says, or the server would wait for more or misread the next.
#[test]
fn requests_have_the_length_the_server_expects() {
    let requests = [
        login("tester", "secret"),
        char_enter(&account()),
        ping(2_000_000),
        select_char(0),
        make_char("Hero", 0, 1, 2, 0, 1),
        map_enter(&account(), 150_000, 1234),
        map_loaded(),
        tick(1234),
        delete_reserve(150_000),
        delete_now(150_000, "990131"),
        delete_cancel(150_000),
        walk_to(156, 191),
        raise_stat(parameter::STR, 1),
        restart(true),
    ];
    assert_lengths(&requests);
}

#[test]
fn builds_the_login_request() {
    let packet = login("tester", "secret");
    assert_eq!(&packet[..6], [0x64, 0x00, 55, 0, 0, 0]);
    assert_eq!(&packet[6..13], b"tester\0");
    assert_eq!(&packet[30..37], b"secret\0");
    assert!(packet[13..30].iter().chain(&packet[37..]).all(|&byte| byte == 0));
}

/// A name that fills its field keeps a terminator.
#[test]
fn long_text_is_cut_to_fit() {
    let packet = login(&"u".repeat(40), "p");
    assert_eq!(&packet[6..29], "u".repeat(23).as_bytes());
    assert_eq!(packet[29], 0);
    assert_eq!(packet.len(), 55);
}

#[test]
fn builds_the_char_and_map_requests() {
    assert_eq!(char_enter(&account()), [0x65, 0x00, 0x80, 0x84, 0x1e, 0x00, 0x22, 0x22, 0x11, 0x11, 0x44, 0x44, 0x33, 0x33, 0, 0, 1]);

    let packet = make_char("Hero", 2, 5, 7, 4218, 0);
    assert_eq!(&packet[2..7], b"Hero\0");
    assert_eq!(&packet[26..], [2, 5, 0, 7, 0, 0x7a, 0x10, 0, 0, 0]);

    let packet = map_enter(&account(), 150_000, 0x0102_0304);
    assert_eq!(&packet[2..6], 2_000_000u32.to_le_bytes());
    assert_eq!(&packet[6..10], 150_000u32.to_le_bytes());
    assert_eq!(&packet[10..14], 0x1111_2222u32.to_le_bytes());
    assert_eq!(&packet[14..18], [4, 3, 2, 1]);
    assert_eq!(packet[22], 1);
}

#[test]
fn reads_the_login_reply() {
    let mut packet = vec![0u8; 64 + 160];
    packet[..2].copy_from_slice(&AC_ACCEPT_LOGIN.to_le_bytes());
    packet[2..4].copy_from_slice(&224u16.to_le_bytes());
    packet[4..8].copy_from_slice(&7u32.to_le_bytes());
    packet[8..12].copy_from_slice(&2_000_000u32.to_le_bytes());
    packet[12..16].copy_from_slice(&9u32.to_le_bytes());
    packet[46] = 1;
    packet[64..68].copy_from_slice(&[127, 0, 0, 1]);
    packet[68..70].copy_from_slice(&6121u16.to_le_bytes());
    packet[70..77].copy_from_slice(b"rAthena");
    packet[90..92].copy_from_slice(&3u16.to_le_bytes());

    let accepted = login_accepted(&packet).unwrap();
    assert_eq!((accepted.login_id1, accepted.account_id, accepted.login_id2, accepted.sex), (7, 2_000_000, 9, 1));
    assert_eq!(accepted.char_servers, [CharServer { ip: [127, 0, 0, 1], port: 6121, name: "rAthena".into(), users: 3 }]);
    assert_eq!(login_accepted(&packet[..40]), None);
}

#[test]
fn reads_why_a_login_was_refused() {
    let mut packet = vec![0u8; 26];
    packet[2] = 6;
    packet[6..16].copy_from_slice(b"2027-01-01");
    assert_eq!(login_refused(&packet), Some((6, "2027-01-01".to_owned())));
    assert_eq!(login_refused(&packet[..10]), None);
    assert_eq!(ban_reason(&[0x81, 0x00, 2]), Some(2));
    assert_eq!(make_char_refused(&[0x6e, 0x00, 3]), 3);
    assert_eq!(char_slots(&[0x2d, 0x08, 29, 0, 9, 0, 0, 9, 15, 0]), Some((9, 15)));
}

fn character_bytes(name: &str, slot: u8) -> Vec<u8> {
    let mut info = vec![0u8; CHARACTER_INFO];
    info[..4].copy_from_slice(&150_000u32.to_le_bytes());
    info[4..12].copy_from_slice(&123_456_789_012u64.to_le_bytes());
    info[12..16].copy_from_slice(&500u32.to_le_bytes());
    info[24..28].copy_from_slice(&10u32.to_le_bytes());
    info[50..58].copy_from_slice(&40u64.to_le_bytes());
    info[58..66].copy_from_slice(&42u64.to_le_bytes());
    info[66..74].copy_from_slice(&11u64.to_le_bytes());
    info[74..82].copy_from_slice(&12u64.to_le_bytes());
    info[84..86].copy_from_slice(&4u16.to_le_bytes());
    info[86..88].copy_from_slice(&3u16.to_le_bytes());
    info[92..94].copy_from_slice(&99u16.to_le_bytes());
    info[108..108 + name.len()].copy_from_slice(name.as_bytes());
    info[132..138].copy_from_slice(&[1, 2, 3, 4, 5, 6]);
    info[138] = slot;
    info[139] = 8;
    info[96..98].copy_from_slice(&5u16.to_le_bytes());
    info[100..102].copy_from_slice(&224u16.to_le_bytes());
    info[102..104].copy_from_slice(&12u16.to_le_bytes());
    info[106..108].copy_from_slice(&2u16.to_le_bytes());
    info[142..154].copy_from_slice(b"prontera.gat");
    info[174] = 1;
    info
}

#[test]
fn reads_the_character_list() {
    let mut packet = vec![0u8; 27];
    packet.extend(character_bytes("Hero", 0));
    packet.extend(character_bytes("Second", 4));

    let characters = char_list(&packet);
    assert_eq!(characters.len(), 2);
    let hero = &characters[0];
    assert_eq!((hero.char_id, hero.base_exp, hero.zeny, hero.job_level), (150_000, 123_456_789_012, 500, 10));
    assert_eq!((hero.hp, hero.max_hp, hero.sp, hero.max_sp), (40, 42, 11, 12));
    assert_eq!((hero.job, hero.hair_style, hero.base_level, hero.hair_color, hero.sex), (4, 3, 99, 8, 1));
    assert_eq!((hero.name.as_str(), hero.map.as_str(), hero.stats), ("Hero", "prontera", [1, 2, 3, 4, 5, 6]));
    assert_eq!((hero.headgear, hero.body_palette, hero.delete_in), ([5, 224, 12], 2, None));
    assert_eq!((characters[1].name.as_str(), characters[1].slot), ("Second", 4));

    // An account without characters.
    assert!(char_list(&packet[..27]).is_empty());
    assert!(char_list(&[]).is_empty());
}

#[test]
fn reads_a_new_character() {
    let mut packet = HC_ACCEPT_MAKECHAR.to_le_bytes().to_vec();
    packet.extend(character_bytes("Fresh", 1));
    assert_eq!(packet.len() as i32, packet_length(HC_ACCEPT_MAKECHAR).unwrap());
    assert_eq!(char_made(&packet).unwrap().name, "Fresh");
    assert_eq!(char_made(&packet[..50]), None);
}

#[test]
fn deleting_a_character() {
    assert_eq!(delete_reserve(150_000), [0x27, 0x08, 0xf0, 0x49, 0x02, 0x00]);
    assert_eq!(&delete_now(150_000, "990131")[6..], b"990131");
    // No birthdate on the account: six zero bytes.
    assert_eq!(&delete_now(150_000, "")[6..], [0; 6]);
    assert_eq!(delete_now(150_000, "19990131").len(), 12);

    let mut reply = vec![0u8; 14];
    reply[2..6].copy_from_slice(&150_000u32.to_le_bytes());
    reply[6..10].copy_from_slice(&1u32.to_le_bytes());
    reply[10..14].copy_from_slice(&86_400u32.to_le_bytes());
    assert_eq!(delete_reserved(&reply), Some((150_000, 1, 86_400)));
    assert_eq!(delete_result(&reply[..10]), Some((150_000, 1)));

    // A character whose wait is over arrives with seconds below zero.
    let mut info = character_bytes("Leaving", 0);
    info[158..162].copy_from_slice(&(-30i32).to_le_bytes());
    let mut list = vec![0u8; 27];
    list.extend(info);
    assert_eq!(char_list(&list)[0].delete_in, Some(-30));
}

#[test]
fn reads_the_map_server_to_go_to() {
    let mut packet = vec![0u8; 156];
    packet[2..6].copy_from_slice(&150_000u32.to_le_bytes());
    packet[6..18].copy_from_slice(b"prontera.gat");
    packet[26..28].copy_from_slice(&5121u16.to_le_bytes());
    assert_eq!(zone_server(&packet), Some(ZoneServer { char_id: 150_000, map: "prontera".into(), port: 5121 }));
}

/// A walk request packs its cell as the server packs positions, and a move packs two (rAthena's WBUFPOS2).
#[test]
fn packs_and_reads_moves() {
    let request = walk_to(156, 191);
    assert_eq!(request[..2], [0x5f, 0x03]);
    let mut accept = vec![0u8; 13];
    accept[6..9].copy_from_slice(&request[2..5]);
    let read = map_accepted(&accept).unwrap();
    assert_eq!((read.x, read.y), (156, 191));
    assert_eq!(walk_to(1023, 1023)[2..], [0xff, 0xff, 0xf0]);

    let ((x0, y0), (x1, y1)) = ((156u16, 191u16), (163u16, 1000u16));
    let mut packet = vec![0u8; 12];
    packet[6] = (x0 >> 2) as u8;
    packet[7] = ((x0 << 6) | ((y0 >> 4) & 0x3f)) as u8;
    packet[8] = ((y0 << 4) | ((x1 >> 6) & 0x0f)) as u8;
    packet[9] = ((x1 << 2) | ((y1 >> 8) & 0x03)) as u8;
    packet[10] = y1 as u8;
    packet[11] = 0x88;
    packet[2..6].copy_from_slice(&77_000u32.to_le_bytes());
    let walk = crate::net::actors::Walked { from: (156, 191), to: (163, 1000), within: (8, 8), started: 77_000 };
    assert_eq!(player_move(&packet), Some(walk));
    assert_eq!(server_time(&[0x7f, 0, 0xc8, 0x2c, 1, 0]), Some(77_000));
    assert_eq!(player_move(&packet[..10]), None);
}

#[test]
fn reads_stops_map_changes_and_parameters() {
    let mut stop = vec![0u8; 10];
    stop[2..6].copy_from_slice(&2_000_000u32.to_le_bytes());
    stop[6..8].copy_from_slice(&150u16.to_le_bytes());
    stop[8..10].copy_from_slice(&180u16.to_le_bytes());
    assert_eq!(stop_move(&stop), Some((2_000_000, 150, 180)));

    let mut moved = vec![0u8; 22];
    moved[2..13].copy_from_slice(b"izlude.gat\0");
    moved[18..20].copy_from_slice(&128u16.to_le_bytes());
    moved[20..22].copy_from_slice(&114u16.to_le_bytes());
    assert_eq!(map_move(&moved), Some(("izlude".to_owned(), 128, 114)));

    let mut speed = vec![0u8; 8];
    speed[4..8].copy_from_slice(&150i32.to_le_bytes());
    assert_eq!(parameter(&speed), Some((PARAMETER_SPEED, 150)));
}

#[test]
fn reads_the_status_window() {
    let mut packet = vec![0u8; 44];
    packet[2..4].copy_from_slice(&48u16.to_le_bytes());
    // Str 9 (2 points to raise), Luk 1 (2 points).
    packet[4] = 9;
    packet[5] = 2;
    packet[14] = 1;
    packet[15] = 2;
    packet[16..18].copy_from_slice(&25i16.to_le_bytes());
    packet[32..34].copy_from_slice(&180i16.to_le_bytes());
    packet[40..42].copy_from_slice(&672i16.to_le_bytes());
    let values = status(&packet);
    let value = |name: u16| values.iter().find(|(parameter, _)| *parameter == name).map(|(_, value)| *value);
    assert_eq!(value(parameter::STATUS_POINTS), Some(48));
    assert_eq!((value(parameter::STR), value(parameter::STR_COST)), (Some(9), Some(2)));
    assert_eq!((value(parameter::STR + 5), value(parameter::STR_COST + 5)), (Some(1), Some(2)));
    assert_eq!((value(parameter::ATK), value(parameter::HIT), value(parameter::ATTACK_DELAY)), (Some(25), Some(180), Some(672)));
    assert!(status(&packet[..40]).is_empty());

    let mut long = vec![0u8; 12];
    long[2] = 22;
    long[4..12].copy_from_slice(&5_000_000_000i64.to_le_bytes());
    assert_eq!(long_parameter(&long), Some((parameter::NEXT_BASE_EXP, 5_000_000_000)));

    let mut couple = vec![0u8; 14];
    couple[2] = 13;
    couple[6] = 9;
    couple[10] = 3;
    assert_eq!(stat(&couple), Some((parameter::STR, 9, 3)));
    assert_eq!(stat_cost(&[0xbe, 0, 32, 0, 3]), Some((parameter::STR_COST, 3)));
    assert_eq!(raise_stat(parameter::STR + 1, 1), [0xbb, 0x00, 14, 0, 1]);
}

#[test]
fn builds_and_reads_chat() {
    let packet = say("Walker", "hello there");
    assert_eq!(packet[..2], [0xf3, 0x00]);
    assert_eq!(usize::from(u16_at(&packet, 2)), packet.len());
    assert_eq!(&packet[4..], b"Walker : hello there\0");

    let mut own = vec![0x8e, 0x00, 0, 0];
    own.extend(b"Walker : hi\0");
    assert_eq!(chat_text(ZC_NOTIFY_PLAYERCHAT, &own).as_deref(), Some("Walker : hi"));
    let mut other = vec![0x8d, 0x00, 0, 0, 1, 2, 3, 4];
    other.extend(b"Friend : hey\0");
    assert_eq!(chat_text(ZC_NOTIFY_CHAT, &other).as_deref(), Some("Friend : hey"));
    assert_eq!(chat_text(0x1234, &own), None);
}

/// The packed position, as rAthena's WBUFPOS writes it.
#[test]
fn reads_the_spawn_position() {
    let (x, y, direction) = (156u16, 191u16, 4u8);
    let mut packet = vec![0u8; 13];
    packet[2..6].copy_from_slice(&77u32.to_le_bytes());
    packet[6] = (x >> 2) as u8;
    packet[7] = ((x << 6) | ((y >> 4) & 0x3f)) as u8;
    packet[8] = ((y << 4) as u8) | (direction & 0x0f);
    assert_eq!(map_accepted(&packet), Some(MapAccepted { server_tick: 77, x: 156, y: 191, direction: 4 }));

    // The largest coordinates the format holds.
    packet[6..9].copy_from_slice(&[0xff, 0xff, 0xf0]);
    let accepted = map_accepted(&packet).unwrap();
    assert_eq!((accepted.x, accepted.y, accepted.direction), (1023, 1023, 0));
}

/// A cell is packed and read back the one way, here and in the files of those in sight.
#[test]
fn packs_a_cell_into_three_bytes_and_reads_it_back() {
    assert_eq!(cell_packed((156, 191)), [0x27, 0x0b, 0xf0]);
    assert_eq!(cell_unpacked(&cell_packed((156, 191))), ((156, 191), 0));
    assert_eq!(cell_unpacked(&[0xff, 0xff, 0xf5]), ((1023, 1023), 5));
    assert_eq!(walk_to(156, 191)[2..], cell_packed((156, 191)));
}

#[test]
fn reads_a_number_and_its_value() {
    assert_eq!(number_and_value(&[0xb0, 0, 5, 0, 0xfe, 0xff, 0xff, 0xff]), Some((5, -2)));
    assert_eq!(number_and_value(&[0xb0, 0, 5, 0, 0xfe, 0xff, 0xff]), None);
    assert_eq!(parameter(&[0xb0, 0, 5, 0, 0xfe, 0xff, 0xff, 0xff, 9]), Some((5, -2)));
}

#[test]
fn reads_an_effect_shown_on_someone() {
    assert_eq!(packet_length(ZC_NOTIFY_EFFECT), Some(10));
    assert_eq!(effect_on(&[0x9b, 0x01, 7, 0, 0, 0, 1, 0, 0, 0]), (Some(7), 1));
    // One that ends early: whoever it is about is still read, and the kind is none there is.
    assert_eq!(effect_on(&[0x9b, 0x01, 7, 0, 0, 0, 1, 0]), (Some(7), u32::MAX));
    assert_eq!(effect_on(&[0x9b, 0x01, 7]), (None, u32::MAX));
}

#[test]
fn reads_whether_the_character_may_leave() {
    assert_eq!((packet_length(ZC_RESTART_ACK), packet_length(ZC_ACK_REQ_DISCONNECT)), (Some(3), Some(4)));
    assert_eq!(restart(true), [0xb2, 0x00, 1]);
    assert_eq!((restart_allowed(&[0xb3, 0, 1]), restart_allowed(&[0xb3, 0, 0]), restart_allowed(&[0xb3, 0])), (true, false, false));
    assert_eq!((leave_refused(&[0x8b, 0x01, 0, 0]), leave_refused(&[0x8b, 0x01, 1, 0]), leave_refused(&[0x8b, 0x01, 0, 1])), (false, true, true));
    // What ends before its result is taken for a refusal.
    assert!(leave_refused(&[0x8b, 0x01, 0]));
}
