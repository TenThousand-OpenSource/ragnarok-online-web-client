//! Chat rooms: opening the window that makes one, what its buttons and the room's own do, a sign over a head
//! entered, what is typed in a room, the room's menu (the client's `UIChatRoomMakeWnd::SendMsg`,
//! `UIChatRoomWnd::SendMsg`, `UIChatRoomTitle::OnLBtnDblClk`, `UIPasswordWnd::SendMsg`).

use super::*;
use crate::net::chat_rooms::{self, Room, Setting, kind};
use crate::press::chat_rooms::Press;

/// For how long after a room was asked to be entered nothing more is asked, in milliseconds
/// (`MM_REQ_ENTER_ROOM`).
const ENTER_WAIT: f64 = 1000.0;

impl App {
    /// A press of this system (`crate::press::chat_rooms`).
    pub(super) fn chat_rooms_pressed(&mut self, press: Press) {
        match press {
            Press::MakeClose => {
                self.chat_rooms.making = None;
                self.redraw(&[&wnd::ROOM_MAKE]);
            }
            Press::MakeOk => self.room_made(),
            Press::PasswordOk => self.room_password_typed(),
            Press::Public(public) => self.room_set(|making| making.public = public),
            Press::Limit(limit) => self.room_set(|making| (making.limit, making.limits_open) = (limit, false)),
            Press::Limits => self.room_set(|making| making.limits_open = !making.limits_open),
            // The cross of the room's window, and `/q` (`MM_REQ_EXIT_ROOM`): the window goes when the server
            // says the character has left.
            Press::Leave => self.room_left(),
            // A member is picked; pressed again it is let go, which is this client's own way out of what
            // follows from one being picked (what is typed then goes to that one alone).
            Press::Member(row) => {
                if let Some(sitting) = &mut self.chat_rooms.sitting {
                    sitting.picked = (sitting.picked != Some(row)).then_some(row);
                }
                self.redraw(&[&wnd::ROOM]);
            }
            Press::Expel | Press::Give | Press::Friend | Press::Change => self.room_menu_pressed(press),
        }
    }

    /// The key or the command that makes a room (`HK_CHATWND_MAKE`, `/chat`: the client's
    /// `BASICINFO_BUTTON_CHATROOM`): the window opens, or closes; nothing for one who sits in a room.
    pub(in crate::web::app) fn room_make_asked(&mut self) {
        if self.chat_rooms.sitting.is_some() {
            return;
        }
        self.chat_rooms.making = if self.chat_rooms.making.is_some() { None } else { Some(Making::default()) };
        self.redraw(&[&wnd::ROOM_MAKE]);
    }

    /// The character leaves the room it sits in (CZ_EXIT_ROOM); nothing for one who sits in none.
    pub(in crate::web::app) fn room_left(&mut self) {
        if self.chat_rooms.sitting.is_some() {
            self.send_on_map(&chat_rooms::leave());
        }
    }

    /// What is typed into the lines of the window that makes a room is kept: the window is written with it.
    fn room_lines_read(&mut self) {
        if let Some(making) = &mut self.chat_rooms.making {
            (making.title, making.password) = (field("room-title"), field("room-word"));
        }
    }

    /// OK of the window that makes or changes a room (`ID_OK`): a setting that will not do is said why in
    /// the client's message box; else the server is asked, and the window closes. (The client also refuses a
    /// room where another sign stands on the character's own cell, with its line 661: not checked here.)
    pub(in crate::web::app) fn room_made(&mut self) {
        self.room_lines_read();
        let Some(making) = &self.chat_rooms.making else { return };
        let setting = Setting { title: making.title.trim().to_owned(), password: making.password.clone(), public: making.public, limit: making.limit };
        if let Some((line, otherwise)) = chat_rooms::wrong(&setting) {
            self.message = Some(Self::words(line, otherwise));
            return self.render();
        }
        if making.changes {
            self.send_on_map(&chat_rooms::change(&setting));
        } else {
            self.send_on_map(&chat_rooms::create(&setting));
            let kind = if setting.public { kind::PUBLIC } else { kind::PRIVATE };
            // The room's number is not known until the server tells of a change (`chatRoom.id` of -1).
            self.chat_rooms.asked = Some(Room { owner: self.own_account(), id: u32::MAX, limit: setting.limit, users: 0, kind, title: setting.title });
        }
        self.chat_rooms.making = None;
        self.redraw(&[&wnd::ROOM_MAKE]);
    }

    /// A sign over a head was double-clicked (`UIChatRoomTitle::OnLBtnDblClk`): a room with a password asks
    /// for it first, a room that is only a sign does nothing, any other is asked to be entered. Nothing for
    /// one who sits in a room already, or who asked a moment ago.
    pub(in crate::web::app) fn room_used(&mut self, id: u32) {
        let Some(room) = self.chat_rooms.rooms.1.get(&id).filter(|_| self.chat_rooms.rooms.0 == world::visit()) else { return };
        match room.kind {
            kind::SIGN => {}
            kind::PRIVATE => {
                self.chat_rooms.password_for = Some(id);
                self.redraw(&[&wnd::ROOM_PASSWORD]);
            }
            _ => self.room_enter_asked(id, ""),
        }
    }

    fn room_enter_asked(&mut self, id: u32, word: &str) {
        let time = js_sys::Date::now();
        if self.chat_rooms.sitting.is_some() || time < self.chat_rooms.enter_asked + ENTER_WAIT {
            return;
        }
        self.chat_rooms.enter_asked = time;
        self.send_on_map(&chat_rooms::enter(id, word));
    }

    /// OK of the box that asks a room's password (`UIPasswordWnd::SendMsg`): the room is asked to be entered
    /// with what was typed, and the box closes.
    pub(in crate::web::app) fn room_password_typed(&mut self) {
        let word = field("room-key");
        if let Some(id) = self.chat_rooms.password_for.take() {
            self.room_enter_asked(id, &word);
        }
        self.redraw(&[&wnd::ROOM_PASSWORD]);
    }

    /// A line was typed into the room's window and sent (`UIChatRoomWnd::SendMsg`, `ID_OK`): a command is
    /// done as the chat's line does it; with a member picked who is not the player's own character the
    /// words go to that one alone, as a whisper; anything else is said in the room, which is the ordinary
    /// CZ_REQUEST_CHAT (`TT_NORMAL_FROM_CHATROOM`).
    pub(in crate::web::app) fn room_typed(&mut self) {
        let words = field("room-input");
        set_field("room-input", "");
        let words = words.trim();
        if words.is_empty() || (words.starts_with('/') && self.typed_command(words)) {
            return;
        }
        let Some(own) = self.game.as_ref().map(|game| game.name.clone()) else { return };
        let picked = self.chat_rooms.sitting.as_ref().and_then(|sitting| sitting.members.get(sitting.picked?)).map(|(_, name)| name.clone()).filter(|name| *name != own);
        match picked {
            Some(to) if !words.starts_with('/') => self.whisper_send(&to, words),
            _ => self.send_on_map(&packets::say(&own, words)),
        }
    }

    /// A line of what is said in the room, in its colour.
    pub(in crate::web::app) fn room_line(&mut self, words: String, colour: &'static str) {
        let Some(sitting) = &mut self.chat_rooms.sitting else { return };
        sitting.lines.push((words, colour));
        if sitting.lines.len() > crate::web::hud::CHAT_KEPT {
            sitting.lines.remove(0);
        }
        self.redraw(&[&wnd::ROOM]);
    }

    /// The right button on the room's window, on a member of its list or elsewhere (`UM_MAKEMENU`): the
    /// member is picked, and the menu opens there if it has a row to show.
    pub(in crate::web::app) fn room_menu_opens(&mut self, member: Option<usize>, at: (f32, f32)) {
        let Some(sitting) = &mut self.chat_rooms.sitting else { return };
        if member.is_some() {
            sitting.picked = member;
        }
        let at = Self::on_stage(at);
        self.chat_rooms.menu = Some((at, member));
        if self.room_menu().is_empty() {
            self.chat_rooms.menu = None;
        }
        self.redraw(&[&wnd::ROOM]);
        self.open_at(&wnd::ROOM_MENU, &wnd::menu_style(at));
    }

    /// A row of the menu was pressed (`UIChatRoomWnd::SendMsg`, `UM_COMMAND` of the menu).
    fn room_menu_pressed(&mut self, press: Press) {
        let member = self.chat_rooms.menu.take().and_then(|(_, member)| self.chat_rooms.sitting.as_ref()?.members.get(member?).map(|(_, name)| name.clone()));
        self.redraw(&[&wnd::ROOM_MENU]);
        match (press, member) {
            (Press::Expel, Some(name)) => self.send_on_map(&chat_rooms::expel(&name)),
            (Press::Give, Some(name)) => self.send_on_map(&chat_rooms::give(&name)),
            (Press::Friend, Some(name)) => self.friend_wanted(&name),
            // The window that changes the room opens with what the room has (`UM_SETINFO` of
            // `UIChatRoomChangeWnd`). The password is not told by the server: its line is empty.
            (Press::Change, _) => {
                if let Some(room) = self.chat_rooms.sitting.as_ref().map(|sitting| &sitting.room) {
                    self.chat_rooms.making = Some(Making { changes: true, limit: room.limit, public: room.kind != kind::PRIVATE, title: room.title.clone(), ..Making::default() });
                    self.redraw(&[&wnd::ROOM_MAKE]);
                }
            }
            _ => {}
        }
    }

    /// A button of the window that makes or changes a room sets something of the room being made: what is
    /// typed there is kept first, and the window drawn anew.
    fn room_set(&mut self, set: impl FnOnce(&mut Making)) {
        self.room_lines_read();
        if let Some(making) = &mut self.chat_rooms.making {
            set(making);
        }
        self.redraw(&[&wnd::ROOM_MAKE]);
    }

    /// Puts the menu of the room's window away, if it is open.
    pub(in crate::web::app) fn close_room_menu(&mut self) {
        if self.chat_rooms.menu.take().is_some() {
            self.redraw(&[&wnd::ROOM_MENU]);
        }
    }

    /// The character sits in no room any more: its window goes, and with it the window that changes the room
    /// (`ReceiveChatRoomMemberExit`: the windows 28 and 30) and the menu.
    pub(in crate::web::app) fn room_closed(&mut self) {
        self.chat_rooms.sitting = None;
        self.chat_rooms.menu = None;
        if self.chat_rooms.making.as_ref().is_some_and(|making| making.changes) {
            self.chat_rooms.making = None;
        }
        self.redraw(&[&wnd::ROOM, &wnd::ROOM_MAKE, &wnd::ROOM_MENU]);
    }

    /// The character was put elsewhere: the room it sat in, the box that asked a password and the signs it
    /// was told of are of the place it left.
    pub(in crate::web::app) fn chat_rooms_moved(&mut self) {
        self.chat_rooms.rooms.1.clear();
        if self.chat_rooms.password_for.take().is_some() {
            self.redraw(&[&wnd::ROOM_PASSWORD]);
        }
        if self.chat_rooms.sitting.is_some() {
            self.room_closed();
        }
    }
}
