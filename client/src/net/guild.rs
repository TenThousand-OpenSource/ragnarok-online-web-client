//! The guild: what it is, who is in it and in which position, its notice, its chat, and the packets that make,
//! join, leave and break one. The layouts are rAthena's for PACKETVER 20260219 (`clif_guild_*` and
//! `clif_parse_Guild*` in `clif.cpp`, `packets_struct.hpp`, `clif_packetdb.hpp`).

use super::bytes::{Writer, body, long, records, text, text_at, word};

/// ZC_RESULT_MAKE_GUILD (0x0167, 3 bytes): `u8 result`: how making a guild went (`made`).
pub const ZC_RESULT_MAKE_GUILD: u16 = 0x0167;
/// ZC_UPDATE_GDID (0x02f7, 47 bytes): the guild the player is of, and what the player may do in it.
pub const ZC_UPDATE_GDID: u16 = 0x02f7;
/// ZC_UPDATE_CHARSTAT (0x01f2, 20 bytes): a member came online or went offline.
pub const ZC_UPDATE_CHARSTAT: u16 = 0x01f2;
/// ZC_ACK_GUILD_MENUINTERFACE (0x014e, 6 bytes): `u32 tabs`: which tabs of the guild window the player may open.
pub const ZC_ACK_GUILD_MENUINTERFACE: u16 = 0x014e;
/// ZC_GUILD_INFO (0x0b7b, 118 bytes): the guild's level, numbers and master.
pub const ZC_GUILD_INFO: u16 = 0x0b7b;
/// ZC_MYGUILD_BASIC_INFO (0x014c): the guilds allied and opposed.
pub const ZC_MYGUILD_BASIC_INFO: u16 = 0x014c;
/// ZC_MEMBERMGR_INFO (0x0b7d): every member.
pub const ZC_MEMBERMGR_INFO: u16 = 0x0b7d;
/// ZC_POSITION_ID_NAME_INFO (0x0166): what the positions are called.
pub const ZC_POSITION_ID_NAME_INFO: u16 = 0x0166;
/// ZC_POSITION_INFO (0x0160): what each position may do and pays.
pub const ZC_POSITION_INFO: u16 = 0x0160;
/// ZC_ACK_CHANGE_GUILD_POSITIONINFO (0x0174): positions were changed.
pub const ZC_ACK_CHANGE_GUILD_POSITIONINFO: u16 = 0x0174;
/// ZC_ACK_REQ_CHANGE_MEMBERS (0x0156): members were given other positions.
pub const ZC_ACK_REQ_CHANGE_MEMBERS: u16 = 0x0156;
/// ZC_GUILD_EMBLEM_IMG (0x0152): a guild's emblem, a BMP packed with zlib.
pub const ZC_GUILD_EMBLEM_IMG: u16 = 0x0152;
/// ZC_CHANGE_GUILD (0x0b47, 14 bytes): someone in sight is of another guild now, or their guild's emblem changed.
pub const ZC_CHANGE_GUILD: u16 = 0x0b47;
/// ZC_GUILD_SKILLINFO (0x0162): the guild's skills.
pub const ZC_GUILD_SKILLINFO: u16 = 0x0162;
/// ZC_GUILD_NOTICE (0x016f, 182 bytes): `char subject[60], char notice[120]`.
pub const ZC_GUILD_NOTICE: u16 = 0x016f;
/// ZC_REQ_JOIN_GUILD (0x016a, 30 bytes): a guild invites the player.
pub const ZC_REQ_JOIN_GUILD: u16 = 0x016a;
/// ZC_ACK_REQ_JOIN_GUILD (0x0169, 3 bytes): `u8 answer`: what became of the player's own invitation (`invite`).
pub const ZC_ACK_REQ_JOIN_GUILD: u16 = 0x0169;
/// ZC_ACK_LEAVE_GUILD (0x0a83, 46 bytes): `u32 character, char reason[40]`: a member left.
pub const ZC_ACK_LEAVE_GUILD: u16 = 0x0a83;
/// ZC_ACK_BAN_GUILD (0x0a82, 46 bytes): `char reason[40], u32 character`: a member was expelled.
pub const ZC_ACK_BAN_GUILD: u16 = 0x0a82;
/// ZC_BAN_LIST (0x0b7c): those expelled, and why.
pub const ZC_BAN_LIST: u16 = 0x0b7c;
/// ZC_GUILD_CHAT (0x017f): a line of guild chat.
pub const ZC_GUILD_CHAT: u16 = 0x017f;
/// ZC_ACK_DISORGANIZE_GUILD_RESULT (0x015e, 6 bytes): `u32 result`: how breaking the guild up went (`broken`).
pub const ZC_ACK_DISORGANIZE_GUILD_RESULT: u16 = 0x015e;
/// ZC_GUILD_POSITION (0x0afd): `u16 length, u32 account, char position[]`: the position someone in sight has
/// in their guild.
pub const ZC_GUILD_POSITION: u16 = 0x0afd;
/// ZC_ACK_REQNAMEALL (0x0a30, 106 bytes): a player's name with the names of the party and the guild and the
/// position in the guild, answered to CZ_REQNAME.
pub const ZC_ACK_REQNAMEALL: u16 = 0x0a30;

/// What ZC_RESULT_MAKE_GUILD answers (the client's `GM_*`).
pub mod made {
    pub const DONE: u8 = 0;
    pub const ALREADY_IN_ONE: u8 = 1;
    pub const SAME_NAME: u8 = 2;
    /// The Emperium is missing.
    pub const NO_ITEM: u8 = 3;
    pub const NOT_HERE: u8 = 4;
}

/// What ZC_ACK_REQ_JOIN_GUILD answers (the client's `REQJOINGUILD_*`).
pub mod invite {
    pub const IN_OTHER_GUILD: u8 = 0;
    pub const REJECTED: u8 = 1;
    pub const ACCEPTED: u8 = 2;
    pub const FULL: u8 = 3;
}

/// What ZC_ACK_DISORGANIZE_GUILD_RESULT answers (the client's `DISRESULT_*`).
pub mod broken {
    pub const DONE: u32 = 0;
    pub const WRONG_KEY: u32 = 1;
    pub const MEMBERS_LEFT: u32 = 2;
}

/// What CZ_REQ_GUILD_MENU asks for (the client's `MM_REQ_GUILD_MENU`).
pub mod menu {
    pub const INFO: u32 = 0;
    pub const MEMBERS: u32 = 1;
    pub const POSITIONS: u32 = 2;
    pub const SKILLS: u32 = 3;
    pub const EXPELLED: u32 = 4;
}

/// The guild the player is of (ZC_UPDATE_GDID).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Belonging {
    pub guild: u32,
    pub emblem: u32,
    /// What the player's position allows: inviting, expelling.
    pub invites: bool,
    pub expels: bool,
    pub master: bool,
    pub name: String,
    /// The master's character.
    pub master_character: u32,
}

/// The guild's numbers (ZC_GUILD_INFO).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Info {
    pub guild: u32,
    pub level: u32,
    pub online: u32,
    pub most: u32,
    pub average_level: u32,
    pub exp: u32,
    pub next_exp: u32,
    pub emblem: u32,
    pub name: String,
    /// The castles it holds, as the server words it.
    pub land: String,
    pub master_character: u32,
    pub master: String,
}

/// A skill of the guild (ZC_GUILD_SKILLINFO).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Skill {
    pub id: u16,
    /// How it is used: 0 for one that is not (the client's "Passive").
    pub kind: u32,
    pub level: u16,
    pub sp: u16,
    pub range: u16,
    pub name: String,
    pub raisable: bool,
}

impl Skill {
    /// The same skill as the skills of the character are kept, for using it as they are used.
    pub fn as_known(&self) -> crate::net::items::Skill {
        crate::net::items::Skill { id: self.id, target: self.kind, level: self.level, sp: self.sp, range: self.range, raisable: self.raisable }
    }
}

/// ZC_RESULT_MAKE_GUILD, ZC_ACK_REQ_JOIN_GUILD, ZC_ACK_REQ_ALLY_GUILD and ZC_ACK_REQ_HOSTILE_GUILD: the one byte
/// each answers with (`made`, `invite`, `ally`, `oppose`), if the packet has it.
pub fn result(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// ZC_ACK_GUILD_MENUINTERFACE: which tabs of the guild window the player may open.
pub fn tabs(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// ZC_ACK_DISORGANIZE_GUILD_RESULT: how breaking the guild up went (`broken`).
pub fn broken_how(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// ZC_UPDATE_GDID: `u32 guild, u32 emblem, u32 mode, u8 master, u32 inter, char name[24], u32 master_character`;
/// `mode` allows inviting (1) and expelling (0x10).
pub fn belonging(packet: &[u8]) -> Option<Belonging> {
    let mode = long(packet, 10)?;
    Some(Belonging {
        guild: long(packet, 2)?,
        emblem: long(packet, 6)?,
        invites: mode & 0x01 != 0,
        expels: mode & 0x10 != 0,
        master: *packet.get(14)? != 0,
        name: text_at(packet, 19, 24)?,
        master_character: long(packet, 43)?,
    })
}

/// ZC_GUILD_INFO: `u32 guild, level, online, most, average_level, exp, next_exp, tax_points, honor, virtue,
/// emblem, char name[24], char land[16], u32 zeny, u32 master_character, char master[24]`.
pub fn info(packet: &[u8]) -> Option<Info> {
    Some(Info {
        guild: long(packet, 2)?,
        level: long(packet, 6)?,
        online: long(packet, 10)?,
        most: long(packet, 14)?,
        average_level: long(packet, 18)?,
        exp: long(packet, 22)?,
        next_exp: long(packet, 26)?,
        emblem: long(packet, 42)?,
        name: text_at(packet, 46, 24)?,
        land: text_at(packet, 70, 16)?,
        master_character: long(packet, 90)?,
        master: text_at(packet, 94, 24)?,
    })
}

/// ZC_MYGUILD_BASIC_INFO: `u16 length`, then `u32 relation, u32 guild, char name[24]` each: the guilds allied
/// (relation 0) and opposed (1). Returns each guild, its name, and whether it is opposed.
pub fn related(packet: &[u8]) -> Vec<(u32, String, bool)> {
    records(packet, 4, 32).filter_map(|record| Some((long(record, 4)?, text_at(record, 8, 24)?, long(record, 0)? != 0))).collect()
}

/// ZC_GUILD_EMBLEM_IMG: `u16 length, u32 guild, u32 emblem`, then the picture as rAthena keeps it: a BMP packed
/// with zlib. Returns the guild, the emblem's version and the packed picture.
pub fn emblem(packet: &[u8]) -> Option<(u32, u32, &[u8])> {
    Some((long(packet, 4)?, long(packet, 8)?, packet.get(12..)?))
}

/// An emblem as a picture: unpacked (zlib) and read as the BMP it is, 24 pixels each way, magenta see-through.
/// Nothing for what is no such picture, or one larger than any emblem (rAthena lets none past 1800 bytes
/// unpacked, `clif_validate_emblem`).
pub fn emblem_picture(packed: &[u8]) -> Option<crate::bmp::Image> {
    let bitmap = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(packed, 4096).ok()?;
    crate::bmp::decode(&bitmap).ok().filter(|picture| picture.width <= 24 && picture.height <= 24)
}

/// ZC_CHANGE_GUILD: `u32 guild, u32 emblem, u32 account`. Returns who, their guild and its emblem's version.
pub fn changed(packet: &[u8]) -> Option<(u32, u32, u32)> {
    Some((long(packet, 10)?, long(packet, 2)?, long(packet, 6)?))
}

/// ZC_GUILD_SKILLINFO: `u16 length, u16 points`, then `u16 id, u32 kind, u16 level, u16 sp, u16 range,
/// char name[24], u8 raisable` each. Returns the points left to give and the skills.
pub fn skills(packet: &[u8]) -> Option<(u16, Vec<Skill>)> {
    let skills = records(packet, 6, 37)
        .filter_map(|record| Some(Skill { id: word(record, 0)?, kind: long(record, 2)?, level: word(record, 6)?, sp: word(record, 8)?, range: word(record, 10)?, name: text_at(record, 12, 24)?, raisable: *record.get(36)? != 0 }))
        .collect();
    Some((word(packet, 4)?, skills))
}

/// ZC_GUILD_NOTICE: the subject and the notice.
pub fn notice(packet: &[u8]) -> Option<(String, String)> {
    Some((text_at(packet, 2, 60)?, text_at(packet, 62, 120)?))
}

/// ZC_REQ_JOIN_GUILD: `u32 guild, char name[24]`.
pub fn invited(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 2)?, text_at(packet, 6, 24)?))
}

/// ZC_ACK_LEAVE_GUILD: who left (the character) and the reason given.
pub fn left(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 2)?, text_at(packet, 6, 40)?))
}

/// ZC_ACK_BAN_GUILD: who was expelled (the character) and the reason given.
pub fn expelled(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 42)?, text_at(packet, 2, 40)?))
}

/// ZC_GUILD_CHAT: `u16 length, char text[]`: "name : words".
pub fn said(packet: &[u8]) -> Option<String> {
    body(packet, 4).map(text)
}

/// ZC_GUILD_POSITION: who, and the position they have in their guild.
pub fn position_of(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 4)?, text_at(packet, 8, packet.len().checked_sub(8)?)?))
}

/// What ZC_ACK_REQNAMEALL tells of a player.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Names {
    pub name: String,
    pub party: String,
    pub guild: String,
    pub position: String,
}

/// ZC_ACK_REQNAMEALL: `u32 id, char name[24], char party[24], char guild[24], char position[24], u32 title`.
pub fn names(packet: &[u8]) -> Option<(u32, Names)> {
    Some((long(packet, 2)?, Names { name: text_at(packet, 6, 24)?, party: text_at(packet, 30, 24)?, guild: text_at(packet, 54, 24)?, position: text_at(packet, 78, 24)? }))
}

mod manage;
mod members;
mod requests;
pub use {manage::*, members::*, requests::*};

#[cfg(test)]
mod tests;
