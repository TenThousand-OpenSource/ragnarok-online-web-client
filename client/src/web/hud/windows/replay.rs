//! The entries of the list of windows for the replays. (The list of recordings is of the login screen, and
//! is written with it: `view/accounts.rs`.)

use super::*;
use crate::press::replay::Press;

/// The recorder (`WID_REPLAYRECCONTROLWND`): 194 wide, where the client first opens it, 259 across its stage
/// and 230 of 480 down.
pub(in crate::web) static REPLAY_RECORDER: Window = Window::usual(
    "replay-recorder",
    |app| app.replay.open,
    |app, view| staged(view, 259, 230, 194, if app.replay.mode == 2 { 200 } else { 34 }),
    App::replay_recorder,
    |app| app.pressed(Press::End),
);

/// The player (`WID_REPLAYCONTROLWND`): 197 by 138 in the lower right corner of the screen, in no order of
/// windows: Escape does not put it away.
pub(in crate::web) static REPLAY_PLAYER: Window = Window {
    id: "replay-player",
    open: |app| app.replay.playing.is_some(),
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:197px;height:138px", view.0 as i32 - 197, view.1 as i32 - 138)),
    draw: App::replay_player,
    order: None,
    layer: Layer::Usual,
    drawn: None,
};
