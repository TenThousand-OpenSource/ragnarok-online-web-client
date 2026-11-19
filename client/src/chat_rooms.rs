//! Chat rooms as rules: what a room is and what its sign and its window say of it (`UIChatRoomTitle`,
//! `UIChatRoomWnd`), what the window that makes one offers and refuses (`UIChatRoomMakeWnd`), and the lines said
//! for a room not made or not entered. The packets that tell of rooms are read in `net/chat_rooms.rs`.

/// The kinds of room (rAthena's `e_status_chatroom`): a player's with a password, a player's open to all, an
/// NPC's that is entered (`waitingroom` with a limit), an NPC's that is only a sign.
pub mod kind {
    pub const PRIVATE: u8 = 0;
    pub const PUBLIC: u8 = 1;
    pub const ARENA: u8 = 2;
    pub const SIGN: u8 = 3;
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
