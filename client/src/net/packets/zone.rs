//! The basics of the map server: entering a map, walking, the character's values, chat.

use super::*;

/// CZ_ENTER (0x0436): `u32 AID, u32 CID, u32 login_id1, u32 client_tick, u8[4] unused, u8 sex`.
pub fn map_enter(account: &LoginAccepted, char_id: u32, tick: u32) -> Vec<u8> {
    Writer::new(0x0436).u32(account.account_id).u32(char_id).u32(account.login_id1).u32(tick).u32(0).u8(account.sex).0
}

/// CZ_NOTIFY_ACTORINIT (0x007d): the map is loaded; the server puts the character into the world.
pub fn map_loaded() -> Vec<u8> {
    Writer::new(0x007d).0
}

/// CZ_REQUEST_TIME (0x0360): `u32 client_tick`. Asks the map server its time, and keeps the connection open.
pub fn tick(tick: u32) -> Vec<u8> {
    Writer::new(0x0360).u32(tick).0
}

/// ZC_NOTIFY_TIME (0x007f): `u32 time`: the server's time, in answer to CZ_REQUEST_TIME.
pub const ZC_NOTIFY_TIME: u16 = 0x007f;

pub fn server_time(packet: &[u8]) -> Option<u32> {
    (packet.len() >= 6).then(|| u32_at(packet, 2))
}

/// CZ_REQUEST_MOVE (0x035f): the cell to walk to, packed as a position.
pub fn walk_to(x: u16, y: u16) -> Vec<u8> {
    let mut packet = Writer::new(0x035f).0;
    packet.extend([(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, (y << 4) as u8]);
    packet
}

/// ZC_NOTIFY_PLAYERMOVE (0x0087): `u32 start_time, u8[6] move`. The move packs the cell walked from and the cell
/// walked to, 10 bits a coordinate (`units::walked`).
pub fn player_move(packet: &[u8]) -> Option<crate::net::units::Walked> {
    crate::net::units::walked(packet.get(6..12)?, u32_at(packet, 2))
}

/// ZC_STOPMOVE (0x0088): `u32 id, u16 x, u16 y`: a unit stands on that cell now, wherever it was thought to be.
pub fn stop_move(packet: &[u8]) -> Option<(u32, u16, u16)> {
    (packet.len() >= 10).then(|| (u32_at(packet, 2), u16_at(packet, 6), u16_at(packet, 8)))
}

/// ZC_NPCACK_MAPMOVE (0x0091): `char[16] map, u16 x, u16 y`: the character is moved to another place, on this map
/// or another of the same server.
pub fn map_move(packet: &[u8]) -> Option<(String, u16, u16)> {
    (packet.len() >= 22).then(|| (map_name(&name_at(packet, 2, 16)), u16_at(packet, 18), u16_at(packet, 20)))
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
/// ZC_NOTIFY_CHAT (0x008d, someone near: `u16 length, u32 id, char[]`). (The server's announcements are
/// read in `net/broadcast.rs`.)
pub fn chat_text(id: u16, packet: &[u8]) -> Option<String> {
    let from = match id {
        ZC_NOTIFY_PLAYERCHAT => 4,
        ZC_NOTIFY_CHAT => 8,
        _ => return None,
    };
    let text = packet.get(from..)?;
    Some(name_at(text, 0, text.len()))
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
