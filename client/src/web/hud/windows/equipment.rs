//! The entry of the list of windows for the Equipment window.

use super::*;

pub(in crate::web) static EQUIPMENT: Window = Window::usual(
    "equipment",
    |app| app.bag.equipment_open,
    |_, _| framed("left:560px;top:290px;width:280px".to_owned()),
    App::equipment_window,
    switched_off!(bag.equipment_open),
);
