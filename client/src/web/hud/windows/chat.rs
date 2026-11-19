//! The entries of the list of windows for the chat's tabs taken out into windows of their own, a window to
//! each of the tabs there can be (`crate::chat_tabs::MOST`), and for the window that sets what a tab lists.

use super::*;

/// The own window of the tab in one place of the list of tabs (the client's `UISubChatHisWnd`): where and as
/// large as the tab has it. It lies under the windows, as the chat does.
macro_rules! own_window {
    ($name:ident, $id:literal, $place:literal) => {
        pub(in crate::web) static $name: Window = Window {
            id: $id,
            open: |app| app.chat_own($place).is_some(),
            frame: |app, _| {
                let (x, y, width, height) = app.chat_own($place).unwrap_or_default();
                framed(at(x, y, width, height))
            },
            draw: |app| app.chat_own_window($place),
            order: None,
            layer: Layer::Under,
            drawn: None,
        };
    };
}

own_window!(CHAT_OWN_0, "chat-own-0", 0);
own_window!(CHAT_OWN_1, "chat-own-1", 1);
own_window!(CHAT_OWN_2, "chat-own-2", 2);
own_window!(CHAT_OWN_3, "chat-own-3", 3);
own_window!(CHAT_OWN_4, "chat-own-4", 4);
own_window!(CHAT_OWN_5, "chat-own-5", 5);
own_window!(CHAT_OWN_6, "chat-own-6", 6);
own_window!(CHAT_OWN_7, "chat-own-7", 7);
own_window!(CHAT_OWN_8, "chat-own-8", 8);
own_window!(CHAT_OWN_9, "chat-own-9", 9);

/// The own windows of the tabs, by the tabs' places.
pub(in crate::web) static CHAT_OWN: [&Window; crate::chat_tabs::MOST] = [&CHAT_OWN_0, &CHAT_OWN_1, &CHAT_OWN_2, &CHAT_OWN_3, &CHAT_OWN_4, &CHAT_OWN_5, &CHAT_OWN_6, &CHAT_OWN_7, &CHAT_OWN_8, &CHAT_OWN_9];

/// The window that sets what a tab lists (`UIBattleMsgOptionWnd`): 280 by 200, 400 across the client's stage
/// and 200 of 480 down.
pub(in crate::web) static CHAT_OPTIONS: Window = Window {
    id: "chat-options",
    open: |app| app.chat.options.is_some(),
    frame: |_, view| staged(view, 400, 200, 280, 200),
    draw: App::chat_options_window,
    order: ordered(|app| app.chat.options.is_some(), |app| app.pressed(crate::press::chat::Press::OptionsClose)),
    layer: Layer::Usual,
    drawn: None,
};
