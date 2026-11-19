//! The quest journal (the client's `CUIRenewQuestUI`), the button that tells of a new quest
//! (`UINotifyQuestWnd`), the list of quests on the screen (`UIQuestDisplay`) and the NPCs' marks on the minimap.
//! The pane of one quest is in `quests/pane.rs`.
//!
//! The client gives its texts' colours blue first (`CUIStaticText::SetColor` hands the number to
//! `DrawTextAt` as it is): 0x955322 is written here as #225395.

use super::*;
use crate::press::quests::{Press, Row};
use crate::quests::{self as rules, tab};

mod pane;

/// The journal's size (`CUIRenewQuestUI::Initialize`).
pub(in crate::web) const JOURNAL: (i32, i32) = (381, 466);
/// The size of the pane of one quest (`CUIOngoingQuestInfo::Initialize`).
pub(in crate::web) const PANE: (i32, i32) = (342, 412);
/// A row's title, the title of the row picked, and the other words of the journal.
const TITLE: &str = "#225395";
const PICKED: &str = "#2e7be4";
const WORDS: &str = "#2e3038";

/// Where the pane of one quest stands: at the journal's right side, where the journal opens
/// (`CUIOngoingQuestInfo::OnInit`). (The client puts it by the journal wherever that was dragged to.)
pub(in crate::web) fn pane_place(view: (f64, f64)) -> String {
    let (left, top) = (((view.0 as i32 - JOURNAL.0) / 2).max(0) + JOURNAL.0, ((view.1 as i32 - JOURNAL.1) / 2).max(0));
    format!("left:{left}px;top:{top}px;width:{}px;height:{}px", PANE.0, PANE.1)
}

/// Where the button that tells of a new quest stands: 43 by 43 in the lower right corner of the screen
/// (`WID_NOTIFYQUESTWND`).
pub(in crate::web) fn notice_place(view: (f64, f64)) -> String {
    format!(r#"class="a" style="{};z-index:10;pointer-events:auto""#, at(view.0 as i32 - 43, view.1 as i32 - 43, 43, 43))
}

/// Where the list of quests on the screen is: 250 by 400, 320 from the right edge and 150 down
/// (`WID_QUESTDISPLAY`).
pub(in crate::web) fn list_place(view: (f64, f64)) -> String {
    format!(r#"class="a" style="{};pointer-events:none""#, at(view.0 as i32 - 320, 150, 250, 400))
}

/// A row of the journal: what it opens, what is written in it, and what its two boxes show (none for a box
/// the row does not have).
struct Line {
    row: Row,
    title: String,
    summary: String,
    icon: String,
    /// The box in its corner: whether the quest is in the list on the screen.
    listed: Option<bool>,
    /// Its lock: open for a quest switched on, or for a recommended one not put away.
    lock: Option<bool>,
}

/// A box that is one of two pictures of the journal's folder, pressed to switch it (the client's `CUICheckBox`).
fn check_box(x: i32, y: i32, (width, height): (i32, i32), picture_name: &str, says: &str, press: Press) -> String {
    format!(
        r#"<div class="a" style="{};background:url('{}') no-repeat" title="{}" {}></div>"#,
        at(x, y, width, height),
        picture(&format!("renew_questui/{picture_name}.bmp")),
        escape(says),
        pressed(press)
    )
}

/// A button of the journal that stays down while it is the one chosen (the client's `CUITabButton`): its
/// `_on` picture then, its `_off` one otherwise.
fn tab_button(x: i32, y: i32, (width, height): (i32, i32), name: &str, chosen: bool, says: &str, press: Press) -> String {
    check_box(x, y, (width, height), &format!("{name}_{}", if chosen { "on" } else { "off" }), says, press)
}

impl App {
    /// What the journal's rows are under its open tab, after the filter and the search
    /// (`CUIRenewQuestUI::UpdateQuestLists`): the server's quests in the order of their numbers; of the
    /// recommended first those not put away, then those that were.
    fn quest_lines(&self) -> Vec<Line> {
        let filters = data::quest_filters();
        let passes = |icon: &str, title: &str, summary: &str| rules::in_filter(self.quests.filter, icon, &filters) && rules::found(&self.quests.search, title, summary);
        if self.quests.tab == tab::RECOMMENDED {
            let mut lines: Vec<Line> = data::recommended_quests()
                .into_iter()
                .filter(|(_, about)| passes(&about.icon, &about.title, &about.summary))
                .map(|(number, about)| Line { row: Row { number, recommended: true }, title: about.title, summary: about.summary, icon: about.icon, listed: None, lock: Some(!self.quests.passed.contains(&number)) })
                .collect();
            lines.sort_by_key(|line| line.lock != Some(true));
            return lines;
        }
        self.quests
            .log
            .values()
            .filter_map(|quest| {
                let about = data::quest_text(quest.id).unwrap_or_default();
                let title = self.quest_title(quest.id);
                (rules::tab_of(about.waits, quest.active) == self.quests.tab && passes(&about.icon, &title, &about.summary)).then(|| Line {
                    row: Row { number: quest.id, recommended: false },
                    title,
                    summary: about.summary,
                    icon: about.icon,
                    // A quest that only waits has neither box (`CUIQuestList::BindItem`).
                    listed: (!about.waits).then(|| !self.quests.unlisted.contains(&quest.id)),
                    lock: (!about.waits).then_some(quest.active),
                })
            })
            .collect()
    }

    /// A row (`CUIQuestItem`, 330 by 46): its ground, which is another picture under the pointer and when
    /// pressed, the picture of the quest's kind, its title in bold over its summary, and at the right the box
    /// and the lock.
    fn quest_row(&self, top: i32, line: &Line) -> String {
        let fitted = "width:260px;overflow:hidden;text-overflow:ellipsis;pointer-events:none";
        let mut row = pictured(0, top, (330, 46), ["renew_questui/bg_questlist.bmp", "renew_questui/bg_questlist_check.bmp", "renew_questui/bg_questlist_press.bmp"], Press::Pick(line.row), "");
        row.push_str(&format!(
            r#"<img class="a" alt="" draggable="false" style="left:8px;top:{}px" src="{}" title="{}" onerror="this.remove()">"#,
            top + 7,
            picture(&format!("renew_questui/{}", rules::icon(&line.icon))),
            escape(&data::quest_tooltip(&line.icon))
        ));
        let picked = self.quests.pane == Some((line.row.number, line.row.recommended));
        row.push_str(&format!(r#"<div class="a t" style="left:42px;top:{}px;color:{};{};{fitted}">{}</div>"#, top + 6, if picked { PICKED } else { TITLE }, BOLD.style(), escape(&line.title)));
        row.push_str(&format!(r#"<div class="a t" style="left:42px;top:{}px;color:{WORDS};{fitted}">{}</div>"#, top + 24, escape(&rules::shown(&line.summary))));
        if let Some(listed) = line.listed {
            // `MSI_PASSWORD` in the decompiled client's names: line 3409, "Display small quest window".
            row.push_str(&check_box(312, top + 3, (14, 14), if listed { "bt_check_on" } else { "bt_check_off" }, &Self::words(3409, "Display small quest window"), Press::Listed(line.row.number)));
        }
        if let Some(open) = line.lock {
            // Lines 3407 ("Show/Hide") for a quest of the server and 3408 ("Hide") for a recommended one.
            let says = if line.row.recommended { Self::words(3408, "Hide") } else { Self::words(3407, "Show/Hide") };
            row.push_str(&check_box(306, top + 22, (20, 17), if open { "bt_lock_open" } else { "bt_lock" }, &says, Press::Lock(line.row)));
        }
        row
    }

    /// The journal, 381 by 466 (`CUIRenewQuestUI::OnCreate`): its four tabs down the left side; right of them
    /// the title bar and the ground; the rows, 46 apart from 7 down, in a list 343 by 394; under it the six
    /// filters, the search with its button, the box that shows the list on the screen, and Close.
    /// The list scrolls with the bar of the quest windows (`.viewer` in index.html: the client's 7-wide bar in
    /// its colours, drawn by the page). (Enter in the search does not search: its button does.)
    pub(in crate::web) fn quest_window(&self) -> String {
        let mut window = format!(r#"<div class="a" style="{}">{}</div>"#, at(29, 0, 352, 17), title_bar(352, &Self::words(3400, "Quest information")));
        window.push_str(&base_button(32, 3));
        window.push_str(&art("renew_questui/bg_quest.bmp", 29, 17, 352, 449));
        for (place, name) in ["tab_process", "tab_recommended", "tab_hold", "tab_wait"].into_iter().enumerate() {
            window.push_str(&tab_button(0, 17 + place as i32 * 101, (30, 100), name, self.quests.tab == place, "", Press::Tab(place)));
        }
        let lines = self.quest_lines();
        let rows: String = lines.iter().enumerate().map(|(place, line)| self.quest_row(7 + place as i32 * 46, line)).collect();
        window.push_str(&format!(r#"<div class="a viewer" style="{}"><div style="position:relative;height:{}px">{rows}</div></div>"#, at(38, 17, 343, 394), 7 + lines.len() * 46));
        window.push_str(&art("renew_questui/bg_search.bmp", 29, 411, 352, 28));
        // The filters, by the client's numbers for them, in the order they stand in.
        for (place, (number, name)) in [(0, "btn_all"), (2, "btn_normal"), (3, "btn_sub"), (4, "btn_daily"), (5, "btn_event"), (1, "btn_etc")].into_iter().enumerate() {
            window.push_str(&tab_button(35 + place as i32 * 31, 414, (26, 22), name, self.quests.filter == number, &data::quest_filter_name(number), Press::Filter(number)));
        }
        window.push_str(&field("quest-search", (223, 417, 119, 16), &self.quests.search, 20, "border:1px solid #c6c6c6;background:transparent;box-sizing:border-box"));
        window.push_str(&pictured_by(Convention::NormalOverPress, 342, 416, (23, 18), "renew_questui/btn_search", Press::Search, ""));
        window.push_str(&check_box(38, 448, (10, 10), if self.quests.display_off { "checkbox_off" } else { "checkbox_on" }, "", Press::Display));
        window.push_str(&text(52, 447, BLACK, &Self::words(2019, "Show Quest")));
        window.push_str(&picture_press(327, 442, "basic_interface/btn_close", Press::Window));
        window
    }

    /// The button that tells of a new quest (`UINotifyQuestWnd`, 43 by 43 in the screen's lower right corner):
    /// a press opens the journal.
    pub(in crate::web) fn quest_notice(&self) -> String {
        pictured(0, 0, (43, 43), ["basic_interface/quest_off.bmp", "basic_interface/quest_off.bmp", "basic_interface/quest_on.bmp"], Press::Notice, "")
    }

    /// The list of quests on the screen (`UIQuestDisplay::Paint`, 250 wide, 320 from the screen's right edge
    /// and 150 down): of the quests switched on and not left out of it, the first five; each its title in
    /// yellow, under it its summary cut to thirty letters, then its hunts with their counts, in white over a
    /// black copy. A quest with one hunt takes 50, any other 40, and one with several hunts as much again.
    /// (The client's menu on a quest of the list, under the right button, is not made.)
    pub(in crate::web) fn quest_list(&self) -> String {
        let shadowed = |x: i32, y: i32, colour: &str, words: &str| format!(r#"<div class="a t" style="left:{x}px;top:{y}px;color:{colour};text-shadow:1px 1px 0 #000">{}</div>"#, escape(words));
        let complete = Self::words(2030, "Complete");
        let (mut top, mut shown, mut list) = (0, 0, String::new());
        for quest in self.quests.log.values().filter(|quest| quest.active && !self.quests.unlisted.contains(&quest.id)) {
            let about = data::quest_text(quest.id);
            // One that only waits is never in the list (`AddOngoingQuest`).
            if about.as_ref().is_some_and(|about| about.waits) || shown >= 5 {
                continue;
            }
            shown += 1;
            let title = about.as_ref().map(|about| about.title.clone()).filter(|title| !title.is_empty()).unwrap_or_else(|| format!("Unknown Quest, ID = {}", quest.id));
            list.push_str(&shadowed(0, top, "#e1d600", &title));
            let summary = about.filter(|about| !about.lines.is_empty()).map(|about| crate::markup::uncoloured(&rules::shown(&about.summary))).unwrap_or_default();
            if !summary.is_empty() {
                let cut = if summary.chars().count() > 30 { format!("{}...", summary.chars().take(30).collect::<String>()) } else { summary };
                list.push_str(&shadowed(0, top + 15, "#fff", &cut));
            }
            for (place, hunt) in quest.hunts.iter().enumerate().filter(|(_, hunt)| hunt.goal > 0) {
                list.push_str(&shadowed(10, top + 28 + place as i32 * 13, "#fff", &rules::hunt_text(hunt, &hunt.name, &complete)));
            }
            top += match quest.hunts.len() {
                1 => 50,
                2 | 3 => 80,
                _ => 40,
            };
        }
        list
    }

    /// The NPCs' marks on the minimap (`UM_MINIMAP_ADD_QUEST_MARK`): for each NPC of this map whose mark has
    /// a colour, the picture of that mark in that colour, its middle on the NPC's cell. (In the client a press
    /// on one starts the navigation there; this client has no navigation yet.)
    pub(in crate::web) fn minimap_quests(&self) -> String {
        let Some(game) = self.game.as_ref().filter(|_| self.quests.marks.0 == crate::web::world::visit()) else { return String::new() };
        self.quests
            .marks
            .1
            .values()
            .filter_map(|mark| {
                let file = rules::minimap_picture(mark.effect, mark.colour)?;
                let (x, y) = crate::web::view::minimap_spot(game, self.minimap_zoom, (f32::from(mark.cell.0), f32::from(mark.cell.1)))?;
                Some(format!(r#"<img class="mark" src="{}" alt="" draggable="false" style="left:{x:.2}%;top:{y:.2}%" onerror="this.remove()">"#, picture(&file)))
            })
            .collect()
    }
}
