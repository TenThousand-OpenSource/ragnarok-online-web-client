//! The navigation's entries of the list of windows.

use super::*;
use crate::press::navigation::Press;

/// Where the navigation's window opens (`NavigationWindowInfo`: in the middle of the screen), or where it
/// stood as a window beside it was opened.
fn corner(app: &App, view: (f64, f64)) -> (i32, i32) {
    app.navigation.at.unwrap_or(((view.0 * 0.5 - 136.0) as i32, (view.1 * 0.5 - 169.0) as i32))
}

/// The navigation's window (`UINavigationV4Wnd`): 272 by 338; its small form 134 by 61, let through a third
/// (`targetAlpha` 0xAA). Escape does what its cross does.
pub(in crate::web) static NAVIGATION: Window = Window::usual(
    "navigation",
    App::navigation_open,
    |app, view| {
        let (x, y) = ((view.0 * 0.5 - 136.0) as i32, (view.1 * 0.5 - 169.0) as i32);
        framed(if app.navigation.guide.small { format!("{};opacity:.67", at(x, y, 134, 61)) } else { at(x, y, 272, 338) })
    },
    App::navigation_window,
    |app| app.pressed(Press::Close),
);

/// The help (`UINavigationHelpWnd`): 310 by 200 at (100, 100).
pub(in crate::web) static NAVIGATION_HELP: Window = Window::usual(
    "navigation-help",
    |app| app.navigation.help,
    |_, _| framed(at(100, 100, 310, 200)),
    App::navigation_help_window,
    |app| app.pressed(Press::Help),
);

/// The sets of arrows (`UINavigationroadiconWnd`): 158 by 102, 113 across and 211 down from the navigation's
/// window.
pub(in crate::web) static NAVIGATION_ICONS: Window = Window::usual(
    "navigation-icons",
    |app| app.navigation.icons,
    |app, view| framed(at(corner(app, view).0 + 113, corner(app, view).1 + 211, 158, 102)),
    App::navigation_icons_window,
    |app| app.pressed(Press::Icons),
);

/// The list of the route (`UINavigationRuideWnd`): 150 by 338, at the right of the navigation's window.
pub(in crate::web) static NAVIGATION_ROUTE: Window = Window::usual(
    "navigation-route",
    |app| app.navigation.route,
    |app, view| framed(at(corner(app, view).0 + 272, corner(app, view).1, 150, 338)),
    App::navigation_route_window,
    |app| app.pressed(Press::Route),
);

/// "Do you want to cancel navigation?".
pub(in crate::web) static NAVIGATION_ASKS: Window = Window::question("navigation-asks", |app| app.navigation.asks, App::navigation_question);
