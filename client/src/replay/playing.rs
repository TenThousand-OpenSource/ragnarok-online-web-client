//! What is held back while a recording plays, as the client holds it: which keys of the interface still act
//! (`UIWindowMgr::ProcessShortcutKey`), which windows the one watching may open (`Replay::AllowsEvent`, asked
//! by `UIMenuIconWnd::SendMsg`), and what the options the recording was made with show and hide
//! (`UIWindowMgr::ApplyReplayOptions`).

use super::{option, Options};
use crate::hotkeys::interface as key;

/// The places of the interface's keys that are the player's own (`HK_REPLAY_STARTSTOP`, `HK_REPLAY_SPEEDP`,
/// `HK_REPLAY_SPEEDN`) and the one that opens the recorder (`HK_REPLAY_ONOFF`).
pub const RECORDER: usize = 38;
pub const START_STOP: usize = 39;
pub const FASTER: usize = 40;
pub const SLOWER: usize = 41;

/// Whether a key of the interface acts while a recording plays (`ProcessShortcutKey`): the player's three
/// always; the minimap, the World Map, the bars over heads, the quests and the commands always; a window of
/// the character only if the recording was made with its option; any other key not at all.
pub fn key_acts(place: usize, options: &Options) -> bool {
    let with = |option: usize| options[option] != 0;
    match place {
        START_STOP | FASTER | SLOWER => true,
        // (20 is `HK_GAGE_ONOFF`, which this client has no entry for.)
        key::MINIMAP | key::WORLD_MAP | 20 | key::QUESTS | key::COMMANDS => true,
        key::BASIC_INFO => with(option::BASIC),
        // (22 is `HK_MEMORIALWND_ONOFF`.)
        key::EQUIPMENT | 22 | key::PET | key::HOMUNCULUS => with(option::EQUIP),
        key::INVENTORY | key::CART => with(option::ITEM),
        key::SKILLS => with(option::SKILL),
        key::FRIENDS | key::PARTY => with(option::PARTY),
        key::HOTKEY_BAR => with(option::SHORTCUT),
        key::CHAT | key::CHAT_SIZE => with(option::CHATTING),
        key::STATUS => with(option::STATUS),
        _ => false,
    }
}

/// The windows of the character the one watching may ask for from the menu's icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    Status,
    Items,
    Equipment,
    Skills,
    Party,
}

/// Whether the menu opens such a window while a recording plays (`AllowsEvent` at `ID_STATUS`, `ID_ITEMS`,
/// `ID_EQUIP`, `ID_SKILL`, `ID_FRIEND`): only if the recording was made with its option. Every other icon of
/// the menu does nothing then, but the one that folds the menu away.
pub fn window_opens(window: Window, options: &Options) -> bool {
    options[match window {
        Window::Status => option::STATUS,
        Window::Items => option::ITEM,
        Window::Equipment => option::EQUIP,
        Window::Skills => option::SKILL,
        Window::Party => option::PARTY,
    }] != 0
}

/// Whether the chat window and what is written into it show while a recording plays (the seventh option:
/// `ApplyReplayOptions`, `UIM_PUSHINTOCHATHISTORY`), and whether the hotkey bar does (the eighth).
pub fn chat_shows(options: &Options) -> bool {
    options[option::CHATTING] != 0
}

pub fn bar_shows(options: &Options) -> bool {
    options[option::SHORTCUT] != 0
}

/// Whether Basic Information shows (`UIWindowMgr` at `WID_BASICINFOWND`: shown by the first option).
pub fn basic_shows(options: &Options) -> bool {
    options[option::BASIC] != 0
}

/// Whether a press of the page does anything while a recording plays. The player's own buttons always, and
/// the fold of the menu (`ID_CLOSE` and the menu's own opening are never held back); the windows of the
/// character and what is pressed in them only if the recording was made with their option (`AllowsEvent`);
/// nothing else: the client hands no other command of the menu on, and its fields, boxes and buttons do
/// nothing then (`UIEditCtrl`, `UICheckBox`, `UITextButton`, `ShortcutPress`).
pub fn press_passes(press: &crate::press::Press, options: &Options) -> bool {
    use crate::press::{general, inventory, Press};
    match press {
        Press::Replay(_) | Press::General(general::Press::Menu) => true,
        Press::Status(_) => window_opens(Window::Status, options),
        Press::Inventory(inventory::Press::Equipment | inventory::Press::EquipmentPage(_)) => window_opens(Window::Equipment, options),
        Press::Inventory(_) => window_opens(Window::Items, options),
        Press::Skills(_) => window_opens(Window::Skills, options),
        Press::Party(_) | Press::Friends(_) => window_opens(Window::Party, options),
        _ => false,
    }
}

/// Whether a press the client has no record of its own for is still written into a recording, by its name
/// (`sections::kind::PRESS`): one of a window the server opens and the player answers (a talk with an NPC, a
/// shop, a list a skill or an item opens, what a script shows, a deal), which nothing of what the server says
/// afterwards would otherwise close when the recording plays. Most presses of those windows have the client's
/// record (`UIWindow::RecordResponse` in its dialogs, shops, lists and accept boxes) and are written as that:
/// `super::told::records_of`, which asks here for the rest.
pub fn press_recorded(press: &crate::press::Press) -> bool {
    use crate::press::Press;
    matches!(press, Press::Talk(_) | Press::Shops(_) | Press::Choosing(_) | Press::Scripted(_) | Press::Exchange(_))
}
