//! Tests of looking at what another wears: the answer read, the switches, what the client sends.

use super::*;

/// A thing worn as the server lists it: 68 bytes.
fn worn(index: u16, id: u32, place: u32, refine: u8) -> Vec<u8> {
    let mut item = vec![0u8; 68];
    item[..2].copy_from_slice(&index.to_le_bytes());
    item[2..6].copy_from_slice(&id.to_le_bytes());
    item[6] = 5;
    item[7..11].copy_from_slice(&place.to_le_bytes());
    item[11..15].copy_from_slice(&place.to_le_bytes());
    item[15..19].copy_from_slice(&4001u32.to_le_bytes());
    item[65] = refine;
    item[67] = 1;
    item
}

#[test]
fn reads_what_another_wears() {
    let mut packet = vec![0x37, 0x0b, 0, 0];
    let mut name = [0u8; 24];
    name[..5].copy_from_slice(b"Other");
    packet.extend(name);
    for value in [4008u16, 7, 11, 22, 33, 44, 3, 2, 0] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(1);
    assert_eq!(packet.len(), 47);
    packet.extend(worn(3, 1101, 2, 7));
    packet.extend(worn(9, 2301, 16, 0));
    let length = packet.len() as u16;
    packet[2..4].copy_from_slice(&length.to_le_bytes());

    let seen = seen(&packet).unwrap();
    assert_eq!((seen.name.as_str(), seen.job, seen.sex), ("Other", 4008, 1));
    assert_eq!((seen.hair_style, seen.hair_colour, seen.body_palette, seen.robe), (7, 3, 2, 44));
    // Bottom, top, middle.
    assert_eq!(seen.headgear, [11, 33, 22]);
    assert_eq!(seen.items.len(), 2);
    assert_eq!((seen.items[0].id, seen.items[0].worn, seen.items[0].refine, seen.items[0].cards[0], seen.items[0].identified), (1101, 2, 7, 4001, true));
    assert_eq!(seen.items[1].worn, 16);
    // One who wears nothing is the 47 bytes alone; less is no answer.
    assert!(super::seen(&packet[..47]).is_some_and(|seen| seen.items.is_empty()));
    assert_eq!(super::seen(&packet[..46]), None);
}

#[test]
fn reads_the_settings() {
    assert_eq!(config(&[0xd9, 0x02, 0, 0, 0, 0, 1, 0, 0, 0]), Some((OPEN_EQUIPMENT, true)));
    assert_eq!(config(&[0xd9, 0x02, 2, 0, 0, 0, 0, 0, 0, 0]), Some((2, false)));
    assert_eq!(config(&[0xd9, 0x02, 0]), None);
    assert_eq!(open(&[0xda, 0x02, 1]), Some(true));
    assert_eq!(open(&[0xda, 0x02, 0]), Some(false));
    assert_eq!(said(&[0x91, 0x02, 0x4d, 0x05]), Some(1357));
}

#[test]
fn writes_what_the_client_sends() {
    assert_eq!(ask(2_000_001), vec![0xd6, 0x02, 0x81, 0x84, 0x1e, 0x00]);
    assert_eq!(set(OPEN_EQUIPMENT, true), vec![0xd8, 0x02, 0, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(set(OPEN_EQUIPMENT, false)[6], 0);
}
