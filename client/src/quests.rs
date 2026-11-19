//! The quest journal's rules that need no page: under which tab a quest lies, which quests a filter and a search
//! leave, how a hunt is written, what a quest's text shows of its tags, and which picture the mark over an NPC
//! is.

use crate::markup::{Piece, pieces};
use crate::net::quests::Hunt;

/// The journal's tabs, down its left side, in the client's order (`CUIRenewQuestUI::OnCreate`: process,
/// recommended, hold, wait).
pub mod tab {
    /// The quests under way.
    pub const PROCESS: usize = 0;
    /// What the client itself recommends: not quests of the server.
    pub const RECOMMENDED: usize = 1;
    /// The quests switched off.
    pub const HOLD: usize = 2;
    /// The quests that only wait for a time to pass (the tables' `CoolTimeQuest`).
    pub const WAIT: usize = 3;
}

/// The tab a quest of the server lies under (`CUIRenewQuestUI::UpdateQuestLists`): one that waits for its time
/// under the last whether switched on or off, the others by that.
pub fn tab_of(waits: bool, active: bool) -> usize {
    match (waits, active) {
        (true, _) => tab::WAIT,
        (false, true) => tab::PROCESS,
        (false, false) => tab::HOLD,
    }
}

/// The picture of a quest's kind in its row, in the interface's `renew_questui` folder: the table's own, or
/// the plain one (`CUIQuestList::BindItem`).
pub fn icon(named: &str) -> &str {
    if named.is_empty() { "ico_nq.bmp" } else { named }
}

/// Whether a quest with that icon passes a filter of the journal's lower row (`CRenewQuestMgr::IsInFilter`).
/// `filters` are the client's (`QuestClassificationInfo.lub`): each its number and the icons it takes. Filter 0
/// is everything; filter 1 is what no filter takes; any other takes its own icons.
pub fn in_filter(filter: u32, icon: &str, filters: &[(u32, Vec<String>)]) -> bool {
    let takes = |icons: &[String]| icons.iter().any(|taken| taken == icon);
    match filter {
        0 => true,
        1 => !filters.iter().any(|(_, icons)| takes(icons)),
        _ => filters.iter().find(|(number, _)| *number == filter).is_some_and(|(_, icons)| takes(icons)),
    }
}

/// Whether a quest is left by what was typed into the journal's search (`CUIRenewQuestUI::MatchesFilter`):
/// nothing typed leaves all, otherwise the words must be in its summary or its title.
pub fn found(search: &str, title: &str, summary: &str) -> bool {
    search.is_empty() || summary.contains(search) || title.contains(search)
}

/// How the client comes by the name of what a hunt wants (`CGameMode::ReceiveAddQuest`).
#[derive(Debug, PartialEq)]
pub enum Named {
    /// A monster: the name the server sent.
    Sent,
    /// A line of the client's message table.
    Message(usize),
    /// A map: the server sent its file's name, and the client shows what it calls the map.
    Map,
    /// Anything else: what the client calls the job of that number.
    Job,
}

/// Where the name of a hunt's monster comes from, by the monster's number.
pub fn named(mob: u32) -> Named {
    match mob {
        1001..=3998 | 20021..=31999 => Named::Sent,
        // `MSI_INVENTORY_EXTEND` and `MSI_INVENTORY_EXTEND_FAILED`, as the decompiled client names lines 3558
        // and 3559 (in this client's table: "KO'ed" and "PvP").
        0x1099 => Named::Message(3558),
        0x109a => Named::Message(3559),
        0x287b => Named::Map,
        _ => Named::Job,
    }
}

/// The levels of the monsters a hunt counts, as the client writes them after the name (`GetHuntText`): none
/// when neither bound is set.
fn levels((least, most): (i16, i16)) -> String {
    match (least, most) {
        (0, most) if most > 0 => format!(" Lv.{most}▼"),
        (least, 0) if least > 0 => format!(" Lv.{least}▲"),
        (least, most) if least > 0 && most > 0 => format!(" Lv.{least}~Lv.{most}"),
        _ => String::new(),
    }
}

/// A hunt as the quest's pane lists it (`RenewQuestOngoingInfo::GetHuntText`): the monster, the levels that
/// count, and how many of how many are hunted, or the client's word for all of them (`complete`, its line
/// 2030). `name` is the monster's name as `named` says to take it. (For a hunt with no one monster, number
/// 31999, the client writes the kind of monster from its navigation texts; that is not done here: the name
/// the server sent stands in.)
pub fn hunt_text(hunt: &Hunt, name: &str, complete: &str) -> String {
    let what = format!("{name}{}", levels(hunt.levels));
    match hunt.goal {
        goal if goal <= 0 => what,
        goal if hunt.killed == goal => format!("{what} ( {complete} )"),
        goal => format!("{what} ( {} / {goal} )", hunt.killed),
    }
}

/// The line the chat gets as a hunt's count changes (`ReceiveQuestHuntUpdate`), from the client's line 1610,
/// "[%s] quest - defeating [%s] progress (%d/%d).": the quest's title, the monster, how many of how many.
pub fn progress_line(line: &str, title: &str, monster: &str, killed: i16, goal: i16) -> String {
    line.replacen("%s", title, 1).replacen("%s", monster, 1).replacen("%d", &killed.to_string(), 1).replacen("%d", &goal.to_string(), 1)
}

/// What a quest's text shows of the links written into it: `<NAVI>words<INFO>where</INFO></NAVI>`, and the
/// like for an item or a tip, show their words; where they lead is left out, as are the tags themselves
/// (the client's `DisplayNavigationTextTag`, `DisplayItemTextTag`, `DisplayTipBoxTextTag`, `RemoveTextLinkInfo`).
/// Colours (`^RRGGBB`) stay as they are.
pub fn shown(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    // Whether what is read stands between `<INFO>` and its `</INFO>`: where a link leads.
    let mut leads = false;
    for piece in pieces(text, &["INFO", "NAVI", "ITEM", "TIPBOX", "URL"]) {
        match piece {
            Piece::Tag { name: "INFO", closes } if leads || !closes => leads = !closes,
            _ if leads => {}
            // An `</INFO>` that closes nothing is text.
            Piece::Tag { name: "INFO", .. } => plain.push_str("</INFO>"),
            Piece::Tag { .. } => {}
            Piece::Colour(code) => {
                plain.push('^');
                plain.push_str(code);
            }
            Piece::Text(words) => plain.push_str(words),
        }
    }
    plain
}

/// A text without the colours written into it (`^RRGGBB`), as the list on the screen shows a summary (the
/// client's `StripColorCodes`): the marks of a text are told apart in `crate::markup`.
pub use crate::markup::uncoloured;

/// The sprite of the marks over NPCs, under `data/sprite`.
pub const MARKS: &str = "이팩트/emotion";

/// Which action of that sprite a mark is, by the number the server gives it (rAthena's `QTYPE_*`). The client
/// (`CNpc::UpdateMinimapMarker`) makes an effect of kind 36 and the number for the first six, 54 and the number
/// from there, and `CGameMode::SpawnMsgEffect` gives those kinds actions 80 to 85 and 93 to 97. None for a
/// number past them, and for the one that takes the mark away.
pub fn mark_action(effect: u16) -> Option<usize> {
    match effect {
        0..=5 => Some(80 + usize::from(effect)),
        6..=10 => Some(93 + usize::from(effect) - 6),
        _ => None,
    }
}

/// The picture of an NPC's mark on the minimap, in the interface's `minimap` folder
/// (`UM_MINIMAP_ADD_QUEST_MARK`): none for a mark with no colour, which the minimap does not show
/// (`ReceiveMinimapMarker`).
pub fn minimap_picture(effect: u16, colour: u16) -> Option<String> {
    (colour != 0 && effect != crate::net::quests::NO_MARK).then(|| format!("minimap/quest_{effect}_{colour}.bmp"))
}

#[cfg(test)]
mod tests;
