//! The client's keys: which key does what, how a key is written, and what differs on a keyboard that lacks
//! some of the keys.
//!
//! The client keeps four tables (`Lua Files\HotKey`): the hotkey bar, the interface, the macros and the second
//! hotkey bar. An entry is known by its table and its place in it, and holds a key and the key held with it,
//! both as Windows' key numbers (74 is J, 18 is Alt).

/// A key, and the key held with it (0 for none): Windows' numbers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chord {
    pub key: u16,
    pub with: u16,
}

pub const SHIFT: u16 = 16;
pub const CTRL: u16 = 17;
pub const ALT: u16 = 18;

/// The tables, in the client's numbering.
pub mod table {
    pub const BAR: usize = 0;
    pub const INTERFACE: usize = 1;
    pub const MACROS: usize = 2;
    pub const BAR_2: usize = 3;
}

/// The tables in the order of the window's tabs: Hotkey Bar, Hotkey Bar 2, Interface, Macros (the client's
/// `CUserHotkeyMgr::TabAt`).
pub const TABS: [usize; 4] = [table::BAR, table::BAR_2, table::INTERFACE, table::MACROS];

/// Places of the interface table (the client's `HK_*`), those this client acts on.
pub mod interface {
    pub const BASIC_INFO: usize = 0;
    pub const EQUIPMENT: usize = 1;
    pub const SKILLS: usize = 2;
    pub const INVENTORY: usize = 9;
    pub const SIT: usize = 12;
    pub const HOTKEY_BAR: usize = 17;
    pub const CHAT_SIZE: usize = 19;
    pub const PET: usize = 23;
    pub const STATUS: usize = 47;
}

/// The places of the interface table the window does not list (the client's `HiddenHotkeyIndex`).
pub const HIDDEN: [usize; 13] = [30, 33, 34, 36, 43, 45, 53, 58, 62, 64, 65, 66, 67];

/// Windows' number of a key, by the browser's name for where it is on the keyboard (`KeyboardEvent.code`).
pub fn key_of(code: &str) -> Option<u16> {
    if let Some(letter) = code.strip_prefix("Key").filter(|letter| letter.len() == 1) {
        return Some(u16::from(letter.as_bytes()[0]));
    }
    if let Some(digit) = code.strip_prefix("Digit").filter(|digit| digit.len() == 1) {
        return Some(u16::from(digit.as_bytes()[0]));
    }
    if let Some(number) = code.strip_prefix("Numpad").and_then(|digit| digit.parse::<u16>().ok()).filter(|number| *number < 10) {
        return Some(96 + number);
    }
    if let Some(number) = code.strip_prefix('F').and_then(|number| number.parse::<u16>().ok()).filter(|number| (1..=12).contains(number)) {
        return Some(111 + number);
    }
    Some(match code {
        "Backspace" => 8,
        "Tab" => 9,
        "Enter" | "NumpadEnter" => 13,
        "Pause" => 19,
        "CapsLock" => 20,
        "Escape" => 27,
        "Space" => 32,
        "PageUp" => 33,
        "PageDown" => 34,
        "End" => 35,
        "Home" => 36,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        "PrintScreen" => 44,
        "Insert" => 45,
        "Delete" => 46,
        "NumpadMultiply" => 106,
        "NumpadAdd" => 107,
        "NumpadSubtract" => 109,
        "NumpadDecimal" => 110,
        "NumpadDivide" => 111,
        "ScrollLock" => 145,
        "Semicolon" => 186,
        "Equal" => 187,
        "Comma" => 188,
        "Minus" => 189,
        "Period" => 190,
        "Slash" => 191,
        "Backquote" => 192,
        "BracketLeft" => 219,
        "Backslash" => 220,
        "BracketRight" => 221,
        "Quote" => 222,
        _ => return None,
    })
}

/// The key held with another, of those held: Alt before Ctrl before Shift, one only, as the client has it.
pub fn held(alt: bool, ctrl: bool, shift: bool) -> u16 {
    match (alt, ctrl, shift) {
        (true, ..) => ALT,
        (_, true, _) => CTRL,
        (_, _, true) => SHIFT,
        _ => 0,
    }
}

/// The keys a MacBook's keyboard does not have: Insert, Home, End, Page Up, Page Down, Print Screen, Scroll
/// Lock, Pause and the number pad.
pub fn missing_on_mac(key: u16) -> bool {
    matches!(key, 19 | 33..=36 | 44 | 45 | 96..=111 | 145)
}

/// What an entry is bound to on a Mac when the client's own key for it is one a MacBook lacks. These are this
/// client's choice (the Windows client has no Mac): Alt with a letter the client's own table leaves free.
pub fn mac_instead(which: usize, place: usize) -> Option<Chord> {
    match (which, place) {
        // Sit and stand: Insert.
        (table::INTERFACE, interface::SIT) => Some(Chord { key: u16::from(b'X'), with: ALT }),
        _ => None,
    }
}

/// How a chord is written (the client's `GetKeyDes`): the key's name, after the held key's and a plus. `name`
/// gives the client's name of a key. On a Mac, Alt is called Option, as its keyboard calls it.
pub fn written(chord: Chord, mac: bool, name: impl Fn(u16) -> Option<String>) -> String {
    if chord.key == 0 {
        return String::new();
    }
    let named = |key: u16| match (key, mac) {
        (ALT, true) => "Option".to_owned(),
        _ => name(key).unwrap_or_else(|| key.to_string()),
    };
    if chord.with == 0 { named(chord.key) } else { format!("{} + {}", named(chord.with), named(chord.key)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_the_keys() {
        assert_eq!((key_of("KeyJ"), key_of("Digit1"), key_of("F1"), key_of("F12"), key_of("Insert"), key_of("Backquote")), (Some(74), Some(49), Some(112), Some(123), Some(45), Some(192)));
        assert_eq!((key_of("F13"), key_of("MetaLeft")), (None, None));
        assert_eq!((held(true, true, false), held(false, true, true), held(false, false, false)), (ALT, CTRL, 0));
    }

    #[test]
    fn writes_them_as_the_client_does() {
        let name = |key: u16| match key {
            74 => Some("J".to_owned()),
            18 => Some("Alt".to_owned()),
            45 => Some("Ins".to_owned()),
            _ => None,
        };
        assert_eq!(written(Chord { key: 74, with: 18 }, false, name), "Alt + J");
        assert_eq!(written(Chord { key: 74, with: 18 }, true, name), "Option + J");
        assert_eq!(written(Chord { key: 45, with: 0 }, false, name), "Ins");
        assert_eq!(written(Chord::default(), false, name), "");
        assert!(missing_on_mac(45) && !missing_on_mac(74));
        assert_eq!(mac_instead(table::INTERFACE, interface::SIT), Some(Chord { key: 88, with: ALT }));
    }
}
