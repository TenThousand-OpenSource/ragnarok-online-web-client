//! The friends list: the list sent on entering, who of it is in the game, asking someone to be a friend and
//! answering one who asks, and removing a friend. The layouts are rAthena's for PACKETVER 20260219
//! (`clif_friendslist_*` and `clif_parse_FriendsList*` in `clif.cpp`, `packets.hpp`); what the client does with
//! each is `CGameMode::ReceiveFriendList`, `ReceiveFriendAddResult` and the arms of its packet switch.

use super::bytes::{name24, text, u32_at};

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

/// How many friends the list holds (the client's title, `%s(%d/%d)` with 40; rAthena's `MAX_FRIENDS`).
pub const MOST: usize = 40;

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
    packet
        .get(4..)
        .unwrap_or_default()
        .chunks_exact(32)
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
    Some((u16::from_le_bytes([packet[2], packet[3]]), Friend { account: u32_at(packet, 4), character: u32_at(packet, 8), name: text(&packet[12..]), offline: false }))
}

/// ZC_DELETE_FRIENDS (0x020a, 10 bytes): `u32 account, u32 character`.
pub fn removed(packet: &[u8]) -> Option<(u32, u32)> {
    let packet = packet.get(..10)?;
    Some((u32_at(packet, 2), u32_at(packet, 6)))
}

/// CZ_ADD_FRIENDS (0x0202, 26 bytes): `char name[24]`: the player of that name is asked to be a friend
/// (`UIWindowMgr::RequestAddFriend`).
pub fn add(name: &str) -> Vec<u8> {
    [&0x0202u16.to_le_bytes()[..], &name24(name)].concat()
}

/// CZ_DELETE_FRIENDS (0x0203, 10 bytes): `u32 account, u32 character` (the client's `MM_DELETE_FRIEND`).
pub fn remove(account: u32, character: u32) -> Vec<u8> {
    [&0x0203u16.to_le_bytes()[..], &account.to_le_bytes(), &character.to_le_bytes()].concat()
}

/// CZ_ACK_REQ_ADD_FRIENDS (0x0208, 14 bytes): `u32 account, u32 character, u32 accept`: the answer to one who
/// asked (`MM_JOIN_FRIEND`).
pub fn answer(account: u32, character: u32, accept: bool) -> Vec<u8> {
    [&0x0208u16.to_le_bytes()[..], &account.to_le_bytes(), &character.to_le_bytes(), &u32::from(accept).to_le_bytes()].concat()
}

/// The colour a friend's name is written in, by the friend's place in the list (the client's
/// `CSession::MemberColor`, whose numbers are read here as red, green and blue).
pub fn colour(place: usize) -> &'static str {
    const COLOURS: [&str; 16] = [
        "#0a3278", "#007878", "#000000", "#780000", "#007800", "#787800", "#780078", "#783200", "#787878", "#000078", "#003232", "#320000", "#003200", "#323200", "#320032", "#780000",
    ];
    COLOURS.get(place).copied().unwrap_or("#323232")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> [u8; 24] {
        name24(name)
    }

    #[test]
    fn reads_the_list() {
        let mut packet = vec![0x01, 0x02, 68, 0];
        for (account, character, name) in [(2_000_001u32, 150_001u32, "One"), (2_000_002, 150_002, "Two Words")] {
            packet.extend(account.to_le_bytes());
            packet.extend(character.to_le_bytes());
            packet.extend(named(name));
        }
        let friends = list(&packet);
        assert_eq!(friends.len(), 2);
        assert_eq!(friends[1], Friend { account: 2_000_002, character: 150_002, name: "Two Words".to_owned(), offline: true });
        // A list with nobody in it is its four bytes.
        assert!(list(&[0x01, 0x02, 4, 0]).is_empty());
    }

    #[test]
    fn reads_who_came_and_went() {
        let mut packet = vec![0x06, 0x02];
        packet.extend(2_000_001u32.to_le_bytes());
        packet.extend(150_001u32.to_le_bytes());
        packet.push(0);
        packet.extend(named("One"));
        assert_eq!(state(&packet), Some(Friend { account: 2_000_001, character: 150_001, name: "One".to_owned(), offline: false }));
        packet[10] = 1;
        assert!(state(&packet).is_some_and(|friend| friend.offline));
        assert_eq!(state(&packet[..34]), None);
    }

    #[test]
    fn reads_asking_and_its_answer() {
        let mut asks = vec![0x07, 0x02];
        asks.extend(7u32.to_le_bytes());
        asks.extend(9u32.to_le_bytes());
        asks.extend(named("Asker"));
        assert_eq!(asked(&asks), Some((7, 9, "Asker".to_owned())));

        let mut answer = vec![0x09, 0x02, 0, 0];
        answer.extend(7u32.to_le_bytes());
        answer.extend(9u32.to_le_bytes());
        answer.extend(named("Asker"));
        assert_eq!(answered(&answer), Some((added::DONE, Friend { account: 7, character: 9, name: "Asker".to_owned(), offline: false })));
        answer[2] = 3;
        assert_eq!(answered(&answer).map(|(result, _)| result), Some(added::THEIR_LIST_FULL));

        let mut gone = vec![0x0a, 0x02];
        gone.extend(7u32.to_le_bytes());
        gone.extend(9u32.to_le_bytes());
        assert_eq!(removed(&gone), Some((7, 9)));
    }

    #[test]
    fn writes_what_the_client_sends() {
        let packet = add("Somebody");
        assert_eq!((packet.len(), &packet[..2], &packet[2..10], packet[10]), (26, &[0x02, 0x02][..], &b"Somebody"[..], 0));
        // A name too long keeps its ending nought.
        assert_eq!(add(&"x".repeat(40))[25], 0);
        assert_eq!(remove(7, 9), vec![0x03, 0x02, 7, 0, 0, 0, 9, 0, 0, 0]);
        assert_eq!(answer(7, 9, true), vec![0x08, 0x02, 7, 0, 0, 0, 9, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(answer(7, 9, false)[10], 0);
    }

    #[test]
    fn colours_go_by_the_place_in_the_list() {
        assert_eq!(colour(0), "#0a3278");
        assert_eq!(colour(3), colour(15));
        assert_eq!(colour(16), "#323232");
        assert_eq!(colour(39), "#323232");
    }
}
