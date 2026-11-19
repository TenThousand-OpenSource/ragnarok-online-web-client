//! The entries of the list of windows for the World Map and the large window of the map one is on.

use super::*;

/// The World Map (`UIRoMapWnd`): over all of the screen, over the other windows (the client keeps it as its
/// one modal window).
pub(in crate::web) static WORLD_MAP: Window = Window {
    id: "worldmap",
    open: |app| app.worldmap.open,
    frame: |_, view| framed(at(0, 0, view.0 as i32, view.1 as i32)),
    draw: App::worldmap_window,
    order: ordered(|app| app.worldmap.open, |app| app.pressed(crate::press::worldmap::Press::Window)),
    layer: Layer::Usual,
    drawn: None,
};

/// The large window of the map one is on (`UIMiniMapWnd`): 516 by 559, in the middle of the screen. (The
/// client opens it where it was last left.)
pub(in crate::web) static LARGE_MAP: Window = Window {
    id: "large-map",
    open: |app| app.worldmap.large,
    frame: |_, view| framed(centred(view, 516, 559)),
    draw: App::large_map_window,
    order: ordered(|app| app.worldmap.large, |app| app.pressed(crate::press::worldmap::Press::Large)),
    layer: Layer::Usual,
    drawn: None,
};
