//! The castles a guild holds: the list the map server sends with the guild's numbers, what it answers of a
//! castle's economy and defence, how going to a castle failed, and the two packets that ask. The layouts are
//! rAthena's for PACKETVER 20260219 (`clif_guild_castle_list`, `clif_guild_castleinfo`,
//! `clif_guild_castle_teleport_res`, `clif_parse_guild_castle_info_request` and
//! `clif_parse_guild_castle_teleport_request` in `clif.cpp`, `packets_struct.hpp`); what the client does with
//! each is `CGameMode::ReceiveGuildAgitList`, `ReceiveAgitNotice` and `ReceiveGuildAgitMoveResult`.
//!
//! A castle is named by the number the client's own table has for it (`AgitConfig.lub`, `Number`; rAthena's
//! `ClientId` of `castle_db.yml`), not by the server's number of it.

use super::{Writer, body, int, short};

/// ZC_GUILD_AGIT_INFO (0x0b27): `u16 length, u8 castle[]`: the castles the player's guild holds. rAthena sends
/// it after the guild's numbers when the first tab of the guild window asks for them (CZ_REQ_GUILD_MENU 0),
/// and not at all for a guild that holds none.
pub const ZC_GUILD_AGIT_INFO: u16 = 0x0b27;
/// ZC_REQ_ACK_AGIT_INVESTMENT (0x0b2d, 11 bytes): `u8 castle, i32 economy, i32 defense`: answers `ask_castle`.
pub const ZC_REQ_ACK_AGIT_INVESTMENT: u16 = 0x0b2d;
/// ZC_REQ_ACK_MOVE_GUILD_AGIT (0x0b2e, 4 bytes): `i16 result`: why `go_to_castle` did not move the character
/// (`moved`). rAthena sends nothing when it did.
pub const ZC_REQ_ACK_MOVE_GUILD_AGIT: u16 = 0x0b2e;

/// What ZC_REQ_ACK_MOVE_GUILD_AGIT answers (rAthena's `e_siege_teleport_result`).
pub mod moved {
    pub const DONE: i16 = 0;
    pub const NO_ZENY: i16 = 1;
    /// Not from where the character is.
    pub const NOT_HERE: i16 = 2;
}

/// What the server tells of one castle (ZC_REQ_ACK_AGIT_INVESTMENT).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Investment {
    pub castle: u8,
    pub economy: i32,
    pub defense: i32,
}

/// ZC_GUILD_AGIT_INFO: the castles, each once, in the order sent (the client's `GuildInfo::AddAgitId` leaves
/// out one it has already).
pub fn castles(packet: &[u8]) -> Option<Vec<u8>> {
    let mut castles = Vec::new();
    for castle in body(packet, 4)? {
        if !castles.contains(castle) {
            castles.push(*castle);
        }
    }
    Some(castles)
}

/// ZC_REQ_ACK_AGIT_INVESTMENT: a castle's economy and defence.
pub fn investment(packet: &[u8]) -> Option<Investment> {
    Some(Investment { castle: *packet.get(2)?, economy: int(packet, 3)?, defense: int(packet, 7)? })
}

/// ZC_REQ_ACK_MOVE_GUILD_AGIT: how going to a castle went (`moved`).
pub fn castle_move(packet: &[u8]) -> Option<i16> {
    short(packet, 2)
}

/// CZ_REQ_MOVE_GUILD_AGIT (0x0b28, 3 bytes): `u8 castle`: asks to be moved to a castle of the guild's (the
/// client's `MM_REQ_MOVE_GUILD_AGIT`, which sends nothing for castle 0).
pub fn go_to_castle(castle: u8) -> Vec<u8> {
    Writer::new(0x0b28).u8(castle).done()
}

/// CZ_REQ_AGIT_INVESTMENT (0x0b2c, 3 bytes): `u8 castle`: asks for a castle's economy and defence (the
/// client's `MM_REQ_AGIT_INVESTMENT`, which sends nothing for castle 0).
pub fn ask_castle(castle: u8) -> Vec<u8> {
    Writer::new(0x0b2c).u8(castle).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{assert_lengths, packet, sized};

    #[test]
    fn the_castles_of_the_guild_are_listed_each_once() {
        assert_eq!(castles(&sized(ZC_GUILD_AGIT_INFO, &[1, 16, 1, 31])), Some(vec![1, 16, 31]));
        assert_eq!(castles(&sized(ZC_GUILD_AGIT_INFO, &[])), Some(Vec::new()));
        // What lies past the length the packet names is not of it; a packet with no length is none.
        let mut longer = sized(ZC_GUILD_AGIT_INFO, &[7]);
        longer.push(9);
        assert_eq!(castles(&longer), Some(vec![7]));
        assert_eq!(castles(&[0x27, 0x0b, 5]), None);
    }

    #[test]
    fn the_economy_and_the_defence_of_a_castle_are_read() {
        let told = packet(ZC_REQ_ACK_AGIT_INVESTMENT, &[&[16], &35i32.to_le_bytes(), &(-2i32).to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_REQ_ACK_AGIT_INVESTMENT), Some(told.len() as i32));
        assert_eq!(investment(&told), Some(Investment { castle: 16, economy: 35, defense: -2 }));
        assert_eq!(investment(&told[..10]), None);
    }

    #[test]
    fn how_going_to_a_castle_went_is_read() {
        let told = packet(ZC_REQ_ACK_MOVE_GUILD_AGIT, &[&moved::NO_ZENY.to_le_bytes()]);
        assert_eq!(crate::net::packet_length(ZC_REQ_ACK_MOVE_GUILD_AGIT), Some(told.len() as i32));
        assert_eq!((castle_move(&told), castle_move(&told[..3])), (Some(moved::NO_ZENY), None));
        assert_eq!(crate::net::packet_length(ZC_GUILD_AGIT_INFO), Some(-1));
    }

    #[test]
    fn what_is_asked_of_a_castle_is_as_the_server_reads_it() {
        assert_lengths(&[go_to_castle(16), ask_castle(16)]);
        assert_eq!((go_to_castle(16), ask_castle(31)), (vec![0x28, 0x0b, 16], vec![0x2c, 0x0b, 31]));
    }
}
