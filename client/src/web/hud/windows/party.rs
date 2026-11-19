//! The party's entries of the list of windows: its window, the one beside it, what it asks, the menu on a member.

use super::*;

/// The party window, where the client first opens it (`MessengerGroupWndInfo::Reset`: 300, 200).
pub(in crate::web) static PARTY: Window = Window::usual(
    "party",
    |app| app.party.open,
    |app, _| framed(at(300, 200, app.party_size().0, app.party_size().1)),
    App::party_window,
    |app| app.pressed(crate::press::party::Press::Window),
);

/// The party's setup, the making of one, or whom to invite: beside the party window, two pixels right of it,
/// while that is open (where the client moves it); otherwise where the client opens it by itself, 120 of 480
/// down the middle of the screen. (The client keeps it beside the party window as that is dragged; here it
/// stands where it was opened.)
pub(in crate::web) static PARTY_SIDE: Window = Window::usual(
    "party-side",
    |app| app.party.side.is_some(),
    |app, view| {
        let (width, height) = app.party_side_size();
        if app.party.open { framed(at(302 + app.party_size().0, 200, width, height)) } else { middle(view, width, 120) }
    },
    App::party_side_window,
    |app| app.pressed(crate::press::party::Press::SetupCancel),
);

/// What the party asks, in the client's message box: 200 of 480 down the middle of the screen.
pub(in crate::web) static PARTY_ASKS: Window = Window::question("party-asks", |app| app.party.asks.is_some(), App::party_question);

/// The menu on a member of the party, opened with the right button in the party window, where the pointer was.
pub(in crate::web) static PARTY_MENU: Window =
    Window::menu("party-menu", |app| app.party.menu.is_some(), |app, _| menu_frame(app.party.menu.map(|(at, _)| at)), App::party_menu);
