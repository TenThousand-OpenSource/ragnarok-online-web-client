//! The entry of the list of windows for what an NPC's script shows: the picture over the screen.

use super::*;
use crate::net::scripted::Stand;

/// Where a picture stands, as a style (`CGameMode::UpdateMiniMapTooltip`): on the bottom edge with its middle
/// 200 from the left, in the middle or 200 from the right, where the pointer goes through it to the world
/// (`UIIllustWnd::CanHitTest`); or in the very middle of the screen, where it has a cross and is pressed.
/// The picture's own size is not known as it opens: the window is put by its middle.
pub(in crate::web) fn cutin_style(stand: Option<Stand>) -> String {
    match stand {
        Some(Stand::Left) => "left:200px;bottom:0;transform:translateX(-50%);pointer-events:none".to_owned(),
        Some(Stand::Middle) => "left:50%;bottom:0;transform:translateX(-50%);pointer-events:none".to_owned(),
        Some(Stand::Right) => "left:calc(100% - 200px);bottom:0;transform:translateX(-50%);pointer-events:none".to_owned(),
        Some(Stand::Titled | Stand::Centre) => "left:50%;top:50%;transform:translate(-50%,-50%)".to_owned(),
        None => String::new(),
    }
}

/// The picture a script shows (the client's `UIIllustWnd`). It lies under the windows; one with a cross is
/// closed by Escape as by its cross.
pub(in crate::web) static CUTIN: Window = Window {
    id: "cutin",
    open: |app| app.scripted.cutin.is_some(),
    frame: |app, _| framed(cutin_style(app.scripted.cutin.as_ref().map(|(_, stand)| *stand))),
    draw: App::cutin_window,
    order: ordered(|app| matches!(app.scripted.cutin, Some((_, Stand::Titled | Stand::Centre))), |app| app.pressed(crate::press::scripted::Press::CutinClose)),
    layer: Layer::Under,
    drawn: None,
};
