//! The homunculus's entries of the list of windows: its window, what the window asks, the list of its skills,
//! the menu on the homunculus itself.

use super::*;

/// The homunculus window, where the client opens it (`WID_HOMUNINFOWND`: 280 by 208, then moved to where
/// `HomunInfoWndInfo::Reset` has it: 150 across and 100 down).
pub(in crate::web) static HOMUNCULUS: Window = Window::usual(
    "homunculus",
    |app| app.homunculus.open && app.homunculus.shown(),
    |_, _| framed(at(150, 100, 280, 208)),
    App::homunculus_window,
    switched_off!(homunculus.open),
);

/// What the homunculus window asks before it acts.
pub(in crate::web) static HOMUNCULUS_ASKS: Window = Window::question("homunculus-asks", |app| app.homunculus.asks.is_some() && app.homunculus.shown(), App::homunculus_question);

/// The list of the homunculus's skills, where the client opens it (`WID_SKILLLISTWND_SKILLTAB_0`: 200 across
/// its 640-wide stage and 200 of 480 down, 280 wide).
pub(in crate::web) static HOMUNCULUS_SKILLS: Window = Window::usual(
    "homunculus-skills",
    |app| app.homunculus.skills_open && app.homunculus.shown(),
    |app, view| staged(view, 200, 200, 280, crate::web::hud::skill_list::list_height(app.homunculus.skills.len())),
    App::homunculus_skills,
    switched_off!(homunculus.skills_open),
);

/// The menu of the homunculus, opened with the right button on it, where the pointer was.
pub(in crate::web) static HOMUNCULUS_MENU: Window = Window::menu("homunculus-menu", |app| app.homunculus.menu_at.is_some() && app.homunculus.shown(), |app, _| menu_frame(app.homunculus.menu_at), App::homunculus_menu);
