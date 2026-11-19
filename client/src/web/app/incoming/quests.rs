//! The quest journal as the server tells it, what the chat says of it, and the marks over NPCs.

use super::*;
use crate::net::quests;
use crate::quests::Named;

/// The kind of line the chat keeps what is said of quests as (the client sends them with 19).
const SAID: u8 = 19;
/// Where the browser keeps what the client keeps in its `SaveData` folder: whether the list on the screen
/// shows (the registry's `QUESTDISPLAY`), the quests left out of it (`OngoingSimpleViewList`), and what of the
/// recommended was put away (`RecommendedActiveList`).
const KEPT: [&str; 3] = ["roweb.quests.display", "roweb.quests.unlisted", "roweb.quests.passed"];

impl App {
    /// Reads a packet of this kind from the map server; `false` for any other.
    pub(super) fn quest_packet(&mut self, id: u16, packet: &[u8]) -> bool {
        match id {
            // The journal on entering (`ReceiveOngoingQuestList`): a quest it has already only has its state
            // set, any other is new to it. Nothing is said of these.
            quests::ZC_ALL_QUEST_LIST => {
                for quest in quests::list(packet) {
                    self.quest_kept(quest);
                }
                self.quests_changed();
            }
            quests::ZC_ADD_QUEST => self.quest_added(packet),
            // What a new quest hunts, as the client reads it (`questUpdateType2`): the quest of that number has
            // the count and the goal of its monster of that number set.
            quests::ZC_HUNTING_QUEST_INFO => {
                for (quest, mob, goal, killed) in quests::hunting_info(packet) {
                    if let Some(hunt) = self.quests.log.get_mut(&quest).and_then(|quest| quest.hunts.iter_mut().find(|hunt| hunt.mob == mob)) {
                        (hunt.killed, hunt.goal) = (killed, goal);
                    }
                }
                self.quests_changed();
            }
            quests::ZC_UPDATE_MISSION_HUNT => self.quest_hunted(packet),
            // A quest is gone, finished or dropped (the client's `ID_ZC_DEL_QUEST`): the chat says which.
            quests::ZC_DEL_QUEST => {
                let Some(quest) = quests::deleted(packet) else { return true };
                let title = self.quests.log.contains_key(&quest).then(|| self.quest_title(quest)).unwrap_or_default();
                self.quests.log.remove(&quest);
                if self.quests.pane == Some((quest, false)) {
                    self.quests.pane = None;
                }
                self.quests_changed();
                self.chat_line_of(SAID, data::message(1611).unwrap_or_else(|| "[%s] quest was deleted.".to_owned()).replacen("%s", &title, 1), YELLOW);
            }
            quests::ZC_ACTIVE_QUEST => {
                if let Some((quest, active)) = quests::activated(packet) {
                    if let Some(quest) = self.quests.log.get_mut(&quest) {
                        quest.active = active;
                    }
                    self.quests_changed();
                }
            }
            quests::ZC_QUEST_NOTIFY_EFFECT => self.quest_marked(packet),
            _ => return false,
        }
        true
    }

    /// Reads, once, what the browser keeps of the journal.
    pub(in crate::web::app) fn quests_remembered(&mut self) {
        if std::mem::replace(&mut self.quests.remembered, true) {
            return;
        }
        let numbers = |name: &str| stored(name).unwrap_or_default().split(',').filter_map(|number| number.parse().ok()).collect();
        // The list shows until it is switched off (`QuestDisplayInfo::Reset`).
        self.quests.display_off = stored(KEPT[0]).is_some();
        (self.quests.unlisted, self.quests.passed) = (numbers(KEPT[1]), numbers(KEPT[2]));
    }

    /// Keeps in the browser what was set in the journal.
    pub(in crate::web::app) fn quests_keep(&self) {
        let numbers = |set: &std::collections::BTreeSet<u32>| (!set.is_empty()).then(|| set.iter().map(u32::to_string).collect::<Vec<_>>().join(","));
        store(KEPT[0], self.quests.display_off.then_some("off"));
        store(KEPT[1], numbers(&self.quests.unlisted).as_deref());
        store(KEPT[2], numbers(&self.quests.passed).as_deref());
    }

    /// What a quest is called: the client's title for it, or its number where the client's table has none (or
    /// the gateway is one that does not give the table yet).
    pub(in crate::web) fn quest_title(&self, quest: u32) -> String {
        data::quest_text(quest).map(|text| text.title).filter(|title| !title.is_empty()).unwrap_or_else(|| quest.to_string())
    }

    /// A quest of the server's list or one just taken goes into the journal, the names of what it hunts
    /// written as the client writes them. Whether it was new to the journal.
    fn quest_kept(&mut self, mut quest: quests::Quest) -> bool {
        self.quests_remembered();
        if let Some(known) = self.quests.log.get_mut(&quest.id) {
            known.active = quest.active;
            return false;
        }
        for hunt in &mut quest.hunts {
            hunt.name = match crate::quests::named(hunt.mob) {
                Named::Sent => continue,
                Named::Message(line) => data::message(line).unwrap_or_default(),
                Named::Map => data::map_sign(&hunt.name).map_or_else(|| hunt.name.clone(), |sign| sign.name),
                Named::Job => data::job_name(hunt.mob as u16),
            };
        }
        self.quests.log.insert(quest.id, quest);
        true
    }

    /// ZC_ADD_QUEST_EX (`ReceiveAddQuest`): a quest new to the journal has the button that tells of it shown
    /// in the screen's corner, and the chat says "You have received a new quest."
    fn quest_added(&mut self, packet: &[u8]) {
        let Some(quest) = quests::added(packet) else { return };
        let new = self.quest_kept(quest);
        if new {
            // The button is not there while the journal is open (`CUIRenewQuestUI::OnInit` closes it).
            self.quests.notice = !self.quests.open;
        }
        self.quests_changed();
        if new {
            self.chat_line_of(SAID, data::message(1374).unwrap_or_else(|| "You have received a new quest.".to_owned()), YELLOW);
        }
    }

    /// ZC_UPDATE_MISSION_HUNT_EX (`ReceiveQuestHuntUpdate`): the count and the goal of each hunt named, and
    /// for each a line in the chat of how far it is. (The client also shows these lines for a moment in the
    /// middle of the screen, in its broadcast balloon: not done here.)
    fn quest_hunted(&mut self, packet: &[u8]) {
        let line = data::message(1610).unwrap_or_else(|| "[%s] quest - defeating [%s] progress (%d/%d).".to_owned());
        let mut said = Vec::new();
        for progress in quests::hunted(packet) {
            let Some(hunt) = self.quests.log.get_mut(&progress.quest).and_then(|quest| quest.hunts.iter_mut().find(|hunt| hunt.id == progress.hunt)) else { continue };
            (hunt.killed, hunt.goal) = (progress.killed, progress.goal);
            let monster = hunt.name.clone();
            // A quest the client's table has no title for is not told of (the client's `m_title.empty()`).
            if let Some(title) = data::quest_text(progress.quest).map(|text| text.title).filter(|title| !title.is_empty()) {
                said.push(crate::quests::progress_line(&line, &title, &monster, progress.killed, progress.goal));
            }
        }
        self.quests_changed();
        for words in said {
            self.chat_line_of(SAID, words, YELLOW);
        }
    }

    /// ZC_QUEST_NOTIFY_EFFECT (`ReceiveMinimapMarker`): the mark over an NPC is set or taken away, in the
    /// world and on the minimap. The marks are those of this visit to the map: the first one told on the
    /// next drops the last one's.
    fn quest_marked(&mut self, packet: &[u8]) {
        let Some(mark) = quests::notified(packet) else { return };
        if self.quests.marks.0 != world::visit() {
            self.quests.marks = (world::visit(), Default::default());
        }
        if mark.effect == quests::NO_MARK {
            self.quests.marks.1.remove(&mark.npc);
        } else {
            self.quests.marks.1.insert(mark.npc, mark);
        }
        world::quest_mark(mark.npc, crate::quests::mark_action(mark.effect));
        self.quest_marks();
    }

    /// Writes the NPCs' marks on the minimap anew.
    pub(in crate::web::app) fn quest_marks(&self) {
        if let Some(marks) = element("quest-marks") {
            marks.set_inner_html(&self.minimap_quests());
        }
    }

    /// The journal changed: its windows and the list on the screen are drawn anew.
    pub(in crate::web::app) fn quests_changed(&self) {
        self.redraw(&[&wnd::QUESTS, &wnd::QUEST_PANE, &wnd::QUEST_NOTICE, &wnd::QUEST_LIST]);
    }
}
