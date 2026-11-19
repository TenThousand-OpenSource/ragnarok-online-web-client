//! What the app keeps of chat rooms: the rooms whose signs it was told of, the room the character sits in,
//! the window that makes or changes one.

use std::collections::HashMap;

use crate::net::chat_rooms::{LIMITS, MEMBER, OWNER, Room};

/// The room the character sits in (the client's `UIChatRoomWnd` and `CGameMode::chatRoom`).
pub(in crate::web) struct Sitting {
    /// What is known of it: of a room just made, its number is not (the server tells it only with a change).
    pub room: Room,
    /// Who is in it, each with their role.
    pub members: Vec<(u32, String)>,
    /// What was said in it, oldest first: each line and its colour.
    pub lines: Vec<(String, &'static str)>,
    /// The member picked in the list: what is typed goes to that one alone (`UIChatRoomWnd::SendMsg`).
    pub picked: Option<usize>,
}

impl Sitting {
    /// Whether that one owns the room.
    pub fn owned_by(&self, name: &str) -> bool {
        self.members.iter().any(|(role, member)| *role == OWNER && member == name)
    }

    /// Someone's role is told (`UM_ROLE_CHANGE`); the list is put in the order of the names, as the client's
    /// is whenever it builds it anew from its map of members.
    pub fn role(&mut self, name: &str, role: u32) {
        if let Some(member) = self.members.iter_mut().find(|(_, member)| member == name) {
            member.0 = role;
        }
        self.sorted();
    }

    /// Someone left (`UM_MEMBER_EXIT`).
    pub fn left(&mut self, name: &str) {
        self.members.retain(|(_, member)| member != name);
        self.sorted();
    }

    /// Someone came in (`UM_ADDMEMBER`): at the end of the list.
    pub fn joined(&mut self, name: String) {
        if !self.members.iter().any(|(_, member)| *member == name) {
            self.members.push((MEMBER, name));
        }
    }

    fn sorted(&mut self) {
        self.picked = None;
        self.members.sort_by(|one, other| one.1.cmp(&other.1));
    }
}

/// The window that makes a room or changes the one the character owns (`UIChatRoomMakeWnd`,
/// `UIChatRoomChangeWnd`), while it is open.
pub(in crate::web) struct Making {
    /// It changes the room the character sits in; else it makes one.
    pub changes: bool,
    /// How many may come in, and whether anyone may (no password).
    pub limit: u16,
    pub public: bool,
    /// The list of numbers to pick the limit from is dropped down.
    pub limits_open: bool,
    /// The title and the password as they were last read from the window's lines: what the lines are written
    /// with when the screen is drawn anew.
    pub title: String,
    pub password: String,
}

impl Default for Making {
    /// As the window opens (`UIChatRoomMakeWnd::OnCreate`): the last of the numbers, anyone may come in.
    fn default() -> Self {
        Self { changes: false, limit: LIMITS[LIMITS.len() - 1], public: true, limits_open: false, title: String::new(), password: String::new() }
    }
}

/// Chat rooms.
#[derive(Default)]
pub(in crate::web) struct ChatRooms {
    /// The rooms whose signs the server told of, by room: those of one visit to a map (`world::visit`).
    pub rooms: (u32, HashMap<u32, Room>),
    pub sitting: Option<Sitting>,
    pub making: Option<Making>,
    /// The room asked to be made, until the server says whether it is.
    pub asked: Option<Room>,
    /// The room whose password is asked for (the client's `UIPasswordWnd`).
    pub password_for: Option<u32>,
    /// When a room was last asked to be entered: nothing more is asked for a second
    /// (`MM_REQ_ENTER_ROOM`'s `isWaitingEnterRoom`).
    pub enter_asked: f64,
    /// The menu of the room's window, opened with the right button: where on the stage, and on which member.
    pub menu: Option<((i32, i32), Option<usize>)>,
}
