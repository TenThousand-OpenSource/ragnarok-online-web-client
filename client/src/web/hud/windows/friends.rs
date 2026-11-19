//! The friends list's entries of the list of windows. The list itself is a side of the party window (`PARTY`).

use super::*;
use crate::web::hud::friends::SETUP;

/// The Friend Setup window, beside the window that lists the friends, two pixels right of it, where the
/// client moves it (`ID_PARTY_SETTING` with `mode` 0). Escape closes it as its cross does.
pub(in crate::web) static FRIEND_SETUP: Window = Window::usual(
    "friend-setup",
    |app| app.party.open && app.friends.shown && app.friends.setup,
    |app, _| framed(at(302 + app.party_size().0, 200, SETUP.0, SETUP.1)),
    App::friend_setup_window,
    |app| app.pressed(crate::press::friends::Press::SetupClose),
);

/// What the friends list asks, in the client's message box: 200 of 480 down the middle of the screen
/// (`WID_JOINFRIENDACCEPTWND`).
pub(in crate::web) static FRIEND_ASKS: Window = Window::question("friend-asks", |app| app.friends.asks.is_some(), App::friend_question);

/// The menu on a friend, opened with the right button in the list, where the pointer was.
pub(in crate::web) static FRIEND_MENU: Window =
    Window::menu(
        "friend-menu",
        |app| app.party.open && app.friends.shown && app.friends.menu.is_some(),
        |app, _| menu_frame(app.friends.menu.map(|(at, _)| at)),
        App::friend_menu
    );
