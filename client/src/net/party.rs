//! The party: who is in it, what they share, what they say to each other, and the packets that make, join and
//! leave one. The layouts are rAthena's for PACKETVER 20260219 (`clif_party_*` in `clif.cpp`,
//! `packets_struct.hpp`, `packets.hpp`).

use super::bytes::{name24, text, u16_at, u32_at};

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
    let members = packet.get(28..)?;
    let members = members
        .chunks_exact(54)
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
    Some((text(&packet[4..28]), members))
}

/// ZC_ADD_MEMBER_TO_GROUP (0x0ae4, 89 bytes): `u32 account`, `u32 character`, `u32 role` (0 the leader),
/// `u16 job`, `u16 level`, `u16 x`, `u16 y`, `u8 offline`, `char party[24]`, `char name[24]`, `char map[16]`,
/// `u8 pickup`, `u8 items`. The member, the party's name, the member's cell, and how items are shared.
pub fn joined(packet: &[u8]) -> Option<(Member, String, (u16, u16), (bool, bool))> {
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
    Some((member, text(&packet[23..47]), (u16_at(packet, 18), u16_at(packet, 20)), (packet[87] != 0, packet[88] != 0)))
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
    Some((u32_at(packet, 2), u32_at(packet, 6) as i32, u32_at(packet, 10) as i32))
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

/// CZ_MAKE_GROUP2 (0x01e8, 28 bytes): `char name[24]`, `u8 pickup`, `u8 items`: a party is made.
pub fn make(name: &str, pickup: bool, items: bool) -> Vec<u8> {
    [&[0xe8, 0x01][..], &name24(name), &[u8::from(pickup), u8::from(items)]].concat()
}

/// CZ_PARTY_JOIN_REQ (0x02c4, 26 bytes): `char name[24]`: that player is invited.
pub fn invite(name: &str) -> Vec<u8> {
    [&[0xc4, 0x02][..], &name24(name)].concat()
}

/// CZ_PARTY_JOIN_REQ_ACK (0x02c7, 7 bytes): `u32 party`, `u8 accept`: the answer to an invitation.
pub fn answer(party: u32, accept: bool) -> Vec<u8> {
    [&[0xc7, 0x02][..], &party.to_le_bytes(), &[u8::from(accept)]].concat()
}

/// CZ_REQ_LEAVE_GROUP (0x0100): the player leaves the party.
pub fn leave() -> Vec<u8> {
    vec![0x00, 0x01]
}

/// CZ_REQ_EXPEL_GROUP_MEMBER (0x0103, 30 bytes): `u32 account`, `char name[24]`: the leader expels a member.
pub fn expel(account: u32, name: &str) -> Vec<u8> {
    [&[0x03, 0x01][..], &account.to_le_bytes(), &name24(name)].concat()
}

/// CZ_GROUPINFO_CHANGE_V2 (0x07d7, 8 bytes): `u32 experience`, `u8 pickup`, `u8 items`: the leader sets how the
/// party shares.
pub fn share(sharing: Sharing) -> Vec<u8> {
    [&[0xd7, 0x07][..], &sharing.experience.to_le_bytes(), &[u8::from(sharing.pickup), u8::from(sharing.items)]].concat()
}

/// CZ_CHANGE_GROUP_MASTER (0x07da, 6 bytes): `u32 account`: the leader hands the party to that member.
pub fn hand_over(account: u32) -> Vec<u8> {
    [&[0xda, 0x07][..], &account.to_le_bytes()].concat()
}

/// CZ_REQUEST_CHAT_PARTY (0x0108): `u16 length`, `char text[]`: a line for the party, "name : words" (the
/// client's `MM_REQUEST_CHAT_PARTY`, which sends no zero after it; rAthena takes it with or without).
pub fn say(name: &str, words: &str) -> Vec<u8> {
    let text = format!("{name} : {words}");
    [&[0x08, 0x01][..], &((text.len() + 4) as u16).to_le_bytes(), text.as_bytes()].concat()
}

/// CZ_PARTY_CONFIG (0x02c8, 3 bytes): `u8 refuse`: party invitations are turned down from now, or let in again.
pub fn refuse_invitations(refuse: bool) -> Vec<u8> {
    vec![0xc8, 0x02, u8::from(refuse)]
}

/// The map a member is on without what the server writes after its name (`prontera.gat`): two members are on
/// one map when these are the same (the client's `UpdatePartyMemberInfo` cuts the same off both).
pub fn map_name(map: &str) -> &str {
    map.split('.').next().unwrap_or(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(text: &str, length: usize) -> Vec<u8> {
        let mut field = text.as_bytes().to_vec();
        field.resize(length, 0);
        field
    }

    #[test]
    fn reads_the_list_of_members() {
        let mut packet = vec![0xe5, 0x0a, 0, 0];
        packet.extend(field("Poring Hunters", 24));
        for (account, name, map, role, offline, job, level) in [(2_000_001u32, "Anna", "prontera.gat", 0u8, 0u8, 4u16, 55u16), (2_000_002, "Boris", "", 1, 1, 7, 99)] {
            packet.extend(account.to_le_bytes());
            packet.extend((account + 150_000).to_le_bytes());
            packet.extend(field(name, 24));
            packet.extend(field(map, 16));
            packet.extend([role, offline]);
            packet.extend(job.to_le_bytes());
            packet.extend(level.to_le_bytes());
        }
        let length = packet.len() as u16;
        packet[2..4].copy_from_slice(&length.to_le_bytes());
        let (name, members) = list(&packet).unwrap();
        assert_eq!(name, "Poring Hunters");
        assert_eq!(members.len(), 2);
        assert_eq!(members[0], Member { account: 2_000_001, character: 2_150_001, name: "Anna".into(), map: "prontera.gat".into(), leader: true, offline: false, job: 4, level: 55 });
        assert_eq!((members[1].leader, members[1].offline, members[1].level), (false, true, 99));
        assert_eq!(list(&packet[..20]), None);
    }

    #[test]
    fn reads_a_member_joining() {
        let mut packet = vec![0xe4, 0x0a];
        packet.extend(2_000_002u32.to_le_bytes());
        packet.extend(150_002u32.to_le_bytes());
        packet.extend(1u32.to_le_bytes());
        for value in [7u16, 99, 156, 187] {
            packet.extend(value.to_le_bytes());
        }
        packet.push(0);
        packet.extend(field("Poring Hunters", 24));
        packet.extend(field("Boris", 24));
        packet.extend(field("prontera.gat", 16));
        packet.extend([1, 0]);
        assert_eq!(packet.len(), 89);
        let (member, party, cell, shared) = joined(&packet).unwrap();
        assert_eq!(member, Member { account: 2_000_002, character: 150_002, name: "Boris".into(), map: "prontera.gat".into(), leader: false, offline: false, job: 7, level: 99 });
        assert_eq!((party.as_str(), cell, shared), ("Poring Hunters", (156, 187), (true, false)));
        assert_eq!(joined(&packet[..88]), None);
    }

    #[test]
    fn reads_the_smaller_packets() {
        let mut gone = vec![0x05, 0x01];
        gone.extend(2_000_002u32.to_le_bytes());
        gone.extend(field("Boris", 24));
        gone.push(gone::EXPELLED);
        assert_eq!(left(&gone), Some((2_000_002, "Boris".into(), 1)));

        let mut asked = vec![0xc6, 0x02];
        asked.extend(77u32.to_le_bytes());
        asked.extend(field("Poring Hunters", 24));
        assert_eq!(invited(&asked), Some((77, "Poring Hunters".into())));

        let mut answered = vec![0xc5, 0x02];
        answered.extend(field("Boris", 24));
        answered.extend(invite::ACCEPTED.to_le_bytes());
        assert_eq!(invite_answered(&answered), Some(("Boris".into(), 2)));

        assert_eq!(sharing(&[0xd8, 0x07, 1, 0, 0, 0, 1, 0]), Some(Sharing { experience: 1, pickup: true, items: false }));
        assert_eq!(said(&[&[0x09, 0x01, 18, 0, 5, 0, 0, 0][..], b"Anna : hi\0"].concat()), Some((5, "Anna : hi".into())));
        assert_eq!(place(&[0x07, 0x01, 5, 0, 0, 0, 156, 0, 187, 0]), Some((5, (156, 187))));
        assert_eq!(life(&[0x0e, 0x08, 5, 0, 0, 0, 0x10, 0x27, 0, 0, 0x20, 0x4e, 0, 0]), Some((5, 10_000, 20_000)));
        assert_eq!(dead(&[0xb2, 0x0a, 5, 0, 0, 0, 1]), Some((5, true)));
        assert_eq!(job_level(&[0xbd, 0x0a, 5, 0, 0, 0, 7, 0, 99, 0]), Some((5, 7, 99)));
        assert_eq!(leader(&[0xfc, 0x07, 5, 0, 0, 0, 6, 0, 0, 0]), Some((5, 6)));
    }

    #[test]
    fn writes_what_the_client_sends() {
        let made = make("Poring Hunters", true, false);
        assert_eq!((made.len(), &made[..4], made[26], made[27]), (28, &[0xe8, 0x01, b'P', b'o'][..], 1, 0));
        let invited = invite("Boris");
        assert_eq!((invited.len(), &invited[..8]), (26, &[0xc4, 0x02, b'B', b'o', b'r', b'i', b's', 0][..]));
        // A name longer than the server keeps is cut, and still ends in a zero.
        assert_eq!(invite("abcdefghijklmnopqrstuvwxyz")[25], 0);
        assert_eq!(answer(77, true), [0xc7, 0x02, 77, 0, 0, 0, 1]);
        assert_eq!(leave(), [0x00, 0x01]);
        let expelled = expel(5, "Boris");
        assert_eq!((expelled.len(), &expelled[..7]), (30, &[0x03, 0x01, 5, 0, 0, 0, b'B'][..]));
        assert_eq!(share(Sharing { experience: 1, pickup: false, items: true }), [0xd7, 0x07, 1, 0, 0, 0, 0, 1]);
        assert_eq!(hand_over(6), [0xda, 0x07, 6, 0, 0, 0]);
        assert_eq!(say("Anna", "hi"), [&[0x08, 0x01, 13, 0][..], b"Anna : hi"].concat());
        assert_eq!(refuse_invitations(true), [0xc8, 0x02, 1]);
    }

    #[test]
    fn two_members_are_on_one_map_by_its_name_alone() {
        assert_eq!(map_name("prontera.gat"), "prontera");
        assert_eq!(map_name("prontera"), "prontera");
    }
}
