//! The cart's entries of the list of windows: the cart's own window, the box that asks how many go into it
//! or out of it, and the window of the looks it may be given.

use super::*;
use crate::press::cart::Press;

/// The cart's window, 280 by 120, 200 across the client's stage and 120 of 480 down (`UIWindowMgr::
/// EnsureWindow`, `WID_MERCHANTITEMWND`).
pub(in crate::web) static CART: Window = Window::usual("cart", |app| app.cart.open, |_, view| staged(view, 200, 120, 280, 120), App::cart_window, |app| app.pressed(Press::Window));

/// How many of a thing go into the cart or out of it.
pub(in crate::web) static CART_COUNT: Window = Window::count("cart-count", |app| app.cart.counting.is_some(), App::cart_count_window);

/// The looks a cart may be given (`WID_SELECTCARTWND`): 385 wide, 200 across the client's stage and 200 of
/// 480 down.
pub(in crate::web) static CART_LOOKS: Window = Window::usual(
    "cart-looks",
    |app| app.cart.looks.is_some(),
    |_, view| framed(format!("left:{}px;top:{}px;width:385px", (view.0 as i32 - 640) / 2 + 200, view.1 as i32 * 200 / 480)),
    App::cart_looks_window,
    |app| app.pressed(Press::LooksClose),
);

/// The windows that show what is in the cart: drawn anew together when it changes.
pub(in crate::web) static CART_WINDOWS: [&Window; 4] = [&CART, &CART_COUNT, &VENDING_STALL, &VENDING_STOCK];
