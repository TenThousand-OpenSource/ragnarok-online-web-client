use super::*;

/// A row of the mailbox as the server writes it.
fn row(tab: u8, id: i64, read: bool, has: u8, sender: &str, expires_in: u32, title: &str) -> Vec<u8> {
    let mut row = vec![tab];
    row.extend_from_slice(&id.to_le_bytes());
    row.extend_from_slice(&[u8::from(read), has]);
    let mut name = [0u8; 24];
    name[..sender.len()].copy_from_slice(sender.as_bytes());
    row.extend_from_slice(&name);
    row.extend_from_slice(&expires_in.to_le_bytes());
    row.extend_from_slice(&(title.len() as u16 + 1).to_le_bytes());
    row.extend_from_slice(title.as_bytes());
    row.push(0);
    row
}

fn listed(tab: u8, id: i64, read: bool, sender: &str, title: &str) -> Listed {
    Listed { tab, id, read, has: 0, sender: sender.to_owned(), expires_in: 600, title: title.to_owned() }
}

#[test]
fn reads_the_mailbox() {
    let mut packet = vec![0xc2, 0x0a, 0, 0, 1];
    packet.extend(row(tab::NORMAL, 7, false, has::ZENY | has::ITEMS, "Alice", 86400, "Hello"));
    packet.extend(row(tab::RETURNED, 0x1_0000_0002, true, has::NPC, "Kafra", 60, ""));
    let length = packet.len() as u16;
    packet[2..4].copy_from_slice(&length.to_le_bytes());
    let (whole, letters) = list(&packet).unwrap();
    assert!(whole);
    assert_eq!(
        letters,
        [
            Listed { tab: 0, id: 7, read: false, has: 6, sender: "Alice".to_owned(), expires_in: 86400, title: "Hello".to_owned() },
            Listed { tab: 2, id: 0x1_0000_0002, read: true, has: 8, sender: "Kafra".to_owned(), expires_in: 60, title: String::new() },
        ]
    );
    // A list cut short gives the letters that are whole in it; an empty one is no letters.
    assert_eq!(list(&packet[..packet.len() - 3]).map(|(_, letters)| letters.len()), Some(1));
    assert_eq!(list(&[0xc2, 0x0a, 5, 0, 0]), Some((false, Vec::new())));
}

#[test]
fn reads_a_letter_with_what_is_in_it() {
    let body = b"Take this.\0";
    let mut packet = vec![0x63, 0x0b, 0, 0, tab::NORMAL];
    packet.extend_from_slice(&9i64.to_le_bytes());
    packet.extend_from_slice(&(body.len() as u16).to_le_bytes());
    packet.extend_from_slice(&1500i64.to_le_bytes());
    packet.push(1);
    packet.extend_from_slice(body);
    let mut thing = vec![0u8; 60];
    thing[..2].copy_from_slice(&3u16.to_le_bytes());
    thing[2..6].copy_from_slice(&1201u32.to_le_bytes());
    thing[6] = 1;
    thing[8..12].copy_from_slice(&4001u32.to_le_bytes());
    thing[24..28].copy_from_slice(&2u32.to_le_bytes());
    thing[28] = 5;
    thing[58] = 7;
    packet.extend(thing);
    let letter = read(&packet).unwrap();
    assert_eq!((letter.tab, letter.id, letter.body.as_str(), letter.zeny), (0, 9, "Take this.", 1500));
    assert_eq!(letter.items, [Item { index: 0, id: 1201, kind: 5, amount: 3, wearable: 2, worn: 0, refine: 7, identified: true, cards: [4001, 0, 0, 0] }]);
    assert_eq!(read(&packet[..20]), None);
}

#[test]
fn reads_the_short_answers() {
    assert_eq!(unread(&[0xe7, 0x09, 1]), Some(true));
    assert_eq!(unread(&[0xe7, 0x09, 0]), Some(false));
    let mut answer = vec![0xf2, 0x09];
    answer.extend_from_slice(&12i64.to_le_bytes());
    answer.extend_from_slice(&[tab::ACCOUNT, 2]);
    assert_eq!(taken(&answer), Some((1, 12, 2)));
    assert_eq!(deleted(&about(ZC_ACK_DELETE_MAIL, tab::RETURNED, 5)), Some((2, 5)));
    let mut opened = named(ZC_ACK_OPEN_WRITE_MAIL, "Bob", 1);
    opened[26] = 1;
    assert_eq!(write_opened(&opened), Some(("Bob".to_owned(), true)));
    assert_eq!(remove_answered(&[0x07, 0x0a, 1, 4, 0, 2, 0, 30, 0]), Some((true, 4, 2, 30)));
}

#[test]
fn reads_who_the_receiver_is() {
    let mut packet = vec![0x51, 0x0a];
    packet.extend_from_slice(&150001u32.to_le_bytes());
    packet.extend_from_slice(&4008u16.to_le_bytes());
    packet.extend_from_slice(&99u16.to_le_bytes());
    let mut name = [0u8; 24];
    name[..3].copy_from_slice(b"Bob");
    packet.extend_from_slice(&name);
    assert_eq!(receiver(&packet), Some(Receiver { character: 150001, job: 4008, level: 99, name: "Bob".to_owned() }));
}

#[test]
fn reads_a_thing_put_into_the_letter() {
    let mut packet = vec![0u8; 64];
    packet[..2].copy_from_slice(&ZC_ACK_ADD_ITEM_RODEX.to_le_bytes());
    packet[3..5].copy_from_slice(&6u16.to_le_bytes());
    packet[5..7].copy_from_slice(&20u16.to_le_bytes());
    packet[7..11].copy_from_slice(&501u32.to_le_bytes());
    packet[12] = 1;
    packet[55..57].copy_from_slice(&140u16.to_le_bytes());
    let (result, item, weight) = add_answered(&packet).unwrap();
    assert_eq!((result, weight), (added::DONE, 140));
    assert_eq!(item, Item { index: 6, id: 501, kind: 0, amount: 20, wearable: 0, worn: 0, refine: 0, identified: true, cards: [0; 4] });
    assert_eq!(add_answered(&packet[..63]), None);
}

#[test]
fn writes_what_the_mailbox_asks() {
    let opened = open(false, [3, 0, 0x0102]);
    assert_eq!((opened.len(), &opened[..2], opened[2], opened[18], opened[19]), (26, &[0xc0, 0x0a][..], 3, 2, 1));
    assert_eq!(open(true, [0; 3])[..2], [0xc1, 0x0a]);
    assert_eq!(close(), [0xe9, 0x09]);
    assert_eq!(ask(tab::ACCOUNT, 5), [0xea, 0x09, 1, 5, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(delete(tab::NORMAL, 5), [0xf5, 0x09, 0, 5, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(take(false, tab::RETURNED, 5), [0xf1, 0x09, 5, 0, 0, 0, 0, 0, 0, 0, 2]);
    assert_eq!(take(true, tab::NORMAL, 5)[..2], [0xf3, 0x09]);
    assert_eq!(give_back(5), [0x98, 0x0b, 5, 0, 0, 0]);
    assert_eq!(cancel_write(), [0x03, 0x0a]);
    assert_eq!(attach(4, 10), [0x04, 0x0a, 4, 0, 10, 0]);
    assert_eq!(detach(4, 10), [0x06, 0x0a, 4, 0, 10, 0]);
}

#[test]
fn writes_the_names_with_a_zero_at_their_end() {
    let packet = write("Bob");
    assert_eq!((packet.len(), &packet[..6]), (26, &[0x08, 0x0a, b'B', b'o', b'b', 0][..]));
    let long = write("a name of far too many letters");
    assert_eq!((long.len(), &long[2..25], long[25]), (26, &b"a name of far too many "[..], 0));
    let checked = check_receiver("Bob", true);
    assert_eq!((checked.len(), &checked[..2], checked[26]), (27, &[0x97, 0x0b][..], 1));
}

#[test]
fn writes_a_letter() {
    let to = Receiver { character: 150001, job: 0, level: 1, name: "Bob".to_owned() };
    let packet = send(&to, "Alice", 300, "Hi there", "Yo");
    assert_eq!(packet.len(), 68 + 9 + 3);
    assert_eq!(u16::from_le_bytes([packet[2], packet[3]]), 80);
    assert_eq!(&packet[4..8], b"Bob\0");
    assert_eq!(&packet[28..34], b"Alice\0");
    assert_eq!(i64::from_le_bytes(packet[52..60].try_into().unwrap()), 300);
    assert_eq!((u16::from_le_bytes([packet[60], packet[61]]), u16::from_le_bytes([packet[62], packet[63]])), (9, 3));
    assert_eq!(u32::from_le_bytes(packet[64..68].try_into().unwrap()), 150001);
    assert_eq!(&packet[68..], b"Hi there\0Yo\0");
}

#[test]
fn every_packet_of_the_mail_has_its_length() {
    for id in [ZC_NOTIFY_UNREADMAIL, ZC_ACK_WRITE_MAIL, ZC_ACK_ZENY_FROM_MAIL, ZC_ACK_ITEM_FROM_MAIL, ZC_ACK_DELETE_MAIL, ZC_ACK_REMOVE_ITEM_MAIL, ZC_ACK_OPEN_WRITE_MAIL, ZC_CHECKNAME, ZC_MAIL_LIST, ZC_ACK_ADD_ITEM_RODEX, ZC_ACK_READ_RODEX] {
        assert!(crate::net::packet_length(id).is_some(), "{id:#06x} has no length");
    }
    assert_eq!(crate::net::packet_length(ZC_ACK_ADD_ITEM_RODEX), Some(64));
    assert_eq!(crate::net::packet_length(ZC_CHECKNAME), Some(34));
}

#[test]
fn keeps_a_letter_it_has_as_it_is() {
    let mut store = Store::default();
    assert!(store.add(listed(0, 1, false, "Alice", "First"), 100.0, true));
    assert!(!store.add(listed(0, 2, true, "Bob", "Second"), 100.0, false));
    // The same letter listed again, now read, changes nothing and is not counted again.
    assert!(!store.add(listed(0, 1, true, "Alice", "First"), 200.0, true));
    assert_eq!(store.get(0, 1).map(|letter| (letter.read, letter.expires)), Some((false, 700.0)));
    assert_eq!(store.get(1, 1), None);
    // A kind the mailbox has not is not taken.
    assert!(!store.add(listed(3, 9, false, "", ""), 100.0, true));
}

#[test]
fn lists_what_is_not_read_first_and_then_the_newer() {
    let mut store = Store::default();
    for (id, read) in [(1, true), (2, false), (3, true), (4, false)] {
        store.add(listed(0, id, read, "Alice", "Hello"), 0.0, true);
    }
    assert_eq!(store.listed(0, 1.0), [(0, 4), (0, 2), (0, 3), (0, 1)]);
    // One whose time has run out is listed no more.
    assert_eq!(store.listed(0, 600.0), []);
    assert_eq!(store.last(0), 1);
    assert_eq!(store.last(2), 0);
    store.get_mut(0, 3).unwrap().gone = true;
    store.purge();
    assert_eq!(store.listed(0, 1.0), [(0, 4), (0, 2), (0, 1)]);
}

#[test]
fn searches_every_kind_by_title_or_sender() {
    let mut store = Store::default();
    store.add(listed(0, 1, true, "Alice", "About the guild"), 0.0, true);
    store.add(listed(1, 2, false, "Notice", "Guild war"), 0.0, true);
    store.add(listed(2, 3, true, "Alice", "Returned"), 0.0, true);
    assert_eq!(store.search(false, "uild", 1.0), [(1, 2), (0, 1)]);
    assert_eq!(store.search(true, "Alice", 1.0), [(2, 3), (0, 1)]);
    assert_eq!(store.search(true, "alice", 1.0), []);
}

#[test]
fn colours_a_title_by_who_sent_it_and_whether_it_was_read() {
    let letter = |known, read, gone| Letter { known, read, gone, ..Letter::default() };
    assert_eq!(title_colour(tab::NORMAL, &letter(false, false, false)), "#ff0000");
    assert_eq!(title_colour(tab::NORMAL, &letter(false, true, false)), "#e6c8c8");
    assert_eq!(title_colour(tab::NORMAL, &letter(true, false, false)), "#000000");
    assert_eq!(title_colour(tab::NORMAL, &letter(true, true, false)), "#c8c8c8");
    assert_eq!(title_colour(tab::ACCOUNT, &letter(false, false, false)), "#000000");
    assert_eq!(title_colour(tab::NORMAL, &letter(false, false, true)), "#c8c8c8");
}

#[test]
fn says_how_long_a_letter_still_lies_there() {
    assert_eq!(time_left(3 * 86400 + 5), (2852, "%d day", 3));
    assert_eq!(time_left(86399), (2851, "%d hour", 23));
    assert_eq!(time_left(3599), (2850, "%d minute", 59));
    assert_eq!(time_left(30), (2850, "%d minute", 0));
}

#[test]
fn counts_the_fee_and_reads_the_zeny_typed() {
    assert_eq!(fee(0, 0), 0);
    assert_eq!(fee(2, 1000), 5020);
    assert_eq!(fee(0, 49), 0);
    assert_eq!(zeny_typed("1,234,567"), 1234567);
    assert_eq!(zeny_typed(" 50 "), 50);
    assert_eq!(zeny_typed("12abc"), 12);
    assert_eq!(zeny_typed("abc"), 0);
    assert_eq!(zeny_typed(""), 0);
}
