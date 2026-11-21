//! The Hotkey Guide as rules (the client's `UIHotkeyGuideWnd`): the keys of the keyboard it shows, each with
//! what is written on it and where it lies in the window's picture, the eighteen things it lists with their
//! keys, and how the keys of one are written beside it.
//!
//! The table of keys is compiled into the client (`UIHotkeyGuideWnd::OnCreate`, `m_labels`) and written out
//! here as it stands there.

use super::{Chord, interface};

/// The window's size (`UIWindowMgr::EnsureWindow`, `WID_HOTKEYGUIDEWND`), which is its picture's
/// (`hotkey_guide\bg_shortcuts.bmp`).
pub const SIZE: (i32, i32) = (934, 498);

/// How what is written on a key lies across it (`UIHotkeyGuideWnd::DrawKeyLabel`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    /// From the key's left edge, `padding` in (the client's 0).
    Left,
    /// Ending `padding` short of the key's right edge (2).
    Right,
    /// In the middle of the key (6).
    Middle,
}

/// A key of the picture: where it lies in the window and how large it is, and how its words lie on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cap {
    pub x: i32,
    pub y: i32,
    pub align: Align,
    pub width: i32,
    pub padding: i32,
    pub height: i32,
}

impl Cap {
    /// Where the words written on the key begin, down the window: the middle of the key's height for a text
    /// 12 high (`DrawKeyLabel`).
    pub fn words_top(&self) -> i32 {
        self.y + (self.height - 12) / 2
    }
}

/// A key as the client's table has it: the place, the way its words lie (0, 2 or 6), its width, the padding
/// and its height.
const fn cap(x: i32, y: i32, align: u8, width: i32, padding: i32, height: i32) -> Cap {
    let align = match align {
        0 => Align::Left,
        2 => Align::Right,
        _ => Align::Middle,
    };
    Cap { x, y, align, width, padding, height }
}

/// The key whose words are a line of the client's message table (`WORDS_OF_25`), not written in the code.
pub const KEY_25: u16 = 25;
pub const WORDS_OF_25: usize = 3147;

/// Every key of the picture: Windows' number of the key, what is written on it, where it lies, and where it
/// lies a second time for the two that are on the keyboard twice (Enter, Shift).
pub const KEYS: [(u16, &str, Cap, Option<Cap>); 99] = [
    (8, "←", cap(528, 93, 0, 75, 15, 36), None),
    (9, "Tab", cap(21, 132, 0, 55, 15, 36), None),
    (13, "Enter", cap(528, 171, 2, 75, 20, 36), Some(cap(876, 210, 6, 36, 0, 75))),
    (16, "Shift", cap(21, 210, 0, 94, 15, 36), Some(cap(508, 210, 2, 95, 20, 36))),
    (17, "Ctrl", cap(21, 249, 0, 55, 15, 36), None),
    (18, "Alt", cap(118, 249, 6, 49, 0, 36), None),
    (19, "Pause", cap(702, 46, 6, 36, 0, 36), None),
    (20, "Caps", cap(21, 171, 0, 75, 15, 36), None),
    (25, "", cap(547, 249, 6, 56, 0, 36), None),
    (27, "Esc", cap(21, 46, 6, 36, 0, 36), None),
    (32, "Space", cap(170, 249, 6, 246, 0, 36), None),
    (33, "PgUp", cap(702, 93, 6, 36, 0, 36), None),
    (34, "PgDn", cap(702, 132, 6, 36, 0, 36), None),
    (35, "End", cap(663, 132, 6, 36, 0, 36), None),
    (36, "Home", cap(663, 93, 6, 36, 0, 36), None),
    (37, "←", cap(624, 249, 6, 36, 0, 36), None),
    (38, "↑", cap(663, 210, 6, 36, 0, 36), None),
    (39, "→", cap(702, 249, 6, 36, 0, 36), None),
    (40, "↓", cap(663, 249, 6, 36, 0, 36), None),
    (44, "PrtSc", cap(624, 46, 6, 36, 0, 36), None),
    (45, "Insert", cap(624, 93, 6, 36, 0, 36), None),
    (46, "Delete", cap(624, 132, 6, 36, 0, 36), None),
    (48, "0", cap(411, 93, 6, 36, 0, 36), None),
    (49, "1", cap(60, 93, 6, 36, 0, 36), None),
    (50, "2", cap(99, 93, 6, 36, 0, 36), None),
    (51, "3", cap(138, 93, 6, 36, 0, 36), None),
    (52, "4", cap(177, 93, 6, 36, 0, 36), None),
    (53, "5", cap(216, 93, 6, 36, 0, 36), None),
    (54, "6", cap(255, 93, 6, 36, 0, 36), None),
    (55, "7", cap(294, 93, 6, 36, 0, 36), None),
    (56, "8", cap(333, 93, 6, 36, 0, 36), None),
    (57, "9", cap(372, 93, 6, 36, 0, 36), None),
    (65, "A", cap(99, 171, 6, 36, 0, 36), None),
    (66, "B", cap(274, 210, 6, 36, 0, 36), None),
    (67, "C", cap(196, 210, 6, 36, 0, 36), None),
    (68, "D", cap(177, 171, 6, 36, 0, 36), None),
    (69, "E", cap(157, 132, 6, 36, 0, 36), None),
    (70, "F", cap(216, 171, 6, 36, 0, 36), None),
    (71, "G", cap(255, 171, 6, 36, 0, 36), None),
    (72, "H", cap(294, 171, 6, 36, 0, 36), None),
    (73, "I", cap(352, 132, 6, 36, 0, 36), None),
    (74, "J", cap(333, 171, 6, 36, 0, 36), None),
    (75, "K", cap(372, 171, 6, 36, 0, 36), None),
    (76, "L", cap(411, 171, 6, 36, 0, 36), None),
    (77, "M", cap(352, 210, 6, 36, 0, 36), None),
    (78, "N", cap(313, 210, 6, 36, 0, 36), None),
    (79, "O", cap(391, 132, 6, 36, 0, 36), None),
    (80, "P", cap(430, 132, 6, 36, 0, 36), None),
    (81, "Q", cap(79, 132, 6, 36, 0, 36), None),
    (82, "R", cap(196, 132, 6, 36, 0, 36), None),
    (83, "S", cap(138, 171, 6, 36, 0, 36), None),
    (84, "T", cap(235, 132, 6, 36, 0, 36), None),
    (85, "U", cap(313, 132, 6, 36, 0, 36), None),
    (86, "V", cap(235, 210, 6, 36, 0, 36), None),
    (87, "W", cap(118, 132, 6, 36, 0, 36), None),
    (88, "X", cap(157, 210, 6, 36, 0, 36), None),
    (89, "Y", cap(274, 132, 6, 36, 0, 36), None),
    (90, "Z", cap(118, 210, 6, 36, 0, 36), None),
    (93, "▤", cap(508, 249, 6, 36, 0, 36), None),
    (96, "0", cap(759, 249, 6, 36, 0, 36), None),
    (97, "1", cap(759, 210, 6, 36, 0, 36), None),
    (98, "2", cap(798, 210, 6, 36, 0, 36), None),
    (99, "3", cap(837, 210, 6, 36, 0, 36), None),
    (100, "4", cap(759, 171, 6, 36, 0, 36), None),
    (101, "5", cap(798, 171, 6, 36, 0, 36), None),
    (102, "6", cap(837, 171, 6, 36, 0, 36), None),
    (103, "7", cap(759, 132, 6, 36, 0, 36), None),
    (104, "8", cap(798, 132, 6, 36, 0, 36), None),
    (105, "9", cap(837, 132, 6, 36, 0, 36), None),
    (106, "*", cap(837, 93, 6, 36, 0, 36), None),
    (107, "+", cap(876, 132, 6, 36, 0, 75), None),
    (109, "-", cap(876, 93, 6, 36, 0, 36), None),
    (110, "Del", cap(837, 249, 6, 36, 0, 36), None),
    (111, "/", cap(798, 93, 6, 36, 0, 36), None),
    (112, "F1", cap(83, 46, 6, 36, 0, 36), None),
    (113, "F2", cap(122, 46, 6, 36, 0, 36), None),
    (114, "F3", cap(161, 46, 6, 36, 0, 36), None),
    (115, "F4", cap(200, 46, 6, 36, 0, 36), None),
    (116, "F5", cap(267, 46, 6, 36, 0, 36), None),
    (117, "F6", cap(306, 46, 6, 36, 0, 36), None),
    (118, "F7", cap(345, 46, 6, 36, 0, 36), None),
    (119, "F8", cap(384, 46, 6, 36, 0, 36), None),
    (120, "F9", cap(450, 46, 6, 36, 0, 36), None),
    (121, "F10", cap(489, 46, 6, 36, 0, 36), None),
    (122, "F11", cap(528, 46, 6, 36, 0, 36), None),
    (123, "F12", cap(567, 46, 6, 36, 0, 36), None),
    (144, "Num", cap(759, 93, 6, 36, 0, 36), None),
    (145, "Scroll", cap(663, 46, 6, 36, 0, 36), None),
    (186, ";", cap(450, 171, 6, 36, 0, 36), None),
    (187, "=", cap(489, 93, 6, 36, 0, 36), None),
    (188, ",", cap(391, 210, 6, 36, 0, 36), None),
    (189, "-", cap(450, 93, 6, 36, 0, 36), None),
    (190, ".", cap(430, 210, 6, 36, 0, 36), None),
    (191, "/", cap(469, 210, 6, 36, 0, 36), None),
    (192, "`", cap(21, 93, 6, 36, 0, 36), None),
    (219, "[", cap(469, 132, 6, 36, 0, 36), None),
    (220, "\\", cap(547, 132, 0, 56, 11, 36), None),
    (221, "]", cap(508, 132, 6, 36, 0, 36), None),
    (222, "'", cap(489, 171, 6, 36, 0, 36), None),
];

/// What the guide lists, in its order, down three columns of six (`UIHotkeyGuideWnd::OnCreate`): the line of
/// the client's message table that names the thing, and the place of its keys in the table of the
/// interface's keys; none for the game's options, whose key is Escape whatever the tables say.
pub const LISTED: [(usize, Option<usize>); 18] = [
    (105, Some(interface::STATUS)),
    (104, Some(interface::EQUIPMENT)),
    (283, Some(interface::SKILLS)),
    (106, Some(interface::INVENTORY)),
    (103, Some(interface::PARTY)),
    (3134, Some(interface::GUILD)),
    (1317, Some(interface::QUESTS)),
    (2859, Some(interface::WORLD_MAP)),
    (2353, Some(42)),
    (265, None),
    (3135, Some(46)),
    (2350, Some(38)),
    (3136, Some(interface::MAIL)),
    (2646, Some(49)),
    (3225, Some(interface::CHAT_ROOM)),
    (3137, Some(37)),
    (574, Some(interface::HOTKEY_BAR)),
    (3138, Some(interface::SIT)),
];

/// Escape, the key of the game's options.
pub const ESCAPE: Chord = Chord { key: 27, with: 0 };

/// Where the button of the thing at that place of the list stands, 118 wide (its keys are written in the 100
/// that begin 130 right of it and 5 lower).
pub fn listed_place(place: usize) -> (i32, i32) {
    ((place / 6 * 246 + 26) as i32, (place % 6 * 30 + 310) as i32)
}

/// The key of that number in the table, if the picture has it.
pub fn key(code: u16) -> Option<&'static (u16, &'static str, Cap, Option<Cap>)> {
    KEYS.iter().find(|(known, ..)| *known == code)
}

/// The keys of a thing as the guide writes them beside it (`UIHotkeyGuideWnd::Paint`): what is written on
/// the key, after what is written on the key held with it and " + " when there is one; "Non" for a thing
/// with no key. `words` gives what is written on a key (nothing for one the picture lacks).
pub fn written(chord: Chord, words: impl Fn(u16) -> String) -> String {
    match (chord.key, chord.with) {
        (0, _) => "Non".to_owned(),
        (key, 0) => words(key),
        (key, with) => format!("{} + {}", words(with), words(key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(code: u16) -> String {
        key(code).map(|(_, words, ..)| (*words).to_owned()).unwrap_or_default()
    }

    #[test]
    fn the_keys_are_the_clients() {
        assert_eq!(key(27), Some(&(27, "Esc", cap(21, 46, 6, 36, 0, 36), None)));
        // Enter is on the keyboard twice: the tall one of the number pad is the second.
        let (_, _, main, pad) = key(13).unwrap();
        assert_eq!((main.align, main.width, pad.unwrap().height), (Align::Right, 75, 75));
        assert_eq!(key(32).unwrap().2.width, 246);
        assert_eq!(key(7), None);
        // No key is in the table twice, and every key lies inside the picture.
        for (place, (code, _, first, second)) in KEYS.iter().enumerate() {
            assert_eq!(KEYS.iter().position(|(other, ..)| other == code), Some(place));
            for cap in [Some(*first), *second].into_iter().flatten() {
                assert!(cap.x >= 0 && cap.x + cap.width <= SIZE.0 && cap.y >= 17 && cap.y + cap.height <= 300, "{code}");
            }
        }
    }

    #[test]
    fn words_stand_in_the_middle_of_a_keys_height() {
        assert_eq!(key(27).unwrap().2.words_top(), 58);
        assert_eq!(key(107).unwrap().2.words_top(), 132 + 31);
    }

    #[test]
    fn the_list_stands_in_three_columns_of_six() {
        assert_eq!((listed_place(0), listed_place(5), listed_place(6), listed_place(17)), ((26, 310), (26, 460), (272, 310), (518, 460)));
        assert_eq!(LISTED[0], (105, Some(47)));
        assert_eq!(LISTED[9], (265, None));
    }

    #[test]
    fn keys_are_written_as_the_guide_writes_them() {
        assert_eq!(written(Chord { key: 65, with: 18 }, words), "Alt + A");
        assert_eq!(written(Chord { key: 45, with: 0 }, words), "Insert");
        assert_eq!(written(ESCAPE, words), "Esc");
        assert_eq!(written(Chord::default(), words), "Non");
    }
}
