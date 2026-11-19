//! The entry of the list of windows for the packets this client does not read (`/packets`).

use super::*;
use crate::web::hud::unread::SIZE;

/// The list of packets not read: a tool of this project's own, so it stands where it was decided here, by the
/// left edge of the screen under the Basic Information window. Escape closes it as its cross does.
pub(in crate::web) static UNREAD: Window = Window {
    id: "unread",
    open: |app| app.unread.open,
    frame: |_, _| framed(format!("left:4px;top:160px;width:{}px;height:{}px", SIZE.0, SIZE.1)),
    draw: App::unread_window,
    order: ordered(|app| app.unread.open, |app| app.pressed(crate::press::unread::Press::Close)),
    layer: Layer::Usual,
    drawn: None,
};
