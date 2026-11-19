//! What the server says of chat rooms (`crate::net::chat_rooms`): the signs over heads, a room made or
//! entered, who comes and goes in the room the character sits in, and what is said there.

use super::*;
use crate::net::chat_rooms::{self, OWNER};
use crate::chat_rooms::Room;

/// The colours the client writes the room's lines in (blue first there): what others say (`0x222222`), what
/// the player says (`0x7800`), someone coming in (`0x808000`), someone leaving (`0x80`).
const THEIRS: &str = "#222222";
const OWN: &str = "#007800";
const CAME: &str = "#008080";
const WENT: &str = "#800000";

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other. It is asked before the reader
    /// of what is said, for what is said near belongs to the room while the character sits in one: it goes to
    /// the room's window and neither to the chat nor over a head (`ReceiveActorChat`, `ReceiveBroadcast`).
    pub(super) fn chat_rooms_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            packets::ZC_NOTIFY_CHAT | packets::ZC_NOTIFY_PLAYERCHAT if self.chat_rooms.sitting.is_some() => {
                if let Some(words) = packets::chat_text(id, packet) {
                    self.room_line(words, if id == packets::ZC_NOTIFY_PLAYERCHAT { OWN } else { THEIRS });
                }
            }
            chat_rooms::ZC_ACK_CREATE_CHATROOM => self.room_answered(packet),
            // `ReceiveChatRoomCreate`: the sign of a room over the one it belongs to.
            chat_rooms::ZC_ROOM_NEWENTRY => {
                if let Some(room) = chat_rooms::room(packet) {
                    world::sign(room.owner, self.own_account(), world::Board::Room, Some(Self::room_sign(&room)));
                    self.rooms_here().insert(room.id, room);
                }
            }
            // `ReceiveChatRoomDestroy`: the sign goes, and the room's window if the character sat in it.
            chat_rooms::ZC_DESTROY_ROOM => {
                let Some(id) = chat_rooms::destroyed(packet) else { return true };
                if let Some(room) = self.rooms_here().remove(&id) {
                    world::sign(room.owner, self.own_account(), world::Board::Room, None);
                }
                if self.chat_rooms.sitting.as_ref().is_some_and(|sitting| sitting.room.id == id) {
                    self.room_closed();
                }
            }
            // `ReceiveChatRoomEnterRefused`: why, in red.
            chat_rooms::ZC_REFUSE_ENTER_ROOM => {
                self.chat_rooms.enter_asked = 0.0;
                if let Some((line, otherwise)) = chat_rooms::result(packet).and_then(crate::chat_rooms::refused) {
                    self.chat_line(Self::words(line, otherwise), RED);
                }
            }
            chat_rooms::ZC_ENTER_ROOM => self.room_entered(packet),
            chat_rooms::ZC_MEMBER_NEWENTRY => self.room_joined(packet),
            chat_rooms::ZC_MEMBER_EXIT => self.room_member_left(packet),
            // `ReceiveActorMessage34`: the room the character sits in is called otherwise, or holds another
            // number of people; only now is the number of a room the player made known.
            chat_rooms::ZC_CHANGE_CHATROOM => {
                if let (Some(room), Some(sitting)) = (chat_rooms::room(packet), &mut self.chat_rooms.sitting) {
                    sitting.room = room;
                    self.redraw(&[&wnd::ROOM]);
                }
            }
            chat_rooms::ZC_ROLE_CHANGE => {
                if let (Some((role, name)), Some(sitting)) = (chat_rooms::role(packet), &mut self.chat_rooms.sitting) {
                    sitting.role(&name, role);
                    self.redraw(&[&wnd::ROOM]);
                }
            }
            _ => return false,
        }
        true
    }

    /// The rooms told of on the map the character is on now: those of another map are forgotten first.
    fn rooms_here(&mut self) -> &mut std::collections::HashMap<u32, Room> {
        let visit = world::visit();
        if self.chat_rooms.rooms.0 != visit {
            self.chat_rooms.rooms = (visit, Default::default());
        }
        &mut self.chat_rooms.rooms.1
    }

    /// The character sits in that room now: its window opens, the line to type in with the keyboard.
    fn room_opened(&mut self, room: Room, members: Vec<(u32, String)>) {
        self.chat_rooms.sitting = Some(Sitting { room, members, lines: Vec::new(), picked: None });
        self.chat_rooms.password_for = None;
        self.redraw(&[&wnd::ROOM, &wnd::ROOM_PASSWORD]);
        if let Some(line) = element("room-input").and_then(|line| line.dyn_into::<web_sys::HtmlElement>().ok()) {
            let _ = line.focus();
        }
    }

    /// ZC_ACK_CREATE_CHATROOM (`ReceiveChatRoomCreateResult`): the room asked for is made, and the player's
    /// own character sits in it as its owner, which the chat says in the client's colour for it
    /// (`0xFFFF00`, blue first); or why it is not, in red.
    fn room_answered(&mut self, packet: &[u8]) {
        self.chat_rooms.enter_asked = 0.0;
        let Some(result) = chat_rooms::result(packet) else { return };
        let asked = self.chat_rooms.asked.take();
        match crate::chat_rooms::not_made(result) {
            Some((line, otherwise)) => self.chat_line(Self::words(line, otherwise), RED),
            None => {
                self.chat_line(Self::words(64, "Room has been successfully created."), CYAN);
                let own = self.game.as_ref().map(|game| game.name.clone()).unwrap_or_default();
                self.room_opened(Room { users: 1, ..asked.unwrap_or_default() }, vec![(OWNER, own)]);
            }
        }
    }

    /// ZC_ENTER_ROOM (`ReceiveChatRoomEnter`): the character is in a room whose sign it was told of; the
    /// sign over the owner goes, and the room's window opens with everyone in it.
    fn room_entered(&mut self, packet: &[u8]) {
        let Some((id, members)) = chat_rooms::entered(packet) else { return };
        let Some(room) = self.rooms_here().get(&id).cloned() else { return };
        world::sign(room.owner, self.own_account(), world::Board::Room, None);
        self.room_opened(Room { users: members.len() as u16, ..room }, members);
    }

    /// ZC_MEMBER_NEWENTRY (`ReceiveChatRoomMemberEnter`): someone came into the room; the window says so
    /// with the client's line 179, and chimes.
    fn room_joined(&mut self, packet: &[u8]) {
        let (Some((users, name)), Some(sitting)) = (chat_rooms::joined(packet), &mut self.chat_rooms.sitting) else { return };
        sitting.room.users = users;
        sitting.joined(name.clone());
        self.chime();
        self.room_line(Self::words(179, "%s has joined.").replacen("%s", &name, 1), CAME);
    }

    /// ZC_MEMBER_EXIT (`ReceiveChatRoomMemberExit`): the player's own character left, and the window goes;
    /// or someone else did, which the window says with the client's line 180, or 181 for one thrown out.
    fn room_member_left(&mut self, packet: &[u8]) {
        let Some((users, name, thrown)) = chat_rooms::left(packet) else { return };
        if self.game.as_ref().is_some_and(|game| game.name == name) {
            return self.room_closed();
        }
        let Some(sitting) = &mut self.chat_rooms.sitting else { return };
        sitting.room.users = users;
        sitting.left(&name);
        let line = if thrown { Self::words(181, "%s was kicked out.") } else { Self::words(180, "%s has left.") };
        self.room_line(line.replacen("%s", &name, 1), WENT);
    }
}
