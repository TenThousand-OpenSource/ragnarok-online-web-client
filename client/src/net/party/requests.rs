//! What the client sends about the party: making one, inviting and answering, leaving and expelling, how it
//! shares, handing it over, its chat, and turning invitations down.

use super::*;

/// CZ_MAKE_GROUP2 (0x01e8, 28 bytes): `char name[24]`, `u8 pickup`, `u8 items`: a party is made.
pub fn make(name: &str, pickup: bool, items: bool) -> Vec<u8> {
    Writer::new(0x01e8).name(name).u8(u8::from(pickup)).u8(u8::from(items)).done()
}

/// CZ_PARTY_JOIN_REQ (0x02c4, 26 bytes): `char name[24]`: that player is invited.
pub fn invite(name: &str) -> Vec<u8> {
    Writer::new(0x02c4).name(name).done()
}

/// CZ_PARTY_JOIN_REQ_ACK (0x02c7, 7 bytes): `u32 party`, `u8 accept`: the answer to an invitation.
pub fn answer(party: u32, accept: bool) -> Vec<u8> {
    Writer::new(0x02c7).u32(party).u8(u8::from(accept)).done()
}

/// CZ_REQ_LEAVE_GROUP (0x0100): the player leaves the party.
pub fn leave() -> Vec<u8> {
    Writer::new(0x0100).done()
}

/// CZ_REQ_EXPEL_GROUP_MEMBER (0x0103, 30 bytes): `u32 account`, `char name[24]`: the leader expels a member.
pub fn expel(account: u32, name: &str) -> Vec<u8> {
    Writer::new(0x0103).u32(account).name(name).done()
}

/// CZ_GROUPINFO_CHANGE_V2 (0x07d7, 8 bytes): `u32 experience`, `u8 pickup`, `u8 items`: the leader sets how the
/// party shares.
pub fn share(sharing: Sharing) -> Vec<u8> {
    Writer::new(0x07d7).u32(sharing.experience).u8(u8::from(sharing.pickup)).u8(u8::from(sharing.items)).done()
}

/// CZ_CHANGE_GROUP_MASTER (0x07da, 6 bytes): `u32 account`: the leader hands the party to that member.
pub fn hand_over(account: u32) -> Vec<u8> {
    Writer::new(0x07da).u32(account).done()
}

/// CZ_REQUEST_CHAT_PARTY (0x0108): `u16 length`, `char text[]`: a line for the party, "name : words" (the
/// client's `MM_REQUEST_CHAT_PARTY`, which sends no zero after it; rAthena takes it with or without).
pub fn say(name: &str, words: &str) -> Vec<u8> {
    Writer::sized(0x0108).bytes(format!("{name} : {words}").as_bytes()).done()
}

/// CZ_PARTY_CONFIG (0x02c8, 3 bytes): `u8 refuse`: party invitations are turned down from now, or let in again.
pub fn refuse_invitations(refuse: bool) -> Vec<u8> {
    Writer::new(0x02c8).u8(u8::from(refuse)).done()
}
