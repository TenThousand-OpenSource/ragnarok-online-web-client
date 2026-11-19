//! The entry of the list of windows for the packets this client does not read (`/packets`).

use super::*;
use crate::web::hud::unread::SIZE;

/// The list of packets not read: a tool of this project's own, so it stands where it was decided here, by the
/// left edge of the screen under the Basic Information window. Escape closes it as its cross does.
pub(in crate::web) static UNREAD: Window = Window::usual(
    "unread",
    |app| app.unread.open,
    |_, _| framed(at(4, 160, SIZE.0, SIZE.1)),
    App::unread_window,
    |app| app.pressed(crate::press::unread::Press::Close),
);
