//! The entries of the list of windows for what is always in the corner of the screen: Basic Information and
//! the menu under it.

use super::*;

pub(in crate::web) static BASIC: Window =
    Window {
        id: "basic",
        open: ALWAYS,
        frame: |_, _| framed("left:0;top:0;width:220px".to_owned()),
        draw: App::basic_info,
        order: None,
        layer: Layer::Usual,
        drawn: None
    };

/// The menu hangs under the basic window, a little into it.
pub(in crate::web) static MENU: Window =
    Window {
        id: "menu",
        open: ALWAYS,
        frame: |app, _| framed(format!("left:0;top:{}px;width:220px", app.basic_height() - 4)),
        draw: App::menu_icons,
        order: None,
        layer: Layer::Usual,
        drawn: None
    };
