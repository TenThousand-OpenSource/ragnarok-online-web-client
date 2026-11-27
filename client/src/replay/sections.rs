//! The sections of a replay file in which the Windows client keeps what its session held when the recording
//! began, as far as this client writes them: the character (section 3, `CSession::BuildCharacterReplayRecords`
//! read back by `CSession::ReadReplayCharacter`) and its values (section 4, `BuildStatsReplayRecords`,
//! `ReadReplayStats`). Each is a row of tagged records between a record that opens it and one that closes it,
//! every record a member of the client's session at that member's size; a member this client has nothing for
//! is written as zeros of its size, so the row is laid out as the client's whatever it holds.
//!
//! Also here: the records the client writes among the packets for what its windows were told
//! (`CC_REPLAYPACKET`, 0x08ae: `UIWindow::RecordResponse` and its kin, played by `CGameMode::ProcessUiMessage`).

use std::collections::HashMap;

use super::{tagged, tags};

/// The numbers of the two sections in the file's table.
pub const CHARACTER: usize = 3;
pub const STATS: usize = 4;

/// The server's numbers of a character's values (`SP_*`), as the app keeps them, by the tag of section 3 that
/// holds each: the job and the looks are not among them and are given apart.
const OF_VALUES: [(u16, u16); 35] = [
    (1015, 1),
    (1016, 11),
    (1017, 9),
    (1018, 22),
    (1019, 55),
    (1020, 12),
    (1024, 13),
    (1025, 14),
    (1026, 15),
    (1027, 16),
    (1028, 17),
    (1029, 18),
    (1030, 32),
    (1031, 33),
    (1032, 34),
    (1033, 35),
    (1034, 36),
    (1035, 37),
    (1036, 53),
    (1037, 41),
    (1038, 47),
    (1040, 45),
    (1041, 46),
    (1042, 42),
    (1043, 43),
    (1044, 44),
    (1045, 48),
    (1046, 49),
    (1047, 50),
    (1048, 52),
    (1049, 51),
    (1051, 20),
    (1052, 0),
    (1054, 23),
    (1055, 2),
];

/// The values of section 4 likewise: weight and the most of it, HP and SP and the most of them.
const OF_STATS: [(u16, u16); 6] = [(2017, 24), (2018, 25), (2029, 5), (2030, 6), (2031, 7), (2032, 8)];

/// Who the character is beyond its values: the account, the character, the job and what it looks like (the
/// head, the weapon, the shield, the head's colour, the three things worn on the head), the colour of its
/// clothes (1063: the session's member the decompiled client calls `m_bodyStyle`, from which its windows take
/// the body's palette), and the states of its body and of what it rides or hides in (1068, 1069).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Looks {
    pub account: u32,
    pub character: u32,
    pub job: u32,
    pub head: u32,
    pub weapon: u32,
    pub shield: u32,
    pub head_colour: u32,
    pub worn: [u32; 3],
    pub clothes_colour: u32,
    pub body_state: u32,
    pub effect_state: u32,
}

/// How many bytes the member behind a tag of section 3 has (`BuildCharacterReplayRecords`): four, but for the
/// two experiences and the weapon's look (eight), the name of the one whose equipment is looked at (24), the
/// team and the chat's font and the emblems' option (one), and the stall's title (as long as it is: none here).
fn character_size(tag: u16) -> usize {
    match tag {
        1015 | 1018 | 1061 => 8,
        1081 => 24,
        1082 | 1085 | 1098 => 1,
        1097 => 0,
        _ => 4,
    }
}

/// Section 3: the records 1010 to 1101 between 1000 and 1001.
pub fn character(looks: &Looks, values: &HashMap<u16, i64>) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 1000, &[]);
    for tag in 1010..=1101u16 {
        let value: i64 = match tag {
            1010 => looks.account.into(),
            1011 => looks.character.into(),
            1014 => looks.job.into(),
            1060 => looks.head.into(),
            1061 => looks.weapon.into(),
            1062 => looks.shield.into(),
            1063 => looks.clothes_colour.into(),
            1064 => looks.head_colour.into(),
            1065..=1067 => looks.worn[usize::from(tag - 1065)].into(),
            1068 => looks.body_state.into(),
            1069 => looks.effect_state.into(),
            _ => OF_VALUES.iter().find(|(of, _)| *of == tag).and_then(|(_, value)| values.get(value)).copied().unwrap_or(0),
        };
        let mut bytes = value.to_le_bytes().to_vec();
        bytes.resize(character_size(tag), 0);
        tagged(&mut section, tag, &bytes);
    }
    tagged(&mut section, 1001, &[]);
    section
}

/// Section 4: the records 2010 to 2038 and 2052 between 2000 and 2001 (`BuildStatsReplayRecords`): the six
/// bonuses of the stats, the weight, five rows of twelve numbers this client has nothing for, HP and SP, and
/// the account's name, which this client does not write.
pub fn stats(bonus: &[i32; 6], values: &HashMap<u16, i64>) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 2000, &[]);
    for tag in (2010..=2038u16).chain([2052]) {
        let value = match tag {
            2010..=2015 => i64::from(bonus[usize::from(tag - 2010)]),
            _ => OF_STATS.iter().find(|(of, _)| *of == tag).and_then(|(_, value)| values.get(value)).copied().unwrap_or(0),
        };
        match tag {
            2024..=2028 => tagged(&mut section, tag, &[0; 48]),
            2038 => tagged(&mut section, tag, &[]),
            _ => tagged(&mut section, tag, &(value as i32).to_le_bytes()),
        }
    }
    tagged(&mut section, 2001, &[]);
    section
}

/// A number out of a record's bytes, however many of the first eight it has.
fn number(bytes: &[u8]) -> i64 {
    let mut whole = [0u8; 8];
    let size = bytes.len().min(8);
    whole[..size].copy_from_slice(&bytes[..size]);
    if size == 4 { i64::from(i32::from_le_bytes([whole[0], whole[1], whole[2], whole[3]])) } else { i64::from_le_bytes(whole) }
}

/// Sections 3 and 4 read back (`ReadReplayCharacter`, `ReadReplayStats`): who the character is, its values by
/// the server's numbers, and the bonuses of its stats. None when section 3 does not open as the client's does.
pub fn told(character: &[u8], stats: &[u8]) -> Option<(Looks, HashMap<u16, i64>, [i32; 6])> {
    let records = tags(character);
    if records.first().map(|(tag, _)| *tag) != Some(1000) {
        return None;
    }
    let (mut looks, mut values, mut bonus) = (Looks::default(), HashMap::new(), [0; 6]);
    for (tag, bytes) in records.into_iter().chain(tags(stats)) {
        let value = number(bytes);
        match tag {
            1010 => looks.account = value as u32,
            1011 => looks.character = value as u32,
            1014 => looks.job = value as u32,
            1060 => looks.head = value as u32,
            1061 => looks.weapon = value as u32,
            1062 => looks.shield = value as u32,
            1063 => looks.clothes_colour = value as u32,
            1064 => looks.head_colour = value as u32,
            1065..=1067 => looks.worn[usize::from(tag - 1065)] = value as u32,
            1068 => looks.body_state = value as u32,
            1069 => looks.effect_state = value as u32,
            2010..=2015 => bonus[usize::from(tag - 2010)] = value as i32,
            _ => {
                if let Some((_, of)) = OF_VALUES.iter().chain(&OF_STATS).find(|(known, _)| *known == tag) {
                    values.insert(*of, value);
                }
            }
        }
    }
    Some((looks, values, bonus))
}

/// The number of the packet the client writes among the recorded ones for what its windows were told.
pub const CC_REPLAYPACKET: u16 = 0x08ae;

/// The kinds of such a record this client writes and plays. 909 and 914 are the client's
/// (`UIWindowMgr::RecordWindowPacket`, and `CloseWindow` at the windows it records the closing of: a window is
/// opened, a window is closed, by the client's number of it). `PRESS` is this project's own: something of this
/// client's page was pressed, by the name and what went with it (`crate::press`); the Windows client has no
/// arm for it in `ProcessUiMessage` and passes it by. It is written only for a press the client has no record
/// of its own for: those, the kinds 900 to 902 and 913, are in `super::told`.
pub mod kind {
    pub const OPENED: u16 = 909;
    pub const CLOSED: u16 = 914;
    pub const PRESS: u16 = 990;
}

/// The client's numbers of the windows whose opening and closing it records, that this client has: the quest
/// window (`WID_RENEWQUESTWND`, written as 0x2718 when it opens) and the Tipbox.
pub mod window {
    pub const QUESTS: u32 = 0x2718;
}

/// A record of the windows: `u16 0x08ae, u16 kind, u16 bytes`, then what the kind holds.
pub(super) fn ui(kind: u16, body: &[u8]) -> Vec<u8> {
    let mut record = Vec::with_capacity(6 + body.len());
    record.extend(CC_REPLAYPACKET.to_le_bytes());
    record.extend(kind.to_le_bytes());
    record.extend((6 + body.len() as u16).to_le_bytes());
    record.extend(body);
    record
}

/// A window was opened or closed (ten bytes, the window's number after the six).
pub fn window_record(opened: bool, window: u32) -> Vec<u8> {
    ui(if opened { kind::OPENED } else { kind::CLOSED }, &window.to_le_bytes())
}

/// Something of this client's page was pressed: its name, a zero, what went with it.
pub fn press_record(what: &str, with: &str) -> Vec<u8> {
    ui(kind::PRESS, &[what.as_bytes(), &[0], with.as_bytes()].concat())
}

/// What a record of the windows says.
#[derive(Debug, PartialEq)]
pub enum Told<'record> {
    Window { opened: bool, window: u32 },
    Press(&'record str, &'record str),
    /// Another kind of the Windows client's (900 to 913, but 909): `super::told::deed` reads those it knows.
    Other(u16),
}

/// Reads a record of the windows; none for a packet that is none.
pub fn ui_record(packet: &[u8]) -> Option<Told<'_>> {
    if packet.len() < 6 || u16::from_le_bytes([packet[0], packet[1]]) != CC_REPLAYPACKET {
        return None;
    }
    let (kind, body) = (u16::from_le_bytes([packet[2], packet[3]]), &packet[6..]);
    Some(match kind {
        kind::OPENED | kind::CLOSED if body.len() >= 4 => {
            Told::Window { opened: kind == kind::OPENED, window: u32::from_le_bytes([body[0], body[1], body[2], body[3]]) }
        }
        kind::PRESS => {
            let text = std::str::from_utf8(body).ok()?;
            let (what, with) = text.split_once('\0')?;
            Told::Press(what, with)
        }
        other => Told::Other(other),
    })
}
