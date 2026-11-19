//! What the client sends about the guild: asking for its tabs and names, making one, inviting and answering, leaving and expelling, breaking it up, its chat and its notice.

use super::*;

/// CZ_REQNAME (0x0368, 6 bytes): `u32 id`: asks what someone in sight is called.
pub fn ask_names(id: u32) -> Vec<u8> {
    [&[0x68, 0x03][..], &id.to_le_bytes()].concat()
}

/// CZ_REQ_MAKE_GUILD (0x0165, 30 bytes): `u32 character, char name[24]`.
pub fn make(character: u32, name: &str) -> Vec<u8> {
    [&[0x65, 0x01][..], &character.to_le_bytes(), &field::<24>(name)].concat()
}

/// CZ_REQ_GUILD_MENUINTERFACE (0x014d, 2 bytes): asks which tabs the player may open.
pub fn ask_tabs() -> Vec<u8> {
    vec![0x4d, 0x01]
}

/// CZ_REQ_GUILD_MENU (0x014f, 6 bytes): `u32 what`: asks for what a tab shows (`menu`).
pub fn ask(what: u32) -> Vec<u8> {
    [&[0x4f, 0x01][..], &what.to_le_bytes()].concat()
}

/// CZ_REQ_GUILD_EMBLEM_IMG (0x0151, 6 bytes): `u32 guild`: asks for a guild's emblem.
pub fn ask_emblem(guild: u32) -> Vec<u8> {
    [&[0x51, 0x01][..], &guild.to_le_bytes()].concat()
}

/// CZ_REQ_JOIN_GUILD (0x0168, 14 bytes): `u32 account, u32 own_account, u32 own_character`: invites someone
/// in sight.
pub fn invite(account: u32, own_account: u32, own_character: u32) -> Vec<u8> {
    [&[0x68, 0x01][..], &account.to_le_bytes(), &own_account.to_le_bytes(), &own_character.to_le_bytes()].concat()
}

/// CZ_REQ_JOIN_GUILD2 (0x0916, 26 bytes): `char name[24]`: invites a player by name (`/guildinvite`).
pub fn invite_named(name: &str) -> Vec<u8> {
    [&[0x16, 0x09][..], &field::<24>(name)].concat()
}

/// CZ_JOIN_GUILD (0x016b, 10 bytes): `u32 guild, u32 answer`: the invitation is taken (1) or not (0).
pub fn answer(guild: u32, accept: bool) -> Vec<u8> {
    [&[0x6b, 0x01][..], &guild.to_le_bytes(), &u32::from(accept).to_le_bytes()].concat()
}

/// CZ_REQ_LEAVE_GUILD (0x0159, 54 bytes): `u32 guild, u32 account, u32 character, char reason[40]`.
pub fn leave(guild: u32, account: u32, character: u32, reason: &str) -> Vec<u8> {
    [&[0x59, 0x01][..], &guild.to_le_bytes(), &account.to_le_bytes(), &character.to_le_bytes(), &field::<40>(reason)].concat()
}

/// CZ_REQ_BAN_GUILD (0x015b, 54 bytes): `u32 guild, u32 account, u32 character, char reason[40]`.
pub fn expel(guild: u32, account: u32, character: u32, reason: &str) -> Vec<u8> {
    [&[0x5b, 0x01][..], &guild.to_le_bytes(), &account.to_le_bytes(), &character.to_le_bytes(), &field::<40>(reason)].concat()
}

/// CZ_REQ_DISORGANIZE_GUILD (0x015d, 42 bytes): `char key[40]`: breaks the guild up; the key is the guild's
/// name (rAthena's `guild_break`).
pub fn break_up(key: &str) -> Vec<u8> {
    [&[0x5d, 0x01][..], &field::<40>(key)].concat()
}

/// CZ_GUILD_CHAT (0x017e): `u16 length, char text[]`: a line for the guild, "name : words", as the party's is
/// sent.
pub fn say(name: &str, words: &str) -> Vec<u8> {
    let text = format!("{name} : {words}");
    [&[0x7e, 0x01][..], &((text.len() + 4) as u16).to_le_bytes(), text.as_bytes()].concat()
}

/// CZ_GUILD_NOTICE (0x016e, 186 bytes): `u32 guild, char subject[60], char notice[120]`.
pub fn set_notice(guild: u32, subject: &str, notice: &str) -> Vec<u8> {
    [&[0x6e, 0x01][..], &guild.to_le_bytes(), &field::<60>(subject), &field::<120>(notice)].concat()
}
