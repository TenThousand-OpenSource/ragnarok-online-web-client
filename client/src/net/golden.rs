//! Every packet this client writes, built with fixed arguments and compared byte for byte with `golden.txt`,
//! which was written by the builders as they were before they were moved onto `bytes::Writer`: so that
//! rewriting how a packet is built cannot change the packet.
//!
//! Each line of `golden.txt` is a builder's name and its bytes in sixteens, in the order of `built` below. A
//! builder that takes a text is built twice: with a short name, and with a text longer than any field, of
//! letters two bytes each, so that where each field cuts its text (at its last byte, or at the end of a whole
//! letter) is held too. A new builder gets its line here; `every_builder_is_here` fails until it has one.
//!
//! To write the file anew, after a packet was changed on purpose: `ROWEB_GOLDEN=write cargo test -p
//! roweb-client golden`, and look at what changed in it.

use super::*;

/// A name that fits any field.
const SHORT: &str = "Poring";
/// A text longer than any field, its letters two bytes each and so laid that fields of even and of odd sizes
/// both end inside a letter.
const LONG: &str = "xЙмя-героя-длиннее-двадцати-четырёх-байт, и ещё длиннее: чтобы не поместиться ни в одно поле пакета, каким бы длинным оно ни было, и чтобы всякое поле обрезало его посреди буквы.";

macro_rules! built {
    ($lines:ident; $($path:ident)::+; $($argument:expr),*) => {
        $lines.push((concat!($(stringify!($path), "::"),+).trim_end_matches("::"), $($path)::+($($argument),*).to_vec()))
    };
}

/// Every builder with its bytes.
fn built() -> Vec<(&'static str, Vec<u8>)> {
    let mut lines = Vec::new();
    way_in(&mut lines);
    world(&mut lines);
    things(&mut lines);
    dealings(&mut lines);
    people(&mut lines);
    guild_and_mail(&mut lines);
    lines
}

/// The login, the characters, entering the map and what keeps the connection.
fn way_in(lines: &mut Vec<(&'static str, Vec<u8>)>) {
    let account = packets::LoginAccepted { login_id1: 0x1122_3344, account_id: 2_000_001, login_id2: 0x5566_7788, sex: 1, char_servers: Vec::new() };
    for text in [SHORT, LONG] {
        built!(lines; packets::login; text, text);
        built!(lines; packets::make_char; text, 3, 5, 7, 4218, 1);
        built!(lines; packets::delete_now; 150_001, text);
        built!(lines; packets::say; text, text);
    }
    built!(lines; packets::login; "", "");
    built!(lines; packets::char_enter; &account);
    built!(lines; packets::ping; 2_000_001);
    built!(lines; packets::select_char; 2);
    built!(lines; packets::delete_reserve; 150_001);
    built!(lines; packets::delete_now; 150_001, "19990101");
    built!(lines; packets::delete_cancel; 150_001);
    built!(lines; packets::map_enter; &account, 150_001, 0x0a0b_0c0d);
    built!(lines; packets::map_loaded;);
    built!(lines; packets::tick; 0x0a0b_0c0d);
    built!(lines; packets::walk_to; 153, 207);
    built!(lines; packets::walk_to; 1023, 1023);
    built!(lines; packets::raise_stat; 13, 1);
    built!(lines; link::ping_live;);
    built!(lines; npc::restart; true);
    built!(lines; npc::restart; false);
}

/// What is done on the map: to those in sight, to NPCs, with skills.
fn world(lines: &mut Vec<(&'static str, Vec<u8>)>) {
    built!(lines; units::stop_attack;);
    built!(lines; units::pick_up; 0x0102_0304);
    built!(lines; units::attack; 0x0102_0304);
    built!(lines; units::attack_once; 0x0102_0304);
    built!(lines; units::face; 5);
    built!(lines; units::turn; 2, 5);
    built!(lines; units::sit; true);
    built!(lines; units::sit; false);
    built!(lines; npc::contact; 0x0102_0304);
    built!(lines; npc::next; 0x0102_0304);
    built!(lines; npc::close; 0x0102_0304);
    built!(lines; npc::choose; 0x0102_0304, 2);
    built!(lines; npc::number; 0x0102_0304, -5);
    built!(lines; npc::number; 0x0102_0304, 99_999);
    built!(lines; trade::deal; 0x0102_0304, true);
    built!(lines; trade::deal; 0x0102_0304, false);
    built!(lines; trade::buy; &[(501, 3), (1201, 1)]);
    built!(lines; trade::buy; &[]);
    built!(lines; trade::sell; &[(2, 30), (7, 1)]);
    built!(lines; trade::quit;);
    built!(lines; skills::use_on_ground; 10, 89, (153, 207));
    built!(lines; items::raise_skill; 28);
    built!(lines; items::use_skill; 10, 28, 0x0102_0304);
    built!(lines; fight::agree_star_place; 1);
    built!(lines; emotions::show; 12);
    built!(lines; quests::activate; 7001, true);
    built!(lines; quests::activate; 7001, false);
    built!(lines; scripted::progress_done;);
    built!(lines; viewequip::ask; 2_000_001);
    built!(lines; viewequip::set; 1, true);
    built!(lines; viewequip::set; 6, false);
    built!(lines; commands::user_count;);
    built!(lines; commands::whisper_list;);
    built!(lines; commands::remember_warp;);
    built!(lines; commands::doridori;);
    built!(lines; commands::whisper_all; true);
    built!(lines; commands::whisper_all; false);
    built!(lines; commands::ranking_ask; 1);
    for text in [SHORT, LONG] {
        built!(lines; npc::words; 0x0102_0304, text);
        built!(lines; npc::whisper; text, text);
        built!(lines; commands::whisper_from; text, true);
        built!(lines; commands::whisper_from; text, false);
        built!(lines; choosing::warp; 27, text);
    }
    built!(lines; choosing::warp; 27, "prontera.gat");
    built!(lines; choosing::make_arrows; 909);
    built!(lines; choosing::pick_spell; 14);
    built!(lines; choosing::make; 1201, [984, 0, 997]);
    built!(lines; choosing::cook; 1, 12_043);
    let listed = choosing::Listed { index: 4, id: 1201, refine: 7, cards: [4001, 0, 0, 255] };
    built!(lines; choosing::repair; Some(&listed));
    built!(lines; choosing::repair; None);
    built!(lines; choosing::refine; Some(4));
    built!(lines; choosing::refine; None);
    built!(lines; choosing::ask_compound; 9);
    built!(lines; choosing::compound; 9, 4);
    built!(lines; choosing::ask_name; 150_001);
}

/// What is carried, worn, kept and set on the bar.
fn things(lines: &mut Vec<(&'static str, Vec<u8>)>) {
    built!(lines; items::throw; 3, 20);
    built!(lines; items::use_item; 3, 2_000_001);
    built!(lines; items::wear; 3, 0x0002_0022);
    built!(lines; items::take_off; 3);
    built!(lines; items::identify; 3);
    built!(lines; items::uninstall; 1);
    built!(lines; items::move_to_tab; 3, true);
    built!(lines; items::move_to_tab; 3, false);
    built!(lines; items::set_hotkey; 5, items::Hotkey { skill: true, id: 28, count: 10 });
    built!(lines; items::set_hotkey; 5, items::Hotkey { skill: false, id: 501, count: 0 });
    built!(lines; items::set_hotkey_of; 2, 5, items::Hotkey { skill: true, id: 28, count: 10 });
    built!(lines; items::set_hotkey_of; 0, 37, items::Hotkey::default());
    built!(lines; storage::to_storage; 3, 100_000);
    built!(lines; storage::from_storage; 3, 100_000);
    built!(lines; storage::cart_to_storage; 3, 100_000);
    built!(lines; storage::storage_to_cart; 3, 100_000);
    built!(lines; storage::close_storage;);
    built!(lines; cart::put; 3, 100_000);
    built!(lines; cart::take; 3, 100_000);
    built!(lines; cart::change; 4);
    built!(lines; cart::select; 0x0102_0304, 9);
    built!(lines; pets::command; 3);
    built!(lines; pets::hatch; 3);
    built!(lines; pets::capture; 0x0102_0304);
    built!(lines; pets::act; 5);
    built!(lines; pets::act; -1);
    built!(lines; homunculus::command; 2);
    built!(lines; homunculus::move_to; 0x0102_0304, (153, 207));
    built!(lines; homunculus::attack; 0x0102_0304, 0x0506_0708);
    built!(lines; homunculus::to_owner; 0x0102_0304);
    built!(lines; mercenary::command; 2);
    for text in [SHORT, LONG] {
        built!(lines; pets::rename; text);
        built!(lines; homunculus::rename; text);
    }
}

/// Dealing with another player, one's own shop and another's.
fn dealings(lines: &mut Vec<(&'static str, Vec<u8>)>) {
    built!(lines; exchange::request; 2_000_002);
    built!(lines; exchange::reply; 3);
    built!(lines; exchange::add; 3, 100_000);
    built!(lines; exchange::add; 0, 2_000_000_000);
    built!(lines; exchange::conclude;);
    built!(lines; exchange::cancel;);
    built!(lines; exchange::trade;);
    for text in [SHORT, LONG] {
        built!(lines; vending::open; text, true, &[(2, 30, 1_000), (7, 1, 999_999_999)]);
        built!(lines; vending::open; text, false, &[]);
    }
    built!(lines; vending::close;);
    built!(lines; vending::ask; 2_000_002);
    built!(lines; vending::buy; 2_000_002, 77, &[(2, 30), (7, 1)]);
    built!(lines; vending::buy; 2_000_002, 77, &[]);
}

/// Parties, friends, chat rooms, a family.
fn people(lines: &mut Vec<(&'static str, Vec<u8>)>) {
    for text in [SHORT, LONG] {
        built!(lines; party::make; text, true, false);
        built!(lines; party::make; text, false, true);
        built!(lines; party::invite; text);
        built!(lines; party::expel; 2_000_002, text);
        built!(lines; party::say; text, text);
        built!(lines; friends::add; text);
        built!(lines; chat_rooms::create; &chat_rooms::Setting { title: text.to_owned(), password: text.to_owned(), public: false, limit: 20 });
        built!(lines; chat_rooms::change; &chat_rooms::Setting { title: text.to_owned(), password: String::new(), public: true, limit: 2 });
        built!(lines; chat_rooms::enter; 0x0102_0304, text);
        built!(lines; chat_rooms::give; text);
        built!(lines; chat_rooms::expel; text);
        built!(lines; family::adoption_answer; &family::Parent { account: 2_000_002, character: 150_002, name: text.to_owned() }, true);
        built!(lines; family::adoption_answer; &family::Parent { account: 2_000_002, character: 150_002, name: text.to_owned() }, false);
    }
    built!(lines; party::answer; 0x0102_0304, true);
    built!(lines; party::answer; 0x0102_0304, false);
    built!(lines; party::leave;);
    built!(lines; party::share; party::Sharing { experience: 1, pickup: true, items: false });
    built!(lines; party::share; party::Sharing { experience: 0, pickup: false, items: true });
    built!(lines; party::hand_over; 2_000_002);
    built!(lines; party::refuse_invitations; true);
    built!(lines; party::refuse_invitations; false);
    built!(lines; friends::remove; 2_000_002, 150_002);
    built!(lines; friends::answer; 2_000_002, 150_002, true);
    built!(lines; friends::answer; 2_000_002, 150_002, false);
    built!(lines; chat_rooms::enter; 0x0102_0304, "");
    built!(lines; chat_rooms::leave;);
}

/// The guild and the mail.
fn guild_and_mail(lines: &mut Vec<(&'static str, Vec<u8>)>) {
    let position = |id: u32, name: &str| guild::Position { id, name: name.to_owned(), invites: id % 2 == 0, expels: id % 3 == 0, storage: id == 0, tax: id * 7 };
    for text in [SHORT, LONG] {
        built!(lines; guild::make; 150_001, text);
        built!(lines; guild::invite_named; text);
        built!(lines; guild::leave; 9, 2_000_001, 150_001, text);
        built!(lines; guild::expel; 9, 2_000_002, 150_002, text);
        built!(lines; guild::break_up; text);
        built!(lines; guild::say; text, text);
        built!(lines; guild::set_notice; 9, text, text);
        built!(lines; guild::change_positions; &[position(0, text), position(3, text), position(19, "")]);
        built!(lines; mail::write; text);
        built!(lines; mail::check_receiver; text, true);
        built!(lines; mail::check_receiver; text, false);
        built!(lines; mail::send; &mail::Receiver { character: 150_002, job: 4012, level: 99, name: text.to_owned() }, text, 123_456_789, text, text);
    }
    built!(lines; guild::ask_names; 9);
    built!(lines; guild::ask_tabs;);
    built!(lines; guild::ask; 4);
    built!(lines; guild::ask_emblem; 9);
    built!(lines; guild::invite; 2_000_002, 2_000_001, 150_001);
    built!(lines; guild::answer; 9, true);
    built!(lines; guild::answer; 9, false);
    built!(lines; guild::ask_ally; 2_000_002, 2_000_001, 150_001);
    built!(lines; guild::answer_ally; 2_000_002, true);
    built!(lines; guild::answer_ally; 2_000_002, false);
    built!(lines; guild::oppose; 2_000_002);
    built!(lines; guild::unrelate; 12, true);
    built!(lines; guild::unrelate; 12, false);
    built!(lines; guild::change_positions; &[]);
    built!(lines; guild::move_members; &[(2_000_002, 150_002, 3), (2_000_003, 150_003, 19)]);
    built!(lines; guild::set_emblem; &[0x78, 0x9c, 1, 2, 3, 4, 5]);
    built!(lines; mail::open; true, [0, 0, 0]);
    built!(lines; mail::open; false, [0x0102_0304_0506_0708, -1, 77]);
    built!(lines; mail::close;);
    built!(lines; mail::ask; 1, 0x0102_0304_0506_0708);
    built!(lines; mail::delete; 2, 0x0102_0304_0506_0708);
    built!(lines; mail::take; true, 1, 0x0102_0304_0506_0708);
    built!(lines; mail::take; false, 1, 0x0102_0304_0506_0708);
    built!(lines; mail::give_back; 0x0102_0304_0506_0708);
    built!(lines; mail::cancel_write;);
    built!(lines; mail::attach; 3, 20);
    built!(lines; mail::detach; 3, 20);
    built!(lines; mail::send; &mail::Receiver { character: 150_002, job: 0, level: 1, name: SHORT.to_owned() }, SHORT, 0, "", "");
}

/// The lines of `golden.txt` as the builders write them now.
fn written() -> String {
    built().iter().map(|(name, bytes)| format!("{name} {}\n", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>())).collect()
}

#[test]
fn every_packet_is_built_as_it_was() {
    let written = written();
    if std::env::var("ROWEB_GOLDEN").as_deref() == Ok("write") {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/src/net/golden.txt"), &written).expect("golden.txt is written");
        return;
    }
    let kept = include_str!("golden.txt");
    for (at, (now, then)) in written.lines().zip(kept.lines()).enumerate() {
        assert_eq!(now, then, "line {} of golden.txt", at + 1);
    }
    assert_eq!(written.lines().count(), kept.lines().count(), "golden.txt has as many lines as there are packets built");
}

/// Every `pub fn … -> Vec<u8>` of the files of packets has a line here, under the name it is called by
/// (a file's builders are reached through its system's module, wherever the file lies under it).
#[test]
fn every_builder_is_here() {
    fn sources(folder: &std::path::Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(folder).expect("the folder of packets is read").flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                sources(&path, found);
            } else if name.ends_with(".rs") && !["tests.rs", "golden.rs", "bytes.rs"].contains(&name.as_str()) {
                found.push(std::fs::read_to_string(&path).expect("a file of packets is read"));
            }
        }
    }
    let mut files = Vec::new();
    sources(std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/net")), &mut files);
    let mut defined = std::collections::BTreeMap::<String, usize>::new();
    for line in files.iter().flat_map(|file| file.lines()) {
        if let Some(name) = line.strip_prefix("pub fn ").filter(|_| line.contains("-> Vec<u8>")).and_then(|rest| rest.split(['(', '<']).next()) {
            *defined.entry(name.to_owned()).or_default() += 1;
        }
    }
    assert!(defined.len() > 100, "the builders were found: {}", defined.len());
    let here: std::collections::BTreeSet<&str> = built().into_iter().map(|(name, _)| name).collect();
    for (name, times) in defined {
        let built = here.iter().filter(|built| built.rsplit("::").next() == Some(name.as_str())).count();
        assert!(built >= times, "`{name}` is defined {times} times and built here under {built} names: add it to net/golden.rs");
    }
}
