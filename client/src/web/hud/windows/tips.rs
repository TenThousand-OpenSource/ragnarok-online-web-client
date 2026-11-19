//! The entry of the list of windows for the Tipbox.

use super::*;

/// The Tipbox (`UITipboxWnd`): 480 by 365, 120 across the client's stage and a quarter of the way down.
pub(in crate::web) static TIPS: Window = Window {
    id: "tips",
    open: |app| app.tips.open,
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:480px;height:365px", (view.0 as i32 - 640) / 2 + 120, view.1 as i32 / 4)),
    draw: App::tip_window,
    order: ordered(|app| app.tips.open, |app| app.pressed(crate::press::tips::Press::Tip(0))),
    layer: Layer::Usual,
    drawn: None,
};
