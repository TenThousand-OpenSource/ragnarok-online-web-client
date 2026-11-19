//! What the chat commands send to the map server and hear back: who may whisper, how many play.

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

/// The answers' results: done, not done, not done because the list is full.
pub const DONE: u8 = 0;
pub const FAILED: u8 = 1;
pub const FULL: u8 = 2;

/// CZ_REQ_USER_COUNT (0x00c1), CZ_REQ_WHISPER_LIST (0x00d3): nothing but the number.
pub fn user_count() -> Vec<u8> {
    CZ_REQ_USER_COUNT.to_le_bytes().to_vec()
}

pub fn whisper_list() -> Vec<u8> {
    0x00d3u16.to_le_bytes().to_vec()
}

/// CZ_REMEMBER_WARPPOINT (0x011d): this place is kept for Warp Portal (`/memo`).
pub fn remember_warp() -> Vec<u8> {
    0x011du16.to_le_bytes().to_vec()
}

/// CZ_DORIDORI (0x01e7): the head shakes; the server gives a novice's SP back faster for it (`/doridori`).
pub fn doridori() -> Vec<u8> {
    0x01e7u16.to_le_bytes().to_vec()
}

/// CZ_SETTING_WHISPER_PC (0x00cf, 27 bytes): `char name[24]`, `u8 allow`: whispers from that character are
/// blocked (0) or let in again (1). The name is cut to what fits with its ending nought.
pub fn whisper_from(name: &str, allow: bool) -> Vec<u8> {
    let mut packet = vec![0u8; 27];
    packet[..2].copy_from_slice(&0x00cfu16.to_le_bytes());
    packet[2..26].copy_from_slice(&super::bytes::name24(name));
    packet[26] = u8::from(allow);
    packet
}

/// CZ_SETTING_WHISPER_STATE (0x00d0, 3 bytes): `u8 allow`: all whispers are blocked (0) or let in (1).
pub fn whisper_all(allow: bool) -> Vec<u8> {
    vec![0xd0, 0x00, u8::from(allow)]
}

/// ZC_SETTING_WHISPER_PC (0x00d1) and ZC_SETTING_WHISPER_STATE (0x00d2), 4 bytes each: `u8 allow`, `u8 result`:
/// what was asked (0 to block, 1 to let in) and how it went.
pub fn whisper_set(packet: &[u8]) -> Option<(bool, u8)> {
    let packet = packet.get(..4)?;
    Some((packet[2] != 0, packet[3]))
}

/// ZC_WHISPER_LIST (0x00d4): `u16 length`, then `char name[24]` for each character blocked.
pub fn whisper_blocked(packet: &[u8]) -> Vec<String> {
    packet
        .get(4..)
        .unwrap_or_default()
        .chunks_exact(24)
        .map(super::bytes::text)
        .collect()
}

/// ZC_USER_COUNT (0x00c2, 6 bytes): `u32 count`.
pub fn users(packet: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(packet.get(2..6)?.try_into().ok()?))
}

/// ZC_ACK_RANKING: the ten first of a ranking and the player's own points.
pub const ZC_ACK_RANKING: u16 = 0x097d;

/// CZ_REQ_RANKING (0x097c, 4 bytes): `u16 kind`: the ten first blacksmiths (0), alchemists (1) or taekwons (2)
/// are asked for (`/blacksmith`, `/alchemist`, `/taekwon`).
pub fn ranking_ask(kind: u16) -> Vec<u8> {
    [0x097cu16.to_le_bytes(), kind.to_le_bytes()].concat()
}

/// ZC_ACK_RANKING (0x097d, 288 bytes): `u16 kind`, ten times `char name[24]`, `u32 points`, then `u32` the
/// player's own points. Gives the kind, the ten and the player's own.
pub fn ranking(packet: &[u8]) -> Option<(u16, Vec<(String, u32)>, u32)> {
    let packet = packet.get(..288)?;
    let ranked = (0..10).map(|place| (super::bytes::name_at(packet, 4 + place * 28, 24), super::bytes::u32_at(packet, 4 + place * 28 + 24))).collect();
    Some((super::bytes::u16_at(packet, 2), ranked, super::bytes::u32_at(packet, 284)))
}

/// A ranking as the client writes it into the chat: a heading with the ranking's name, a line to each of the
/// ten ("[ %2d ] %-18s  :  %4d POINT"), a rule, the player's own points and a rule again.
pub fn ranking_lines(kind: u16, ranked: &[(String, u32)], own: u32) -> Vec<String> {
    let named = ["BLACKSMITH", "ALCHEMIST", "TAEKWON"].get(usize::from(kind)).copied().unwrap_or("KILLER");
    let rule = "=".repeat(39);
    let mut lines = vec![format!("========== {named} RANK ============")];
    lines.extend(ranked.iter().enumerate().map(|(place, (name, points))| format!("[ {:2} ] {name:<18}  :  {points:4} POINT", place + 1)));
    lines.extend([rule.clone(), format!("MY POINT    :  {own:4} POINT "), rule]);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ranking_is_asked_for_read_and_written() {
        assert_eq!(ranking_ask(1), [0x7c, 0x09, 1, 0]);
        let mut packet = vec![0u8; 288];
        packet[..4].copy_from_slice(&[0x7d, 0x09, 2, 0]);
        packet[4..8].copy_from_slice(b"Kick");
        packet[28..32].copy_from_slice(&1200u32.to_le_bytes());
        packet[284..288].copy_from_slice(&7u32.to_le_bytes());
        let (kind, ranked, own) = ranking(&packet).unwrap();
        assert_eq!((kind, ranked.len(), ranked[0].clone(), own), (2, 10, ("Kick".to_owned(), 1200), 7));
        assert_eq!(ranking(&packet[..200]), None);
        let lines = ranking_lines(kind, &ranked[..1], own);
        assert_eq!(lines[0], "========== TAEKWON RANK ============");
        assert_eq!(lines[1], "[  1 ] Kick                :  1200 POINT");
        assert_eq!(lines[3], "MY POINT    :     7 POINT ");
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
