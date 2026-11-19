//! Tests of the party's packets: each is built as rAthena writes it and read back, and what the client sends
//! is compared byte by byte.

use super::*;
use crate::net::bytes::testing::field;

#[test]
fn reads_the_list_of_members() {
    let mut packet = vec![0xe5, 0x0a, 0, 0];
    packet.extend(field("Poring Hunters", 24));
    for (account, name, map, role, offline, job, level) in [(2_000_001u32, "Anna", "prontera.gat", 0u8, 0u8, 4u16, 55u16), (2_000_002, "Boris", "", 1, 1, 7, 99)] {
        packet.extend(account.to_le_bytes());
        packet.extend((account + 150_000).to_le_bytes());
        packet.extend(field(name, 24));
        packet.extend(field(map, 16));
        packet.extend([role, offline]);
        packet.extend(job.to_le_bytes());
        packet.extend(level.to_le_bytes());
    }
    let length = packet.len() as u16;
    packet[2..4].copy_from_slice(&length.to_le_bytes());
    let (name, members) = list(&packet).unwrap();
    assert_eq!(name, "Poring Hunters");
    assert_eq!(members.len(), 2);
    assert_eq!(members[0], Member { account: 2_000_001, character: 2_150_001, name: "Anna".into(), map: "prontera.gat".into(), leader: true, offline: false, job: 4, level: 55 });
    assert_eq!((members[1].leader, members[1].offline, members[1].level), (false, true, 99));
    assert_eq!(list(&packet[..20]), None);
}

#[test]
fn reads_a_member_joining() {
    let mut packet = vec![0xe4, 0x0a];
    packet.extend(2_000_002u32.to_le_bytes());
    packet.extend(150_002u32.to_le_bytes());
    packet.extend(1u32.to_le_bytes());
    for value in [7u16, 99, 156, 187] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(0);
    packet.extend(field("Poring Hunters", 24));
    packet.extend(field("Boris", 24));
    packet.extend(field("prontera.gat", 16));
    packet.extend([1, 0]);
    assert_eq!(packet.len(), 89);
    let (member, party, cell, shared) = joined(&packet).unwrap();
    assert_eq!(member, Member { account: 2_000_002, character: 150_002, name: "Boris".into(), map: "prontera.gat".into(), leader: false, offline: false, job: 7, level: 99 });
    assert_eq!((party.as_str(), cell, shared), ("Poring Hunters", (156, 187), (true, false)));
    assert_eq!(joined(&packet[..88]), None);
}

#[test]
fn reads_the_smaller_packets() {
    let mut gone = vec![0x05, 0x01];
    gone.extend(2_000_002u32.to_le_bytes());
    gone.extend(field("Boris", 24));
    gone.push(gone::EXPELLED);
    assert_eq!(left(&gone), Some((2_000_002, "Boris".into(), 1)));

    let mut asked = vec![0xc6, 0x02];
    asked.extend(77u32.to_le_bytes());
    asked.extend(field("Poring Hunters", 24));
    assert_eq!(invited(&asked), Some((77, "Poring Hunters".into())));

    let mut answered = vec![0xc5, 0x02];
    answered.extend(field("Boris", 24));
    answered.extend(invite::ACCEPTED.to_le_bytes());
    assert_eq!(invite_answered(&answered), Some(("Boris".into(), 2)));

    assert_eq!(sharing(&[0xd8, 0x07, 1, 0, 0, 0, 1, 0]), Some(Sharing { experience: 1, pickup: true, items: false }));
    assert_eq!(said(&[&[0x09, 0x01, 18, 0, 5, 0, 0, 0][..], b"Anna : hi\0"].concat()), Some((5, "Anna : hi".into())));
    assert_eq!(place(&[0x07, 0x01, 5, 0, 0, 0, 156, 0, 187, 0]), Some((5, (156, 187))));
    assert_eq!(life(&[0x0e, 0x08, 5, 0, 0, 0, 0x10, 0x27, 0, 0, 0x20, 0x4e, 0, 0]), Some((5, 10_000, 20_000)));
    assert_eq!(dead(&[0xb2, 0x0a, 5, 0, 0, 0, 1]), Some((5, true)));
    assert_eq!(job_level(&[0xbd, 0x0a, 5, 0, 0, 0, 7, 0, 99, 0]), Some((5, 7, 99)));
    assert_eq!(leader(&[0xfc, 0x07, 5, 0, 0, 0, 6, 0, 0, 0]), Some((5, 6)));
}

#[test]
fn reads_the_one_byte_of_the_answers() {
    assert_eq!((made_how(&[0xfa, 0x00, made::NOT_HERE]), made_how(&[0xfa, 0x00])), (Some(3), None));
    assert_eq!((refuses(&[0xc9, 0x02, 1]), refuses(&[0xc9, 0x02, 0]), refuses(&[0xc9, 0x02])), (Some(true), Some(false), None));
}

#[test]
fn writes_what_the_client_sends() {
    let made = make("Poring Hunters", true, false);
    assert_eq!((made.len(), &made[..4], made[26], made[27]), (28, &[0xe8, 0x01, b'P', b'o'][..], 1, 0));
    let invited = invite("Boris");
    assert_eq!((invited.len(), &invited[..8]), (26, &[0xc4, 0x02, b'B', b'o', b'r', b'i', b's', 0][..]));
    // A name longer than the server keeps is cut, and still ends in a zero.
    assert_eq!(invite("abcdefghijklmnopqrstuvwxyz")[25], 0);
    assert_eq!(answer(77, true), [0xc7, 0x02, 77, 0, 0, 0, 1]);
    assert_eq!(leave(), [0x00, 0x01]);
    let expelled = expel(5, "Boris");
    assert_eq!((expelled.len(), &expelled[..7]), (30, &[0x03, 0x01, 5, 0, 0, 0, b'B'][..]));
    assert_eq!(share(Sharing { experience: 1, pickup: false, items: true }), [0xd7, 0x07, 1, 0, 0, 0, 0, 1]);
    assert_eq!(hand_over(6), [0xda, 0x07, 6, 0, 0, 0]);
    assert_eq!(say("Anna", "hi"), [&[0x08, 0x01, 13, 0][..], b"Anna : hi"].concat());
    assert_eq!(refuse_invitations(true), [0xc8, 0x02, 1]);
}
