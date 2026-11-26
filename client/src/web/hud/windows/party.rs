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

/// The small window of a member of the party, by its place among the small windows (the client's
/// `UIMiniPartyWnd`, 174 by 52): in its cell of the grid over the screen. It is moved by all of it and is
/// let go into the nearest cell (`OnLBtnDown`, `OnLBtnUp`), docks to nothing (the client keeps these out of
/// the docking, `snapExcluded`), and lies among the usual windows.
macro_rules! mini_window {
    ($name:ident, $id:literal, $place:literal) => {
        pub(in crate::web) static $name: Window = Window {
            id: $id,
            open: |app| app.party_mini_member($place).is_some(),
            frame: |app, view| {
                // (One whose cell the screen no longer has is out of sight, as in the client, until the
                // windows are put in order.)
                let (x, y) = app.party_mini_place($place, view).unwrap_or((-1000, -1000));
                format!(r#"class="a hud" data-whole data-apart style="{}""#, at(x, y, crate::party::mini::SIZE.0, crate::party::mini::SIZE.1))
            },
            draw: |app| app.party_mini_window($place),
            order: None,
            layer: Layer::Usual,
            drawn: None,
        };
    };
}

mini_window!(PARTY_MINI_0, "party-mini-0", 0);
mini_window!(PARTY_MINI_1, "party-mini-1", 1);
mini_window!(PARTY_MINI_2, "party-mini-2", 2);
mini_window!(PARTY_MINI_3, "party-mini-3", 3);
mini_window!(PARTY_MINI_4, "party-mini-4", 4);
mini_window!(PARTY_MINI_5, "party-mini-5", 5);
mini_window!(PARTY_MINI_6, "party-mini-6", 6);
mini_window!(PARTY_MINI_7, "party-mini-7", 7);
mini_window!(PARTY_MINI_8, "party-mini-8", 8);
mini_window!(PARTY_MINI_9, "party-mini-9", 9);
mini_window!(PARTY_MINI_10, "party-mini-10", 10);
mini_window!(PARTY_MINI_11, "party-mini-11", 11);

/// The small windows of the party's members, by their places: one to each of the twelve a party holds.
pub(in crate::web) static PARTY_MINI: [&Window; 12] = [
    &PARTY_MINI_0, &PARTY_MINI_1, &PARTY_MINI_2, &PARTY_MINI_3, &PARTY_MINI_4, &PARTY_MINI_5, &PARTY_MINI_6, &PARTY_MINI_7, &PARTY_MINI_8,
    &PARTY_MINI_9, &PARTY_MINI_10, &PARTY_MINI_11,
];
