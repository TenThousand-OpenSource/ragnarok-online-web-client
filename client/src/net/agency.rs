//! The Adventurer's Agency on the map server: asking a party's leader to be let in, the leader's answer, and
//! what the one who asked is told. (The notices themselves go over the game's web service: `crate::agency`.)
//! The layouts are rAthena's for PACKETVER 20260219 (`PACKET_CZ_PARTY_REQ_MASTER_TO_JOIN` and the three
//! beside it in `packets.hpp`, `clif_partybooking_ask`, `clif_partybooking_reply`) and the client's
//! (`MM_PARTY_REQ_MASTER_TO_JOIN`, `MM_PARTY_REQ_ACK_MASTER_TO_JOIN`, `ReceivePartyJoinResult`).

use super::bytes::{long, text_at, word, Writer};

/// ZC_PARTY_REQ_MASTER_TO_JOIN (0x0ae7, 38 bytes): someone asks the leader to be let into the party.
pub const ZC_PARTY_REQ_MASTER_TO_JOIN: u16 = 0x0ae7;
/// ZC_PARTY_JOIN_REQ_ACK_FROM_MASTER (0x0afa, 58 bytes): what became of asking to join.
pub const ZC_PARTY_JOIN_REQ_ACK_FROM_MASTER: u16 = 0x0afa;

/// One who asks to be let into the party.
#[derive(Clone, Debug, PartialEq)]
pub struct Asking {
    pub character: u32,
    pub account: u32,
    pub name: String,
    pub level: u16,
    pub job: u16,
}

/// ZC_PARTY_REQ_MASTER_TO_JOIN: `u32 character, u32 account, char name[24], u16 level, u16 job` (rAthena's
/// names for the last two are `x` and `y`).
pub fn asking(packet: &[u8]) -> Option<Asking> {
    Some(Asking { character: long(packet, 2)?, account: long(packet, 6)?, name: text_at(packet, 10, 24)?, level: word(packet, 34)?, job: word(packet, 36)? })
}

/// ZC_PARTY_JOIN_REQ_ACK_FROM_MASTER: `char first[24], char second[24], u32 leader, u32 status`: two names
/// (rAthena writes the leader's, then the party's; the client reads the first as the party's and the second
/// as the player's), the leader's account and what happened (`crate::agency::join_result`).
pub fn joined(packet: &[u8]) -> Option<(String, String, u32, u32)> {
    Some((text_at(packet, 2, 24)?, text_at(packet, 26, 24)?, long(packet, 50)?, long(packet, 54)?))
}

/// CZ_PARTY_REQ_MASTER_TO_JOIN (0x0ae6, 10 bytes): `u32 character, u32 account`: the leader of that notice
/// is asked to let the player in.
pub fn ask_to_join(character: u32, account: u32) -> Vec<u8> {
    Writer::new(0x0ae6).u32(character).u32(account).done()
}

/// CZ_PARTY_REQ_ACK_MASTER_TO_JOIN (0x0af8, 11 bytes): `u32 character, u32 account, u8 accept`: the leader's
/// answer to one who asked.
pub fn answer_join(character: u32, account: u32, accept: bool) -> Vec<u8> {
    Writer::new(0x0af8).u32(character).u32(account).u8(accept.into()).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{field, packet};

    #[test]
    fn reads_one_who_asks_and_what_became_of_asking() {
        let asks = packet(0x0ae7, &[&150_007u32.to_le_bytes(), &2_000_007u32.to_le_bytes(), &field("Seven", 24), &99u16.to_le_bytes(), &4008u16.to_le_bytes()]);
        assert_eq!(asks.len(), 38);
        assert_eq!(asking(&asks), Some(Asking { character: 150_007, account: 2_000_007, name: "Seven".to_owned(), level: 99, job: 4008 }));
        assert_eq!(asking(&asks[..30]), None);
        let answer = packet(0x0afa, &[&field("Six", 24), &field("Sixes", 24), &2_000_006u32.to_le_bytes(), &1u32.to_le_bytes()]);
        assert_eq!(answer.len(), 58);
        assert_eq!(joined(&answer), Some(("Six".to_owned(), "Sixes".to_owned(), 2_000_006, 1)));
    }

    #[test]
    fn what_is_sent_is_as_the_server_reads_it() {
        assert_eq!(ask_to_join(150_007, 2_000_007), [0xe6, 0x0a, 0xf7, 0x49, 0x02, 0x00, 0x87, 0x84, 0x1e, 0x00]);
        assert_eq!(answer_join(150_007, 2_000_007, true), [0xf8, 0x0a, 0xf7, 0x49, 0x02, 0x00, 0x87, 0x84, 0x1e, 0x00, 1]);
        assert_eq!(answer_join(150_007, 2_000_007, false)[10], 0);
    }
}
