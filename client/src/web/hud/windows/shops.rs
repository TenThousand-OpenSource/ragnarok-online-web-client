//! The entries of the list of windows for an NPC's shop: its question, its list, its basket, and the box that
//! asks how many go across.

use super::*;

fn dealing(app: &App) -> bool {
    matches!(app.deal.shop, Some(Shop::Buy(_) | Shop::Sell(_)))
}

/// The shop's question whether to buy or to sell.
pub(in crate::web) static TRADE: Window = Window {
    id: "trade",
    open: |app| matches!(app.deal.shop, Some(Shop::Choose(_))),
    frame: |_, view| middle(view, 280, 100),
    draw: App::trade_window,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// The shop's list, where the client first puts it (`ItemShopWndInfo::Reset`: 0, 120, 280 by 312). Escape
/// leaves the shop, as the basket's Cancel does.
pub(in crate::web) static SHOP: Window = Window {
    id: "shop",
    open: dealing,
    frame: |_, _| framed(at(0, 120, 280, 312)),
    draw: App::shop_window,
    order: ordered(dealing, |app| app.pressed(crate::press::shops::Press::Cancel)),
    layer: Layer::Usual,
    drawn: None,
};

/// The basket (`ItemPurchaseWndInfo::Reset`, `ItemSellWndInfo::Reset`: 280, 280, 280 by 152).
pub(in crate::web) static SHOP_BASKET: Window = Window {
    id: "shop-basket",
    open: dealing,
    frame: |_, _| framed(at(280, 280, 280, 152)),
    draw: App::shop_basket_window,
    order: ordered(dealing, |app| app.pressed(crate::press::shops::Press::Cancel)),
    layer: Layer::Usual,
    drawn: None,
};

/// How many of a row go across the shop's windows.
pub(in crate::web) static SHOP_COUNT: Window = Window::count("shop-count", |app| app.deal.counting.is_some(), App::shop_count_window);
