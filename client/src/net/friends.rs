//! The friends list: the list sent on entering, who of it is in the game, asking someone to be a friend and
//! answering one who asks, and removing a friend. The layouts are rAthena's for PACKETVER 20260219
//! (`clif_friendslist_*` and `clif_parse_FriendsList*` in `clif.cpp`, `packets.hpp`); what the client does with
//! each is `CGameMode::ReceiveFriendList`, `ReceiveFriendAddResult` and the arms of its packet switch.

use super::bytes::{Writer, records, text, u16_at, u32_at};

/// ZC_FRIENDS_LIST (0x0201): every friend at once.
pub const ZC_FRIENDS_LIST: u16 = 0x0201;
/// ZC_FRIENDS_STATE (0x0206, 35 bytes): a friend came into the game or left it.
pub const ZC_FRIENDS_STATE: u16 = 0x0206;
/// ZC_REQ_ADD_FRIENDS (0x0207, 34 bytes): someone asks to be the player's friend.
pub const ZC_REQ_ADD_FRIENDS: u16 = 0x0207;
/// ZC_ADD_FRIENDS_LIST (0x0209, 36 bytes): what became of asking someone.
pub const ZC_ADD_FRIENDS_LIST: u16 = 0x0209;
/// ZC_DELETE_FRIENDS (0x020a, 10 bytes): a friend is one no more.
pub const ZC_DELETE_FRIENDS: u16 = 0x020a;

/// What ZC_ADD_FRIENDS_LIST answers (the client's `FRIEND_ADD_*`; the server here sends the first four).
pub mod added {
    pub const DONE: u16 = 0;
    pub const REFUSED: u16 = 1;
    pub const OWN_LIST_FULL: u16 = 2;
    pub const THEIR_LIST_FULL: u16 = 3;
}

/// A friend: the account and the character, the name, and whether the friend is out of the game.
#[derive(Debug, Clone, PartialEq)]
pub struct Friend {
    pub account: u32,
    pub character: u32,
    pub name: String,
    pub offline: bool,
}

/// ZC_FRIENDS_LIST (0x0201): `u16 length`, then 32 bytes a friend: `u32 account, u32 character, char name[24]`.
/// Each is out of the game until the server says otherwise (`ReceiveFriendList` sets `offline`).
pub fn list(packet: &[u8]) -> Vec<Friend> {
    records(packet, 4, 32)
        .map(|friend| Friend { account: u32_at(friend, 0), character: u32_at(friend, 4), name: text(&friend[8..]), offline: true })
        .collect()
}

/// ZC_FRIENDS_STATE (0x0206, 35 bytes): `u32 account, u32 character, u8 offline, char name[24]`.
pub fn state(packet: &[u8]) -> Option<Friend> {
    let packet = packet.get(..35)?;
    Some(Friend { account: u32_at(packet, 2), character: u32_at(packet, 6), offline: packet[10] != 0, name: text(&packet[11..]) })
}

/// ZC_REQ_ADD_FRIENDS (0x0207, 34 bytes): `u32 account, u32 character, char name[24]`: who asks.
pub fn asked(packet: &[u8]) -> Option<(u32, u32, String)> {
    let packet = packet.get(..34)?;
    Some((u32_at(packet, 2), u32_at(packet, 6), text(&packet[10..])))
}

/// ZC_ADD_FRIENDS_LIST (0x0209, 36 bytes): `u16 result, u32 account, u32 character, char name[24]`: how asking
/// went (`added`), and the friend made, who is in the game.
pub fn answered(packet: &[u8]) -> Option<(u16, Friend)> {
    let packet = packet.get(..36)?;
    Some((u16_at(packet, 2), Friend { account: u32_at(packet, 4), character: u32_at(packet, 8), name: text(&packet[12..]), offline: false }))
}

/// ZC_DELETE_FRIENDS (0x020a, 10 bytes): `u32 account, u32 character`.
pub fn removed(packet: &[u8]) -> Option<(u32, u32)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u32_at(packet, 6)))
}

/// CZ_ADD_FRIENDS (0x0202, 26 bytes): `char name[24]`: the player of that name is asked to be a friend
/// (`UIWindowMgr::RequestAddFriend`).
pub fn add(name: &str) -> Vec<u8> {
    Writer::new(0x0202).name(name).done()
}

/// CZ_DELETE_FRIENDS (0x0203, 10 bytes): `u32 account, u32 character` (the client's `MM_DELETE_FRIEND`).
pub fn remove(account: u32, character: u32) -> Vec<u8> {
    Writer::new(0x0203).u32(account).u32(character).done()
}

/// CZ_ACK_REQ_ADD_FRIENDS (0x0208, 14 bytes): `u32 account, u32 character, u32 accept`: the answer to one who
/// asked (`MM_JOIN_FRIEND`).
pub fn answer(account: u32, character: u32, accept: bool) -> Vec<u8> {
    Writer::new(0x0208).u32(account).u32(character).u32(u32::from(accept)).done()
}

#[cfg(test)]
mod tests;
