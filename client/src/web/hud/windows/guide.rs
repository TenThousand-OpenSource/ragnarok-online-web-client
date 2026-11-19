//! The Adventure Guide's entry of the list of windows.

use super::*;

/// The Adventure Guide, 750 by 468, in the middle.
pub(in crate::web) static GUIDE: Window = Window {
    id: "guide",
    open: |app| app.guide.open,
    frame: |_, view| framed(format!("{};height:468px", centred(view, 750, 468))),
    draw: App::guide_window,
    order: ordered(|app| app.guide.open, switched_off!(guide.open)),
    layer: Layer::Usual,
    drawn: None,
};
