//! The entry of the list of windows for the family: the question of one who asks to adopt the character.

use super::*;

/// The question (`WID_BABYACCEPTWND`, `WID_BABYACCEPTWND2`): the client's message box, 200 of 480 down the
/// middle of the screen.
pub(in crate::web) static FAMILY_ASKS: Window = Window::question("family-asks", |app| app.family.asks.is_some(), App::family_question);
