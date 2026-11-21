//! The party: who is in it, what they share, what they say to each other, and the packets that make, join and
//! leave one. The layouts are rAthena's for PACKETVER 20260219 (`clif_party_*` in `clif.cpp`,
//! `packets_struct.hpp`, `packets.hpp`).

use super::bytes::{Writer, i32_at, records, text, text_at, u16_at, u32_at};

/// ZC_ACK_MAKE_GROUP (0x00fa, 3 bytes): `u8 result`: how making a party went (`made`).
pub const ZC_ACK_MAKE_GROUP: u16 = 0x00fa;
/// ZC_GROUP_LIST (0x0ae5): everyone of the party at once.
pub const ZC_GROUP_LIST: u16 = 0x0ae5;
/// ZC_ADD_MEMBER_TO_GROUP (0x0ae4): one member joined, came online, went offline or to another map.
pub const ZC_ADD_MEMBER_TO_GROUP: u16 = 0x0ae4;
/// ZC_DELETE_MEMBER_FROM_GROUP (0x0105): one member left or was expelled.
pub const ZC_DELETE_MEMBER_FROM_GROUP: u16 = 0x0105;
/// ZC_PARTY_JOIN_REQ (0x02c6): a party invites the player.
pub const ZC_PARTY_JOIN_REQ: u16 = 0x02c6;
/// ZC_PARTY_JOIN_REQ_ACK (0x02c5): what became of the player's own invitation.
pub const ZC_PARTY_JOIN_REQ_ACK: u16 = 0x02c5;
/// ZC_REQ_GROUPINFO_CHANGE_V2 (0x07d8): how the party shares experience and items.
pub const ZC_REQ_GROUPINFO_CHANGE_V2: u16 = 0x07d8;
/// ZC_NOTIFY_CHAT_PARTY (0x0109): a line of party chat.
pub const ZC_NOTIFY_CHAT_PARTY: u16 = 0x0109;
/// ZC_NOTIFY_POSITION_TO_GROUPM (0x0107): where a member on the same map is.
pub const ZC_NOTIFY_POSITION_TO_GROUPM: u16 = 0x0107;
/// ZC_NOTIFY_HP_TO_GROUPM (0x080e): a member's life.
pub const ZC_NOTIFY_HP_TO_GROUPM: u16 = 0x080e;
/// ZC_GROUP_ISALIVE (0x0ab2): a member died or lives again.
pub const ZC_GROUP_ISALIVE: u16 = 0x0ab2;
/// ZC_NOTIFY_MEMBERINFO_TO_GROUPM (0x0abd): a member's job and level changed.
pub const ZC_NOTIFY_MEMBERINFO_TO_GROUPM: u16 = 0x0abd;
/// ZC_CHANGE_GROUP_MASTER (0x07fc): the party has another leader.
pub const ZC_CHANGE_GROUP_MASTER: u16 = 0x07fc;
/// ZC_PARTY_CONFIG (0x02c9, 3 bytes): `u8 refused`: whether the player turns party invitations down.
pub const ZC_PARTY_CONFIG: u16 = 0x02c9;
/// ZC_ITEM_PICKUP_PARTY (0x0b67): a member of the party obtained a thing (`clif_party_show_picker`).
pub const ZC_ITEM_PICKUP_PARTY: u16 = 0x0b67;

/// What ZC_ACK_MAKE_GROUP answers (the client's `GROUP_MAKE_*`).
pub mod made {
    pub const DONE: u8 = 0;
    pub const SAME_NAME: u8 = 1;
    pub const ALREADY_IN_ONE: u8 = 2;
    /// No party can be made on this map.
    pub const NOT_HERE: u8 = 3;
}

/// What ZC_PARTY_JOIN_REQ_ACK answers (rAthena's `e_party_invite_reply`).
pub mod invite {
    pub const IN_OTHER_PARTY: u32 = 0;
    pub const REJECTED: u32 = 1;
    pub const ACCEPTED: u32 = 2;
    pub const FULL: u32 = 3;
    pub const SAME_ACCOUNT: u32 = 4;
    pub const REFUSES_ALL: u32 = 5;
    pub const OFFLINE: u32 = 7;
    pub const THEIR_MAP_FORBIDS: u32 = 8;
    pub const THIS_MAP_FORBIDS: u32 = 9;
    pub const MEMORIAL_DUNGEON: u32 = 10;
    pub const LEVEL: u32 = 11;
}

/// What ZC_DELETE_MEMBER_FROM_GROUP tells (the client's `GROUPMEMBER_DELETE_*`).
pub mod gone {
    pub const LEFT: u8 = 0;
    pub const EXPELLED: u8 = 1;
    pub const CANNOT_LEAVE: u8 = 2;
    pub const CANNOT_EXPEL: u8 = 3;
}

/// Someone of the party.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Member {
    pub account: u32,
    pub character: u32,
    pub name: String,
    /// The map the member is on, as the server names it (with `.gat`).
    pub map: String,
    pub leader: bool,
    pub offline: bool,
    pub job: u16,
    pub level: u16,
}

/// How the party shares: experience evenly, items anyone may pick up, items handed out among the members.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Sharing {
    /// 0 each takes its own, 1 shared evenly, 2 it could not be changed (the levels are too far apart).
    pub experience: u32,
    pub pickup: bool,
    pub items: bool,
}

/// ZC_GROUP_LIST (0x0ae5): `u16 length`, `char party[24]`, then for each member 54 bytes: `u32 account`,
/// `u32 character`, `char name[24]`, `char map[16]`, `u8 role` (0 the leader), `u8 offline`, `u16 job`,
/// `u16 level`. The party's name, and its members.
pub fn list(packet: &[u8]) -> Option<(String, Vec<Member>)> {
    let name = text_at(packet, 4, 24)?;
    let members = records(packet, 28, 54)
        .map(|member| Member {
            account: u32_at(member, 0),
            character: u32_at(member, 4),
            name: text(&member[8..32]),
            map: text(&member[32..48]),
            leader: member[48] == 0,
            offline: member[49] != 0,
            job: u16_at(member, 50),
            level: u16_at(member, 52),
        })
        .collect();
    Some((name, members))
}

/// What ZC_ADD_MEMBER_TO_GROUP tells: the member, the party's name, the member's cell, and how items are
/// shared (anyone may pick them up; they are handed out among the members).
#[derive(Debug, Clone, PartialEq)]
pub struct Joined {
    pub member: Member,
    pub party: String,
    pub cell: (u16, u16),
    pub pickup: bool,
    pub items: bool,
}

/// ZC_ADD_MEMBER_TO_GROUP (0x0ae4, 89 bytes): `u32 account`, `u32 character`, `u32 role` (0 the leader),
/// `u16 job`, `u16 level`, `u16 x`, `u16 y`, `u8 offline`, `char party[24]`, `char name[24]`, `char map[16]`,
/// `u8 pickup`, `u8 items`.
pub fn joined(packet: &[u8]) -> Option<Joined> {
    let packet = packet.get(..89)?;
    let member = Member {
        account: u32_at(packet, 2),
        character: u32_at(packet, 6),
        name: text(&packet[47..71]),
        map: text(&packet[71..87]),
        leader: u32_at(packet, 10) == 0,
        offline: packet[22] != 0,
        job: u16_at(packet, 14),
        level: u16_at(packet, 16),
    };
    Some(Joined {
        member,
        party: text(&packet[23..47]),
        cell: (u16_at(packet, 18), u16_at(packet, 20)),
        pickup: packet[87] != 0,
        items: packet[88] != 0,
    })
}

/// ZC_ACK_MAKE_GROUP: how making a party went (`made`), if the packet has it.
pub fn made_how(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// ZC_PARTY_CONFIG: whether the player turns party invitations down, if the packet says.
pub fn refuses(packet: &[u8]) -> Option<bool> {
    Some(*packet.get(2)? != 0)
}

/// ZC_DELETE_MEMBER_FROM_GROUP (0x0105, 31 bytes): `u32 account`, `char name[24]`, `u8 result` (`gone`).
pub fn left(packet: &[u8]) -> Option<(u32, String, u8)> {
    let packet = packet.get(..31)?;
    Some((u32_at(packet, 2), text(&packet[6..30]), packet[30]))
}

/// ZC_PARTY_JOIN_REQ (0x02c6, 30 bytes): `u32 party`, `char name[24]`: the party that invites, and its name.
pub fn invited(packet: &[u8]) -> Option<(u32, String)> {
    let packet = packet.get(..30)?;
    Some((u32_at(packet, 2), text(&packet[6..30])))
}

/// ZC_PARTY_JOIN_REQ_ACK (0x02c5, 30 bytes): `char name[24]`, `u32 result` (`invite`).
pub fn invite_answered(packet: &[u8]) -> Option<(String, u32)> {
    let packet = packet.get(..30)?;
    Some((text(&packet[2..26]), u32_at(packet, 26)))
}

/// ZC_REQ_GROUPINFO_CHANGE_V2 (0x07d8, 8 bytes): `u32 experience`, `u8 pickup`, `u8 items`.
pub fn sharing(packet: &[u8]) -> Option<Sharing> {
    let packet = packet.get(..8)?;
    Some(Sharing { experience: u32_at(packet, 2), pickup: packet[6] != 0, items: packet[7] != 0 })
}

/// ZC_NOTIFY_CHAT_PARTY (0x0109): `u16 length`, `u32 account`, `char text[]`: who says it, and the line as it is
/// shown ("name : words").
pub fn said(packet: &[u8]) -> Option<(u32, String)> {
    Some((u32_at(packet.get(..8)?, 4), text(&packet[8..])))
}

/// ZC_NOTIFY_POSITION_TO_GROUPM (0x0107, 10 bytes): `u32 account`, `u16 x`, `u16 y`.
pub fn place(packet: &[u8]) -> Option<(u32, (u16, u16))> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), (u16_at(packet, 6), u16_at(packet, 8))))
}

/// ZC_NOTIFY_HP_TO_GROUPM (0x080e, 14 bytes): `u32 account`, `i32 hp`, `i32 most`.
pub fn life(packet: &[u8]) -> Option<(u32, i32, i32)> {
    let packet = packet.get(..14)?;
    Some((u32_at(packet, 2), i32_at(packet, 6), i32_at(packet, 10)))
}

/// ZC_GROUP_ISALIVE (0x0ab2, 7 bytes): `u32 account`, `u8 dead`.
pub fn dead(packet: &[u8]) -> Option<(u32, bool)> {
    let packet = packet.get(..7)?;
    Some((u32_at(packet, 2), packet[6] != 0))
}

/// ZC_NOTIFY_MEMBERINFO_TO_GROUPM (0x0abd, 10 bytes): `u32 account`, `u16 job`, `u16 level`.
pub fn job_level(packet: &[u8]) -> Option<(u32, u16, u16)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u16_at(packet, 6), u16_at(packet, 8)))
}

/// ZC_CHANGE_GROUP_MASTER (0x07fc, 10 bytes): `u32 before`, `u32 now`: the accounts of the old leader and the new.
pub fn leader(packet: &[u8]) -> Option<(u32, u32)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u32_at(packet, 6)))
}

mod requests;
pub use requests::*;

/// A thing a member of the party obtained, as ZC_ITEM_PICKUP_PARTY tells it: what the thing's name is
/// written from.
#[derive(Debug, Clone, PartialEq)]
pub struct Picked {
    /// The member's account.
    pub account: u32,
    pub id: u32,
    /// The item's type, as the server tells it.
    pub kind: u8,
    pub identified: bool,
    pub refine: u8,
    pub cards: [u32; 4],
}

/// ZC_ITEM_PICKUP_PARTY (0x0b67, 33 bytes): `u32 account, u32 item, u8 identified, u8 damaged, u32 card[4],
/// u16 location, u8 type, u8 refine, u8 grade` (the client's `PartyItemPickupPacket`).
pub fn picked(packet: &[u8]) -> Option<Picked> {
    let packet = packet.get(..33)?;
    Some(Picked {
        account: u32_at(packet, 2),
        id: u32_at(packet, 6),
        kind: packet[30],
        identified: packet[10] != 0,
        refine: packet[31],
        cards: std::array::from_fn(|card| u32_at(packet, 12 + card * 4)),
    })
}

#[cfg(test)]
mod tests;
