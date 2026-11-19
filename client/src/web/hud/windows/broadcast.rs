//! The entry of the list of windows for the server's announcements across the top of the screen.

use super::*;

/// The announcements, where the client puts them (`CGameMode::EnsureBroadcastBalloon`): as wide as the
/// screen, 51 down it, in front of everything (`AddRenderedFront`); the pointer goes through it
/// (`UIBroadcastBalloon::CanHitTest`).
pub(in crate::web) static BROADCAST: Window = Window {
    id: "broadcast",
    open: |app| !app.broadcast.shown.is_empty(),
    frame: |_, _| r#"class="a" style="left:0;top:51px;width:100%;pointer-events:none""#.to_owned(),
    draw: App::broadcast_window,
    order: None,
    layer: Layer::Top,
    drawn: None,
};
