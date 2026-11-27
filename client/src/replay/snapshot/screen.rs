//! The sections of a replay file about what stands on the screen. The status icons (18,
//! `CGameMode::SaveActiveSkillEffects`, read at the top of `CGameMode::Initialize`, `GameMode.cpp`), the
//! windows open (19, `UIWindowMgr::SaveReplayWindows`, `LoadReplayWindows`, `UI/Window/UIWindowMgr.cpp`), the
//! one setting kept (20, `SaveReplaySettings`), the section kept for later (21, `SaveReplayReserved`), whether
//! the character is on a quest (23, `CGameMode::SaveQuestState`) and the navigation (24,
//! `CNavigationManager::WriteReplayState`, `LoadReplayState`, `Contents/Navigation/NavigationRoute.cpp`).

use super::*;

/// A status the character is in (`ActiveSkillEffect`, 28 bytes).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatusIcon {
    /// The status (`EFST_`).
    pub id: u32,
    /// The milliseconds left when the recording began (the client takes its clock off the tick it ends at);
    /// 0 and 999999 are written as they are, for one that does not end.
    pub expires: u32,
    pub values: [u32; 3],
    /// How long it lasts in all, and how much of that is left.
    pub total: u32,
    pub remaining: u32,
}

/// How many bytes an icon's chunk holds.
pub const ICON_BYTES: usize = 28;

/// Section 18: a chunk 19011 an icon between 19000 and 19001.
pub fn icons(icons: &[StatusIcon]) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 19000, &[]);
    for icon in icons {
        let [first, second, third] = icon.values;
        let numbers = [icon.id, icon.expires, first, second, third, icon.total, icon.remaining];
        tagged(&mut section, 19011, &numbers.iter().flat_map(|number| number.to_le_bytes()).collect::<Vec<u8>>());
    }
    tagged(&mut section, 19001, &[]);
    section
}

/// Section 18 read back.
pub fn icons_read(section: &[u8]) -> Option<Vec<StatusIcon>> {
    let chunks = opened(section, 19000)?.into_iter().filter(|(tag, _)| *tag == 19011);
    let icon = |bytes: &[u8]| {
        let number = |place: usize| bytes.get(place * 4..).map_or(0, u32_of);
        StatusIcon { id: number(0), expires: number(1), values: [number(2), number(3), number(4)], total: number(5), remaining: number(6) }
    };
    Some(chunks.map(|(_, bytes)| icon(bytes)).collect())
}

/// The tag of a window of section 19: this, and the client's number of the window.
const WINDOW: u16 = 20100;

/// Section 19: between 20000 and 20001 a record a window, its tag 20100 and the window's number, holding
/// what the window kept of itself (`GetReplayData`; nothing for most: such a window is only opened again).
/// The client writes the windows whose number is 10040 at most and that its table marks for it
/// (`policy[id].actions[7]`); a number that would not fit the tag is left out here.
pub fn windows(windows: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 20000, &[]);
    for (window, kept) in windows {
        if let Some(tag) = u16::try_from(*window).ok().and_then(|window| window.checked_add(WINDOW)) {
            tagged(&mut section, tag, kept);
        }
    }
    tagged(&mut section, 20001, &[]);
    section
}

/// Section 19 read back: every window written, also those the client's own reader does not know again (it
/// takes the tags 20100 to 20499 alone, and so loses the exchange's window, 10011, which its writer writes).
pub fn windows_read(section: &[u8]) -> Option<Vec<(u32, Vec<u8>)>> {
    let records = opened(section, 20000)?.into_iter().filter(|(tag, _)| *tag >= WINDOW);
    Some(records.map(|(tag, kept)| (u32::from(tag - WINDOW), kept.to_vec())).collect())
}

/// Whether the client's own reader opens a window of that number again from section 19 (`LoadReplayWindows`).
pub fn window_known(window: u32) -> bool {
    window < 400
}

/// A section that is one number of four bytes between a record that opens it and one that closes it.
fn one(begin: u16, tag: u16, value: i32) -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, begin, &[]);
    tagged(&mut section, tag, &value.to_le_bytes());
    tagged(&mut section, begin + 1, &[]);
    section
}

fn one_read(section: &[u8], begin: u16, tag: u16) -> Option<i32> {
    Some(opened(section, begin)?.into_iter().find(|(of, _)| *of == tag).map_or(0, |(_, bytes)| i32_of(bytes)))
}

/// Section 20: whether all of a pile is dragged at once (`dragAllItems`, 25010) between 25000 and 25001.
pub fn settings(drag_all: i32) -> Vec<u8> {
    one(25000, 25010, drag_all)
}

pub fn settings_read(section: &[u8]) -> Option<i32> {
    one_read(section, 25000, 25010)
}

/// Section 21: nothing between 26000 and 26001.
pub fn reserved() -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 26000, &[]);
    tagged(&mut section, 26001, &[]);
    section
}

/// Section 23: whether the character is on a quest (`isOnQuest`, 14011) between 14000 and 14001.
pub fn on_quest(on_quest: i32) -> Vec<u8> {
    one(14000, 14011, on_quest)
}

pub fn on_quest_read(section: &[u8]) -> Option<i32> {
    one_read(section, 14000, 14011)
}

/// The records of the navigation the client always writes, each its tag and its size, in its order; those of
/// a route, of a search and of what is sought (8022, 8015, 8006 to 8008, 8021) come only with one.
const NAVIGATION: [(u16, usize); 16] = [
    (8002, 1),
    (8003, 1),
    (8004, 1),
    (8005, 4),
    (8009, 4),
    (8010, 4),
    (8011, 1),
    (8013, 4),
    (8014, 4),
    (8016, 4),
    (8017, 1),
    (8018, 4),
    (8019, 1),
    (8020, 4),
    (8023, 1),
    (8012, 4),
];

/// Section 24: a navigation that seeks nothing and shows no route, 151 bytes. Every member is written as
/// zeros: what a navigation the player never touched holds in them was not found in the client.
pub fn navigation() -> Vec<u8> {
    let mut section = Vec::new();
    tagged(&mut section, 8000, &[]);
    for (tag, size) in NAVIGATION {
        tagged(&mut section, tag, &vec![0; size]);
    }
    tagged(&mut section, 8001, &[]);
    section
}

/// Section 24 looked at: whether it opens as the client's does and its records stand whole to its end. This
/// client keeps nothing of it.
pub fn navigation_read(section: &[u8]) -> bool {
    let records = tags(section);
    let whole = records.iter().map(|(_, bytes)| 6 + bytes.len()).sum::<usize>() == section.len();
    whole && records.first().map(|(tag, _)| *tag) == Some(8000)
}
