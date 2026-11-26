//! The entries of the list of windows for the battlegrounds.

use super::*;
use crate::press::battleground::Press;

/// The list (`WID_JOINTO_BATTLEFIELDWND`): 350 by 350, where the client first opens it, 350 across and 150
/// down (`EntryQueueWndInfo::Reset`).
pub(in crate::web) static BATTLEGROUND: Window = Window::usual(
    "battleground",
    |app| app.battleground.open,
    |_, _| framed("left:350px;top:150px;width:350px;height:350px".to_owned()),
    App::battleground_window,
    |app| app.pressed(Press::Close),
);

/// Its help (`WID_ENTRYQUEUEHELPWND`): 280 by 280. (The client makes it at the corner of the screen, having
/// given it no place; here it opens beside the list.)
pub(in crate::web) static BATTLEGROUND_HELP: Window = Window::usual(
    "battleground-help",
    |app| app.battleground.help,
    |_, _| framed("left:60px;top:150px;width:280px;height:280px".to_owned()),
    App::battleground_help,
    |app| app.pressed(Press::HelpClose),
);

/// The window of the queue (`WID_ENTRYQUEUESTANDBYWND`): 280 by 120, 185 across the client's stage and 180
/// down it (`UIEntryQueueStandByWnd::Create`).
pub(in crate::web) static BATTLEGROUND_STANDBY: Window = Window::usual(
    "battleground-standby",
    |app| app.battleground.standby,
    |_, view| staged(view, 185, 180, 280, 120),
    App::battleground_standby,
    |app| app.pressed(Press::StandbyClose),
);

/// The window that asks whether to enter (`WID_ENTRYQUEUEREQUESTWND`): 300 by 250 in the same place, over the
/// windows.
pub(in crate::web) static BATTLEGROUND_ENTRY: Window = Window {
    id: "battleground-entry",
    open: |app| app.battleground.entry.is_some(),
    frame: |_, view| staged(view, 185, 180, 300, 250),
    draw: App::battleground_entry,
    order: None,
    layer: Layer::Asks,
    drawn: None,
};

/// The question before applying, in the client's message box.
pub(in crate::web) static BATTLEGROUND_ASKS: Window =
    Window::question("battleground-asks", |app| app.battleground.asks.is_some(), App::battleground_question);

/// Every window of the battlegrounds: drawn anew together when what they show changes.
pub(in crate::web) static BATTLEGROUND_WINDOWS: [&Window; 5] =
    [&BATTLEGROUND, &BATTLEGROUND_HELP, &BATTLEGROUND_STANDBY, &BATTLEGROUND_ENTRY, &BATTLEGROUND_ASKS];
