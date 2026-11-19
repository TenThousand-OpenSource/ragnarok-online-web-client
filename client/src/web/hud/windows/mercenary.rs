//! The mercenary's entries of the list of windows: its window, what the window asks, the list of its skills,
//! the menu on the mercenary itself.

use super::*;

/// The mercenary window, where the client opens it (`WID_MERINFOWND`: 280 by 180, then moved to where
/// `MerInfoWndInfo::Reset` has it: 150 across and 100 down, as the homunculus window; the two are seldom
/// open together).
pub(in crate::web) static MERCENARY: Window = Window::usual(
    "mercenary",
    |app| app.mercenary.open && app.mercenary.shown(),
    |_, _| framed(at(150, 100, 280, 180)),
    App::mercenary_window,
    switched_off!(mercenary.open),
);

/// Whether to dismiss the mercenary.
pub(in crate::web) static MERCENARY_ASKS: Window = Window::question("mercenary-asks", |app| app.mercenary.asks && app.mercenary.shown(), App::mercenary_question);

/// The list of the mercenary's skills, where the client opens it (`WID_SKILLLISTWND_SKILLTAB_1`: 200 across
/// its 640-wide stage and 200 of 480 down, 280 wide).
pub(in crate::web) static MERCENARY_SKILLS: Window = Window::usual(
    "mercenary-skills",
    |app| app.mercenary.skills_open && app.mercenary.shown(),
    |app, view| staged(view, 200, 200, 280, crate::web::hud::skill_list::list_height(app.mercenary.skills.len())),
    App::mercenary_skills,
    switched_off!(mercenary.skills_open),
);

/// The menu of the mercenary, opened with the right button on it, where the pointer was.
pub(in crate::web) static MERCENARY_MENU: Window = Window::menu("mercenary-menu", |app| app.mercenary.menu_at.is_some() && app.mercenary.shown(), |app, _| menu_frame(app.mercenary.menu_at), App::mercenary_menu);
