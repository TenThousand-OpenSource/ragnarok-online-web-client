//! The presses of chat rooms: the window that makes or changes a room (the client's `UIChatRoomMakeWnd`,
//! `UIChatRoomChangeWnd`), the room's own window (`UIChatRoomWnd`) and its menu.

presses! {
    /// What can be pressed here.
    Press {
        /// The cross and Cancel of the window that makes or changes a room.
        MakeClose = "room-make-close",
        /// Its OK: the room is made, or changed.
        MakeOk = "room-make-ok",
        /// The OK of the box that asks a room's password.
        PasswordOk = "room-password-ok",
        /// Its round buttons: anyone may come in (`true`), or only who knows the password.
        Public(bool) = "room-public",
        /// The button that drops down the numbers of people to pick from.
        Limits = "room-limits",
        /// A number of that list.
        Limit(u16) = "room-limit",
        /// The cross of the room's window: the character leaves the room.
        Leave = "room-leave",
        /// A member in the room's list is picked.
        Member(usize) = "room-member",
        /// The menu's rows: the member is thrown out, is given the room, is asked to be a friend; the room's
        /// setting is changed (`MENUID_EXPEL`, `MENUID_GIVE_ROOM_POWER`, `MENUID_CHANGE_ROOM_SETTING`).
        Expel = "room-expel",
        Give = "room-give",
        Friend = "room-friend",
        Change = "room-change",
    }
}

#[cfg(test)]
mod tests {
    use super::Press;
    use crate::press::Named;

    #[test]
    fn every_press_is_read_back_from_what_it_is_written_as() {
        for press in [Press::MakeClose, Press::MakeOk, Press::PasswordOk, Press::Public(true), Press::Limits, Press::Limit(15), Press::Leave, Press::Member(2), Press::Expel, Press::Give, Press::Friend, Press::Change] {
            let (what, arg) = press.name();
            assert_eq!(Press::parse(what, &arg), Some(press));
        }
        assert_eq!(Press::parse("room-limit", "many"), None);
    }
}
