//! Where the windows stand, as their tags say it: what the entries of the list of windows are placed with.

/// The tag of a usual window: one that is dragged by its title and comes to the front when pressed.
pub(super) fn framed(style: String) -> String {
    format!(r#"class="a hud" style="{style}""#)
}

/// Where a window `width` wide stands that the client puts in the middle, so far down a screen 480 high.
pub(super) fn middle(view: (f64, f64), width: i32, down: i32) -> String {
    framed(format!("left:{}px;top:{}px;width:{width}px", (view.0 as i32 - width) / 2, view.1 as i32 * down / 480))
}

/// Where a window of that size stands in the very middle of the screen.
pub(super) fn centred(view: (f64, f64), width: i32, height: i32) -> String {
    format!("left:{}px;top:{}px;width:{width}px", ((view.0 as i32 - width) / 2).max(0), ((view.1 as i32 - height) / 2).max(0))
}

/// Where the client opens a window so far across its 640-wide stage and so far of 480 down
/// (`CenterOnViewportX`, `ScaleByScreenHeight`).
pub(super) fn staged(view: (f64, f64), across: i32, down: i32, width: i32, height: i32) -> String {
    framed(format!("left:{}px;top:{}px;width:{width}px;height:{height}px", (view.0 as i32 - 640) / 2 + across, view.1 as i32 * down / 480))
}

/// Where a menu opened with the right button stands, as a style: a white box with a grey rim, its corner at
/// that place of the stage. `App::open_at` is given this when a menu opens without the screen being written.
pub(in crate::web) fn menu_style((x, y): (i32, i32)) -> String {
    format!("left:{x}px;top:{y}px;background:#fff;border:1px solid #9a9a9a")
}

/// The tag of a menu that was opened at that place.
pub(super) fn menu_frame(at: Option<(i32, i32)>) -> String {
    framed(menu_style(at.unwrap_or_default()))
}
