//! The entries of the list of windows for the Shortcut Settings and the Game Settings.

use super::*;

/// The Shortcut Settings window, in the middle of the view.
pub(in crate::web) static KEYS: Window = Window::usual(
    "keys",
    |app| app.keys.open,
    |_, view| framed(centred(view, 436, 432)),
    App::key_window,
    |app| app.pressed(crate::press::hotkeys::Press::Close),
);

/// The Game Settings window, in the middle of the view, as the client puts it.
pub(in crate::web) static SETTINGS: Window = Window::usual("settings", |app| app.settings.open, |_, view| framed(centred(view, 400, 350)), App::settings_window, switched_off!(settings.open));
