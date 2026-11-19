//! The packets of a pet, read and written. (The rules they carry are tested in `crate::pets`.)

use super::*;
use crate::net::bytes::testing::assert_lengths;

#[test]
fn the_eggs_and_the_bait_go_as_the_server_expects() {
    assert_eq!(eggs(&[0xa6, 0x01, 8, 0, 3, 0, 9, 0]), vec![3, 9]);
    assert_eq!(hatch(9), vec![0xa7, 0x01, 9, 0]);
    assert_eq!(capture(0x0102_0304), vec![0x9f, 0x01, 4, 3, 2, 1]);
    assert_lengths(&[hatch(9), capture(1)]);
}

#[test]
fn reads_a_pet() {
    let mut packet = vec![0u8; 37];
    packet[..2].copy_from_slice(&ZC_PROPERTY_PET.to_le_bytes());
    packet[2..8].copy_from_slice(b"Poring");
    packet[26] = 1;
    packet[27..29].copy_from_slice(&3u16.to_le_bytes());
    packet[29..31].copy_from_slice(&70u16.to_le_bytes());
    packet[31..33].copy_from_slice(&1000u16.to_le_bytes());
    packet[33..35].copy_from_slice(&10013u16.to_le_bytes());
    packet[35..37].copy_from_slice(&1002u16.to_le_bytes());
    assert_eq!(property(&packet), Some(Pet { name: "Poring".into(), renamed: true, level: 3, hunger: 70, intimacy: 1000, accessory: 10013, job: 1002 }));

    let mut change = vec![0xa4, 0x01, change::HUNGER];
    change.extend(110_000u32.to_le_bytes());
    change.extend(55i32.to_le_bytes());
    assert_eq!(changed(&change), Some((change::HUNGER, 110_000, 55)));
    assert_eq!(fed(&[0xa3, 0x01, 1, 0x13, 0x02, 0, 0]), Some((true, 531)));
    assert_eq!((caught(&[0xa0, 0x01, 1]), caught(&[0xa0, 0x01, 0]), caught(&[0xa0, 0x01])), (Some(true), Some(false), None));
}

#[test]
fn writes_the_commands() {
    assert_eq!(command(order::FEED), [0xa1, 0x01, 1]);
    let named = rename("Bob");
    assert_eq!((named.len(), &named[..6], named[25]), (26, &[0xa5, 0x01, b'B', b'o', b'b', 0][..], 0));
}

#[test]
fn feelings_and_words() {
    let packet = |number: i32| [&[0xaa, 0x01, 7, 0, 0, 0][..], &number.to_le_bytes()].concat();
    assert_eq!(acted(&packet(322)), Some((7, Act::Emotion(32))));
    assert_eq!(acted(&packet(1_002_031)), Some((7, Act::Speech { job: 1002, moment: 0, hunger: 3 })));
    assert_eq!(act(322), [0xa9, 0x01, 0x42, 0x01, 0, 0]);
}
