//! The basics of the map server: entering a map, its clock, walking, the character's values and raising a
//! stat, the chat, and leaving the game or going back to the characters.

use super::*;

/// ZC_NOTIFY_TIME (0x007f): `u32 time`: the server's time, in answer to CZ_REQUEST_TIME.
pub const ZC_NOTIFY_TIME: u16 = 0x007f;
/// ZC_STATUS_CHANGE_ACK (0x00bc, 6 bytes): `u16 stat, u8 done, u8 value`: the answer to raising a stat.
pub const ZC_STATUS_CHANGE_ACK: u16 = 0x00bc;
/// ZC_NOTIFY_EFFECT (0x019b, 10 bytes): `u32 who, u32 kind`: something to show on someone: a level gained, a
/// refining, a potion brewed.
pub const ZC_NOTIFY_EFFECT: u16 = 0x019b;
/// ZC_RESTART_ACK (0x00b3, 3 bytes): `u8 type`: the answer to CZ_RESTART.
pub const ZC_RESTART_ACK: u16 = 0x00b3;
/// ZC_ACK_REQ_DISCONNECT (0x018b, 4 bytes): `u16 result`: whether the character may leave the game now
/// (rAthena's `clif_disconnect_ack`; its headers give the packet no name).
pub const ZC_ACK_REQ_DISCONNECT: u16 = 0x018b;

/// A cell and a direction out of the three bytes the server packs them into (rAthena's `WBUFPOS`): x in 10
/// bits, y in 10 bits, the direction in 4. The bytes are three at least.
pub(in crate::net) fn cell_unpacked(packed: &[u8]) -> ((u16, u16), u8) {
    let x = u16::from(packed[0]) << 2 | u16::from(packed[1]) >> 6;
    let y = (u16::from(packed[1]) & 0x3f) << 4 | u16::from(packed[2]) >> 4;
    ((x, y), packed[2] & 0x0f)
}

/// A cell as the three bytes a request packs it into, the direction left at nought.
pub(in crate::net) fn cell_packed((x, y): (u16, u16)) -> [u8; 3] {
    [(x >> 2) as u8, ((x << 6) | ((y >> 4) & 0x3f)) as u8, (y << 4) as u8]
}

/// What many small packets are: a `u16` and an `i32` after the packet's number (a parameter and its value, a
/// line of the message table and the number to write into it); none of a packet under 8 bytes.
pub(in crate::net) fn number_and_value(packet: &[u8]) -> Option<(u16, i32)> {
    (packet.len() >= 8).then(|| (u16_at(packet, 2), i32_at(packet, 4)))
}

/// CZ_ENTER (0x0436): `u32 AID, u32 CID, u32 login_id1, u32 client_tick, u8[4] unused, u8 sex`.
pub fn map_enter(account: &LoginAccepted, char_id: u32, tick: u32) -> Vec<u8> {
    Writer::new(0x0436).u32(account.account_id).u32(char_id).u32(account.login_id1).u32(tick).u32(0).u8(account.sex).done()
}

/// CZ_NOTIFY_ACTORINIT (0x007d): the map is loaded; the server puts the character into the world.
pub fn map_loaded() -> Vec<u8> {
    Writer::new(0x007d).done()
}

/// CZ_REQUEST_TIME (0x0360): `u32 client_tick`. Asks the map server its time, and keeps the connection open.
pub fn tick(tick: u32) -> Vec<u8> {
    Writer::new(0x0360).u32(tick).done()
}

/// ZC_NOTIFY_TIME: the server's time.
pub fn server_time(packet: &[u8]) -> Option<u32> {
    (packet.len() >= 6).then(|| u32_at(packet, 2))
}

/// CZ_REQUEST_MOVE (0x035f): the cell to walk to, packed as a position.
pub fn walk_to(x: u16, y: u16) -> Vec<u8> {
    Writer::new(0x035f).bytes(&cell_packed((x, y))).done()
}

/// ZC_NOTIFY_PLAYERMOVE (0x0087): `u32 start_time, u8[6] move`. The move packs the cell walked from and the cell
/// walked to, 10 bits a coordinate (`actors::walked`).
pub fn player_move(packet: &[u8]) -> Option<crate::net::actors::Walked> {
    crate::net::actors::walked(packet.get(6..12)?, u32_at(packet, 2))
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
    number_and_value(packet)
}

/// ZC_LONGLONGPAR_CHANGE (0x0acb): `u16 parameter, i64 value`: experience.
pub fn long_parameter(packet: &[u8]) -> Option<(u16, i64)> {
    (packet.len() >= 12).then(|| (u16_at(packet, 2), i64_at(packet, 4)))
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
        values.push((name, i64::from(i16_at(packet, 16 + index * 2))));
    }
    values
}

/// ZC_STATUS_CHANGE (0x00be): `u16 parameter, u8 value`: what raising a stat costs now.
pub fn stat_cost(packet: &[u8]) -> Option<(u16, u8)> {
    (packet.len() >= 5).then(|| (u16_at(packet, 2), packet[4]))
}

/// ZC_COUPLESTATUS (0x0141): `u32 parameter, i32 base, i32 bonus`: a stat, and what equipment and the like add.
pub fn stat(packet: &[u8]) -> Option<(u16, i32, i32)> {
    (packet.len() >= 14).then(|| (u32_at(packet, 2) as u16, i32_at(packet, 6), i32_at(packet, 10)))
}

/// CZ_STATUS_CHANGE (0x00bb): `u16 parameter, u8 amount`: raises a stat.
pub fn raise_stat(parameter: u16, amount: u8) -> Vec<u8> {
    Writer::new(0x00bb).u16(parameter).u8(amount).done()
}

/// ZC_STATUS_CHANGE_ACK: the stat that was raised (`parameter::STR` and the five after it) and what it is
/// now; nothing when the server did not raise it (`CGameMode::ReceiveStatusChangeAck`).
pub fn stat_raised(packet: &[u8]) -> Option<(u16, u8)> {
    let packet = packet.get(..6)?;
    (packet[4] != 0).then_some((u16_at(packet, 2), packet[5]))
}

/// ZC_NOTIFY_EFFECT: on whom, if the packet reaches so far, and which kind (`u32::MAX`, which is no kind, of a
/// packet that ends before it).
pub fn effect_on(packet: &[u8]) -> (Option<u32>, u32) {
    (long(packet, 2), long(packet, 6).unwrap_or(u32::MAX))
}

/// CZ_REQUEST_CHAT (0x00f3): `u16 length, char[] text`: says something to everyone near. The text is the
/// character's name, " : ", and the words, with a terminator.
pub fn say(name: &str, words: &str) -> Vec<u8> {
    Writer::sized(0x00f3).bytes(format!("{name} : {words}").as_bytes()).u8(0).done()
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

/// Who says a line of ZC_NOTIFY_CHAT (0x008d): the ID after the length.
pub fn chat_speaker(packet: &[u8]) -> Option<u32> {
    long(packet, 4)
}

/// ZC_ACCEPT_ENTER (0x02eb): `u32 start_time, u8[3] position, u8 x_size, u8 y_size, u16 font`. The position packs
/// x (10 bits), y (10 bits) and the direction (4 bits).
pub fn map_accepted(packet: &[u8]) -> Option<MapAccepted> {
    let ((x, y), direction) = cell_unpacked(packet.get(6..9)?);
    Some(MapAccepted { server_tick: u32_at(packet, 2), x, y, direction })
}

/// CZ_RESTART (0x00b2): `u8 type`: 0 returns a dead character to its save point, 1 goes back to the characters.
pub fn restart(to_characters: bool) -> Vec<u8> {
    Writer::new(0x00b2).u8(u8::from(to_characters)).done()
}

/// ZC_RESTART_ACK: whether the server lets the character out of the game, back to the characters (type 1).
pub fn restart_allowed(packet: &[u8]) -> bool {
    packet.get(2) == Some(&1)
}

/// ZC_ACK_REQ_DISCONNECT (0x018b, 4 bytes: `u16 result`): whether leaving was refused (a fight just now, for
/// one). The client says so for the result 1 alone (`NOTDISCONNECTABLE_STATE`; 0 lets it out, and it has no
/// word for any other); a packet that ends before its result tells nothing.
pub fn leave_refused(packet: &[u8]) -> bool {
    word(packet, 2) == Some(1)
}
