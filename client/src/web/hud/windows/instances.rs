//! The entry of the list of windows for the party's instance.

use super::*;

/// The instance's window (`UIMemorialDunWnd`): 200 by 150, at the left edge 310 down
/// (`UIWindowMgr::EnsureWindow`, `WID_MEMORIALDUNWND`). Escape puts it away as its cross does: that the
/// client's Escape does so was not found in its code and is taken from what its other framed windows do.
pub(in crate::web) static INSTANCE: Window = Window::usual(
    "instance",
    |app| app.instances.shown && app.instances.known.is_some(),
    |_, _| {
        let ((x, y), (width, height)) = (crate::instances::PLACE, crate::instances::SIZE);
        framed(format!("left:{x}px;top:{y}px;width:{width}px;height:{height}px"))
    },
    App::instance_window,
    |app| app.pressed(crate::press::instances::Press::Close),
);
