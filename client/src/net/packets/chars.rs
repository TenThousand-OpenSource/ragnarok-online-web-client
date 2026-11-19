//! The character server: the list of characters, making and deleting one, and the map server to go to.

use super::*;

/// CH_ENTER (0x0065): `u32 AID, u32 login_id1, u32 login_id2, u16 unused, u8 sex`.
pub fn char_enter(account: &LoginAccepted) -> Vec<u8> {
    Writer::new(0x0065).u32(account.account_id).u32(account.login_id1).u32(account.login_id2).u16(0).u8(account.sex).done()
}

/// PING (0x0187): `u32 AID`. Keeps the char server connection open.
pub fn ping(account_id: u32) -> Vec<u8> {
    Writer::new(0x0187).u32(account_id).done()
}

/// CH_SELECT_CHAR (0x0066): `u8 slot`.
pub fn select_char(slot: u8) -> Vec<u8> {
    Writer::new(0x0066).u8(slot).done()
}

/// CH_MAKE_CHAR (0x0a39): `char[24] name, u8 slot, u16 hair_color, u16 hair_style, u32 job, u8 sex`.
pub fn make_char(name: &str, slot: u8, hair_color: u16, hair_style: u16, job: u32, sex: u8) -> Vec<u8> {
    Writer::new(0x0a39).text(name, NAME_LENGTH).u8(slot).u16(hair_color).u16(hair_style).u32(job).u8(sex).done()
}

/// HC_ACCEPT_ENTER2 (0x082d): `u16 length, u8 normal, u8 premium, u8 billing, u8 producible, u8 total, char[20]`.
/// Returns how many slots the account may create characters in, and how many slots there are in all.
pub fn char_slots(packet: &[u8]) -> Option<(u8, u8)> {
    Some((*packet.get(7)?, *packet.get(8)?))
}

/// One `CHARACTER_INFO`, 175 bytes at this PACKETVER.
pub(super) fn character(info: &[u8]) -> Character {
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
        name: name_at(info, 108, NAME_LENGTH),
        stats: info[132..138].try_into().unwrap(),
        slot: info[138],
        hair_color: info[139],
        headgear: [u16_at(info, 96), u16_at(info, 100), u16_at(info, 102)],
        body_palette: u16_at(info, 106),
        // Zero stands for "not being deleted"; a date already passed arrives as a negative number.
        delete_in: Some(i32_at(info, 158)).filter(|seconds| *seconds != 0),
        map: map_name(&name_at(info, 142, 16)),
        sex: info[174],
    }
}

pub(super) fn map_name(name: &str) -> String {
    name.strip_suffix(".gat").unwrap_or(name).to_owned()
}

/// HC_ACCEPT_ENTER (0x006b): `u16 length, u8 total, u8 premium_start, u8 premium_end, char[20]`, then the
/// account's characters.
pub fn char_list(packet: &[u8]) -> Vec<Character> {
    records(packet, 27, CHARACTER_INFO).map(character).collect()
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
    Writer::new(0x0827).u32(char_id).done()
}

/// CH_DELETE_CHAR3 (0x0829): `u32 CID, char[6] birthdate` (YYMMDD). Deletes a character whose wait is over. An
/// account without a birthdate sends none.
pub fn delete_now(char_id: u32, birthdate: &str) -> Vec<u8> {
    let digits = &birthdate.as_bytes()[..birthdate.len().min(6)];
    Writer::new(0x0829).u32(char_id).bytes(digits).bytes(&[0; 6][digits.len()..]).done()
}

/// CH_DELETE_CHAR3_CANCEL (0x082b): `u32 CID`.
pub fn delete_cancel(char_id: u32) -> Vec<u8> {
    Writer::new(0x082b).u32(char_id).done()
}

/// HC_DELETE_CHAR3_RESERVED (0x0828): `u32 CID, u32 result, u32 seconds`. Result 1 is success, and then the
/// seconds are how long the deletion waits; 3 is a character not found, 4 one in a guild, 5 one in a party.
pub fn delete_reserved(packet: &[u8]) -> Option<(u32, u32, i32)> {
    (packet.len() >= 14).then(|| (u32_at(packet, 2), u32_at(packet, 6), i32_at(packet, 10)))
}

/// HC_DELETE_CHAR3 (0x082a) and HC_DELETE_CHAR3_CANCEL (0x082c): `u32 CID, u32 result`. Result 1 is success; for
/// a deletion, 4 is a wait not over yet and 5 a wrong birthdate.
pub fn delete_result(packet: &[u8]) -> Option<(u32, u32)> {
    (packet.len() >= 10).then(|| (u32_at(packet, 2), u32_at(packet, 6)))
}

/// HC_NOTIFY_ZONESVR (0x0ac5): `u32 CID, char[16] map, u32 ip, u16 port, char[128] domain`.
pub fn zone_server(packet: &[u8]) -> Option<ZoneServer> {
    (packet.len() >= 28).then(|| ZoneServer { char_id: u32_at(packet, 2), map: map_name(&name_at(packet, 6, 16)), port: u16_at(packet, 26) })
}
