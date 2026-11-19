//! The entries of the list of windows for the window of emotions and the Shortcut List.

use super::*;

/// The window of emotions (`UICashEmotionListWnd`): 298 by 380, 185 across the client's stage and 60 above
/// 400 of 480 down.
pub(in crate::web) static EMOTIONS: Window = Window::usual(
    "emotions",
    |app| app.emotions.open,
    |_, view| framed(format!("left:{}px;top:{}px;width:298px;height:380px", (view.0 as i32 - 640) / 2 + 185, (view.1 as i32 * 400 / 480 - 60 - 380).max(0))),
    App::emotion_window,
    |app| app.pressed(crate::press::emotions::Press::Window),
);

/// The Shortcut List (`UIEmotionWnd`): 375 by 260, 185 across the stage and 60 above 300 of 480 down.
pub(in crate::web) static SHORTCUTS: Window = Window::usual(
    "shortcuts",
    |app| app.emotions.shortcuts,
    |_, view| framed(format!("left:{}px;top:{}px;width:375px;height:260px", (view.0 as i32 - 640) / 2 + 185, (view.1 as i32 * 300 / 480 - 60).max(0))),
    App::shortcut_window,
    |app| app.pressed(crate::press::emotions::Press::Shortcuts),
);
