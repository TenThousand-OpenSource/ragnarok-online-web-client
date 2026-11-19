//! The entry of the list of windows for looking at another player's equipment.

use super::*;

/// The window that shows another's equipment, where the client opens it (`WID_EQUIPWND_VIEWOTHER_1`): 185
/// across its 640-wide stage, 300 of 480 down, 280 by 167.
pub(in crate::web) static VIEW_EQUIP: Window = Window {
    id: "viewequip",
    open: |app| app.viewequip.seen.is_some(),
    frame: |_, view| staged(view, 185, 300, 280, 167),
    draw: App::view_equip_window,
    order: ordered(|app| app.viewequip.seen.is_some(), |app| app.pressed(crate::press::viewequip::Press::Close)),
    layer: Layer::Usual,
    drawn: None,
};
