//! The entries of the list of windows for chat rooms: the window that makes or changes one, the room's own
//! window and its menu, the box that asks a room's password, and the place of the signs over heads.

use super::*;
use crate::press::chat_rooms::Press;
use crate::web::parts::menued;

/// Where the signs over heads are kept (`world/signs.rs` writes and moves them, frame after frame): the whole
/// stage, under the windows, itself letting the pointer through.
pub(in crate::web) static SIGNS: Window = Window { id: "signs", open: ALWAYS, frame: |_, _| r#"class="a" style="left:0;top:0;width:0;height:0""#.to_owned(), draw: |_| String::new(), order: None, layer: Layer::Under, drawn: None };

/// The window that makes a room or changes one (`WID_CHATROOMMAKEWND`, `WID_CHATROOMCHANGEWND`): 280 by 120,
/// 185 across the client's stage and 300 of 480 down. Its title has the keyboard as it opens.
pub(in crate::web) static ROOM_MAKE: Window = Window {
    id: "room-make",
    open: |app| app.chat_rooms.making.is_some(),
    frame: |_, view| staged(view, 185, 300, 280, 120),
    draw: App::room_make_window,
    order: ordered(|app| app.chat_rooms.making.is_some(), |app| app.pressed(Press::MakeClose)),
    layer: Layer::Usual,
    drawn: None,
};

/// The room the character sits in (`WID_CHATROOMWND`): 280 by 120 where the client has it at first (500,
/// 180: `ChatRoomWndInfo`), with the right button's menu on all of it. Escape does not close it: leaving a
/// room is its cross, or `/q`.
pub(in crate::web) static ROOM: Window = Window {
    id: "room",
    open: |app| app.chat_rooms.sitting.is_some(),
    frame: |_, _| format!("{} {}", framed(crate::web::parts::at(500, 180, 280, 120)), menued(crate::press::things::Menu::Room)),
    draw: App::room_window,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};

/// The box that asks the password of a room (`WID_PASSWORDWND`): 176 by 46, 200 across the client's stage
/// and 200 of 480 down, over the windows; its line has the keyboard.
pub(in crate::web) static ROOM_PASSWORD: Window = Window {
    id: "room-password",
    open: |app| app.chat_rooms.password_for.is_some(),
    frame: |_, view| staged(view, 200, 200, 176, 46),
    draw: App::room_password_window,
    order: None,
    layer: Layer::Asks,
    drawn: Some(typed_over),
};

/// The menu of the room's window, where the pointer was.
pub(in crate::web) static ROOM_MENU: Window =
    Window::menu("room-menu", |app| app.chat_rooms.menu.is_some(), |app, _| menu_frame(app.chat_rooms.menu.map(|(at, _)| at)), App::room_menu);
