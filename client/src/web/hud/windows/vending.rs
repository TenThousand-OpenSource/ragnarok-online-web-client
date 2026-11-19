//! The entries of the list of windows for merchants' shops: the window a shop is made in and the one beside
//! it, one's own shop, a vendor's list with its basket and the window that asks whether to buy, the question
//! the making of a shop asks, and the box that asks how many go across.

use super::*;
use crate::press::vending::Press;
use crate::web::app::Stage;

fn making(app: &App) -> bool {
    app.vending.making.is_some()
}

fn choosing(app: &App) -> bool {
    app.vending.visit.as_ref().is_some_and(|visit| visit.stage == Stage::Choosing)
}

fn confirming(app: &App) -> bool {
    app.vending.visit.as_ref().is_some_and(|visit| visit.stage == Stage::Confirming)
}

/// The window a shop is made in (`WID_MERCHANTSHOPMAKEWND`): 400 wide, where the client has it at first (0,
/// 240: `MerchantShopMakeWndInfo`). Escape leaves it as its Cancel does. The shop's title has the keyboard
/// as the window opens.
pub(in crate::web) static VENDING_STALL: Window = Window::usual(
    "vending-stall",
    making,
    |app, _| framed(at(0, 240, 400, app.stall_height())),
    App::stall_window,
    |app| app.pressed(Press::StallCancel),
);

/// The name of the field of the price of a row of the shop being made.
pub(in crate::web) fn stall_price_field(row: usize) -> String {
    format!("{}-{row}", VENDING_STALL.field())
}

/// What the cart has for the shop being made (`WID_MERCHANTMIRRORITEMWND`): 280 by 120 at 0, 120.
pub(in crate::web) static VENDING_STOCK: Window =
    Window::usual("vending-stock", making, |_, _| framed(at(0, 120, 280, 120)), App::stall_stock_window, |app| app.pressed(
        Press::StallCancel
    ));

/// One's own shop while it is open (`WID_MERCHANTITEMMYSHOPWND`): 280 by 120 where the client has it at
/// first (200, 120: `MerchantItemMyShopWndInfo`). Escape does not close it: closing the shop is its button.
pub(in crate::web) static VENDING_MINE: Window =
    Window {
        id: "vending-mine",
        open: |app| app.vending.mine.is_some(),
        frame: |_, _| framed(at(200, 120, 280, 120)),
        draw: App::vending_mine_window,
        order: None,
        layer: Layer::Usual,
        drawn: None
    };

/// A vendor's list (`WID_MERCHANTITEMSHOPWND`): 280 wide at 0, 200. Escape leaves the vendor, as the
/// basket's Cancel does.
pub(in crate::web) static VENDOR: Window = Window::usual(
    "vendor",
    choosing,
    |app, _| framed(at(0, 200, 280, app.vendor_rows() * 32 + 56)),
    App::vendor_window,
    |app| app.pressed(Press::Leave),
);

/// The basket (`WID_MERCHANTITEMPURCHASEWND`): 280 by 120 at 300, 200.
pub(in crate::web) static VENDOR_BASKET: Window = Window::usual(
    "vendor-basket",
    choosing,
    |_, _| framed(at(300, 200, 280, 120)),
    App::vendor_basket_window,
    |app| app.pressed(Press::Leave)
);

/// The window that asks whether to buy for the sum (`UIMerchantItemPurchaseMSGWnd`): 300 by 150 in the very
/// middle of the screen.
pub(in crate::web) static VENDOR_CONFIRM: Window = Window::usual(
    "vendor-confirm",
    confirming,
    |_, view| framed(centred(view, 300, 150)),
    App::vendor_confirm_window,
    |app| app.pressed(Press::Refuse)
);

/// What the making of a shop asks, in the client's message box.
pub(in crate::web) static VENDING_ASKS: Window = Window::question("vending-asks", |app| app.vending.asks.is_some(), App::vending_question);

/// How many of a thing go into a shop being made or into the basket at a vendor's.
pub(in crate::web) static VENDING_COUNT: Window = Window::count("vending-count", |app| app.vending.counting.is_some(), App::vending_count_window);

/// Every window of the shops: drawn anew together when what they show changes.
pub(in crate::web) static VENDING_WINDOWS: [&Window; 9] =
    [&VENDING_STALL, &VENDING_STOCK, &VENDING_MINE, &VENDOR, &VENDOR_BASKET, &VENDOR_CONFIRM, &VENDING_ASKS, &VENDING_COUNT, &CART];
