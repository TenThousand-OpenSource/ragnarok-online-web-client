//! The entries of the list of windows for an NPC's shop: its question, its list, its basket, and the box that
//! asks how many go across.

use super::*;

fn dealing(app: &App) -> bool {
    matches!(app.shops.shop, Some(Shop::Buy(_) | Shop::Sell(_)))
}

/// The shop's question whether to buy or to sell.
pub(in crate::web) static TRADE: Window = Window {
    id: "trade",
    open: |app| matches!(app.shops.shop, Some(Shop::Choose(_))),
    frame: |_, view| middle(view, 280, 100),
    draw: App::trade_window,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// The shop's list, where the client first puts it (`ItemShopWndInfo::Reset`: 0, 120, 280 by 312). Escape
/// leaves the shop, as the basket's Cancel does.
pub(in crate::web) static SHOP: Window =
    Window::usual("shop", dealing, |_, _| framed(at(0, 120, 280, 312)), App::shop_window, |app| app.pressed(
        crate::press::shops::Press::Cancel
    ));

/// The basket (`ItemPurchaseWndInfo::Reset`, `ItemSellWndInfo::Reset`: 280, 280, 280 by 152).
pub(in crate::web) static SHOP_BASKET: Window = Window::usual(
    "shop-basket",
    dealing,
    |_, _| framed(at(280, 280, 280, 152)),
    App::shop_basket_window,
    |app| app.pressed(crate::press::shops::Press::Cancel),
);

/// How many of a row go across the shop's windows.
pub(in crate::web) static SHOP_COUNT: Window = Window::count("shop-count", |app| app.shops.counting.is_some(), App::shop_count_window);
