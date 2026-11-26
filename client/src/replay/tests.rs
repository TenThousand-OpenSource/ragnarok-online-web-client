//! Tests of the replays: the file written and read back, its cipher, the options, names, times and speeds.

use super::*;

fn file() -> File {
    File {
        header: Header {
            when: [2026, 11, 26, 15, 4, 59],
            service: 0,
            service_type: 0,
            name: "PlaySix".to_owned(),
            map: "prt_fild08.gat".to_owned(),
            account: 2_000_006,
            x: 150,
            y: 198,
            direction: 4,
            duration: 61_500,
            options: [1; 30].to_vec(),
        },
        who: Who { character: 150_006, job: 7, level: 99, job_level: 50, zeny: 4_956_300, look: [5, 3, 1, 0, 224, 0, 1], stats: [99, 1, 2, 3, 4, 5] },
        before: vec![vec![0xeb, 0x02, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], vec![0x7f, 0x00, 1, 0, 0, 0]],
        packets: vec![(0, vec![0x8d, 0x00, 9, 0, 1, 2, 3, 4, b'h']), (61_000, vec![0x80, 0x00, 1, 2, 3, 4, 0])],
    }
}

#[test]
fn a_file_written_is_read_back_whole() {
    let bytes = written(&file());
    assert!(bytes.starts_with(b"<< Ragnarok Replay File Version 0.05 >>\r\n"));
    assert_eq!(&bytes[100..104], [0x05, 0x47, 0x41, 0x52]);
    assert_eq!(header(&bytes), Some(file().header));
    assert_eq!(read(&bytes), Some(file()));
}

#[test]
fn the_keys_are_the_date_and_the_time_and_what_is_written_is_ciphered() {
    let bytes = written(&file());
    let key0 = u32::from_le_bytes(bytes[104..108].try_into().unwrap());
    let key1 = u32::from_le_bytes(bytes[108..112].try_into().unwrap());
    assert_eq!((key0 & 0xffff, key0 >> 16 & 0xff, key0 >> 24), (2026, 11, 26));
    assert_eq!((key1 >> 8 & 0xff, key1 >> 16 & 0xff, key1 >> 24), (15, 4, 59));
    // Nowhere in the file does the character's name stand as it is.
    assert!(!bytes.windows(7).any(|window| window == b"PlaySix"));
    // The cipher laid on twice is taken off.
    let mut words = *b"twelve bytes!";
    cipher(key0, key1, &mut words);
    assert_ne!(&words[..12], b"twelve bytes");
    assert_eq!(words[12], b'!');
    cipher(key0, key1, &mut words);
    assert_eq!(&words, b"twelve bytes!");
}

#[test]
fn what_is_no_replay_of_this_client_is_not_read() {
    assert_eq!(read(b"not a replay"), None);
    let mut bytes = written(&file());
    // A file whose table has no state of this client (as the Windows client's): its header is read, the
    // file is not played.
    bytes[TABLE_AT + STATE * 10] = 24;
    assert!(header(&bytes).is_some());
    assert_eq!(read(&bytes), None);
    bytes[100] = 0;
    assert_eq!(header(&bytes), None);
}

#[test]
fn the_boxes_of_the_options_go_together_as_the_clients_do() {
    let none: Options = [0; 30];
    let all = ticked(none, option::ALL, true);
    assert_eq!(&all[..10], [1; 10]);
    let less = ticked(all, option::ITEM, false);
    assert_eq!((less[option::ITEM], less[option::ALL], less[option::BASIC]), (0, 0, 1));
    let one = ticked(none, option::CHATTING, true);
    assert_eq!((one[option::CHATTING], one[option::BASIC], one[option::ALL]), (1, 1, 0));
    assert_eq!(&ticked(all, option::BASIC, false)[..10], [0; 10]);
    assert_eq!(ticked(all, option::ALL, false)[..10], [1, 1, 1, 1, 1, 1, 1, 1, 1, 0]);
}

#[test]
fn the_chat_is_recorded_only_with_its_option() {
    let all: Options = [1; 30];
    let silent = ticked(all, option::CHATTING, false);
    assert!(recorded(0x008d, &all) && recorded(0x0080, &all));
    assert!(!recorded(0x008d, &silent) && !recorded(0x0097, &silent) && !recorded(0x02dc, &silent));
    assert!(recorded(0x0080, &silent));
}

#[test]
fn names_times_and_speeds_are_the_clients() {
    assert_eq!(next_name([]), "RagnarokReplay_0001.rrf");
    assert_eq!(next_name(["RagnarokReplay_0001.rrf", "ragnarokreplay_0002.RRF", "mine.rrf"]), "RagnarokReplay_0003.rrf");
    assert_eq!(named(" mine "), "mine.rrf");
    assert_eq!(clock(0), "000:00:00");
    assert_eq!(clock(3_725_999), "001:02:05");
    assert_eq!(parts(61_500), (0, 1, 1));
    assert_eq!([turning(0), turning(250), turning(999), turning(1000)], [0, 1, 3, 0]);
    assert_eq!(stepped(1.0, true), 2.0);
    assert_eq!(stepped(4.0, true), 4.0);
    assert_eq!(stepped(0.25, false), 0.25);
    assert_eq!(stepped(2.0, false), 1.0);
    assert_eq!([speed_line(0.25), speed_line(1.0), speed_line(4.0), speed_line(3.0)], [1956, 1958, 1960, 1963]);
    assert_eq!([service_name(0), service_name(5), service_name(6), service_name(-1)], [Some("Primary"), Some("PK"), None, None]);
}
