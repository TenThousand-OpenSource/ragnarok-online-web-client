//! The packets of the way in. Requests are built as bytes; replies are read out of a whole packet (ID included).

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
pub const ZC_BROADCAST: u16 = 0x009a;
pub const ZC_BROADCAST2: u16 = 0x01c3;

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

struct Writer(Vec<u8>);

impl Writer {
    fn new(id: u16) -> Self {
        Self(id.to_le_bytes().to_vec())
    }

    fn u8(mut self, value: u8) -> Self {
        self.0.push(value);
        self
    }

    fn u16(mut self, value: u16) -> Self {
        self.0.extend(value.to_le_bytes());
        self
    }

    fn u32(mut self, value: u32) -> Self {
        self.0.extend(value.to_le_bytes());
        self
    }

    /// A fixed-size text field, zero-padded. The last byte always stays zero, as the server expects a terminator.
    fn text(mut self, text: &str, size: usize) -> Self {
        let bytes = &text.as_bytes()[..text.len().min(size - 1)];
        self.0.extend(bytes);
        self.0.resize(self.0.len() + size - bytes.len(), 0);
        self
    }
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

/// A zero-terminated text field. Names on this server are ASCII; other bytes are read as Latin-1 so nothing is lost.
fn text_at(bytes: &[u8], at: usize, size: usize) -> String {
    let field = &bytes[at..at + size];
    let end = field.iter().position(|&byte| byte == 0).unwrap_or(size);
    match std::str::from_utf8(&field[..end]) {
        Ok(text) => text.to_owned(),
        Err(_) => field[..end].iter().map(|&byte| char::from(byte)).collect(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Login server

/// CA_LOGIN (0x0064): `u32 version, char[24] user, char[24] password, u8 clienttype`.
pub fn login(user: &str, password: &str) -> Vec<u8> {
    Writer::new(0x0064).u32(55).text(user, NAME_LENGTH).text(password, NAME_LENGTH).u8(0).0
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
    let char_servers = packet[HEADER..]
        .chunks_exact(SERVER)
        .map(|server| CharServer {
            ip: server[..4].try_into().unwrap(),
            port: u16_at(server, 4),
            name: text_at(server, 6, 20),
            users: u16_at(server, 26),
        })
        .collect();
    Some(LoginAccepted { login_id1: u32_at(packet, 4), account_id: u32_at(packet, 8), login_id2: u32_at(packet, 12), sex: packet[46], char_servers })
}

/// AC_REFUSE_LOGIN (0x083e): `u32 error, char[20] unblock_time`. Error 0 is an unknown ID, 1 a wrong password,
/// 6 a block until the time given.
pub fn login_refused(packet: &[u8]) -> Option<(u32, String)> {
    (packet.len() >= 26).then(|| (u32_at(packet, 2), text_at(packet, 6, 20)))
}

/// SC_NOTIFY_BAN (0x0081): `u8 reason`. 1 is a closed server, 2 another login on the account, 8 a login the
/// server has not forgotten yet.
pub fn ban_reason(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

// ---------------------------------------------------------------------------------------------------------------
// Char server

/// CH_ENTER (0x0065): `u32 AID, u32 login_id1, u32 login_id2, u16 unused, u8 sex`.
pub fn char_enter(account: &LoginAccepted) -> Vec<u8> {
    Writer::new(0x0065).u32(account.account_id).u32(account.login_id1).u32(account.login_id2).u16(0).u8(account.sex).0
}

/// PING (0x0187): `u32 AID`. Keeps the char server connection open.
pub fn ping(account_id: u32) -> Vec<u8> {
    Writer::new(0x0187).u32(account_id).0
}

/// CH_SELECT_CHAR (0x0066): `u8 slot`.
pub fn select_char(slot: u8) -> Vec<u8> {
    Writer::new(0x0066).u8(slot).0
}

/// CH_MAKE_CHAR (0x0a39): `char[24] name, u8 slot, u16 hair_color, u16 hair_style, u32 job, u8 sex`.
pub fn make_char(name: &str, slot: u8, hair_color: u16, hair_style: u16, job: u32, sex: u8) -> Vec<u8> {
    Writer::new(0x0a39).text(name, NAME_LENGTH).u8(slot).u16(hair_color).u16(hair_style).u32(job).u8(sex).0
}

/// HC_ACCEPT_ENTER2 (0x082d): `u16 length, u8 normal, u8 premium, u8 billing, u8 producible, u8 total, char[20]`.
/// Returns how many slots the account may create characters in, and how many slots there are in all.
pub fn char_slots(packet: &[u8]) -> Option<(u8, u8)> {
    Some((*packet.get(7)?, *packet.get(8)?))
}

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

/// One `CHARACTER_INFO`, 175 bytes at this PACKETVER.
fn character(info: &[u8]) -> Character {
    Character {
        char_id: u32_at(info, 0),
        base_exp: u64_at(info, 4),
        zeny: u32_at(info, 12),
        job_level: u32_at(info, 24),
        hp: u64_at(info, 50),
        max_hp: u64_at(info, 58),
        sp: u64_at(info, 66),
        max_sp: u64_at(info, 74),
        job: u16_at(info, 84),
        hair_style: u16_at(info, 86),
        base_level: u16_at(info, 92),
        name: text_at(info, 108, NAME_LENGTH),
        stats: info[132..138].try_into().unwrap(),
        slot: info[138],
        hair_color: info[139],
        headgear: [u16_at(info, 96), u16_at(info, 100), u16_at(info, 102)],
        body_palette: u16_at(info, 106),
        // Zero stands for "not being deleted"; a date already passed arrives as a negative number.
        delete_in: Some(u32_at(info, 158) as i32).filter(|seconds| *seconds != 0),
        map: map_name(&text_at(info, 142, 16)),
        sex: info[174],
    }
}

fn map_name(name: &str) -> String {
    name.strip_suffix(".gat").unwrap_or(name).to_owned()
}

/// HC_ACCEPT_ENTER (0x006b): `u16 length, u8 total, u8 premium_start, u8 premium_end, char[20]`, then the
/// account's characters.
pub fn char_list(packet: &[u8]) -> Vec<Character> {
    packet.get(27..).unwrap_or_default().chunks_exact(CHARACTER_INFO).map(character).collect()
}

/// HC_ACCEPT_MAKECHAR (0x0b6f): the new character.
pub fn char_made(packet: &[u8]) -> Option<Character> {
    packet.get(2..2 + CHARACTER_INFO).map(character)
}

/// HC_REFUSE_MAKECHAR (0x006e): `u8 error`. 0 is a taken name, 1 an underaged player, 2 forbidden symbols,
/// 3 a slot the account may not use; anything else is a plain refusal.
pub fn make_char_refused(packet: &[u8]) -> u8 {
    packet.get(2).copied().unwrap_or(0xff)
}

/// CH_DELETE_CHAR3_RESERVED (0x0827): `u32 CID`. Asks for a character's deletion, which waits a day.
pub fn delete_reserve(char_id: u32) -> Vec<u8> {
    Writer::new(0x0827).u32(char_id).0
}

/// CH_DELETE_CHAR3 (0x0829): `u32 CID, char[6] birthdate` (YYMMDD). Deletes a character whose wait is over. An
/// account without a birthdate sends none.
pub fn delete_now(char_id: u32, birthdate: &str) -> Vec<u8> {
    let mut packet = Writer::new(0x0829).u32(char_id).0;
    let digits = &birthdate.as_bytes()[..birthdate.len().min(6)];
    packet.extend(digits);
    packet.resize(12, 0);
    packet
}

/// CH_DELETE_CHAR3_CANCEL (0x082b): `u32 CID`.
pub fn delete_cancel(char_id: u32) -> Vec<u8> {
    Writer::new(0x082b).u32(char_id).0
}

/// HC_DELETE_CHAR3_RESERVED (0x0828): `u32 CID, u32 result, u32 seconds`. Result 1 is success, and then the
/// seconds are how long the deletion waits; 3 is a character not found, 4 one in a guild, 5 one in a party.
pub fn delete_reserved(packet: &[u8]) -> Option<(u32, u32, i32)> {
    (packet.len() >= 14).then(|| (u32_at(packet, 2), u32_at(packet, 6), u32_at(packet, 10) as i32))
}

/// HC_DELETE_CHAR3 (0x082a) and HC_DELETE_CHAR3_CANCEL (0x082c): `u32 CID, u32 result`. Result 1 is success; for
/// a deletion, 4 is a wait not over yet and 5 a wrong birthdate.
pub fn delete_result(packet: &[u8]) -> Option<(u32, u32)> {
    (packet.len() >= 10).then(|| (u32_at(packet, 2), u32_at(packet, 6)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneServer {
    pub char_id: u32,
    pub map: String,
    pub port: u16,
}

/// HC_NOTIFY_ZONESVR (0x0ac5): `u32 CID, char[16] map, u32 ip, u16 port, char[128] domain`.
pub fn zone_server(packet: &[u8]) -> Option<ZoneServer> {
    (packet.len() >= 28).then(|| ZoneServer { char_id: u32_at(packet, 2), map: map_name(&text_at(packet, 6, 16)), port: u16_at(packet, 26) })
}

// ---------------------------------------------------------------------------------------------------------------
// Map server

/// CZ_ENTER (0x0436): `u32 AID, u32 CID, u32 login_id1, u32 client_tick, u8[4] unused, u8 sex`.
pub fn map_enter(account: &LoginAccepted, char_id: u32, tick: u32) -> Vec<u8> {
    Writer::new(0x0436).u32(account.account_id).u32(char_id).u32(account.login_id1).u32(tick).u32(0).u8(account.sex).0
}

/// CZ_NOTIFY_ACTORINIT (0x007d): the map is loaded; the server puts the character into the world.
pub fn map_loaded() -> Vec<u8> {
    Writer::new(0x007d).0
}

/// CZ_REQUEST_TIME (0x0360): `u32 client_tick`. Keeps the map server connection open.
pub fn tick(tick: u32) -> Vec<u8> {
    Writer::new(0x0360).u32(tick).0
}

/// CZ_REQUEST_MOVE (0x035f): the cell to walk to, packed as a position.
pub fn walk_to(x: u16, y: u16) -> Vec<u8> {
    let mut packet = Writer::new(0x035f).0;
    packet.extend([(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, (y << 4) as u8]);
    packet
}

/// ZC_NOTIFY_PLAYERMOVE (0x0087): `u32 start_time, u8[6] move`. The move packs the cell walked from and the cell
/// walked to, 10 bits a coordinate. Returns `(from, to)`.
pub fn player_move(packet: &[u8]) -> Option<((u16, u16), (u16, u16))> {
    let data: Vec<u16> = packet.get(6..12)?.iter().map(|&byte| u16::from(byte)).collect();
    let from = (data[0] << 2 | data[1] >> 6, (data[1] & 0x3f) << 4 | data[2] >> 4);
    let to = ((data[2] & 0x0f) << 6 | data[3] >> 2, (data[3] & 0x03) << 8 | data[4]);
    Some((from, to))
}

/// ZC_STOPMOVE (0x0088): `u32 id, u16 x, u16 y`: a unit stands on that cell now, wherever it was thought to be.
pub fn stop_move(packet: &[u8]) -> Option<(u32, u16, u16)> {
    (packet.len() >= 10).then(|| (u32_at(packet, 2), u16_at(packet, 6), u16_at(packet, 8)))
}

/// ZC_NPCACK_MAPMOVE (0x0091): `char[16] map, u16 x, u16 y`: the character is moved to another place, on this map
/// or another of the same server.
pub fn map_move(packet: &[u8]) -> Option<(String, u16, u16)> {
    (packet.len() >= 22).then(|| (map_name(&text_at(packet, 2, 16)), u16_at(packet, 18), u16_at(packet, 20)))
}

/// ZC_PAR_CHANGE (0x00b0) and ZC_LONGPAR_CHANGE (0x00b1): `u16 parameter, i32 value`.
pub fn parameter(packet: &[u8]) -> Option<(u16, i32)> {
    (packet.len() >= 8).then(|| (u16_at(packet, 2), u32_at(packet, 4) as i32))
}

/// ZC_LONGLONGPAR_CHANGE (0x0acb): `u16 parameter, i64 value`: experience.
pub fn long_parameter(packet: &[u8]) -> Option<(u16, i64)> {
    (packet.len() >= 12).then(|| (u16_at(packet, 2), u64_at(packet, 4) as i64))
}

/// ZC_STATUS (0x00bd): the whole status window at once, as `(parameter, value)` pairs: the status points, each
/// stat with what raising it costs, and the values worked out from them.
pub fn status(packet: &[u8]) -> Vec<(u16, i64)> {
    if packet.len() < 44 {
        return Vec::new();
    }
    let mut values = vec![(parameter::STATUS_POINTS, i64::from(u16_at(packet, 2)))];
    for stat in 0..6u16 {
        let at = 4 + usize::from(stat) * 2;
        values.push((parameter::STR + stat, i64::from(packet[at])));
        values.push((parameter::STR_COST + stat, i64::from(packet[at + 1])));
    }
    let signed = |at: usize| i64::from(u16_at(packet, at) as i16);
    for (index, name) in [
        parameter::ATK,
        parameter::ATK_BONUS,
        parameter::MATK_MAX,
        parameter::MATK_MIN,
        parameter::DEF,
        parameter::DEF_BONUS,
        parameter::MDEF,
        parameter::MDEF_BONUS,
        parameter::HIT,
        parameter::FLEE,
        parameter::FLEE_BONUS,
        parameter::CRITICAL,
        parameter::ATTACK_DELAY,
    ]
    .into_iter()
    .enumerate()
    {
        values.push((name, signed(16 + index * 2)));
    }
    values
}

/// ZC_STATUS_CHANGE (0x00be): `u16 parameter, u8 value`: what raising a stat costs now.
pub fn stat_cost(packet: &[u8]) -> Option<(u16, u8)> {
    (packet.len() >= 5).then(|| (u16_at(packet, 2), packet[4]))
}

/// ZC_COUPLESTATUS (0x0141): `u32 parameter, i32 base, i32 bonus`: a stat, and what equipment and the like add.
pub fn stat(packet: &[u8]) -> Option<(u16, i32, i32)> {
    (packet.len() >= 14).then(|| (u32_at(packet, 2) as u16, u32_at(packet, 6) as i32, u32_at(packet, 10) as i32))
}

/// CZ_STATUS_CHANGE (0x00bb): `u16 parameter, u8 amount`: raises a stat.
pub fn raise_stat(parameter: u16, amount: u8) -> Vec<u8> {
    Writer::new(0x00bb).u16(parameter).u8(amount).0
}

/// CZ_REQUEST_CHAT (0x00f3): `u16 length, char[] text`: says something to everyone near. The text is the
/// character's name, " : ", and the words, with a terminator.
pub fn say(name: &str, words: &str) -> Vec<u8> {
    let text = format!("{name} : {words}");
    let mut packet = Writer::new(0x00f3).u16((text.len() + 5) as u16).0;
    packet.extend(text.as_bytes());
    packet.push(0);
    packet
}

/// The text of a chat line: ZC_NOTIFY_PLAYERCHAT (0x008e, the player's own words back: `u16 length, char[]`),
/// ZC_NOTIFY_CHAT (0x008d, someone near: `u16 length, u32 id, char[]`), ZC_BROADCAST (0x009a, an announcement)
/// and ZC_BROADCAST2 (0x01c3, one with a colour: `u16 length, u32 colour, u16[4], char[]`).
pub fn chat_text(id: u16, packet: &[u8]) -> Option<String> {
    let from = match id {
        ZC_NOTIFY_PLAYERCHAT | ZC_BROADCAST => 4,
        ZC_NOTIFY_CHAT => 8,
        ZC_BROADCAST2 => 16,
        _ => return None,
    };
    let text = packet.get(from..)?;
    Some(text_at(text, 0, text.len()))
}

/// The colour of a ZC_BROADCAST2, as red, green, blue.
pub fn broadcast_colour(packet: &[u8]) -> Option<[u8; 3]> {
    let colour = packet.get(4..8)?;
    Some([colour[2], colour[1], colour[0]])
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapAccepted {
    pub server_tick: u32,
    pub x: u16,
    pub y: u16,
    pub direction: u8,
}

/// ZC_ACCEPT_ENTER (0x02eb): `u32 start_time, u8[3] position, u8 x_size, u8 y_size, u16 font`. The position packs
/// x (10 bits), y (10 bits) and the direction (4 bits).
pub fn map_accepted(packet: &[u8]) -> Option<MapAccepted> {
    let position = packet.get(6..9)?;
    Some(MapAccepted {
        server_tick: u32_at(packet, 2),
        x: u16::from(position[0]) << 2 | u16::from(position[1]) >> 6,
        y: (u16::from(position[1]) & 0x3f) << 4 | u16::from(position[2]) >> 4,
        direction: position[2] & 0x0f,
    })
}

/// What the client calls a job, for the first jobs; later ones wait for the client's own tables.
pub fn job_name(job: u16) -> String {
    let name = match job {
        0 => "Novice",
        1 => "Swordman",
        2 => "Magician",
        3 => "Archer",
        4 => "Acolyte",
        5 => "Merchant",
        6 => "Thief",
        7 => "Knight",
        8 => "Priest",
        9 => "Wizard",
        10 => "Blacksmith",
        11 => "Hunter",
        12 => "Assassin",
        14 => "Crusader",
        15 => "Monk",
        16 => "Sage",
        17 => "Rogue",
        18 => "Alchemist",
        19 => "Bard",
        20 => "Dancer",
        23 => "Super Novice",
        4218 => "Summoner",
        _ => return format!("Job {job}"),
    };
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::packet_length;

    fn account() -> LoginAccepted {
        LoginAccepted { login_id1: 0x1111_2222, account_id: 2_000_000, login_id2: 0x3333_4444, sex: 1, char_servers: Vec::new() }
    }

    /// Every request is as long as the server's table says, or the server would wait for more or misread the next.
    #[test]
    fn requests_have_the_length_the_server_expects() {
        let requests = [
            login("tester", "secret"),
            char_enter(&account()),
            ping(2_000_000),
            select_char(0),
            make_char("Hero", 0, 1, 2, 0, 1),
            map_enter(&account(), 150_000, 1234),
            map_loaded(),
            tick(1234),
            delete_reserve(150_000),
            delete_now(150_000, "990131"),
            delete_cancel(150_000),
            walk_to(156, 191),
            raise_stat(parameter::STR, 1),
        ];
        for request in requests {
            let id = u16_at(&request, 0);
            assert_eq!(packet_length(id), Some(request.len() as i32), "{id:#06x}");
        }
    }

    #[test]
    fn builds_the_login_request() {
        let packet = login("tester", "secret");
        assert_eq!(&packet[..6], [0x64, 0x00, 55, 0, 0, 0]);
        assert_eq!(&packet[6..13], b"tester\0");
        assert_eq!(&packet[30..37], b"secret\0");
        assert!(packet[13..30].iter().chain(&packet[37..]).all(|&byte| byte == 0));
    }

    /// A name that fills its field keeps a terminator.
    #[test]
    fn long_text_is_cut_to_fit() {
        let packet = login(&"u".repeat(40), "p");
        assert_eq!(&packet[6..29], "u".repeat(23).as_bytes());
        assert_eq!(packet[29], 0);
        assert_eq!(packet.len(), 55);
    }

    #[test]
    fn builds_the_char_and_map_requests() {
        assert_eq!(char_enter(&account()), [0x65, 0x00, 0x80, 0x84, 0x1e, 0x00, 0x22, 0x22, 0x11, 0x11, 0x44, 0x44, 0x33, 0x33, 0, 0, 1]);

        let packet = make_char("Hero", 2, 5, 7, 4218, 0);
        assert_eq!(&packet[2..7], b"Hero\0");
        assert_eq!(&packet[26..], [2, 5, 0, 7, 0, 0x7a, 0x10, 0, 0, 0]);

        let packet = map_enter(&account(), 150_000, 0x0102_0304);
        assert_eq!(&packet[2..6], 2_000_000u32.to_le_bytes());
        assert_eq!(&packet[6..10], 150_000u32.to_le_bytes());
        assert_eq!(&packet[10..14], 0x1111_2222u32.to_le_bytes());
        assert_eq!(&packet[14..18], [4, 3, 2, 1]);
        assert_eq!(packet[22], 1);
    }

    #[test]
    fn reads_the_login_reply() {
        let mut packet = vec![0u8; 64 + 160];
        packet[..2].copy_from_slice(&AC_ACCEPT_LOGIN.to_le_bytes());
        packet[2..4].copy_from_slice(&224u16.to_le_bytes());
        packet[4..8].copy_from_slice(&7u32.to_le_bytes());
        packet[8..12].copy_from_slice(&2_000_000u32.to_le_bytes());
        packet[12..16].copy_from_slice(&9u32.to_le_bytes());
        packet[46] = 1;
        packet[64..68].copy_from_slice(&[127, 0, 0, 1]);
        packet[68..70].copy_from_slice(&6121u16.to_le_bytes());
        packet[70..77].copy_from_slice(b"rAthena");
        packet[90..92].copy_from_slice(&3u16.to_le_bytes());

        let accepted = login_accepted(&packet).unwrap();
        assert_eq!((accepted.login_id1, accepted.account_id, accepted.login_id2, accepted.sex), (7, 2_000_000, 9, 1));
        assert_eq!(accepted.char_servers, [CharServer { ip: [127, 0, 0, 1], port: 6121, name: "rAthena".into(), users: 3 }]);
        assert_eq!(login_accepted(&packet[..40]), None);
    }

    #[test]
    fn reads_why_a_login_was_refused() {
        let mut packet = vec![0u8; 26];
        packet[2] = 6;
        packet[6..16].copy_from_slice(b"2027-01-01");
        assert_eq!(login_refused(&packet), Some((6, "2027-01-01".to_owned())));
        assert_eq!(login_refused(&packet[..10]), None);
        assert_eq!(ban_reason(&[0x81, 0x00, 2]), Some(2));
        assert_eq!(make_char_refused(&[0x6e, 0x00, 3]), 3);
        assert_eq!(char_slots(&[0x2d, 0x08, 29, 0, 9, 0, 0, 9, 15, 0]), Some((9, 15)));
    }

    fn character_bytes(name: &str, slot: u8) -> Vec<u8> {
        let mut info = vec![0u8; CHARACTER_INFO];
        info[..4].copy_from_slice(&150_000u32.to_le_bytes());
        info[4..12].copy_from_slice(&123_456_789_012u64.to_le_bytes());
        info[12..16].copy_from_slice(&500u32.to_le_bytes());
        info[24..28].copy_from_slice(&10u32.to_le_bytes());
        info[50..58].copy_from_slice(&40u64.to_le_bytes());
        info[58..66].copy_from_slice(&42u64.to_le_bytes());
        info[66..74].copy_from_slice(&11u64.to_le_bytes());
        info[74..82].copy_from_slice(&12u64.to_le_bytes());
        info[84..86].copy_from_slice(&4u16.to_le_bytes());
        info[86..88].copy_from_slice(&3u16.to_le_bytes());
        info[92..94].copy_from_slice(&99u16.to_le_bytes());
        info[108..108 + name.len()].copy_from_slice(name.as_bytes());
        info[132..138].copy_from_slice(&[1, 2, 3, 4, 5, 6]);
        info[138] = slot;
        info[139] = 8;
        info[96..98].copy_from_slice(&5u16.to_le_bytes());
        info[100..102].copy_from_slice(&224u16.to_le_bytes());
        info[102..104].copy_from_slice(&12u16.to_le_bytes());
        info[106..108].copy_from_slice(&2u16.to_le_bytes());
        info[142..154].copy_from_slice(b"prontera.gat");
        info[174] = 1;
        info
    }

    #[test]
    fn reads_the_character_list() {
        let mut packet = vec![0u8; 27];
        packet.extend(character_bytes("Hero", 0));
        packet.extend(character_bytes("Second", 4));

        let characters = char_list(&packet);
        assert_eq!(characters.len(), 2);
        let hero = &characters[0];
        assert_eq!((hero.char_id, hero.base_exp, hero.zeny, hero.job_level), (150_000, 123_456_789_012, 500, 10));
        assert_eq!((hero.hp, hero.max_hp, hero.sp, hero.max_sp), (40, 42, 11, 12));
        assert_eq!((hero.job, hero.hair_style, hero.base_level, hero.hair_color, hero.sex), (4, 3, 99, 8, 1));
        assert_eq!((hero.name.as_str(), hero.map.as_str(), hero.stats), ("Hero", "prontera", [1, 2, 3, 4, 5, 6]));
        assert_eq!((hero.headgear, hero.body_palette, hero.delete_in), ([5, 224, 12], 2, None));
        assert_eq!((characters[1].name.as_str(), characters[1].slot), ("Second", 4));

        // An account without characters.
        assert!(char_list(&packet[..27]).is_empty());
        assert!(char_list(&[]).is_empty());
    }

    #[test]
    fn reads_a_new_character() {
        let mut packet = HC_ACCEPT_MAKECHAR.to_le_bytes().to_vec();
        packet.extend(character_bytes("Fresh", 1));
        assert_eq!(packet.len() as i32, packet_length(HC_ACCEPT_MAKECHAR).unwrap());
        assert_eq!(char_made(&packet).unwrap().name, "Fresh");
        assert_eq!(char_made(&packet[..50]), None);
    }

    #[test]
    fn deleting_a_character() {
        assert_eq!(delete_reserve(150_000), [0x27, 0x08, 0xf0, 0x49, 0x02, 0x00]);
        assert_eq!(&delete_now(150_000, "990131")[6..], b"990131");
        // No birthdate on the account: six zero bytes.
        assert_eq!(&delete_now(150_000, "")[6..], [0; 6]);
        assert_eq!(delete_now(150_000, "19990131").len(), 12);

        let mut reply = vec![0u8; 14];
        reply[2..6].copy_from_slice(&150_000u32.to_le_bytes());
        reply[6..10].copy_from_slice(&1u32.to_le_bytes());
        reply[10..14].copy_from_slice(&86_400u32.to_le_bytes());
        assert_eq!(delete_reserved(&reply), Some((150_000, 1, 86_400)));
        assert_eq!(delete_result(&reply[..10]), Some((150_000, 1)));

        // A character whose wait is over arrives with seconds below zero.
        let mut info = character_bytes("Leaving", 0);
        info[158..162].copy_from_slice(&(-30i32).to_le_bytes());
        let mut list = vec![0u8; 27];
        list.extend(info);
        assert_eq!(char_list(&list)[0].delete_in, Some(-30));
    }

    #[test]
    fn reads_the_map_server_to_go_to() {
        let mut packet = vec![0u8; 156];
        packet[2..6].copy_from_slice(&150_000u32.to_le_bytes());
        packet[6..18].copy_from_slice(b"prontera.gat");
        packet[26..28].copy_from_slice(&5121u16.to_le_bytes());
        assert_eq!(zone_server(&packet), Some(ZoneServer { char_id: 150_000, map: "prontera".into(), port: 5121 }));
    }

    /// A walk request packs its cell as the server packs positions, and a move packs two (rAthena's WBUFPOS2).
    #[test]
    fn packs_and_reads_moves() {
        let request = walk_to(156, 191);
        assert_eq!(request[..2], [0x5f, 0x03]);
        let mut accept = vec![0u8; 13];
        accept[6..9].copy_from_slice(&request[2..5]);
        let read = map_accepted(&accept).unwrap();
        assert_eq!((read.x, read.y), (156, 191));
        assert_eq!(walk_to(1023, 1023)[2..], [0xff, 0xff, 0xf0]);

        let ((x0, y0), (x1, y1)) = ((156u16, 191u16), (163u16, 1000u16));
        let mut packet = vec![0u8; 12];
        packet[6] = (x0 >> 2) as u8;
        packet[7] = ((x0 << 6) | ((y0 >> 4) & 0x3f)) as u8;
        packet[8] = ((y0 << 4) | ((x1 >> 6) & 0x0f)) as u8;
        packet[9] = ((x1 << 2) | ((y1 >> 8) & 0x03)) as u8;
        packet[10] = y1 as u8;
        packet[11] = 0x88;
        assert_eq!(player_move(&packet), Some(((156, 191), (163, 1000))));
        assert_eq!(player_move(&packet[..10]), None);
    }

    #[test]
    fn reads_stops_map_changes_and_parameters() {
        let mut stop = vec![0u8; 10];
        stop[2..6].copy_from_slice(&2_000_000u32.to_le_bytes());
        stop[6..8].copy_from_slice(&150u16.to_le_bytes());
        stop[8..10].copy_from_slice(&180u16.to_le_bytes());
        assert_eq!(stop_move(&stop), Some((2_000_000, 150, 180)));

        let mut moved = vec![0u8; 22];
        moved[2..13].copy_from_slice(b"izlude.gat\0");
        moved[18..20].copy_from_slice(&128u16.to_le_bytes());
        moved[20..22].copy_from_slice(&114u16.to_le_bytes());
        assert_eq!(map_move(&moved), Some(("izlude".to_owned(), 128, 114)));

        let mut speed = vec![0u8; 8];
        speed[4..8].copy_from_slice(&150i32.to_le_bytes());
        assert_eq!(parameter(&speed), Some((PARAMETER_SPEED, 150)));
    }

    #[test]
    fn reads_the_status_window() {
        let mut packet = vec![0u8; 44];
        packet[2..4].copy_from_slice(&48u16.to_le_bytes());
        // Str 9 (2 points to raise), Luk 1 (2 points).
        packet[4] = 9;
        packet[5] = 2;
        packet[14] = 1;
        packet[15] = 2;
        packet[16..18].copy_from_slice(&25i16.to_le_bytes());
        packet[32..34].copy_from_slice(&180i16.to_le_bytes());
        packet[40..42].copy_from_slice(&672i16.to_le_bytes());
        let values = status(&packet);
        let value = |name: u16| values.iter().find(|(parameter, _)| *parameter == name).map(|(_, value)| *value);
        assert_eq!(value(parameter::STATUS_POINTS), Some(48));
        assert_eq!((value(parameter::STR), value(parameter::STR_COST)), (Some(9), Some(2)));
        assert_eq!((value(parameter::STR + 5), value(parameter::STR_COST + 5)), (Some(1), Some(2)));
        assert_eq!((value(parameter::ATK), value(parameter::HIT), value(parameter::ATTACK_DELAY)), (Some(25), Some(180), Some(672)));
        assert!(status(&packet[..40]).is_empty());

        let mut long = vec![0u8; 12];
        long[2] = 22;
        long[4..12].copy_from_slice(&5_000_000_000i64.to_le_bytes());
        assert_eq!(long_parameter(&long), Some((parameter::NEXT_BASE_EXP, 5_000_000_000)));

        let mut couple = vec![0u8; 14];
        couple[2] = 13;
        couple[6] = 9;
        couple[10] = 3;
        assert_eq!(stat(&couple), Some((parameter::STR, 9, 3)));
        assert_eq!(stat_cost(&[0xbe, 0, 32, 0, 3]), Some((parameter::STR_COST, 3)));
        assert_eq!(raise_stat(parameter::STR + 1, 1), [0xbb, 0x00, 14, 0, 1]);
    }

    #[test]
    fn builds_and_reads_chat() {
        let packet = say("Walker", "hello there");
        assert_eq!(packet[..2], [0xf3, 0x00]);
        assert_eq!(usize::from(u16_at(&packet, 2)), packet.len());
        assert_eq!(&packet[4..], b"Walker : hello there\0");

        let mut own = vec![0x8e, 0x00, 0, 0];
        own.extend(b"Walker : hi\0");
        assert_eq!(chat_text(ZC_NOTIFY_PLAYERCHAT, &own).as_deref(), Some("Walker : hi"));
        let mut other = vec![0x8d, 0x00, 0, 0, 1, 2, 3, 4];
        other.extend(b"Friend : hey\0");
        assert_eq!(chat_text(ZC_NOTIFY_CHAT, &other).as_deref(), Some("Friend : hey"));
        let mut loud = vec![0u8; 16];
        loud[4..8].copy_from_slice(&[0x00, 0xff, 0xff, 0x00]);
        loud.extend(b"Server restart\0");
        assert_eq!(chat_text(ZC_BROADCAST2, &loud).as_deref(), Some("Server restart"));
        assert_eq!(broadcast_colour(&loud), Some([0xff, 0xff, 0x00]));
        assert_eq!(chat_text(0x1234, &own), None);
    }

    /// The packed position, as rAthena's WBUFPOS writes it.
    #[test]
    fn reads_the_spawn_position() {
        let (x, y, direction) = (156u16, 191u16, 4u8);
        let mut packet = vec![0u8; 13];
        packet[2..6].copy_from_slice(&77u32.to_le_bytes());
        packet[6] = (x >> 2) as u8;
        packet[7] = ((x << 6) | ((y >> 4) & 0x3f)) as u8;
        packet[8] = ((y << 4) as u8) | (direction & 0x0f);
        assert_eq!(map_accepted(&packet), Some(MapAccepted { server_tick: 77, x: 156, y: 191, direction: 4 }));

        // The largest coordinates the format holds.
        packet[6..9].copy_from_slice(&[0xff, 0xff, 0xf0]);
        let accepted = map_accepted(&packet).unwrap();
        assert_eq!((accepted.x, accepted.y, accepted.direction), (1023, 1023, 0));
    }
}
