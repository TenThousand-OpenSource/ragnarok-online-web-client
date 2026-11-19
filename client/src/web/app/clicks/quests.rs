//! The quest journal: its tabs, filters, search and rows, the pane of one quest, and the list on the screen.

use super::*;
use crate::net::quests;
use crate::press::quests::{Press, Row};

impl App {
    /// A press of this system (`crate::press::quests`).
    pub(super) fn quests_pressed(&mut self, press: Press) {
        self.quests_remembered();
        match press {
            // The journal is opened or closed (the client's `ID_QUEST` of its menu): opened, the button that
            // tells of a new quest goes (`CUIRenewQuestUI::OnInit`); closed, the pane of one quest goes with
            // it (`~CUIRenewQuestUI`).
            Press::Window | Press::Notice => {
                self.quests.open = press == Press::Notice || !self.quests.open;
                if self.quests.open {
                    self.quests.notice = false;
                } else {
                    self.quests.pane = None;
                }
                self.render();
            }
            // A tab shows its quests from the top (`OnProcessTabClick` and the others). The first time the
            // recommended are looked at, the first of them is opened (`OnRecommendedTabClick`).
            Press::Tab(tab) => {
                self.quests.tab = tab.min(crate::quests::tab::WAIT);
                self.redraw(&[&wnd::QUESTS]);
                if let Some(list) = element("quests").and_then(|journal| journal.query_selector(".scroll").ok().flatten()) {
                    list.set_scroll_top(0);
                }
                if tab == crate::quests::tab::RECOMMENDED && !std::mem::replace(&mut self.quests.recommended_seen, true) {
                    if let Some((number, _)) = data::recommended_quests().into_iter().find(|(number, _)| !self.quests.passed.contains(number)) {
                        self.quest_picked(Row { number, recommended: true });
                    }
                }
            }
            Press::Filter(filter) => {
                self.quests.filter = filter;
                self.redraw(&[&wnd::QUESTS]);
            }
            Press::Search => {
                self.quests.search = field("quest-search");
                self.redraw(&[&wnd::QUESTS]);
            }
            Press::Pick(row) => self.quest_picked(row),
            // Whether a quest is in the list on the screen (`CUIQuestItem::OnSimpleView`).
            Press::Listed(quest) => {
                if !self.quests.unlisted.remove(&quest) {
                    self.quests.unlisted.insert(quest);
                }
                self.quests_keep();
                self.quests_changed();
            }
            // The lock (`CUIQuestItem::OnActiveStateChanged`): the server is asked to switch its quest, and
            // answers with ZC_ACTIVE_QUEST; a recommended one is put away, or taken back, by the client alone.
            Press::Lock(Row { number, recommended: true }) => {
                if !self.quests.passed.remove(&number) {
                    self.quests.passed.insert(number);
                }
                self.quests_keep();
                self.quests_changed();
            }
            Press::Lock(Row { number, recommended: false }) => {
                if let Some(quest) = self.quests.log.get(&number) {
                    self.send(&quests::activate(number, !quest.active));
                }
            }
            // The box under the journal (`UIWindow::RecordWindowFlag`): the list on the screen shows or not.
            Press::Display => {
                self.quests.display_off = !self.quests.display_off;
                self.quests_keep();
                self.quests_changed();
            }
            Press::PaneClose => {
                self.quests.pane = None;
                self.quests_changed();
            }
        }
    }

    /// The client's key for the journal (`HK_QUESTWND_ONOFF`) does what its icon in the menu does.
    pub(in crate::web::app) fn quest_key(&mut self) -> bool {
        self.pressed(Press::Window);
        true
    }

    /// A row of the journal was pressed: its pane opens beside the journal, in the place of the one that was
    /// open (`CUIQuestList::OnSelectItem`). The names of what a quest gives are asked for.
    fn quest_picked(&mut self, row: Row) {
        self.quests.pane = Some((row.number, row.recommended));
        if !row.recommended {
            let rewards = data::quest_text(row.number).map(|about| about.rewards).unwrap_or_default();
            data::want_items(rewards.into_iter().map(|(item, _)| item), || dispatch(Action::ItemsKnown));
        }
        self.quests_changed();
    }
}
