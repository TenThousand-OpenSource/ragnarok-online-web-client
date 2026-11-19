//! Chat rooms: a player's room and the waiting room of an NPC, the sign of one over its owner's head, who is
//! in the room the character sits in, and what the client sends to make, enter, change and leave one. The
//! layouts are rAthena's for PACKETVER 20260219 (`clif_dispchat`, `clif_joinchatok` and their neighbours in
//! `src/map/clif.cpp`); what the client does with each is `CGameMode`'s (`GameModePackets.cpp`,
//! `GameModeHandlers.cpp`) and its `UIChatRoomWnd`, `UIChatRoomMakeWnd` and `UIChatRoomTitle`.

use super::bytes::{long, name24, text, text_at, word};

/// ZC_ACK_CREATE_CHATROOM (0x00d6, 3 bytes): `u8 result`: 0 the room is made, 1 there are too many rooms, 2
/// there is a room of that title.
pub const ZC_ACK_CREATE_CHATROOM: u16 = 0x00d6;
/// ZC_ROOM_NEWENTRY (0x00d7): `u16 length, u32 owner, u32 room, u16 limit, u16 users, u8 kind, char title[]`:
/// the sign of a room over its owner.
pub const ZC_ROOM_NEWENTRY: u16 = 0x00d7;
/// ZC_DESTROY_ROOM (0x00d8, 6 bytes): `u32 room`: the room is no more.
pub const ZC_DESTROY_ROOM: u16 = 0x00d8;
/// ZC_REFUSE_ENTER_ROOM (0x00da, 3 bytes): `u8 why`.
pub const ZC_REFUSE_ENTER_ROOM: u16 = 0x00da;
/// ZC_ENTER_ROOM (0x00db): `u16 length, u32 room`, then for everyone in it `u32 role, char name[24]`.
pub const ZC_ENTER_ROOM: u16 = 0x00db;
/// ZC_MEMBER_NEWENTRY (0x00dc, 28 bytes): `u16 users, char name[24]`: someone came in.
pub const ZC_MEMBER_NEWENTRY: u16 = 0x00dc;
/// ZC_MEMBER_EXIT (0x00dd, 29 bytes): `u16 users, char name[24], u8 kicked`: someone left.
pub const ZC_MEMBER_EXIT: u16 = 0x00dd;
/// ZC_CHANGE_CHATROOM (0x00df): as ZC_ROOM_NEWENTRY: the room the character sits in was changed.
pub const ZC_CHANGE_CHATROOM: u16 = 0x00df;
/// ZC_ROLE_CHANGE (0x00e1, 30 bytes): `u32 role, char name[24]`: who owns the room now.
pub const ZC_ROLE_CHANGE: u16 = 0x00e1;

/// The role of the one a room belongs to; everyone else has `MEMBER`.
pub const OWNER: u32 = 0;
pub const MEMBER: u32 = 1;

/// The kinds of room (rAthena's `e_status_chatroom`): a player's with a password, a player's open to all, an
/// NPC's that is entered (`waitingroom` with a limit), an NPC's that is only a sign.
pub mod kind {
    pub const PRIVATE: u8 = 0;
    pub const PUBLIC: u8 = 1;
    pub const ARENA: u8 = 2;
    pub const SIGN: u8 = 3;
}

/// The longest title the client keeps of a room it is told of (`ReceiveChatRoomCreate`: 60 bytes).
const TITLE_KEPT: usize = 60;

/// A room as its sign tells of it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Room {
    pub owner: u32,
    pub id: u32,
    pub limit: u16,
    pub users: u16,
    pub kind: u8,
    pub title: String,
}

impl Room {
    /// What the sign over the owner says (`UIChatRoomTitle::SendMsg`, `UM_SETINFO`): the title up to its
    /// first line break, and how many of how many are in; a room that is only a sign has its title alone.
    pub fn sign(&self) -> String {
        let title = self.title.split('\r').next().unwrap_or_default();
        if self.kind == kind::SIGN { title.to_owned() } else { format!("{title} ({}/{})", self.users, self.limit) }
    }

    /// What the room's window is called (`UIChatRoomWnd::SendMsg`, `UM_SETINFO`): `"%s%s (%d/%d)"` with the
    /// client's word for a room with a password or for one without (its lines 89 and 90) before the title.
    pub fn heading(&self, word: &str) -> String {
        format!("{word}{} ({}/{})", self.title.split('\r').next().unwrap_or_default(), self.users, self.limit)
    }
}

/// ZC_ROOM_NEWENTRY and ZC_CHANGE_CHATROOM. The title has no zero after it.
pub fn room(packet: &[u8]) -> Option<Room> {
    let title = packet.get(17..)?;
    Some(Room { owner: long(packet, 4)?, id: long(packet, 8)?, limit: word(packet, 12)?, users: word(packet, 14)?, kind: packet[16], title: text(&title[..title.len().min(TITLE_KEPT)]) })
}

/// ZC_DESTROY_ROOM: which room.
pub fn destroyed(packet: &[u8]) -> Option<u32> {
    long(packet, 2)
}

/// ZC_ACK_CREATE_CHATROOM and ZC_REFUSE_ENTER_ROOM: the one byte each says.
pub fn result(packet: &[u8]) -> Option<u8> {
    packet.get(2).copied()
}

/// The line of the client's message table said for a room not made (`ReceiveChatRoomCreateResult`), and the
/// words until the table is read: none for a room that was made.
pub fn not_made(result: u8) -> Option<(usize, &'static str)> {
    match result {
        1 => Some((65, "Room limit exceeded.")),
        2 => Some((66, "Same room already exists.")),
        _ => None,
    }
}

/// The same for a room not entered (`ReceiveChatRoomEnterRefused`): the client has no line for 3.
pub fn refused(why: u8) -> Option<(usize, &'static str)> {
    Some(match why {
        0 => (67, "Room is full."),
        1 => (7, "Incorrect Password."),
        2 => (68, "You have been kicked out of this room."),
        4 => (55, "You don't have enough zeny."),
        5 => (432, "Your level is too low to enter."),
        6 => (433, "Your level is too high to enter."),
        7 => (434, "Your job is not suitable for this room."),
        _ => return None,
    })
}

/// ZC_ENTER_ROOM: the room, and everyone in it with their roles.
pub fn entered(packet: &[u8]) -> Option<(u32, Vec<(u32, String)>)> {
    let members = packet.get(8..)?.chunks_exact(28).filter_map(|member| Some((long(member, 0)?, text_at(member, 4, 24)?))).collect();
    Some((long(packet, 4)?, members))
}

/// ZC_MEMBER_NEWENTRY: how many are in now, and who came.
pub fn joined(packet: &[u8]) -> Option<(u16, String)> {
    Some((word(packet, 2)?, text_at(packet, 4, 24)?))
}

/// ZC_MEMBER_EXIT: how many are in now, who left, and whether they were thrown out.
pub fn left(packet: &[u8]) -> Option<(u16, String, bool)> {
    Some((word(packet, 2)?, text_at(packet, 4, 24)?, *packet.get(28)? == 1))
}

/// ZC_ROLE_CHANGE: the role, and whose it is.
pub fn role(packet: &[u8]) -> Option<(u32, String)> {
    Some((long(packet, 2)?, text_at(packet, 6, 24)?))
}

/// What the player sets of a room in the window that makes or changes one (`ChatRoomSetting`).
#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub title: String,
    pub password: String,
    pub public: bool,
    pub limit: u16,
}

/// The numbers of people the window offers (`UIChatRoomMakeWnd::OnCreate`); the last is what it has at first.
pub const LIMITS: [u16; 6] = [2, 3, 5, 10, 15, 20];

/// What is wrong with a setting, as a line of the client's message table and the words until it is read
/// (`UIChatRoomMakeWnd::SendMsg`, `ID_OK`): no title; a room with a password whose password is missing or
/// shorter than 4. (The client also refuses a title with a word of its list of bad words, which this client
/// does not have.)
pub fn wrong(setting: &Setting) -> Option<(usize, &'static str)> {
    if setting.title.is_empty() {
        Some((13, "Please enter room title."))
    } else if !setting.public && setting.password.is_empty() {
        Some((15, "Please enter the password."))
    } else if !setting.public && setting.password.len() < 4 {
        Some((16, "Please enter at least 4 characters."))
    } else {
        None
    }
}

/// CZ_CREATE_CHATROOM (0x00d5) and CZ_CHANGE_CHATROOM (0x00de), which are laid out alike: `u16 length, u16
/// limit, u8 public, char password[8], char title[]`, the title without a zero after it
/// (`CGameMode::SendMsg`, `MM_CREATE_CHATROOM`).
fn setting(id: u16, setting: &Setting) -> Vec<u8> {
    let mut packet = id.to_le_bytes().to_vec();
    packet.extend(((setting.title.len() + 15) as u16).to_le_bytes());
    packet.extend(setting.limit.to_le_bytes());
    packet.push(u8::from(setting.public));
    packet.extend(password(&setting.password));
    packet.extend(setting.title.as_bytes());
    packet
}

fn password(password: &str) -> [u8; 8] {
    let mut field = [0u8; 8];
    let length = password.len().min(8);
    field[..length].copy_from_slice(&password.as_bytes()[..length]);
    field
}

pub fn create(room: &Setting) -> Vec<u8> {
    setting(0x00d5, room)
}

pub fn change(room: &Setting) -> Vec<u8> {
    setting(0x00de, room)
}

/// CZ_REQ_ENTER_ROOM (0x00d9, 14 bytes): `u32 room, char password[8]`.
pub fn enter(room: u32, word: &str) -> Vec<u8> {
    let mut packet = 0x00d9u16.to_le_bytes().to_vec();
    packet.extend(room.to_le_bytes());
    packet.extend(password(word));
    packet
}

/// CZ_REQ_ROLE_CHANGE (0x00e0, 30 bytes): `u32 role (0), char name[24]`: the room is given to that one.
pub fn give(name: &str) -> Vec<u8> {
    let mut packet = 0x00e0u16.to_le_bytes().to_vec();
    packet.extend(OWNER.to_le_bytes());
    packet.extend(name24(name));
    packet
}

/// CZ_REQ_EXPEL_MEMBER (0x00e2, 26 bytes): `char name[24]`: that one is thrown out.
pub fn expel(name: &str) -> Vec<u8> {
    let mut packet = 0x00e2u16.to_le_bytes().to_vec();
    packet.extend(name24(name));
    packet
}

/// CZ_EXIT_ROOM (0x00e3, 2 bytes): the character leaves the room.
pub fn leave() -> Vec<u8> {
    0x00e3u16.to_le_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign(id: u16, kind: u8, title: &[u8]) -> Vec<u8> {
        let mut packet = id.to_le_bytes().to_vec();
        packet.extend(((17 + title.len()) as u16).to_le_bytes());
        packet.extend(110_000_005u32.to_le_bytes());
        packet.extend(42u32.to_le_bytes());
        packet.extend(20u16.to_le_bytes());
        packet.extend(3u16.to_le_bytes());
        packet.push(kind);
        packet.extend(title);
        packet
    }

    #[test]
    fn reads_a_sign_and_writes_what_it_says() {
        let told = room(&sign(ZC_ROOM_NEWENTRY, kind::PUBLIC, b"Selling apples")).unwrap();
        assert_eq!(told, Room { owner: 110_000_005, id: 42, limit: 20, users: 3, kind: kind::PUBLIC, title: "Selling apples".to_owned() });
        assert_eq!(told.sign(), "Selling apples (3/20)");
        assert_eq!(told.heading("Public: "), "Public: Selling apples (3/20)");
        assert_eq!(room(&sign(ZC_ROOM_NEWENTRY, kind::SIGN, b"Arena\rmore")).unwrap().sign(), "Arena");
        assert_eq!(room(&sign(ZC_CHANGE_CHATROOM, kind::PRIVATE, &[b'x'; 80])).unwrap().title.len(), 60);
        assert_eq!(room(&[0xd7, 0, 10, 0, 1, 2, 3, 4, 5, 6]), None);
    }

    #[test]
    fn reads_who_is_in_a_room_and_who_comes_and_goes() {
        let mut packet = vec![0xdb, 0, 64, 0, 42, 0, 0, 0];
        for (role, name) in [(OWNER, "Waiting Room"), (MEMBER, "Player")] {
            packet.extend(role.to_le_bytes());
            packet.extend(name24(name));
        }
        assert_eq!(entered(&packet), Some((42, vec![(OWNER, "Waiting Room".to_owned()), (MEMBER, "Player".to_owned())])));
        let mut came = vec![0xdc, 0, 3, 0];
        came.extend(name24("Guest"));
        assert_eq!(joined(&came), Some((3, "Guest".to_owned())));
        let mut went = vec![0xdd, 0, 2, 0];
        went.extend(name24("Guest"));
        went.push(1);
        assert_eq!(left(&went), Some((2, "Guest".to_owned(), true)));
        let mut owns = vec![0xe1, 0, 0, 0, 0, 0];
        owns.extend(name24("Guest"));
        assert_eq!(role(&owns), Some((OWNER, "Guest".to_owned())));
        for (id, length) in [(ZC_MEMBER_NEWENTRY, came.len()), (ZC_MEMBER_EXIT, went.len()), (ZC_ROLE_CHANGE, owns.len()), (ZC_DESTROY_ROOM, 6), (ZC_ACK_CREATE_CHATROOM, 3), (ZC_REFUSE_ENTER_ROOM, 3)] {
            assert_eq!(crate::net::packet_length(id), Some(length as i32), "{id:#06x}");
        }
        for id in [ZC_ROOM_NEWENTRY, ZC_ENTER_ROOM, ZC_CHANGE_CHATROOM] {
            assert_eq!(crate::net::packet_length(id), Some(-1), "{id:#06x}");
        }
    }

    #[test]
    fn says_why_a_room_was_not_made_or_not_entered() {
        assert_eq!((not_made(0), not_made(1).map(|said| said.0), not_made(2).map(|said| said.0)), (None, Some(65), Some(66)));
        assert_eq!([0, 1, 2, 3, 4, 5, 6, 7].map(|why| refused(why).map(|said| said.0)), [Some(67), Some(7), Some(68), None, Some(55), Some(432), Some(433), Some(434)]);
    }

    #[test]
    fn a_setting_is_checked_as_the_window_checks_it() {
        let room = |title: &str, password: &str, public: bool| Setting { title: title.to_owned(), password: password.to_owned(), public, limit: 20 };
        assert_eq!(wrong(&room("", "", true)).map(|said| said.0), Some(13));
        assert_eq!(wrong(&room("Chat", "", false)).map(|said| said.0), Some(15));
        assert_eq!(wrong(&room("Chat", "abc", false)).map(|said| said.0), Some(16));
        assert_eq!(wrong(&room("Chat", "abcd", false)), None);
        assert_eq!(wrong(&room("Chat", "", true)), None);
    }

    #[test]
    fn builds_what_the_client_sends() {
        let room = Setting { title: "Chat".to_owned(), password: "abcd".to_owned(), public: false, limit: 10 };
        assert_eq!(create(&room), [0xd5, 0, 19, 0, 10, 0, 0, b'a', b'b', b'c', b'd', 0, 0, 0, 0, b'C', b'h', b'a', b't']);
        assert_eq!(change(&room)[..4], [0xde, 0, 19, 0]);
        assert_eq!(enter(42, "abcd"), [0xd9, 0, 42, 0, 0, 0, b'a', b'b', b'c', b'd', 0, 0, 0, 0]);
        for packet in [enter(1, ""), give("Guest"), expel("Guest"), leave()] {
            let id = u16::from_le_bytes([packet[0], packet[1]]);
            assert_eq!(crate::net::packet_length(id), Some(packet.len() as i32), "{id:#06x}");
        }
        assert_eq!(give("Guest")[..8], [0xe0, 0, 0, 0, 0, 0, b'G', b'u']);
    }
}
