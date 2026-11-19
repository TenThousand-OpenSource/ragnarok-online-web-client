//! The items' entries of the list of windows: the inventory, the equipment, an item's description, the note of
//! a thing picked up, how many to drop, what to identify.

use super::*;

/// The inventory stands aside while the storage is open beside it.
pub(in crate::web) static INVENTORY: Window = Window {
    id: "inventory",
    open: |app| app.bag.open,
    frame: |app, _| framed(at(if app.storage.open.is_some() { 515 } else { 225 }, 80, 40 + 32 * ITEM_COLUMNS + 16, 17 + 32 * ITEM_ROWS + 21)),
    draw: App::inventory_window,
    order: ordered(|app| app.bag.open, switched_off!(bag.open)),
    layer: Layer::Usual,
    drawn: None,
};

pub(in crate::web) static EQUIPMENT: Window = Window {
    id: "equipment",
    open: |app| app.bag.equipment_open,
    frame: |_, _| framed("left:560px;top:290px;width:280px".to_owned()),
    draw: App::equipment_window,
    order: ordered(|app| app.bag.equipment_open, switched_off!(bag.equipment_open)),
    layer: Layer::Usual,
    drawn: None,
};

pub(in crate::web) static ITEM_INFO: Window = Window {
    id: "iteminfo",
    open: |app| app.bag.info.is_some(),
    frame: |_, _| framed("left:300px;top:230px;width:280px".to_owned()),
    draw: App::item_info_window,
    order: ordered(|app| app.bag.info.is_some(), |app| app.pressed(crate::press::inventory::Press::InfoClose)),
    layer: Layer::Usual,
    drawn: None,
};

/// A card's illustration, 300 by 417, at the left of the client's 640-wide stage and the top of the screen
/// (`WID_CARDITEMILLUSTWND`). Escape closes it.
pub(in crate::web) static CARD_ILLUST: Window = Window {
    id: "card-illust",
    open: |app| app.bag.illust.is_some(),
    frame: |_, view| staged(view, 0, 0, 300, 417),
    draw: App::card_illust_window,
    order: ordered(|app| app.bag.illust.is_some(), |app| app.pressed(crate::press::inventory::Press::IllustClose)),
    layer: Layer::Usual,
    drawn: None,
};

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
pub(in crate::web) static IDENTIFY: Window =
    Window { id: "identify", open: |app| app.bag.identify.is_some(), frame: |_, view| middle(view, 200, 200), draw: App::identify_window, order: None, layer: Layer::Asks, drawn: None };
