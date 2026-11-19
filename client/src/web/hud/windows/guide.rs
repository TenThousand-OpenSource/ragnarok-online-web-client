//! The Adventure Guide's entry of the list of windows.

use super::*;

/// The Adventure Guide, 750 by 468, in the middle.
pub(in crate::web) static GUIDE: Window = Window::usual(
    "guide",
    |app| app.guide.open,
    |_, view| framed(format!("{};height:468px", centred(view, 750, 468))),
    App::guide_window,
    switched_off!(guide.open),
);
