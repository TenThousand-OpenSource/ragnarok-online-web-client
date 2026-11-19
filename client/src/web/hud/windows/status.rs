//! The entries of the list of windows for what is always on the screen of the character: Basic Information,
//! the menu under it, the Status window, the states down the right side.

use super::*;

pub(in crate::web) static BASIC: Window =
    Window { id: "basic", open: ALWAYS, frame: |_, _| framed("left:0;top:0;width:220px".to_owned()), draw: App::basic_info, order: None, layer: Layer::Usual, drawn: None };

/// The menu hangs under the basic window, a little into it.
pub(in crate::web) static MENU: Window =
    Window { id: "menu", open: ALWAYS, frame: |app, _| framed(format!("left:0;top:{}px;width:220px", app.basic_height() - 4)), draw: App::menu_icons, order: None, layer: Layer::Usual, drawn: None };

pub(in crate::web) static STATUS: Window = Window {
    id: "status",
    open: |app| app.hero.status_open,
    frame: |_, _| framed(at(225, 285, 280, 141)),
    draw: App::status_window,
    order: ordered(|app| app.hero.status_open, switched_off!(hero.status_open)),
    layer: Layer::Usual,
    drawn: None,
};

/// The states the character is in, down the right side of the screen: its pictures stand by the right edge, so
/// the element is a point at the corner; nothing of it but the pictures takes the mouse.
pub(in crate::web) static STATUSES: Window =
    Window { id: "statuses", open: ALWAYS, frame: |_, _| r#"class="a" style="right:0;top:0;width:0;height:0""#.to_owned(), draw: App::statuses_column, order: None, layer: Layer::Usual, drawn: None };
