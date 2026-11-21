//! The entries of the list of windows for the Shortcut Settings, the Hotkey Guide and the Game Settings.

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
pub(in crate::web) static SETTINGS: Window = Window::usual(
    "settings",
    |app| app.settings.open,
    |_, view| framed(centred(view, 400, 350)),
    App::settings_window,
    switched_off!(settings.open)
);

/// The Hotkey Guide (`WID_HOTKEYGUIDEWND`): 934 by 498 in the very middle of the screen.
pub(in crate::web) static KEY_GUIDE: Window = Window::usual(
    "key-guide",
    |app| app.keys.guide,
    |_, view| framed(format!("{};height:498px", centred(view, 934, 498))),
    App::key_guide,
    switched_off!(keys.guide)
);
