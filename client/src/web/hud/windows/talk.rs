//! The entries of the list of windows for what is said: an NPC's window with the game's menu in its place, and
//! the chat.

use super::*;

/// What an NPC says, and in the same place the game's menu (the client's `UIEscOptionWnd`). The original puts
/// the NPC's window a fifth of the way down the screen, in the middle, and what it asks under it. Only the menu
/// lies in the order: a window it opens (Game Settings, the keys) goes over it, as `EnsureWindow` puts it in
/// front there, and the menu stays open behind; what an NPC says is over the windows, by its layer.
pub(in crate::web) static DIALOG: Window = Window {
    id: "dialog",
    open: |app| app.esc != 0 || app.dialog.is_some(),
    frame: |_, view| middle(view, 280, 100),
    draw: App::dialog_window,
    order: ordered(|app| app.esc == 1, |app| app.pressed(crate::press::general::Press::EscClose)),
    layer: Layer::Asks,
    // What is said is read from its beginning: the client's viewer stays where it is scrolled to, the top, as
    // text is added (`TagTextViewer::UIScroll::UpdateScroll`).
    drawn: None,
};

/// The chat, along the bottom of the screen: as tall as its rows make it.
pub(in crate::web) static CHAT: Window = Window {
    id: "chat",
    open: |app| !app.chat.hidden,
    frame: |app, view| format!(
        r#"class="a hud" data-locked="{}" style="{}""#,
        u8::from(app.chat.locked),
        at(0, view.1 as i32 - app.chat_height(), 600, app.chat_height())
    ),
    draw: |app| app.chat_window(app.chat.rows, app.chat_height()),
    order: None,
    layer: Layer::Under,
    drawn: None,
};
