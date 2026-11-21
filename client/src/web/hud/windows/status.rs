//! The entries of the list of windows for the Status window and for the button that tells of a base level
//! gained.

use super::*;

pub(in crate::web) static STATUS: Window =
    Window::usual("status", |app| app.hero.status_open, |_, _| framed(at(225, 285, 280, 141)), App::status_window, switched_off!(
        hero.status_open
    ));

/// The button that tells of a base level gained (`WID_NOTIFYLEVELUPWND`): 43 by 43 in the lower right corner
/// of the screen. It is no window that is dragged, and has no place in the order
/// (`UIWindowMgr::IsOrderFixed`).
pub(in crate::web) static LEVEL_TOLD: Window = Window {
    id: "level-told",
    open: |app| app.hero.level_told,
    frame: |_, view| crate::web::hud::status::told_place(view.0 as i32 - 43, view),
    draw: App::level_told,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};
