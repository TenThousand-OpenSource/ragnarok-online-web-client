//! The battlegrounds: applying for one, the queue, the question whether to enter, and what a battle's map
//! tells (teams, score, where the others of the team are and their life). The layouts are rAthena's for
//! PACKETVER 20260219 (`clif_bg_queue_*`, `clif_parse_bg_queue_*`, `clif_sendbgemblem_area`,
//! `clif_bg_updatescore`, `clif_bg_xy`, `clif_bg_hp` in `clif.cpp`) and the client's `EntryQueue*Packet` of
//! `Source/Contents/EntryQueue/EntryQueueMgr.cpp`.

use super::bytes::{body, int, long, short, text, text_at, word, Writer};

/// ZC_ACK_ENTRY_QUEUE_APPLY (0x08d8, 27 bytes): the answer to applying.
pub const ZC_ACK_ENTRY_QUEUE_APPLY: u16 = 0x08d8;
/// ZC_NOTIFY_ENTRY_QUEUE_APPLY (0x08d9, 30 bytes): one's place in the queue.
pub const ZC_NOTIFY_ENTRY_QUEUE_APPLY: u16 = 0x08d9;
/// ZC_ACK_ENTRY_QUEUE_CANCEL (0x08db, 27 bytes): the answer to leaving the queue.
pub const ZC_ACK_ENTRY_QUEUE_CANCEL: u16 = 0x08db;
/// ZC_NOTIFY_ENTRY_QUEUE_ADMISSION (0x08dc, 26 bytes): the client reads nothing of it.
pub const ZC_NOTIFY_ENTRY_QUEUE_ADMISSION: u16 = 0x08dc;
/// ZC_REPLY_ACK_ENTRY_QUEUE_ADMISSION (0x08de, 27 bytes): a refusal said in a box. rAthena does not send it.
pub const ZC_REPLY_ACK_ENTRY_QUEUE_ADMISSION: u16 = 0x08de;
/// ZC_NOTIFY_LOBBY_ADMISSION (0x08df, 50 bytes): the battle is ready; enter?
pub const ZC_NOTIFY_LOBBY_ADMISSION: u16 = 0x08df;
/// ZC_REPLY_ACK_LOBBY_ADMISSION (0x08e1, 51 bytes): someone of the queue agreed to enter (a gong).
pub const ZC_REPLY_ACK_LOBBY_ADMISSION: u16 = 0x08e1;
/// ZC_ENTRY_QUEUE_INIT (0x090e, 2 bytes): the queue is forgotten.
pub const ZC_ENTRY_QUEUE_INIT: u16 = 0x090e;
/// ZC_BATTLEFIELD_NOTIFY_CAMPINFO (0x02dd, 32 bytes): which team someone is of.
pub const ZC_BATTLEFIELD_NOTIFY_CAMPINFO: u16 = 0x02dd;
/// ZC_BATTLEFIELD_NOTIFY_POINT (0x02de, 6 bytes): the score.
pub const ZC_BATTLEFIELD_NOTIFY_POINT: u16 = 0x02de;
/// ZC_BATTLEFIELD_NOTIFY_POSITION (0x02df, 36 bytes): where one of the team is.
pub const ZC_BATTLEFIELD_NOTIFY_POSITION: u16 = 0x02df;
/// ZC_BATTLEFIELD_NOTIFY_HP (0x02e0, 34 bytes): the life of one of the team, as older servers say it.
pub const ZC_BATTLEFIELD_NOTIFY_HP: u16 = 0x02e0;
/// ZC_BATTLEFIELD_NOTIFY_HP2 (0x0a0e, 14 bytes): the same, as rAthena says it to this client.
pub const ZC_BATTLEFIELD_NOTIFY_HP2: u16 = 0x0a0e;

/// ZC_BATTLEFIELD_CHAT (0x02dc): a line of the battleground's own chat.
pub const ZC_BATTLEFIELD_CHAT: u16 = 0x02dc;

/// ZC_BATTLEFIELD_CHAT: `u16 length, u32 account, char name[24], char text[]`: who said it and the line, which
/// has the name in it already (`CGameMode::ReceiveActorBroadcastText`, which passes by a line of more than 256
/// bytes or of none).
pub fn chat_heard(packet: &[u8]) -> Option<(u32, String)> {
    let words = body(packet, 32).filter(|words| !words.is_empty() && words.len() <= 256)?;
    Some((long(packet, 4)?, text(words)))
}

/// CZ_BATTLEFIELD_CHAT (0x02db): `u16 length, char text[]`: a line for one's team, "name : words" with its
/// zero (`CGameMode::SendMsg`, `MM_CHAT_BATTLEFIELD`).
pub fn chat(name: &str, words: &str) -> Vec<u8> {
    Writer::sized(0x02db).bytes(format!("{name} : {words}").as_bytes()).u8(0).done()
}

/// ZC_ACK_ENTRY_QUEUE_APPLY, ZC_ACK_ENTRY_QUEUE_CANCEL (27 bytes): `u8 result, char name[24]`: the result and
/// the battleground's name.
pub fn answered(packet: &[u8]) -> Option<(u8, String)> {
    Some((*packet.get(2)?, text_at(packet, 3, 24)?))
}

/// The result alone of a packet that begins with it: ZC_REPLY_ACK_ENTRY_QUEUE_ADMISSION and
/// ZC_REPLY_ACK_LOBBY_ADMISSION (`u8 result`, then names the client does not read).
pub fn result(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// ZC_NOTIFY_ENTRY_QUEUE_APPLY (30 bytes): `char name[24], i32 place`: the battleground and one's place in
/// its queue (rAthena says how many wait in all).
pub fn queued(packet: &[u8]) -> Option<(String, i32)> {
    Some((text_at(packet, 2, 24)?, int(packet, 26)?))
}

/// ZC_NOTIFY_LOBBY_ADMISSION (50 bytes): `char name[24], char lobby[24]`: the battleground that is ready and
/// the name of its lobby, which the answer gives back.
pub fn ready(packet: &[u8]) -> Option<(String, String)> {
    Some((text_at(packet, 2, 24)?, text_at(packet, 26, 24)?))
}

/// ZC_BATTLEFIELD_NOTIFY_CAMPINFO (32 bytes): `u32 account, char name[24], u16 camp`: who is of which team
/// (the client keeps the camp's low byte).
pub fn camp(packet: &[u8]) -> Option<(u32, u8)> {
    Some((long(packet, 2)?, word(packet, 30)? as u8))
}

/// ZC_BATTLEFIELD_NOTIFY_POINT (6 bytes): `i16 first, i16 second`: the score of the two teams.
pub fn points(packet: &[u8]) -> Option<(i32, i32)> {
    Some((short(packet, 2)?.into(), short(packet, 4)?.into()))
}

/// ZC_BATTLEFIELD_NOTIFY_POSITION (36 bytes): `u32 account, char name[24], u16 job, i16 x, i16 y`: where one
/// of the team is; -1 and -1 when they are gone from the map.
pub fn place(packet: &[u8]) -> Option<(u32, Option<(u16, u16)>)> {
    let (account, x, y) = (long(packet, 2)?, short(packet, 32)?, short(packet, 34)?);
    Some((account, (x >= 0 && y >= 0).then_some((x as u16, y as u16))))
}

/// The life of one of the team: ZC_BATTLEFIELD_NOTIFY_HP (34 bytes: `u32 account, char name[24], i16 hp,
/// i16 most`) or ZC_BATTLEFIELD_NOTIFY_HP2 (14 bytes: `u32 account, i32 hp, i32 most`).
pub fn life(id: u16, packet: &[u8]) -> Option<(u32, i32, i32)> {
    let account = long(packet, 2)?;
    if id == ZC_BATTLEFIELD_NOTIFY_HP {
        Some((account, short(packet, 30)?.into(), short(packet, 32)?.into()))
    } else {
        Some((account, int(packet, 6)?, int(packet, 10)?))
    }
}

/// CZ_REQ_ENTRY_QUEUE_APPLY (0x08d7, 28 bytes): `u16 way, char name[24]`: to queue for that battleground, alone
/// (1), as a party (2) or as a guild (4) (`CEntryQueueMgr::SendApply`).
pub fn apply(way: u16, name: &str) -> Vec<u8> {
    Writer::new(0x08d7).u16(way).name(name).done()
}

/// CZ_REQ_ENTRY_QUEUE_CANCEL (0x08da, 26 bytes): `char name[24]`: to leave its queue (`SendCancel`).
pub fn cancel(name: &str) -> Vec<u8> {
    Writer::new(0x08da).name(name).done()
}

/// CZ_REQ_ENTRY_QUEUE_RANKING (0x090a, 26 bytes): `char name[24]`: what is one's place in its queue?
/// (`SendRankingRequest`.)
pub fn ask_place(name: &str) -> Vec<u8> {
    Writer::new(0x090a).name(name).done()
}

/// CZ_REPLY_LOBBY_ADMISSION (0x08e0, 51 bytes): `u8 result, char name[24], char lobby[24]`: to enter the
/// battle that is ready (1) or not (2) (`SendAdmissionReply`).
pub fn admit(enter: bool, name: &str, lobby: &str) -> Vec<u8> {
    Writer::new(0x08e0).u8(if enter { 1 } else { 2 }).name(name).name(lobby).done()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::bytes::testing::{field, packet};

    #[test]
    fn reads_the_answers_of_the_queue() {
        let answer = packet(0x08d8, &[&[7], &field("Flavius", 24)]);
        assert_eq!(answered(&answer), Some((7, "Flavius".to_owned())));
        assert_eq!(result(&answer), Some(7));
        assert_eq!(answered(&answer[..20]), None);
        let place = packet(0x08d9, &[&field("KVM (Level 80 and up)", 24), &3i32.to_le_bytes()]);
        assert_eq!(queued(&place), Some(("KVM (Level 80 and up)".to_owned(), 3)));
        let begins = packet(0x08df, &[&field("Flavius", 24), &field("Lobby", 24)]);
        assert_eq!(ready(&begins), Some(("Flavius".to_owned(), "Lobby".to_owned())));
        assert_eq!(ready(&begins[..40]), None);
    }

    #[test]
    fn reads_what_a_battles_map_tells() {
        let team = packet(0x02dd, &[&2_000_006u32.to_le_bytes(), &field("Six", 24), &2u16.to_le_bytes()]);
        assert_eq!(camp(&team), Some((2_000_006, 2)));
        assert_eq!(points(&packet(0x02de, &[&3i16.to_le_bytes(), &12i16.to_le_bytes()])), Some((3, 12)));
        let at = |x: i16, y: i16| {
            packet(0x02df, &[&2_000_007u32.to_le_bytes(), &field("Seven", 24), &4008u16.to_le_bytes(), &x.to_le_bytes(), &y.to_le_bytes()])
        };
        assert_eq!(place(&at(150, 62)), Some((2_000_007, Some((150, 62)))));
        assert_eq!(place(&at(-1, -1)), Some((2_000_007, None)));
        let old = packet(0x02e0, &[&2_000_007u32.to_le_bytes(), &field("Seven", 24), &40i16.to_le_bytes(), &100i16.to_le_bytes()]);
        assert_eq!(life(ZC_BATTLEFIELD_NOTIFY_HP, &old), Some((2_000_007, 40, 100)));
        let new = packet(0x0a0e, &[&2_000_007u32.to_le_bytes(), &123_456i32.to_le_bytes(), &200_000i32.to_le_bytes()]);
        assert_eq!(life(ZC_BATTLEFIELD_NOTIFY_HP2, &new), Some((2_000_007, 123_456, 200_000)));
    }

    #[test]
    fn the_battlegrounds_own_chat_is_written_and_read() {
        assert_eq!(chat("Six", "go"), [&[0xdb, 0x02, 13, 0][..], b"Six : go\0"].concat());
        let mut heard = packet(0x02dc, &[&[0, 0], &2_000_006u32.to_le_bytes(), &field("Six", 24), b"Six : go\0"]);
        let length = heard.len() as u16;
        heard[2..4].copy_from_slice(&length.to_le_bytes());
        assert_eq!(chat_heard(&heard), Some((2_000_006, "Six : go".to_owned())));
        // A line of nothing is passed by.
        heard[2..4].copy_from_slice(&32u16.to_le_bytes());
        assert_eq!(chat_heard(&heard), None);
    }

    #[test]
    fn what_is_sent_is_as_long_as_the_server_expects() {
        assert_eq!(apply(4, "Tierra Gorge").len(), 28);
        assert_eq!(&apply(4, "Tierra Gorge")[..4], [0xd7, 0x08, 4, 0]);
        assert_eq!(cancel("Flavius").len(), 26);
        assert_eq!(ask_place("Flavius").len(), 26);
        let enter = admit(true, "Flavius", "Flavius");
        assert_eq!((enter.len(), enter[2], &enter[27..34]), (51, 1, &b"Flavius"[..]));
        assert_eq!(admit(false, "Flavius", "")[2], 2);
    }
}
