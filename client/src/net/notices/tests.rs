//! Tests of the server's notices: the packets read, and which line is said in which colour.

use super::*;

#[test]
fn an_action_that_failed_says_its_line() {
    assert_eq!(kind(&[0x3b, 0x01, 3, 0]), Some(3));
    assert_eq!(action_failure(0, 0), Some((242, RED)));
    assert_eq!(action_failure(1, 0), Some((243, PALE_CYAN)));
    assert_eq!(action_failure(2, 0), Some((244, PALE_CYAN)));
    // Ammunition put on: arrows, an assassin's daggers, a gunslinger's bullets, nothing for a ninja.
    assert_eq!(action_failure(3, 3), Some((245, PALE_CYAN)));
    assert_eq!(action_failure(3, 12), Some((1040, PALE_CYAN)));
    assert_eq!(action_failure(3, 24), Some((1175, PALE_CYAN)));
    assert_eq!(action_failure(3, 25), None);
    assert_eq!(action_failure(4, 0), None);
}

#[test]
fn what_a_map_forbids_is_said() {
    assert_eq!(map_info(0), Some((500, RED)));
    assert_eq!(map_info(1).map(|said| said.0), Some(501));
    assert_eq!(map_info(2).map(|said| said.0), Some(1334));
    assert_eq!(map_info(3).map(|said| said.0), Some(1335));
    assert_eq!(map_info(4), None);
}

#[test]
fn lines_of_the_message_table_are_read_with_what_goes_into_them() {
    assert_eq!(value(&[0xe2, 0x07, 0x10, 0x02, 0xfe, 0xff, 0xff, 0xff]), Some((0x210, -2)));
    assert_eq!(skill(&[0xe6, 0x07, 28, 0, 0x4e, 0x04, 0, 0]), Some((28, 0x44e)));
    // A colour is the client's 0xBBGGRR: the red comes first in the packet.
    assert_eq!(coloured(&[0xcd, 0x09, 0x10, 0x00, 0x11, 0x22, 0x33, 0x00]), Some((16, "#112233".to_owned())));
    // The party's call that failed is red whatever the server names.
    assert_eq!(coloured(&[0xcd, 0x09, 0xc5, 0x04, 0x11, 0x22, 0x33, 0x00]), Some((1221, RED.to_owned())));
    assert_eq!(value(&[0xe2, 0x07, 0x10]), None);
}

#[test]
fn gospel_says_its_ten_lines() {
    let said = |kind: u8| skill_message(&[0x15, 0x02, kind, 0, 0, 0]);
    assert_eq!((said(21), said(25), said(28), said(32)), (Some(889), Some(893), Some(894), Some(898)));
    assert_eq!(said(40), Some(915));
    assert_eq!((said(26), said(0)), (None, None));
}

#[test]
fn experience_is_read_and_said() {
    let mut packet = vec![0xcc, 0x0a, 0x40, 0x42, 0x0f, 0x00];
    packet.extend(1234i64.to_le_bytes());
    packet.extend([1, 0, 1, 0]);
    let gained = experience(&packet).unwrap();
    assert_eq!(gained, Experience { who: 1_000_000, amount: 1234, kind: 1, quest: true });
    assert_eq!((gained.lines(), gained.colour()), (Some((Some(3220), 1615)), QUEST_EXPERIENCE));
    let lost = Experience { who: 1, amount: -5, kind: 2, quest: false };
    assert_eq!((lost.lines(), lost.colour()), (Some((Some(3227), 1616)), YELLOW));
    assert_eq!(Experience { who: 1, amount: 0, kind: 1, quest: false }.lines(), None);
    assert_eq!(Experience { who: 1, amount: 7, kind: 3, quest: false }.lines(), Some((None, 1615)));
    assert_eq!(experience(&packet[..17]), None);
}

#[test]
fn the_small_packets_are_read() {
    assert_eq!(overweight(&[0xde, 0x0a, 50, 0, 0, 0]), Some(50));
    assert_eq!(overweight(&[0xde, 0x0a, 0, 0, 0, 0]), None);
    assert_eq!(user_value(&[0xab, 0x01, 7, 0, 0, 0, 4, 0, 0xf6, 0xff, 0xff, 0xff]), Some((7, MANNER, -10)));
    assert_eq!(stat_raised(&[0xbc, 0x00, 13, 0, 1, 42]), Some((13, 42)));
    assert_eq!(stat_raised(&[0xbc, 0x00, 13, 0, 0, 42]), None);
    assert_eq!(damaged(&[0xbb, 0x02, 0x10, 0, 9, 0, 0, 0]), Some((0x10, 9)));
    assert_eq!(damaged(&[0xbb, 0x02, 0, 0, 9, 0, 0, 0]), None);
    assert_eq!((damaged_part(0x10), damaged_part(0x20), damaged_part(0x40)), (Some(1303), Some(0x109a), None));
    assert_eq!(rental_time(&[0x98, 0x02, 0xf5, 0x01, 0, 0, 90, 0, 0, 0]), Some((501, 90)));
    assert_eq!(rental_over(&[0x99, 0x02, 4, 0, 0xf5, 0x01, 0, 0]), Some((4, 501)));
}

#[test]
fn a_rented_item_tells_its_time_in_minutes_then_seconds() {
    assert_eq!(rental_lines(3600), (1256, 60, None));
    assert_eq!(rental_lines(90), (1256, 1, Some(1257)));
    assert_eq!(rental_lines(59), (1270, 59, None));
}

#[test]
fn a_character_is_named_by_its_number() {
    let mut packet = vec![0xf7, 0x0a, 3, 0, 0x39, 0x30, 0, 0];
    packet.extend(crate::net::bytes::name24("Smith"));
    assert_eq!(character_name(&packet), Some((12345, Some("Smith".to_owned()))));
    packet[2] = 4;
    assert_eq!(character_name(&packet), Some((12345, None)));
    packet[2] = 0;
    assert_eq!(character_name(&packet), None);
}

#[test]
fn a_line_takes_what_it_asks_for() {
    assert_eq!(filled("The [%s] item will disappear in %d minutes.", &["Apple", "5"]), "The [Apple] item will disappear in 5 minutes.");
    assert_eq!(filled("'%lld' Base Experience", &["-12"]), "'-12' Base Experience");
    assert_eq!(filled("100%% of %s", &["it"]), "100% of it");
    assert_eq!(filled("%s and %s", &["one"]), "one and ");
    assert_eq!(filled("50% off", &[]), "50% off");
    assert_eq!(filled("in %02d hour(s) and %02d minute(s)", &["3", "15"]), "in 03 hour(s) and 15 minute(s)");
    assert_eq!(filled("[%3d]", &["7"]), "[  7]");
}
