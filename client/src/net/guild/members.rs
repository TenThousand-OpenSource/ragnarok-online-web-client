//! The guild's members and its positions as the server lists them, and those it has expelled.

use super::*;

/// A member (ZC_MEMBERMGR_INFO).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Member {
    pub account: u32,
    pub character: u32,
    pub hair: u16,
    pub hair_colour: u16,
    pub sex: u16,
    pub job: u16,
    pub level: u16,
    /// The experience given to the guild.
    pub given: u32,
    pub online: bool,
    pub position: u32,
    /// When the member was last in the game, in seconds since 1970; 0 when unknown.
    pub seen: u32,
    pub name: String,
}

/// A position: what it is called, what it allows and what part of its experience goes to the guild.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Position {
    pub id: u32,
    pub name: String,
    pub invites: bool,
    pub expels: bool,
    /// Whether it opens the guild's storage.
    pub storage: bool,
    pub tax: u32,
}

/// ZC_MEMBERMGR_INFO: `u16 length`, then `u32 account, u32 character, u16 hair, u16 hair_colour, u16 sex,
/// u16 job, u16 level, u32 given, u32 online, u32 position, u32 seen, char name[24]` each.
pub fn members(packet: &[u8]) -> Vec<Member> {
    records(packet, 4, 58)
        .filter_map(|record| {
            Some(Member {
                account: long(record, 0)?,
                character: long(record, 4)?,
                hair: word(record, 8)?,
                hair_colour: word(record, 10)?,
                sex: word(record, 12)?,
                job: word(record, 14)?,
                level: word(record, 16)?,
                given: long(record, 18)?,
                online: long(record, 22)? != 0,
                position: long(record, 26)?,
                seen: long(record, 30)?,
                name: text_at(record, 34, 24)?,
            })
        })
        .collect()
}

/// ZC_UPDATE_CHARSTAT: `u32 account, u32 character, u32 online, u16 sex, u16 hair, u16 hair_colour`. Returns
/// the character and whether it is in the game.
pub fn member_online(packet: &[u8]) -> Option<(u32, bool)> {
    Some((long(packet, 6)?, long(packet, 10)? != 0))
}

/// ZC_POSITION_ID_NAME_INFO: `u16 length`, then `u32 position, char name[24]` each.
pub fn position_names(packet: &[u8]) -> Vec<(u32, String)> {
    records(packet, 4, 28).filter_map(|record| Some((long(record, 0)?, text_at(record, 4, 24)?))).collect()
}

/// ZC_POSITION_INFO: `u16 length`, then `u32 position, u32 mode, u32 ranking, u32 tax` each; `mode` allows
/// inviting (1), expelling (0x10) and the guild's storage (0x100). The names are not in it.
pub fn position_rights(packet: &[u8]) -> Vec<Position> {
    records(packet, 4, 16)
        .filter_map(|record| {
            let mode = long(record, 4)?;
            Some(Position { id: long(record, 0)?, name: String::new(), invites: mode & 0x01 != 0, expels: mode & 0x10 != 0, storage: mode & 0x100 != 0, tax: long(record, 12)? })
        })
        .collect()
}

/// ZC_ACK_CHANGE_GUILD_POSITIONINFO: `u16 length`, then `u32 position, u32 mode, u32 ranking, u32 tax,
/// char name[24]` each.
pub fn positions_changed(packet: &[u8]) -> Vec<Position> {
    records(packet, 4, 40)
        .filter_map(|record| {
            let mode = long(record, 4)?;
            Some(Position { id: long(record, 0)?, name: text_at(record, 16, 24)?, invites: mode & 0x01 != 0, expels: mode & 0x10 != 0, storage: mode & 0x100 != 0, tax: long(record, 12)? })
        })
        .collect()
}

/// ZC_ACK_REQ_CHANGE_MEMBERS: `u16 length`, then `u32 account, u32 character, u32 position` each. Returns each
/// character and its position.
pub fn members_moved(packet: &[u8]) -> Vec<(u32, u32)> {
    records(packet, 4, 12).filter_map(|record| Some((long(record, 4)?, long(record, 8)?))).collect()
}

/// ZC_BAN_LIST: `u16 length`, then `u32 character, char reason[40], char name[24]` each. Returns each name and
/// reason.
pub fn expelled_list(packet: &[u8]) -> Vec<(String, String)> {
    records(packet, 4, 68).filter_map(|record| Some((text_at(record, 44, 24)?, text_at(record, 4, 40)?))).collect()
}
