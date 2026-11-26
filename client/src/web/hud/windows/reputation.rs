//! The reputation's entry of the list of windows.

use super::*;

/// The reputation's window (`UIReputeWnd`): 640 by 450 in the middle of the screen
/// (`UIWindowMgr::EnsureWindow`, `WID_REPUTEWND`).
pub(in crate::web) static REPUTATION: Window = Window::usual(
    "reputation",
    |app| app.reputation.open,
    |_, view| framed(format!("{};height:450px", centred(view, 640, 450))),
    App::reputation_window,
    |app| app.pressed(crate::press::reputation::Press::Window),
);
