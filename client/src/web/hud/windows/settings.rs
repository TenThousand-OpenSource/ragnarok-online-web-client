//! The entries of the list of windows for the Shortcut Settings and the Game Settings.

use super::*;

/// The Shortcut Settings window, in the middle of the view.
pub(in crate::web) static KEYS: Window = Window {
    id: "keys",
    open: |app| app.keys.open,
    frame: |_, view| framed(centred(view, 436, 432)),
    draw: App::key_window,
    order: ordered(|app| app.keys.open, |app| app.pressed(crate::press::hotkeys::Press::Close)),
    layer: Layer::Usual,
    drawn: None,
};

/// The Game Settings window, in the middle of the view, as the client puts it.
pub(in crate::web) static SETTINGS: Window = Window {
    id: "settings",
    open: |app| app.settings.open,
    frame: |_, view| framed(centred(view, 400, 350)),
    draw: App::settings_window,
    order: ordered(|app| app.settings.open, switched_off!(settings.open)),
    layer: Layer::Usual,
    drawn: None,
};
