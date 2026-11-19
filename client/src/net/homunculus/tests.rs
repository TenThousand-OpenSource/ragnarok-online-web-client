//! The packets of a homunculus, read and written. (Its rules are tested in `crate::homunculus`.)

use super::*;
use crate::net::bytes::name24;

/// A ZC_PROPERTY_HOMUN of the newest layout, field by field.
fn newest() -> Vec<u8> {
    let mut packet = vec![0xa4, 0x0b];
    packet.extend_from_slice(&name24("Lif"));
    packet.push(0b101);
    for word in [12u16, 55, 430] {
        packet.extend_from_slice(&word.to_le_bytes());
    }
    for stat in [31u16, 22, 40, 2, 15, 9, 33, 1400] {
        packet.extend_from_slice(&stat.to_le_bytes());
    }
    for long in [320u32, 410, 60, 75] {
        packet.extend_from_slice(&long.to_le_bytes());
    }
    packet.extend_from_slice(&5_000_000_000i64.to_le_bytes());
    packet.extend_from_slice(&9_000_000_000i64.to_le_bytes());
    packet.extend_from_slice(&3u16.to_le_bytes());
    packet.extend_from_slice(&1u16.to_le_bytes());
    packet
}

#[test]
fn reads_everything_about_a_homunculus() {
    let packet = newest();
    assert_eq!(packet.len(), 85);
    let read = property(ZC_PROPERTY_HOMUN_4, &packet).unwrap();
    assert_eq!((read.name.as_str(), read.flags, read.level, read.hunger, read.intimacy), ("Lif", 5, 12, 55, 430));
    assert_eq!(read.stats, [31, 22, 40, 2, 15, 9, 33, 1400]);
    assert_eq!((read.hp, read.max_hp, read.sp, read.max_sp), (320, 410, 60, 75));
    assert_eq!((read.exp, read.next_exp, read.skill_points, read.range), (5_000_000_000, 9_000_000_000, 3, 1));
    assert_eq!(property(ZC_PROPERTY_HOMUN_4, &packet[..84]), None);
}

#[test]
fn reads_the_older_layouts_too() {
    // 0x022e: an accessory after the intimacy, and everything after the stats in 16 and 32 bits.
    let mut packet = vec![0x2e, 0x02];
    packet.extend_from_slice(&name24("Amistr"));
    packet.push(1);
    for word in [3u16, 20, 900, 0, 1, 2, 3, 4, 5, 6, 7, 8, 100, 200, 30, 40] {
        packet.extend_from_slice(&word.to_le_bytes());
    }
    packet.extend_from_slice(&7u32.to_le_bytes());
    packet.extend_from_slice(&50u32.to_le_bytes());
    packet.extend_from_slice(&[2, 0, 1, 0]);
    assert_eq!(packet.len(), 71);
    let read = property(ZC_PROPERTY_HOMUN, &packet).unwrap();
    assert_eq!(
        (read.level, read.intimacy, read.stats, read.hp, read.max_sp, read.exp, read.next_exp, read.skill_points, read.range),
        (3, 900, [1, 2, 3, 4, 5, 6, 7, 8], 100, 40, 7, 50, 2, 1)
    );
    // 0x09f7: the same with the life in 32 bits.
    let mut packet = packet[..51].to_vec();
    packet[..2].copy_from_slice(&[0xf7, 0x09]);
    packet.extend_from_slice(&70_000u32.to_le_bytes());
    packet.extend_from_slice(&80_000u32.to_le_bytes());
    packet.extend_from_slice(&[30, 0, 40, 0, 7, 0, 0, 0, 50, 0, 0, 0, 2, 0, 1, 0]);
    assert_eq!(packet.len(), 75);
    let read = property(ZC_PROPERTY_HOMUN_2, &packet).unwrap();
    assert_eq!((read.hp, read.max_hp, read.sp, read.max_sp, read.next_exp, read.range), (70_000, 80_000, 30, 40, 50, 1));
}

#[test]
fn reads_what_changed() {
    assert_eq!(changed(&[0x30, 0x02, 0, 1, 0x10, 0x27, 0, 0, 0xb0, 0x01, 0, 0]), Some((change::INTIMACY, 10000, 432)));
    assert_eq!(changed(&[0x30, 0x02, 0, 0]), None);
    assert_eq!(parameter_changed(ZC_HO_PAR_CHANGE2, &[0xa5, 0x0b, 5, 0, 0x2c, 1, 0, 0, 0, 0, 0, 0]), Some((parameter::HP, 300)));
    assert_eq!(parameter_changed(ZC_HO_PAR_CHANGE, &[0xdb, 0x07, 7, 0, 40, 0, 0, 0]), Some((parameter::SP, 40)));
    assert_eq!(fed(&[0x2f, 0x02, 1, 0x19, 0x02, 0, 0]), Some((true, 537)));
}

#[test]
fn asks_for_it_as_the_client_does() {
    assert_eq!(command(order::FEED), [0x2d, 0x02, 0, 0, 1]);
    assert_eq!(rename("Mimi").len(), 26);
    // The cell packed as rAthena's `RBUFPOS` unpacks it: x in the first ten bits, y in the next ten.
    let packet = move_to(0x01020304, (155, 187));
    assert_eq!(&packet[..6], &[0x32, 0x02, 4, 3, 2, 1]);
    let (a, b, c) = (u16::from(packet[6]), u16::from(packet[7]), u16::from(packet[8]));
    assert_eq!(((a << 2) | (b >> 6), ((b & 0x3f) << 4) | (c >> 4)), (155, 187));
    assert_eq!(attack(7, 9), [0x33, 0x02, 7, 0, 0, 0, 9, 0, 0, 0, 0]);
    assert_eq!(to_owner(7), [0x34, 0x02, 7, 0, 0, 0]);
}

#[test]
fn words_it_as_its_window_does() {
    assert!(is_skill(8001) && is_skill(8060) && !is_skill(8061) && !is_skill(7999));
}
