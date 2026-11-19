//! The homunculus's entries of the list of windows: its window, what the window asks, the list of its skills,
//! the menu on the homunculus itself.

use super::*;

/// The homunculus window, where the client opens it (`WID_HOMUNINFOWND`: 280 by 208, then moved to where
/// `HomunInfoWndInfo::Reset` has it: 150 across and 100 down).
pub(in crate::web) static HOMUNCULUS: Window = Window {
    id: "homunculus",
    open: |app| app.homunculus.open && app.homunculus.shown(),
    frame: |_, _| framed("left:150px;top:100px;width:280px;height:208px".to_owned()),
    draw: App::homunculus_window,
    order: ordered(|app| app.homunculus.open && app.homunculus.shown(), switched_off!(homunculus.open)),
    layer: Layer::Usual,
    drawn: None,
};

/// What the homunculus window asks before it acts.
pub(in crate::web) static HOMUNCULUS_ASKS: Window = Window::question("homunculus-asks", |app| app.homunculus.asks.is_some() && app.homunculus.shown(), App::homunculus_question);

/// The list of the homunculus's skills, where the client opens it (`WID_SKILLLISTWND_SKILLTAB_0`: 200 across
/// its 640-wide stage and 200 of 480 down, 280 wide).
pub(in crate::web) static HOMUNCULUS_SKILLS: Window = Window {
    id: "homunculus-skills",
    open: |app| app.homunculus.skills_open && app.homunculus.shown(),
    frame: |app, view| staged(view, 200, 200, 280, crate::web::hud::skill_list::list_height(app.homunculus.skills.len())),
    draw: App::homunculus_skills,
    order: ordered(|app| app.homunculus.skills_open && app.homunculus.shown(), switched_off!(homunculus.skills_open)),
    layer: Layer::Usual,
    drawn: None,
};

/// The menu of the homunculus, opened with the right button on it, where the pointer was.
pub(in crate::web) static HOMUNCULUS_MENU: Window = Window::menu("homunculus-menu", |app| app.homunculus.menu_at.is_some() && app.homunculus.shown(), |app, _| menu_frame(app.homunculus.menu_at), App::homunculus_menu);
