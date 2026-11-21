//! What the chat commands send to the map server and hear back: how many play (`/who`), whose whispers are let
//! in (`/ex`, `/in` and their kin), a place kept for Warp Portal (`/memo`), the shake of the head
//! (`/doridori`), and the rankings (`/blacksmith`, `/alchemist`, `/taekwon`); the lines the client writes of
//! one into the chat are `crate::commands::ranking_lines`. And what a game master's commands hear back.

use super::bytes::{i16_at, long, name_at, records, text, u16_at, u32_at, Writer};

/// CZ_REQ_USER_COUNT: how many play on the server? The answer is ZC_USER_COUNT.
pub const CZ_REQ_USER_COUNT: u16 = 0x00c1;
/// ZC_USER_COUNT: `u32 count`.
pub const ZC_USER_COUNT: u16 = 0x00c2;
/// ZC_SETTING_WHISPER_PC: whether whispers from one character were blocked or let in.
pub const ZC_SETTING_WHISPER_PC: u16 = 0x00d1;
/// ZC_SETTING_WHISPER_STATE: whether all whispers were blocked or let in.
pub const ZC_SETTING_WHISPER_STATE: u16 = 0x00d2;
/// ZC_WHISPER_LIST: the characters whose whispers are blocked.
pub const ZC_WHISPER_LIST: u16 = 0x00d4;
/// ZC_ACK_RANKING: the ten first of a ranking and the player's own points.
pub const ZC_ACK_RANKING: u16 = 0x097d;
/// ZC_ACK_RANKING2: the same with the ten told by their characters' numbers; what rAthena sends at this
/// PACKETVER (`clif_ranklist`).
pub const ZC_ACK_RANKING2: u16 = 0x0af6;
/// ZC_UPDATE_RANKING_POINT: points of fame were gained (`clif_update_rankingpoint`).
pub const ZC_UPDATE_RANKING_POINT: u16 = 0x097e;
/// ZC_ACK_DISCONNECT_CHARACTER: whether a game master's kick threw the player out.
pub const ZC_ACK_DISCONNECT_CHARACTER: u16 = 0x00cd;
/// ZC_ACK_ACCOUNTNAME (0x01e0, 30 bytes): `u32 account, char name[24]`, the answer to a game master's
/// `/accountname`. The client's handler does nothing with it.
pub const ZC_ACK_ACCOUNTNAME: u16 = 0x01e0;
/// ZC_ACK_STATUS_GM: a player's stats, told to a game master.
pub const ZC_ACK_STATUS_GM: u16 = 0x0214;

/// The answers' results: done, not done, not done because the list is full.
pub const DONE: u8 = 0;
pub const FAILED: u8 = 1;
pub const FULL: u8 = 2;

/// CZ_REQ_USER_COUNT (0x00c1), CZ_REQ_WHISPER_LIST (0x00d3): nothing but the number.
pub fn user_count() -> Vec<u8> {
    Writer::new(CZ_REQ_USER_COUNT).done()
}

pub fn whisper_list() -> Vec<u8> {
    Writer::new(0x00d3).done()
}

/// CZ_REMEMBER_WARPPOINT (0x011d): this place is kept for Warp Portal (`/memo`).
pub fn remember_warp() -> Vec<u8> {
    Writer::new(0x011d).done()
}

/// CZ_DORIDORI (0x01e7): the head shakes; the server gives a novice's SP back faster for it (`/doridori`).
pub fn doridori() -> Vec<u8> {
    Writer::new(0x01e7).done()
}

/// CZ_SETTING_WHISPER_PC (0x00cf, 27 bytes): `char name[24]`, `u8 allow`: whispers from that character are
/// blocked (0) or let in again (1). The name is cut to what fits with its ending nought.
pub fn whisper_from(name: &str, allow: bool) -> Vec<u8> {
    Writer::new(0x00cf).name(name).u8(u8::from(allow)).done()
}

/// CZ_SETTING_WHISPER_STATE (0x00d0, 3 bytes): `u8 allow`: all whispers are blocked (0) or let in (1).
pub fn whisper_all(allow: bool) -> Vec<u8> {
    Writer::new(0x00d0).u8(u8::from(allow)).done()
}

/// ZC_SETTING_WHISPER_PC (0x00d1) and ZC_SETTING_WHISPER_STATE (0x00d2), 4 bytes each: `u8 allow`, `u8 result`:
/// what was asked (0 to block, 1 to let in) and how it went.
pub fn whisper_set(packet: &[u8]) -> Option<(bool, u8)> {
    let packet = packet.get(..4)?;
    Some((packet[2] != 0, packet[3]))
}

/// ZC_WHISPER_LIST (0x00d4): `u16 length`, then `char name[24]` for each character blocked.
pub fn whisper_blocked(packet: &[u8]) -> Vec<String> {
    records(packet, 4, 24).map(text).collect()
}

/// ZC_USER_COUNT (0x00c2, 6 bytes): `u32 count`.
pub fn users(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// CZ_REQ_RANKING (0x097c, 4 bytes): `u16 kind`: the ten first blacksmiths (0), alchemists (1) or taekwons (2)
/// are asked for (`/blacksmith`, `/alchemist`, `/taekwon`).
pub fn ranking_ask(kind: u16) -> Vec<u8> {
    Writer::new(0x097c).u16(kind).done()
}

/// A ranking, as ZC_ACK_RANKING tells it.
#[derive(Debug, PartialEq)]
pub struct Ranking {
    /// Blacksmiths (0), alchemists (1) or taekwons (2).
    pub kind: u16,
    /// The ten first, each by name and points.
    pub ranked: Vec<(String, u32)>,
    /// The player's own points.
    pub own: u32,
}

/// ZC_ACK_RANKING (0x097d, 288 bytes): `u16 kind`, ten times `char name[24]`, `u32 points`, then `u32` the
/// player's own points.
pub fn ranking(packet: &[u8]) -> Option<Ranking> {
    let packet = packet.get(..288)?;
    let ranked = (0..10)
        .map(|place| (name_at(packet, 4 + place * 28, 24), u32_at(packet, 4 + place * 28 + 24)))
        .collect();
    Some(Ranking { kind: u16_at(packet, 2), ranked, own: u32_at(packet, 284) })
}

/// A ranking as ZC_ACK_RANKING2 tells it: the ten are characters' numbers (0 for a place nobody has).
#[derive(Debug, PartialEq)]
pub struct RankingByNumber {
    pub kind: u16,
    /// The ten first, each by its character's number and points.
    pub ranked: Vec<(u32, u32)>,
    pub own: u32,
}

/// ZC_ACK_RANKING2 (0x0af6, 88 bytes): `u16 kind`, `u32 character[10]`, `u32 points[10]`, `u32` the player's
/// own points.
pub fn ranking_by_number(packet: &[u8]) -> Option<RankingByNumber> {
    let packet = packet.get(..88)?;
    let ranked = (0..10).map(|place| (u32_at(packet, 4 + place * 4), u32_at(packet, 44 + place * 4))).collect();
    Some(RankingByNumber { kind: u16_at(packet, 2), ranked, own: u32_at(packet, 84) })
}

/// ZC_UPDATE_RANKING_POINT (0x097e, 12 bytes): `u16 kind, u32 points, u32 total`: the kind of ranking, the
/// points just gained and what the player has now.
pub fn fame_gained(packet: &[u8]) -> Option<(u16, u32, u32)> {
    let packet = packet.get(..12)?;
    Some((u16_at(packet, 2), u32_at(packet, 4), u32_at(packet, 8)))
}

/// ZC_ACK_DISCONNECT_CHARACTER (0x00cd, 3 bytes): `u8 result`: whether the player was thrown out (0: not).
pub fn kicked(packet: &[u8]) -> Option<bool> {
    packet.get(2).map(|result| *result != 0)
}

/// ZC_ACK_STATUS_GM (0x0214, 42 bytes): twelve `u8` (STR, AGI, VIT, INT, DEX, LUK, each followed by its
/// standard value), then fourteen `i16`: attack, refining, the largest and the smallest magic attack, the
/// defence of things and its bonus, the magic defence and its bonus, hit, flee and its bonus, critical, the
/// attack speed and its bonus (the client's `PACKET_ZC_ACK_STATUS_GM`).
pub fn gm_stats(packet: &[u8]) -> Option<([u8; 12], [i16; 14])> {
    let packet = packet.get(..42)?;
    Some((packet[2..14].try_into().ok()?, std::array::from_fn(|value| i16_at(packet, 14 + value * 2))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::longs;

    #[test]
    fn a_ranking_is_asked_for_read_and_written() {
        assert_eq!(ranking_ask(1), [0x7c, 0x09, 1, 0]);
        let mut packet = vec![0u8; 288];
        packet[..4].copy_from_slice(&[0x7d, 0x09, 2, 0]);
        packet[4..8].copy_from_slice(b"Kick");
        packet[28..32].copy_from_slice(&1200u32.to_le_bytes());
        packet[284..288].copy_from_slice(&7u32.to_le_bytes());
        let Ranking { kind, ranked, own } = ranking(&packet).unwrap();
        assert_eq!((kind, ranked.len(), ranked[0].clone(), own), (2, 10, ("Kick".to_owned(), 1200), 7));
        assert_eq!(ranking(&packet[..200]), None);
    }

    #[test]
    fn reads_a_ranking_by_numbers_and_the_fame_gained() {
        let mut packet = vec![0xf6, 0x0a, 1, 0];
        packet.extend(longs(&[150001, 150002, 0, 0, 0, 0, 0, 0, 0, 0]));
        packet.extend(longs(&[900, 40, 0, 0, 0, 0, 0, 0, 0, 0]));
        packet.extend(longs(&[12]));
        assert_eq!(packet.len(), 88);
        let RankingByNumber { kind, ranked, own } = ranking_by_number(&packet).unwrap();
        assert_eq!((kind, ranked.len(), ranked[0], ranked[1], ranked[2], own), (1, 10, (150001, 900), (150002, 40), (0, 0), 12));
        assert_eq!(ranking_by_number(&packet[..87]), None);
        assert_eq!(fame_gained(&[0x7e, 0x09, 2, 0, 10, 0, 0, 0, 0x2c, 1, 0, 0]), Some((2, 10, 300)));
        assert_eq!(fame_gained(&[0x7e, 0x09, 2, 0, 10, 0, 0, 0]), None);
    }

    #[test]
    fn reads_what_a_game_master_hears() {
        assert_eq!((kicked(&[0xcd, 0x00, 0]), kicked(&[0xcd, 0x00, 1]), kicked(&[0xcd, 0x00])), (Some(false), Some(true), None));
        let mut packet = vec![0x14, 0x02];
        packet.extend(1..=12u8);
        for value in 100..114i16 {
            packet.extend(value.to_le_bytes());
        }
        packet[40..42].copy_from_slice(&(-3i16).to_le_bytes());
        let (stats, more) = gm_stats(&packet).unwrap();
        assert_eq!((stats, more[0], more[12], more[13]), ([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], 100, 112, -3));
        assert_eq!(gm_stats(&packet[..41]), None);
    }

    #[test]
    fn writes_the_whisper_settings() {
        let packet = whisper_from("Somebody", false);
        assert_eq!(packet.len(), 27);
        assert_eq!(&packet[..2], &[0xcf, 0x00]);
        assert_eq!(&packet[2..10], b"Somebody");
        assert_eq!((packet[10], packet[26]), (0, 0));
        // A name too long keeps its ending nought.
        let long = whisper_from(&"x".repeat(40), true);
        assert_eq!((long.len(), long[24], long[25], long[26]), (27, b'x', 0, 1));
        assert_eq!(whisper_all(true), vec![0xd0, 0x00, 1]);
        assert_eq!(user_count(), vec![0xc1, 0x00]);
        assert_eq!(whisper_list(), vec![0xd3, 0x00]);
        assert_eq!((remember_warp(), doridori()), (vec![0x1d, 0x01], vec![0xe7, 0x01]));
    }

    #[test]
    fn reads_the_answers() {
        assert_eq!(whisper_set(&[0xd1, 0x00, 1, 2]), Some((true, FULL)));
        assert_eq!(whisper_set(&[0xd1, 0x00, 1]), None);
        let mut list = vec![0xd4, 0x00, 52, 0];
        for name in ["One", "Two Words"] {
            let mut field = [0u8; 24];
            field[..name.len()].copy_from_slice(name.as_bytes());
            list.extend(field);
        }
        assert_eq!(whisper_blocked(&list), vec!["One".to_owned(), "Two Words".to_owned()]);
        assert_eq!(users(&[0xc2, 0x00, 7, 0, 0, 0]), Some(7));
    }
}
