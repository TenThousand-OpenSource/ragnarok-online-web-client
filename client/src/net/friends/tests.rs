//! Tests of the friends list's packets: each is built as rAthena writes it and read back.

use super::*;
use crate::net::bytes::name24;

fn named(name: &str) -> [u8; 24] {
    name24(name)
}

#[test]
fn reads_the_list() {
    let mut packet = vec![0x01, 0x02, 68, 0];
    for (account, character, name) in [(2_000_001u32, 150_001u32, "One"), (2_000_002, 150_002, "Two Words")] {
        packet.extend(account.to_le_bytes());
        packet.extend(character.to_le_bytes());
        packet.extend(named(name));
    }
    let friends = list(&packet);
    assert_eq!(friends.len(), 2);
    assert_eq!(friends[1], Friend { account: 2_000_002, character: 150_002, name: "Two Words".to_owned(), offline: true });
    // A list with nobody in it is its four bytes.
    assert!(list(&[0x01, 0x02, 4, 0]).is_empty());
}

#[test]
fn reads_who_came_and_went() {
    let mut packet = vec![0x06, 0x02];
    packet.extend(2_000_001u32.to_le_bytes());
    packet.extend(150_001u32.to_le_bytes());
    packet.push(0);
    packet.extend(named("One"));
    assert_eq!(state(&packet), Some(Friend { account: 2_000_001, character: 150_001, name: "One".to_owned(), offline: false }));
    packet[10] = 1;
    assert!(state(&packet).is_some_and(|friend| friend.offline));
    assert_eq!(state(&packet[..34]), None);
}

#[test]
fn reads_asking_and_its_answer() {
    let mut asks = vec![0x07, 0x02];
    asks.extend(7u32.to_le_bytes());
    asks.extend(9u32.to_le_bytes());
    asks.extend(named("Asker"));
    assert_eq!(asked(&asks), Some((7, 9, "Asker".to_owned())));

    let mut answer = vec![0x09, 0x02, 0, 0];
    answer.extend(7u32.to_le_bytes());
    answer.extend(9u32.to_le_bytes());
    answer.extend(named("Asker"));
    assert_eq!(answered(&answer), Some((added::DONE, Friend { account: 7, character: 9, name: "Asker".to_owned(), offline: false })));
    answer[2] = 3;
    assert_eq!(answered(&answer).map(|(result, _)| result), Some(added::THEIR_LIST_FULL));

    let mut gone = vec![0x0a, 0x02];
    gone.extend(7u32.to_le_bytes());
    gone.extend(9u32.to_le_bytes());
    assert_eq!(removed(&gone), Some((7, 9)));
}

#[test]
fn writes_what_the_client_sends() {
    let packet = add("Somebody");
    assert_eq!((packet.len(), &packet[..2], &packet[2..10], packet[10]), (26, &[0x02, 0x02][..], &b"Somebody"[..], 0));
    // A name too long keeps its ending nought.
    assert_eq!(add(&"x".repeat(40))[25], 0);
    assert_eq!(remove(7, 9), vec![0x03, 0x02, 7, 0, 0, 0, 9, 0, 0, 0]);
    assert_eq!(answer(7, 9, true), vec![0x08, 0x02, 7, 0, 0, 0, 9, 0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(answer(7, 9, false)[10], 0);
}
