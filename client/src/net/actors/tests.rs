//! The tests of `net/actors.rs`.

use super::*;
use crate::net::spirits::{told as spirits, ZC_SPIRITS, ZC_SPIRITS2};

#[test]
fn stopping_the_fight_is_the_number_alone() {
    assert_eq!(stop_attack(), vec![0x18, 0x01]);
    assert_eq!(crate::net::packet_length(0x0118), Some(2));
}

#[test]
fn a_thing_on_the_ground_says_how_many_lie_there() {
    // ZC_ITEM_FALL_ENTRY: the count after the place within the cell.
    let mut falls = vec![0u8; 24];
    falls[2..6].copy_from_slice(&77u32.to_le_bytes());
    falls[6..10].copy_from_slice(&512u32.to_le_bytes());
    falls[12] = 1;
    falls[13..15].copy_from_slice(&150u16.to_le_bytes());
    falls[15..17].copy_from_slice(&160u16.to_le_bytes());
    (falls[17], falls[18]) = (3, 9);
    falls[19..21].copy_from_slice(&11u16.to_le_bytes());
    let fell = Dropped { id: 77, item: 512, cell: (150, 160), within: (3, 9), identified: true, count: 11 };
    assert_eq!(dropped(ZC_ITEM_FALL_ENTRY, &falls), Some(fell));
    // ZC_ITEM_ENTRY: the count before it.
    let mut lies = vec![0u8; 19];
    lies[2..6].copy_from_slice(&78u32.to_le_bytes());
    lies[6..10].copy_from_slice(&909u32.to_le_bytes());
    lies[11..13].copy_from_slice(&150u16.to_le_bytes());
    lies[13..15].copy_from_slice(&160u16.to_le_bytes());
    lies[15..17].copy_from_slice(&2u16.to_le_bytes());
    (lies[17], lies[18]) = (5, 6);
    let lay = Dropped { id: 78, item: 909, cell: (150, 160), within: (5, 6), identified: false, count: 2 };
    assert_eq!(dropped(ZC_ITEM_ENTRY, &lies), Some(lay));
}

/// A packet of `length` bytes with the fields the three kinds share at their fixed places.
fn base(id: u16, length: usize, object: u8) -> Vec<u8> {
    let mut packet = vec![0u8; length];
    packet[..2].copy_from_slice(&id.to_le_bytes());
    packet[2..4].copy_from_slice(&(length as u16).to_le_bytes());
    packet[4] = object;
    packet[5..9].copy_from_slice(&110_000_001u32.to_le_bytes());
    packet[13..15].copy_from_slice(&200u16.to_le_bytes());
    packet[15..17].copy_from_slice(&2u16.to_le_bytes());
    packet[17..19].copy_from_slice(&0x81u16.to_le_bytes());
    packet[23..25].copy_from_slice(&1002u16.to_le_bytes());
    packet[25..27].copy_from_slice(&7u16.to_le_bytes());
    packet[35..37].copy_from_slice(&5u16.to_le_bytes());
    packet
}

fn pack(x: u16, y: u16, direction: u8) -> [u8; 3] {
    [(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, ((y << 4) as u8) | direction]
}

#[test]
fn reads_a_standing_unit() {
    let mut packet = base(ZC_NOTIFY_STANDENTRY, 108, 5);
    packet[37..39].copy_from_slice(&224u16.to_le_bytes());
    packet[39..41].copy_from_slice(&12u16.to_le_bytes());
    packet[41..43].copy_from_slice(&3u16.to_le_bytes());
    packet[43..45].copy_from_slice(&2u16.to_le_bytes());
    packet[57..61].copy_from_slice(&0x18u32.to_le_bytes());
    packet[62] = 1;
    packet[63..66].copy_from_slice(&pack(150, 180, 4));
    packet[69..71].copy_from_slice(&99u16.to_le_bytes());
    packet[84..90].copy_from_slice(b"Poring");

    let unit = unit(ZC_NOTIFY_STANDENTRY, &packet).unwrap();
    assert_eq!((unit.id, unit.kind, unit.job, unit.speed), (110_000_001, Kind::Monster, 1002, 200));
    assert_eq!((unit.hair_style, unit.hair_colour, unit.body_palette, unit.headgear, unit.sex), (7, 3, 2, [5, 224, 12], 1));
    assert_eq!((unit.cell, unit.direction, unit.walks), ((150, 180), 4, None));
    assert_eq!((unit.level, unit.name.as_str()), (99, "Poring"));
    // `bodyState` and `healthState` stand between the speed and the effect state in all three packets.
    assert_eq!((unit.body, unit.health), (2, 0x81));
    // The third option stands after the guild's emblem and the honour.
    assert_eq!(unit.third, 0x18);
    assert_eq!((unit.weapon, unit.shield, unit.head, unit.posture), (0, 0, 0, Posture::Stands));
}

#[test]
fn reads_what_a_player_holds_and_how_it_is_there() {
    let mut packet = base(ZC_NOTIFY_STANDENTRY, 108, 0);
    packet[27..31].copy_from_slice(&1201u32.to_le_bytes());
    packet[31..35].copy_from_slice(&2101u32.to_le_bytes());
    packet[45..47].copy_from_slice(&2u16.to_le_bytes());
    packet[68] = 2;
    let seen = unit(ZC_NOTIFY_STANDENTRY, &packet).unwrap();
    assert_eq!((seen.weapon, seen.shield, seen.head, seen.posture), (1201, 2101, 2, Posture::Sits));
    packet[68] = 1;
    assert_eq!(unit(ZC_NOTIFY_STANDENTRY, &packet).unwrap().posture, Posture::Lies);
    // A new unit has no such byte: the one before its level is the height of its cell's box.
    let mut new = base(ZC_NOTIFY_NEWENTRY, 107, 0);
    new[67] = 2;
    assert_eq!(unit(ZC_NOTIFY_NEWENTRY, &new).unwrap().posture, Posture::Stands);
    // A walker has its head's turn four bytes on.
    let mut walker = base(ZC_NOTIFY_MOVEENTRY, 114, 0);
    walker[49..51].copy_from_slice(&1u16.to_le_bytes());
    assert_eq!(unit(ZC_NOTIFY_MOVEENTRY, &walker).unwrap().head, 1);
}

#[test]
fn reads_the_small_packets_about_someone() {
    assert_eq!(turned(&[0x9c, 0, 7, 0, 0, 0, 2, 0, 5]), Some(Turned { who: 7, head: 2, direction: 5 }));
    assert_eq!(turned(&[0x9c, 0, 7, 0, 0, 0, 2, 0]), None);
    assert_eq!(spirits(&[0xd0, 0x01, 7, 0, 0, 0, 5, 0]), Some((7, 5)));
    assert_eq!(risen(&[0x48, 0x01, 9, 0, 0, 0, 0, 0]), Some(9));
    let mut changed = vec![0xd7, 0x01, 9, 0, 0, 0, 2];
    changed.extend(1201u32.to_le_bytes());
    changed.extend(2101u32.to_le_bytes());
    assert_eq!(look(&changed), Some(Look { who: 9, kind: 2, value: 1201, second: 2101 }));
    assert_eq!(turn(1, 3), vec![0x61, 0x03, 1, 0, 3]);
    for (id, length) in [(ZC_CHANGE_DIRECTION, 9), (ZC_FASTMOVE, 10), (ZC_SPIRITS, 8), (ZC_SPIRITS2, 8), (ZC_RESURRECTION, 8), (ZC_SPRITE_CHANGE, 15)] {
        assert_eq!(crate::net::packet_length(id), Some(length));
    }
}

#[test]
fn reads_the_third_option_of_a_walker_and_of_one_whose_states_changed() {
    // A walking unit has the time it set off before its headgear: everything after stands four on.
    let mut packet = base(ZC_NOTIFY_MOVEENTRY, 114, 0);
    packet[61..65].copy_from_slice(&0x1000u32.to_le_bytes());
    assert_eq!(unit(ZC_NOTIFY_MOVEENTRY, &packet).unwrap().third, 0x1000);
    let mut changed = vec![0x8a, 0x02];
    changed.extend([7u32, 0x40, 99, 0x10].iter().flat_map(|field| field.to_le_bytes()));
    assert_eq!(third_option(&changed), Some((7, 0x40, 0x10)));
    assert_eq!(third_option(&changed[..17]), None);
}

/// A new unit is laid out as a standing one, less the byte that says whether it sits or lies.
#[test]
fn reads_a_new_unit() {
    let mut packet = base(ZC_NOTIFY_NEWENTRY, 107, 6);
    packet[63..66].copy_from_slice(&pack(10, 20, 0));
    packet[68..70].copy_from_slice(&1u16.to_le_bytes());
    packet[83..89].copy_from_slice(b"Kafra\0");
    let unit = unit(ZC_NOTIFY_NEWENTRY, &packet).unwrap();
    assert_eq!((unit.kind, unit.cell, unit.level, unit.name.as_str()), (Kind::Npc, (10, 20), 1, "Kafra"));
}

#[test]
fn reads_a_walking_unit() {
    let mut packet = base(ZC_NOTIFY_MOVEENTRY, 114, 0);
    packet[41..43].copy_from_slice(&224u16.to_le_bytes());
    packet[66] = 0;
    // From (150, 180) to (155, 1000), as the server packs two cells into six bytes.
    let ((x0, y0), (x1, y1)) = ((150u16, 180u16), (155u16, 1000u16));
    packet[67] = (x0 >> 2) as u8;
    packet[68] = ((x0 << 6) | ((y0 >> 4) & 0x3f)) as u8;
    packet[69] = ((y0 << 4) | ((x1 >> 6) & 0x0f)) as u8;
    packet[70] = ((x1 << 2) | ((y1 >> 8) & 0x03)) as u8;
    packet[71] = y1 as u8;
    packet[72] = 0x5a;
    packet[37..41].copy_from_slice(&123_456u32.to_le_bytes());
    packet[90..96].copy_from_slice(b"Walker");
    let unit = unit(ZC_NOTIFY_MOVEENTRY, &packet).unwrap();
    let walk = Walked { from: (150, 180), to: (155, 1000), within: (5, 10), started: 123_456 };
    assert_eq!((unit.kind, unit.cell, unit.walks), (Kind::Player, (150, 180), Some(walk)));
    // Someone in sight who sets out: the ID, the same six bytes, then the time.
    let mut moving = vec![0x86, 0];
    moving.extend(7u32.to_le_bytes());
    moving.extend_from_slice(&packet[67..73]);
    moving.extend(123_456u32.to_le_bytes());
    assert_eq!(moved(&moving), Some((7, walk)));
    assert_eq!(moved(&moving[..15]), None);
    assert_eq!(slid(&[0xff, 0x01, 7, 0, 0, 0, 150, 0, 180, 0]), Some((7, (150, 180))));
    assert_eq!((unit.headgear, unit.name.as_str()), ([5, 224, 0], "Walker"));
}

#[test]
fn refuses_what_is_not_a_unit() {
    assert_eq!(unit(0x1234, &[0u8; 120]), None);
    assert_eq!(unit(ZC_NOTIFY_STANDENTRY, &[0u8; 40]), None);
    assert_eq!(vanished(&[0x80, 0, 1, 0, 0, 0, 1]), Some((1, true)));
    assert_eq!(vanished(&[0x80, 0, 1, 0, 0, 0, 0]), Some((1, false)));
}

/// The tests that stood in `net/actors.rs` itself: a name told, a blow dealt.
mod act_tests {
    use super::*;

    #[test]
    fn a_monsters_name_with_its_title_and_group() {
        let mut packet = vec![0xdf, 0x0a];
        packet.extend(9u32.to_le_bytes());
        packet.extend(3u32.to_le_bytes());
        for words in ["Poring", "Angry"] {
            let mut field = words.as_bytes().to_vec();
            field.resize(24, 0);
            packet.extend(field);
        }
        assert_eq!(named(&packet), Some(Named { who: 9, group: 3, name: "Poring".to_owned(), title: "Angry".to_owned() }));
        assert_eq!(named(&packet[..57]), None);
    }

    #[test]
    fn a_blow_of_several_hits_with_the_other_hands_damage() {
        let mut packet = vec![0xc8, 0x08];
        for number in [7u32, 9, 1000, 432, 288, 300] {
            packet.extend(number.to_le_bytes());
        }
        packet.push(0);
        packet.extend(3u16.to_le_bytes());
        packet.push(8);
        packet.extend(40i32.to_le_bytes());
        assert_eq!(act(&packet), Some(Act { source: 7, target: 9, speed: 432, delay: 288, damage: 300, kind: 8, hits: 3, extra: 40 }));
        assert_eq!(act(&packet[..33]), None);
    }
}

#[test]
fn reads_a_monster_becoming_another_and_how_one_is_gone() {
    // ZC_NPCSPRITE_CHANGE: a Fabre (1007) becomes a Pupa (1008).
    let mut packet = vec![0xb0, 0x01];
    packet.extend(77u32.to_le_bytes());
    packet.push(0);
    packet.extend(1008u32.to_le_bytes());
    assert_eq!(crate::net::packet_length(ZC_NPCSPRITE_CHANGE), Some(packet.len() as i32));
    assert_eq!((became(&packet), became(&packet[..10])), (Some((77, 1008)), None));
    // ZC_NOTIFY_VANISH: out of sight, and teleported away.
    assert_eq!((vanished_how(&[0x80, 0, 7, 0, 0, 0, 0]), vanished_how(&[0x80, 0, 7, 0, 0, 0, 3])), (Some(0), Some(3)));
    assert_eq!(vanished_how(&[0x80, 0, 7, 0, 0, 0]), None);
}
