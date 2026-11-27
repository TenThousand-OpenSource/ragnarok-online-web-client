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
        client: vec![(3, sections::character(&looks(), &values())), (4, sections::stats(&[1, 2, 3, 4, 5, 6], &values()))],
    }
}

fn looks() -> sections::Looks {
    let (account, character, worn) = (2_000_006, 150_006, [0, 224, 5]);
    sections::Looks { account, character, job: 7, head: 5, weapon: 2, shield: 1, head_colour: 3, worn, clothes_colour: 2, body_state: 1, effect_state: 32 }
}

fn values() -> std::collections::HashMap<u16, i64> {
    [(1, 5_000_000_000), (11, 99), (13, 99), (18, 7), (20, 4_956_300), (0, 150), (5, 7000), (6, 9000), (24, 1200), (25, 30_000), (55, 50)].into()
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
fn what_is_no_replay_is_not_read_and_a_file_begins_from_its_packets_or_from_the_clients_sections() {
    assert_eq!(read(b"not a replay"), None);
    let mut bytes = written(&file());
    assert!(read(&bytes).is_some_and(|file| file.begins() && file.begins_from_packets()));
    // A file whose table has no state of this client (as the Windows client's): it is read, its sections of
    // the client's own with it, and it begins from those.
    bytes[TABLE_AT + STATE * 10] = 0;
    let read_back = read(&bytes).unwrap();
    assert!(read_back.begins() && !read_back.begins_from_packets());
    assert_eq!(read_back.client, file().client);
    // One with neither has nothing to begin from.
    let bare = read(&written(&File { before: Vec::new(), client: Vec::new(), ..file() })).unwrap();
    assert!(!bare.begins());
    bytes[100] = 0;
    assert_eq!(header(&bytes), None);
}

#[test]
fn a_file_without_the_packets_before_has_no_section_of_this_clients_own() {
    let given = File { before: Vec::new(), who: Who::default(), ..file() };
    let bytes = written(&given);
    assert_eq!(bytes[TABLE_AT + STATE * 10..TABLE_AT + STATE * 10 + 10], [0; 10]);
    assert_eq!(read(&bytes), Some(given));
    // The character is read out of the client's third section.
    let character = snapshot::session::character(&read(&bytes).unwrap()).unwrap();
    assert_eq!((character.who.character, character.who.job, character.who.level, character.who.zeny), (150_006, 7, 99, 4_956_300));
}

#[test]
fn the_state_of_this_clients_first_files_is_still_read() {
    // (As they were written: the state in the 24th place under the number 124.)
    let mut bytes = written(&File { client: Vec::new(), ..file() });
    let entry: [u8; 10] = bytes[TABLE_AT + STATE * 10..TABLE_AT + STATE * 10 + 10].try_into().unwrap();
    bytes[TABLE_AT + STATE * 10..TABLE_AT + STATE * 10 + 10].fill(0);
    bytes[TABLE_AT + 240..TABLE_AT + 250].copy_from_slice(&entry);
    bytes[TABLE_AT + 240..TABLE_AT + 242].copy_from_slice(&124i16.to_le_bytes());
    assert_eq!(read(&bytes).map(|file| file.before), Some(file().before));
}

#[test]
fn the_table_is_laid_out_so_that_the_windows_client_reads_it_to_its_end() {
    // Its reading (`ReplayFileList::Open`) stops at a number under 0 or over 26, and takes each entry into
    // the place of its number: every entry up to the 25th keeps a number it accepts, the place's own or none.
    let bytes = written(&file());
    for place in 0..25 {
        let id = i16::from_le_bytes([bytes[TABLE_AT + place * 10], bytes[TABLE_AT + place * 10 + 1]]);
        assert!(id == 0 || id as usize == place, "place {place} holds {id}");
    }
    assert_eq!(i16::from_le_bytes([bytes[TABLE_AT + 250], bytes[TABLE_AT + 251]]), 111);
    // The first section stands where the client's first does (`fileOffset = 0x174`).
    assert_eq!(u32::from_le_bytes(bytes[TABLE_AT + 36..TABLE_AT + 40].try_into().unwrap()), 0x174);
}

#[test]
fn the_character_and_its_values_are_written_as_the_clients_members_and_read_back() {
    let character = sections::character(&looks(), &values());
    let records = tags(&character);
    // 1000, the 92 members from 1010 to 1101, 1001; each of its member's size.
    assert_eq!(records.len(), 94);
    assert_eq!((records[0].0, records[93].0), (1000, 1001));
    let size = |tag: u16| records.iter().find(|(id, _)| *id == tag).map(|(_, bytes)| bytes.len());
    assert_eq!([size(1010), size(1015), size(1061), size(1081), size(1082), size(1097), size(1054)], [Some(4), Some(8), Some(8), Some(24), Some(1), Some(0), Some(4)]);
    let stats = sections::stats(&[1, 2, 3, 4, 5, 6], &values());
    assert_eq!(tags(&stats).len(), 32);
    assert_eq!(tags(&stats).iter().find(|(id, _)| *id == 2026).map(|(_, bytes)| bytes.len()), Some(48));
    let (read_looks, read_values, bonus) = sections::told(&character, &stats).unwrap();
    assert_eq!(read_looks, looks());
    assert_eq!(bonus, [1, 2, 3, 4, 5, 6]);
    for (value, is) in values() {
        assert_eq!(read_values.get(&value), Some(&is), "value {value}");
    }
    assert_eq!(sections::told(&stats, &stats), None);
}

#[test]
fn the_records_of_the_windows_are_written_and_read() {
    use sections::{press_record, ui_record, window_record, Told};
    let opened = window_record(true, sections::window::QUESTS);
    // `UIWindowMgr::RecordWindowPacket`: the marker 0x038D08AE, ten bytes, the window.
    assert_eq!(opened, [0xae, 0x08, 0x8d, 0x03, 10, 0, 0x18, 0x27, 0, 0]);
    assert_eq!(ui_record(&opened), Some(Told::Window { opened: true, window: 0x2718 }));
    assert_eq!(window_record(false, 7)[..6], [0xae, 0x08, 0x92, 0x03, 10, 0]);
    let press = press_record("talk-next", "");
    assert_eq!(ui_record(&press), Some(Told::Press("talk-next", "")));
    assert_eq!(ui_record(&press_record("talk-choice", "2")), Some(Told::Press("talk-choice", "2")));
    assert_eq!(ui_record(&[0xae, 0x08, 0x84, 0x03, 26, 0, 1, 2]), Some(Told::Other(900)));
    assert_eq!(ui_record(&[0x8d, 0x00, 4, 0]), None);
}

#[test]
fn what_is_held_back_while_a_recording_plays_goes_by_its_options() {
    use crate::hotkeys::interface as key;
    use playing::{key_acts, window_opens, Window};
    let all: Options = [1; 30];
    let bare = ticked(ticked(all, option::ITEM, false), option::CHATTING, false);
    assert!(key_acts(playing::START_STOP, &bare) && key_acts(playing::FASTER, &bare) && key_acts(playing::SLOWER, &bare));
    assert!(key_acts(key::MINIMAP, &bare) && key_acts(key::WORLD_MAP, &bare) && key_acts(key::QUESTS, &bare));
    assert!(key_acts(key::INVENTORY, &all) && !key_acts(key::INVENTORY, &bare) && !key_acts(key::CART, &bare));
    assert!(key_acts(key::EQUIPMENT, &bare) && key_acts(key::STATUS, &bare) && !key_acts(key::CHAT, &bare));
    // What has no place in the client's list does nothing: sitting down, the guild, the recorder itself.
    assert!(!key_acts(key::SIT, &all) && !key_acts(key::GUILD, &all) && !key_acts(playing::RECORDER, &all));
    assert!(window_opens(Window::Items, &all) && !window_opens(Window::Items, &bare) && window_opens(Window::Skills, &bare));
    assert!(playing::chat_shows(&all) && !playing::chat_shows(&bare) && playing::bar_shows(&bare));
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
    // What the client's table of packets does not mark is in no recording: experience as rAthena tells it
    // (0x0acb), a packet it does not know.
    assert!(!recorded(0x0acb, &all) && !recorded(0xfff0, &all) && recorded(0x09ff, &all) && recorded(0x08ae, &silent));
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

#[test]
fn presses_pass_while_playing_by_the_options_and_are_recorded_by_their_window() {
    use crate::press::{general, inventory, replay, skills, talk, Press};
    use playing::{press_passes, press_recorded};
    let all: Options = [1; 30];
    let bare = ticked(all, option::ITEM, false);
    assert!(press_passes(&replay::Press::Hold.into(), &bare) && press_passes(&general::Press::Menu.into(), &bare));
    assert!(press_passes(&inventory::Press::Window.into(), &all) && !press_passes(&inventory::Press::Window.into(), &bare));
    assert!(press_passes(&inventory::Press::Equipment.into(), &bare) && press_passes(&skills::Press::Window.into(), &bare));
    assert!(!press_passes(&general::Press::Options.into(), &all) && !press_passes(&talk::Press::Next.into(), &all));
    assert!(press_recorded(&talk::Press::Next.into()) && !press_recorded(&inventory::Press::Window.into()));
}
