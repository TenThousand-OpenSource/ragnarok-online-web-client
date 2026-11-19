//! The packets of the way in (the login server, the character server, entering the map server) and the basics
//! of the map server: the clock, walking, the character's values, the chat, leaving. Requests are built as
//! bytes; replies are read out of a whole packet (ID included). The numbers of the packets, the parameters and
//! what is read of the login are here; the character server is `packets/chars.rs`, the map server
//! `packets/zone.rs`.

use super::bytes::{i16_at, i32_at, i64_at, long, name_at, records, u16_at, u32_at, u64_at, word, Writer};

mod chars;
mod zone;

pub use chars::*;
pub use zone::*;

const NAME_LENGTH: usize = 24;

// Login server.
pub const AC_ACCEPT_LOGIN: u16 = 0x0ac4;
pub const AC_REFUSE_LOGIN: u16 = 0x083e;
/// Sent by every server: the connection is refused or ended, with a reason code.
pub const SC_NOTIFY_BAN: u16 = 0x0081;
// Char server.
pub const HC_ACCEPT_ENTER2: u16 = 0x082d;
pub const HC_ACCEPT_ENTER: u16 = 0x006b;
pub const HC_REFUSE_ENTER: u16 = 0x006c;
pub const HC_ACCEPT_MAKECHAR: u16 = 0x0b6f;
pub const HC_REFUSE_MAKECHAR: u16 = 0x006e;
pub const HC_NOTIFY_ZONESVR: u16 = 0x0ac5;
pub const HC_DELETE_CHAR3_RESERVED: u16 = 0x0828;
pub const HC_DELETE_CHAR3: u16 = 0x082a;
pub const HC_DELETE_CHAR3_CANCEL: u16 = 0x082c;
// Map server.
pub const ZC_ACCEPT_ENTER: u16 = 0x02eb;
pub const ZC_REFUSE_ENTER: u16 = 0x0074;
pub const ZC_NOTIFY_PLAYERMOVE: u16 = 0x0087;
pub const ZC_STOPMOVE: u16 = 0x0088;
pub const ZC_NPCACK_MAPMOVE: u16 = 0x0091;
pub const ZC_PAR_CHANGE: u16 = 0x00b0;
pub const ZC_LONGPAR_CHANGE: u16 = 0x00b1;
pub const ZC_LONGLONGPAR_CHANGE: u16 = 0x0acb;
pub const ZC_STATUS: u16 = 0x00bd;
pub const ZC_STATUS_CHANGE: u16 = 0x00be;
pub const ZC_COUPLESTATUS: u16 = 0x0141;
pub const ZC_NOTIFY_CHAT: u16 = 0x008d;
pub const ZC_NOTIFY_PLAYERCHAT: u16 = 0x008e;

/// The character's parameters, as the servers number them (rAthena's `_sp`).
pub mod parameter {
    /// Milliseconds per cell walked.
    pub const SPEED: u16 = 0;
    pub const BASE_EXP: u16 = 1;
    pub const JOB_EXP: u16 = 2;
    pub const HP: u16 = 5;
    pub const MAX_HP: u16 = 6;
    pub const SP: u16 = 7;
    pub const MAX_SP: u16 = 8;
    pub const STATUS_POINTS: u16 = 9;
    pub const BASE_LEVEL: u16 = 11;
    pub const SKILL_POINTS: u16 = 12;
    /// The six stats follow each other from here: Str, Agi, Vit, Int, Dex, Luk.
    pub const STR: u16 = 13;
    pub const ZENY: u16 = 20;
    /// Activity Points, which the fourth jobs have, and the most of them.
    pub const AP: u16 = 232;
    pub const MAX_AP: u16 = 233;
    pub const NEXT_BASE_EXP: u16 = 22;
    pub const NEXT_JOB_EXP: u16 = 23;
    /// Tenths of a unit of weight.
    pub const WEIGHT: u16 = 24;
    pub const MAX_WEIGHT: u16 = 25;
    /// What raising each stat by one costs, from here: Str, Agi, Vit, Int, Dex, Luk.
    pub const STR_COST: u16 = 32;
    pub const ATK: u16 = 41;
    pub const ATK_BONUS: u16 = 42;
    pub const MATK_MAX: u16 = 43;
    pub const MATK_MIN: u16 = 44;
    pub const DEF: u16 = 45;
    pub const DEF_BONUS: u16 = 46;
    pub const MDEF: u16 = 47;
    pub const MDEF_BONUS: u16 = 48;
    pub const HIT: u16 = 49;
    pub const FLEE: u16 = 50;
    pub const FLEE_BONUS: u16 = 51;
    pub const CRITICAL: u16 = 52;
    /// The delay between attacks, in units of 10 milliseconds... as the server keeps it: 2000 less ten times ASPD.
    pub const ATTACK_DELAY: u16 = 53;
    pub const JOB_LEVEL: u16 = 55;
}
/// The parameter of ZC_PAR_CHANGE that is the walking speed, in milliseconds per cell.
pub const PARAMETER_SPEED: u16 = parameter::SPEED;

/// The size of one character in the char server's lists (`CHARACTER_INFO`).
const CHARACTER_INFO: usize = 175;

// ---------------------------------------------------------------------------------------------------------------
// Login server

/// CA_LOGIN (0x0064): `u32 version, char[24] user, char[24] password, u8 clienttype`.
pub fn login(user: &str, password: &str) -> Vec<u8> {
    Writer::new(0x0064).u32(55).text(user, NAME_LENGTH).text(password, NAME_LENGTH).u8(0).done()
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharServer {
    pub ip: [u8; 4],
    pub port: u16,
    pub name: String,
    pub users: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAccepted {
    pub login_id1: u32,
    pub account_id: u32,
    pub login_id2: u32,
    pub sex: u8,
    pub char_servers: Vec<CharServer>,
}

/// AC_ACCEPT_LOGIN (0x0ac4): `u16 length, u32 login_id1, u32 AID, u32 login_id2, u32 last_ip, char[26] last_login,
/// u8 sex, char[17] token`, then per char server `u32 ip, u16 port, char[20] name, u16 users, u16 type, u16 new,
/// u8[128]`.
pub fn login_accepted(packet: &[u8]) -> Option<LoginAccepted> {
    const HEADER: usize = 64;
    const SERVER: usize = 160;
    if packet.len() < HEADER {
        return None;
    }
    let char_servers = records(packet, HEADER, SERVER)
        .map(|server| CharServer {
            ip: server[..4].try_into().unwrap(),
            port: u16_at(server, 4),
            name: name_at(server, 6, 20),
            users: u16_at(server, 26),
        })
        .collect();
    Some(LoginAccepted { login_id1: u32_at(packet, 4), account_id: u32_at(packet, 8), login_id2: u32_at(packet, 12), sex: packet[46], char_servers })
}

/// AC_REFUSE_LOGIN (0x083e): `u32 error, char[20] unblock_time`. Error 0 is an unknown ID, 1 a wrong password,
/// 6 a block until the time given.
pub fn login_refused(packet: &[u8]) -> Option<(u32, String)> {
    (packet.len() >= 26).then(|| (u32_at(packet, 2), name_at(packet, 6, 20)))
}

/// SC_NOTIFY_BAN (0x0081): `u8 reason`. 1 is a closed server, 2 another login on the account, 8 a login the
/// server has not forgotten yet.
pub fn ban_reason(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

// ---------------------------------------------------------------------------------------------------------------
// Char server

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub char_id: u32,
    pub base_exp: u64,
    pub zeny: u32,
    pub job_level: u32,
    pub hp: u64,
    pub max_hp: u64,
    pub sp: u64,
    pub max_sp: u64,
    pub job: u16,
    pub hair_style: u16,
    pub base_level: u16,
    pub name: String,
    /// Str, Agi, Vit, Int, Dex, Luk.
    pub stats: [u8; 6],
    pub slot: u8,
    pub hair_color: u8,
    /// The headgear worn, as view IDs (0 for none): bottom, top, middle.
    pub headgear: [u16; 3],
    /// Which recolouring of the job's clothes; 0 for the sprite's own colours.
    pub body_palette: u16,
    /// Seconds until the character can be deleted for good, if its deletion was asked for.
    pub delete_in: Option<i32>,
    /// The map the character is saved on, without the `.gat` the server appends.
    pub map: String,
    pub sex: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneServer {
    pub char_id: u32,
    pub map: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapAccepted {
    pub server_tick: u32,
    pub x: u16,
    pub y: u16,
    pub direction: u8,
}

#[cfg(test)]
mod tests;
