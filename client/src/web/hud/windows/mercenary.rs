//! The mercenary's entries of the list of windows: its window, what the window asks, the list of its skills,
//! the menu on the mercenary itself.

use super::*;

/// The mercenary window, where the client opens it (`WID_MERINFOWND`: 280 by 180, then moved to where
/// `MerInfoWndInfo::Reset` has it: 150 across and 100 down, as the homunculus window; the two are seldom
/// open together).
pub(in crate::web) static MERCENARY: Window = Window {
    id: "mercenary",
    open: |app| app.mercenary.open && app.mercenary.shown(),
    frame: |_, _| framed("left:150px;top:100px;width:280px;height:180px".to_owned()),
    draw: App::mercenary_window,
    order: ordered(|app| app.mercenary.open && app.mercenary.shown(), switched_off!(mercenary.open)),
    layer: Layer::Usual,
    drawn: None,
};

/// Whether to dismiss the mercenary.
pub(in crate::web) static MERCENARY_ASKS: Window = Window::question("mercenary-asks", |app| app.mercenary.asks && app.mercenary.shown(), App::mercenary_question);

/// The list of the mercenary's skills, where the client opens it (`WID_SKILLLISTWND_SKILLTAB_1`: 200 across
/// its 640-wide stage and 200 of 480 down, 280 wide).
pub(in crate::web) static MERCENARY_SKILLS: Window = Window {
    id: "mercenary-skills",
    open: |app| app.mercenary.skills_open && app.mercenary.shown(),
    frame: |app, view| staged(view, 200, 200, 280, crate::web::hud::skill_list::list_height(app.mercenary.skills.len())),
    draw: App::mercenary_skills,
    order: ordered(|app| app.mercenary.skills_open && app.mercenary.shown(), switched_off!(mercenary.skills_open)),
    layer: Layer::Usual,
    drawn: None,
};

/// The menu of the mercenary, opened with the right button on it, where the pointer was.
pub(in crate::web) static MERCENARY_MENU: Window = Window::menu("mercenary-menu", |app| app.mercenary.menu_at.is_some() && app.mercenary.shown(), |app, _| menu_frame(app.mercenary.menu_at), App::mercenary_menu);
