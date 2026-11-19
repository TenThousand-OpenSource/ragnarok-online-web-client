//! The entry of the list of windows for the menu on another player.

use super::*;

/// The menu on another player, opened with the right button on them, where the pointer was.
pub(in crate::web) static PLAYER_MENU: Window =
    Window::menu("player-menu", |app| app.player_menu.is_some(), |app, _| menu_frame(app.player_menu.as_ref().map(|menu| menu.at)), App::player_menu);
