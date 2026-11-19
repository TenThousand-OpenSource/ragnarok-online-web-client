//! Tests of the guild's packets: each is built as rAthena writes it (`packets_struct.hpp`) and read back.

use super::*;

/// A text in a field of `size` bytes.
fn padded(words: &str, size: usize) -> Vec<u8> {
    let mut field = words.as_bytes().to_vec();
    field.resize(size, 0);
    field
}

/// A packet with a length after its number.
fn sized(id: u16, body: &[u8]) -> Vec<u8> {
    [&id.to_le_bytes()[..], &((body.len() + 4) as u16).to_le_bytes(), body].concat()
}

fn longs(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

#[test]
fn the_guild_one_is_of_is_read() {
    let packet = [&[0xf7, 0x02][..], &longs(&[7, 3, 0x11]), &[1], &longs(&[0]), &padded("Midgard", 24), &longs(&[150_001])].concat();
    assert_eq!(packet.len(), 47);
    let belonging = belonging(&packet).unwrap();
    assert_eq!(belonging, Belonging { guild: 7, emblem: 3, invites: true, expels: true, master: true, name: "Midgard".to_owned(), master_character: 150_001 });
    assert_eq!(super::belonging(&packet[..46]), None);
}

#[test]
fn the_guild_s_numbers_are_read() {
    let packet = [&[0x7b, 0x0b][..], &longs(&[7, 2, 3, 16, 45, 1000, 2000, 0, 0, 0, 3]), &padded("Midgard", 24), &padded("None Taken", 16), &longs(&[0, 150_001]), &padded("Thor", 24)].concat();
    assert_eq!(packet.len(), 118);
    let info = info(&packet).unwrap();
    assert_eq!((info.guild, info.level, info.online, info.most, info.average_level, info.exp, info.next_exp, info.emblem), (7, 2, 3, 16, 45, 1000, 2000, 3));
    assert_eq!((info.name.as_str(), info.land.as_str(), info.master.as_str(), info.master_character), ("Midgard", "None Taken", "Thor", 150_001));
}

#[test]
fn the_members_are_read() {
    let member = |account: u32, name: &str, online: u32, position: u32| {
        [&longs(&[account, account + 100])[..], &[5, 0, 2, 0, 1, 0, 7, 0, 50, 0], &longs(&[1234, online, position, 1_700_000_000]), &padded(name, 24)].concat()
    };
    let packet = sized(ZC_MEMBERMGR_INFO, &[member(2_000_000, "Thor", 1, 0), member(2_000_001, "Loki", 0, 19)].concat());
    let members = members(&packet);
    assert_eq!(members.len(), 2);
    assert_eq!(members[0], Member { account: 2_000_000, character: 2_000_100, hair: 5, hair_colour: 2, sex: 1, job: 7, level: 50, given: 1234, online: true, position: 0, seen: 1_700_000_000, name: "Thor".to_owned() });
    assert_eq!((members[1].name.as_str(), members[1].online, members[1].position), ("Loki", false, 19));
    // A record cut short is not a member.
    assert_eq!(super::members(&packet[..packet.len() - 1]).len(), 1);
}

#[test]
fn a_member_coming_and_going_is_read() {
    let packet = [&[0xf2, 0x01][..], &longs(&[2_000_001, 150_002, 1]), &[1, 0, 5, 0, 2, 0]].concat();
    assert_eq!(packet.len(), 20);
    assert_eq!(member_online(&packet), Some((150_002, true)));
}

#[test]
fn the_positions_are_read() {
    let names = sized(ZC_POSITION_ID_NAME_INFO, &[&longs(&[0])[..], &padded("GuildMaster", 24), &longs(&[1]), &padded("Newbie", 24)].concat());
    assert_eq!(position_names(&names), vec![(0, "GuildMaster".to_owned()), (1, "Newbie".to_owned())]);
    let rights = sized(ZC_POSITION_INFO, &longs(&[0, 0x11, 0, 0, 1, 0x01, 1, 50]));
    assert_eq!(position_rights(&rights).iter().map(|position| (position.id, position.invites, position.expels, position.tax)).collect::<Vec<_>>(), vec![(0, true, true, 0), (1, true, false, 50)]);
    let changed = sized(ZC_ACK_CHANGE_GUILD_POSITIONINFO, &[&longs(&[1, 0x10, 1, 25])[..], &padded("Guard", 24)].concat());
    assert_eq!(positions_changed(&changed), vec![Position { id: 1, name: "Guard".to_owned(), invites: false, expels: true, storage: false, tax: 25 }]);
    let moved = sized(ZC_ACK_REQ_CHANGE_MEMBERS, &longs(&[2_000_001, 150_002, 1]));
    assert_eq!(members_moved(&moved), vec![(150_002, 1)]);
}

#[test]
fn the_related_guilds_are_read() {
    let packet = sized(ZC_MYGUILD_BASIC_INFO, &[&longs(&[0, 8])[..], &padded("Asgard", 24), &longs(&[1, 9]), &padded("Jotun", 24)].concat());
    assert_eq!(related(&packet), vec![(8, "Asgard".to_owned(), false), (9, "Jotun".to_owned(), true)]);
}

#[test]
fn the_emblem_and_a_change_of_guild_are_read() {
    let packet = sized(ZC_GUILD_EMBLEM_IMG, &[&longs(&[7, 3])[..], &[0x78, 0x9c, 1, 2, 3]].concat());
    assert_eq!(emblem(&packet), Some((7, 3, &[0x78, 0x9c, 1, 2, 3][..])));
    let packet = [&[0x47, 0x0b][..], &longs(&[7, 3, 2_000_001])].concat();
    assert_eq!(changed(&packet), Some((2_000_001, 7, 3)));
}

#[test]
fn an_emblem_is_unpacked_into_its_picture() {
    // A BMP of 24 bits, 2 by 2: the bottom row first, blue green red, each row filled to four bytes.
    let mut bitmap = vec![b'B', b'M'];
    bitmap.extend(70u32.to_le_bytes());
    bitmap.extend([0; 4]);
    bitmap.extend(54u32.to_le_bytes());
    bitmap.extend(40u32.to_le_bytes());
    bitmap.extend(2i32.to_le_bytes());
    bitmap.extend(2i32.to_le_bytes());
    bitmap.extend([1, 0, 24, 0]);
    bitmap.extend([0; 24]);
    bitmap.extend([255, 0, 0, 0, 255, 0, 0, 0, 0, 0, 255, 255, 0, 255, 0, 0]);
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(&bitmap, 6);
    let picture = emblem_picture(&packed).unwrap();
    assert_eq!((picture.width, picture.height), (2, 2));
    // The top row is the file's second: red, then magenta, which is see-through.
    assert_eq!(&picture.rgba[..4], &[255, 0, 0, 255]);
    assert_eq!(picture.rgba[7], 0);
    assert!(emblem_picture(&[1, 2, 3]).is_none());
    assert!(emblem_picture(&miniz_oxide::deflate::compress_to_vec_zlib(b"not a picture", 6)).is_none());
}

#[test]
fn the_skills_are_read() {
    let skill = [&[0x10, 0x27][..], &longs(&[0]), &[1, 0, 0, 0, 0, 0], &padded("GD_APPROVAL", 24), &[1]].concat();
    assert_eq!(skill.len(), 37);
    let packet = sized(ZC_GUILD_SKILLINFO, &[&[2, 0][..], &skill].concat());
    assert_eq!(skills(&packet), Some((2, vec![Skill { id: 10000, kind: 0, level: 1, sp: 0, range: 0, name: "GD_APPROVAL".to_owned(), raisable: true }])));
}

#[test]
fn the_notice_and_the_invitation_are_read() {
    let packet = [&[0x6f, 0x01][..], &padded("War tonight", 60), &padded("Be at the castle at nine.", 120)].concat();
    assert_eq!(packet.len(), 182);
    assert_eq!(notice(&packet), Some(("War tonight".to_owned(), "Be at the castle at nine.".to_owned())));
    let packet = [&[0x6a, 0x01][..], &longs(&[7]), &padded("Midgard", 24)].concat();
    assert_eq!(invited(&packet), Some((7, "Midgard".to_owned())));
}

#[test]
fn leaving_and_expelling_are_read() {
    let packet = [&[0x83, 0x0a][..], &longs(&[150_002]), &padded("bye", 40)].concat();
    assert_eq!(packet.len(), 46);
    assert_eq!(left(&packet), Some((150_002, "bye".to_owned())));
    // The expulsion has the reason first.
    let packet = [&[0x82, 0x0a][..], &padded("idle", 40), &longs(&[150_002])].concat();
    assert_eq!(expelled(&packet), Some((150_002, "idle".to_owned())));
    let packet = sized(ZC_BAN_LIST, &[&longs(&[150_002])[..], &padded("idle", 40), &padded("Loki", 24)].concat());
    assert_eq!(expelled_list(&packet), vec![("Loki".to_owned(), "idle".to_owned())]);
}

#[test]
fn chat_and_names_are_read() {
    let packet = sized(ZC_GUILD_CHAT, b"Thor : hello\0");
    assert_eq!(said(&packet), Some("Thor : hello".to_owned()));
    let packet = sized(ZC_GUILD_POSITION, &[&longs(&[2_000_001])[..], &padded("Newbie", 24)].concat());
    assert_eq!(position_of(&packet), Some((2_000_001, "Newbie".to_owned())));
    let packet = [&[0x30, 0x0a][..], &longs(&[2_000_001]), &padded("Loki", 24), &padded("", 24), &padded("Midgard", 24), &padded("Newbie", 24), &longs(&[0])].concat();
    assert_eq!(packet.len(), 106);
    assert_eq!(names(&packet), Some((2_000_001, Names { name: "Loki".to_owned(), party: String::new(), guild: "Midgard".to_owned(), position: "Newbie".to_owned() })));
}

#[test]
fn what_is_sent_has_the_server_s_lengths() {
    use crate::net::packet_length;
    let sent = [
        make(150_001, "Midgard"),
        ask_tabs(),
        ask(menu::MEMBERS),
        ask_emblem(7),
        invite(2_000_001, 2_000_000, 150_001),
        invite_named("Loki"),
        answer(7, true),
        leave(7, 2_000_000, 150_001, "bye"),
        expel(7, 2_000_001, 150_002, "idle"),
        break_up("Midgard"),
        set_notice(7, "War tonight", "Be at the castle at nine."),
        ask_names(2_000_001),
    ];
    for packet in sent {
        let id = u16::from_le_bytes([packet[0], packet[1]]);
        assert_eq!(packet_length(id), Some(packet.len() as i32), "{id:#06x}");
    }
    let said = say("Thor", "hello");
    assert_eq!(packet_length(0x017e), Some(-1));
    assert_eq!(&said[2..4], &[(said.len() as u16).to_le_bytes()[0], 0]);
    assert_eq!(&said[4..], b"Thor : hello");
}

#[test]
fn what_is_sent_is_laid_out_as_the_server_reads_it() {
    let made = make(150_001, "Midgard");
    assert_eq!((&made[..2], &made[2..6], &made[6..13], made[13]), (&[0x65, 0x01][..], &150_001u32.to_le_bytes()[..], &b"Midgard"[..], 0));
    let left = leave(7, 2_000_000, 150_001, "bye");
    assert_eq!((&left[2..6], &left[6..10], &left[10..14], &left[14..17]), (&7u32.to_le_bytes()[..], &2_000_000u32.to_le_bytes()[..], &150_001u32.to_le_bytes()[..], &b"bye"[..]));
    assert_eq!(answer(7, false)[6..], [0, 0, 0, 0]);
    // A name too long for its field is cut, and the field still ends with a zero.
    let long = invite_named("abcdefghijklmnopqrstuvwxyz");
    assert_eq!((&long[2..25], long[25]), (&b"abcdefghijklmnopqrstuvw"[..], 0));
    let notice = set_notice(7, "s", "n");
    assert_eq!((notice[6], notice[66]), (b's', b'n'));
}
