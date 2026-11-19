//! The entries of the list of windows for the lists a skill or a card opens to pick one thing of: one to each
//! of the client's three shapes of such a window. They ask something, and lie over the windows.

use super::*;
use crate::web::app::Shape;

/// The places of a warp, 280 by 120: 200 across the client's 640-wide stage, 300 of 480 down
/// (`WID_CHOOSEWARPWND`).
pub(in crate::web) static CHOOSE_PLACE: Window = Window {
    id: "choose-place",
    open: |app| app.choosing_in(Shape::Places),
    frame: |_, view| staged(view, 200, 300, 280, 120),
    draw: App::choose_place_window,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// A list of rows with pictures, 200 by 200, where the list of items to identify stands
/// (`WID_MAKINGARROWLISTWND`, `WID_SPELLLISTWND`, `WID_ITEMCOMPOSITIONWND`: the same place as
/// `WID_ITEMIDENTIFYWND`).
pub(in crate::web) static CHOOSE_PICTURED: Window = Window {
    id: "choose-pictured",
    open: |app| app.choosing_in(Shape::Pictured),
    frame: |_, view| middle(view, 200, 200),
    draw: App::choose_pictured_window,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// A list of lines or a thing being forged, 280 by 150: 185 across the stage, 60 above 300 of 480 down
/// (`WID_MAKETARGETLISTWND`, `WID_MAKETARGETPROCESSWND`, `WID_ITEMREPAIRWND`, `WID_WEAPONREFINEWND`).
pub(in crate::web) static CHOOSE_LINE: Window = Window {
    id: "choose-line",
    open: |app| app.choosing_in(Shape::Lines),
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px;height:150px", (view.0 as i32 - 640) / 2 + 185, view.1 as i32 * 300 / 480 - 60)),
    draw: App::choose_line_window,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};
