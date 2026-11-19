//! The items' entries of the list of windows: the inventory, an item's description, a card's illustration,
//! the note of a thing picked up, how many to drop, what to identify.

use super::*;

/// The inventory stands aside while the storage is open beside it.
pub(in crate::web) static INVENTORY: Window = Window::usual(
    "inventory",
    |app| app.bag.open,
    |app, _| framed(at(if app.storage.open.is_some() { 515 } else { 225 }, 80, 40 + 32 * ITEM_COLUMNS + 16, 17 + App::inventory_body() + 21)),
    App::inventory_window,
    switched_off!(bag.open),
);

pub(in crate::web) static ITEM_INFO: Window = Window::usual(
    "iteminfo",
    |app| app.bag.info.is_some(),
    |_, _| framed("left:300px;top:230px;width:280px".to_owned()),
    App::item_info_window,
    |app| app.pressed(crate::press::inventory::Press::InfoClose),
);

/// A card's illustration, 300 by 417, at the left of the client's 640-wide stage and the top of the screen
/// (`WID_CARDITEMILLUSTWND`). Escape closes it.
pub(in crate::web) static CARD_ILLUST: Window = Window::usual(
    "card-illust",
    |app| app.bag.illust.is_some(),
    |_, view| staged(view, 0, 0, 300, 417),
    App::card_illust_window,
    |app| app.pressed(crate::press::inventory::Press::IllustClose),
);

/// The note of what was just picked up: not a window, nothing on it is pressed.
pub(in crate::web) static OBTAINED: Window = Window {
    id: "obtained",
    open: ALWAYS,
    frame: |_, view| format!(r#"class="a" style="left:{}px;top:75px;width:260px;pointer-events:none""#, (view.0 as i32 - 260) / 2),
    draw: App::obtained_note,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};

/// How many of a stack to drop.
pub(in crate::web) static THROW: Window = Window::count("throw", |app| app.bag.throwing.is_some(), App::throw_window);

/// The list of items to identify: the same window, in the same place.
pub(in crate::web) static IDENTIFY: Window = Window::pick("identify", |app| app.bag.identify.is_some(), App::identify_window);
