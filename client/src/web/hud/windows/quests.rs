//! The quests' entries of the list of windows: the journal, the pane of one quest, the button that tells of a
//! new one, the list on the screen.

use super::*;
use crate::web::hud::quests::{JOURNAL, list_place, notice_place, pane_place};

/// The quest journal, in the middle of the view (`CUIRenewQuestUI::Initialize`).
pub(in crate::web) static QUESTS: Window = Window {
    id: "quests",
    open: |app| app.quests.open,
    frame: |_, view| framed(format!("{};height:{}px", centred(view, JOURNAL.0, JOURNAL.1), JOURNAL.1)),
    draw: App::quest_window,
    order: ordered(|app| app.quests.open, |app| app.pressed(crate::press::quests::Press::Window)),
    layer: Layer::Usual,
    drawn: None,
};

/// The pane of one quest, at the right side of where the journal opens (`quests::pane_place`).
pub(in crate::web) static QUEST_PANE: Window = Window {
    id: "quest-pane",
    open: |app| app.quests.open && app.quests.pane.is_some(),
    frame: |_, view| framed(pane_place(view)),
    draw: App::quest_pane,
    order: ordered(|app| app.quests.open && app.quests.pane.is_some(), |app| app.pressed(crate::press::quests::Press::PaneClose)),
    layer: Layer::Usual,
    drawn: None,
};

/// The button that tells of a new quest, and the list of quests on the screen: neither is a window that is
/// dragged, and nothing of the list is pressed (`notice_place`, `list_place`).
pub(in crate::web) static QUEST_NOTICE: Window =
    Window { id: "quest-notice", open: |app| app.quests.notice, frame: |_, view| notice_place(view), draw: App::quest_notice, order: None, layer: Layer::Usual, drawn: None };

pub(in crate::web) static QUEST_LIST: Window =
    Window { id: "quest-list", open: |app| !app.quests.display_off && !app.quests.log.is_empty(), frame: |_, view| list_place(view), draw: App::quest_list, order: None, layer: Layer::Usual, drawn: None };
