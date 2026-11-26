//! The entries of the list of windows for the cash shop: its window, and the message box it asks and tells
//! with.

use super::*;

/// The name of the field for the free points to use, beside the window's own field, which is the line that
/// is searched with.
pub(in crate::web) fn cash_shop_points() -> String {
    format!("{}-points", CASH_SHOP.id)
}

/// The cash shop (`UICashShopWnd`): 723 by 540, in the middle of the screen (`UIWindowMgr::EnsureWindow`,
/// `WID_CASHSHOP_WND`). Escape closes it as its cross does: that the client's Escape does so was not found in
/// its code and is taken from what its other framed windows do.
pub(in crate::web) static CASH_SHOP: Window = Window::usual(
    "cashshop",
    |app| app.cashshop.open,
    |_, view| framed(format!("{};height:540px", centred(view, 723, 540))),
    App::cashshop_window,
    |app| app.pressed(crate::press::cashshop::Press::Close),
);

/// What the shop asks or tells (`UIWindowMgr::Confirm`: the client's message box, 280 by 120).
pub(in crate::web) static CASH_SHOP_ASKS: Window = Window::question("cashshop-asks", |app| app.cashshop.asks.is_some(), App::cashshop_question);
