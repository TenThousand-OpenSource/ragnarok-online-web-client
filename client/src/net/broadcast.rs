//! The server's announcements: ZC_BROADCAST and its coloured form, as the client reads them
//! (`CGameMode::ReceiveTaggedBroadcast`, `ReceiveColorBroadcast`): which words are said, in which colour, and
//! how they are cut into the lines shown across the top of the screen (`UIBroadcastBalloon`).

use super::bytes::{name_at, word};

/// ZC_BROADCAST (0x009a): `u16 length, char words[]`.
pub const ZC_BROADCAST: u16 = 0x009a;
/// ZC_BROADCAST2 (0x01c3): `u16 length, u8 colour[4], i16 font, i16 size, i16 align, i16 y, char words[]`.
pub const ZC_BROADCAST2: u16 = 0x01c3;

/// How long an announcement stays at the top of the screen, how many stay there at once, the height of its
/// letters when the server names none, and how many letters a line of a plain one holds
/// (`BroadcastBalloonMessage`, `UIBroadcastBalloon::AddMessage`, `WrapPlainText(.., 0x48)`).
pub const SHOWN_FOR: f64 = 20_000.0;
pub const SHOWN_AT_ONCE: usize = 3;
pub const LETTERS: u16 = 12;
const LINE: usize = 72;

/// An announcement as it is said.
#[derive(Debug, PartialEq)]
pub struct Announcement {
    pub words: String,
    /// Red, green, blue.
    pub colour: [u8; 3],
    /// The kind of line it is in the chat (`crate::said`).
    pub kind: u8,
    /// Who sent it, for one that names its sender (the tag `micc`): the client drops it when the player has
    /// turned that character's whispers away.
    pub from: Option<String>,
    /// The lines across the top of the screen, and the height of their letters.
    pub lines: Vec<String>,
    pub letters: u16,
}

/// The words without what the server put after the speaker's name: all from a `#` (not the first letter) up
/// to the first colon, as both readers cut it.
fn untagged(text: &[u8]) -> Vec<u8> {
    let hash = text.iter().position(|byte| *byte == b'#').filter(|at| *at > 0);
    let colon = text.iter().position(|byte| *byte == b':').filter(|at| *at > 0);
    match (hash, colon) {
        (Some(hash), Some(colon)) => [&text[..hash], &text[colon..]].concat(),
        (Some(hash), None) => text[..hash].to_vec(),
        _ => text.to_vec(),
    }
}

/// Six letters of a colour written `RRGGBB`.
fn hex(letters: Option<&[u8]>) -> Option<[u8; 3]> {
    let value = u32::from_str_radix(std::str::from_utf8(letters?).ok()?, 16).ok()?;
    Some([(value >> 16) as u8, (value >> 8) as u8, value as u8])
}

/// The text of a field up to its first nought.
fn ended(text: &[u8]) -> &[u8] {
    &text[..text.iter().position(|byte| *byte == 0).unwrap_or(text.len())]
}

/// ZC_BROADCAST (`ReceiveTaggedBroadcast`): yellow, unless its first four letters say otherwise: `tool` and
/// six letters of a colour, `blue`, `micc` with the sender's name in 24 bytes and a colour, `ssss` for a
/// line of the system's.
pub fn plain(packet: &[u8]) -> Option<Announcement> {
    let whole = packet.get(4..)?;
    let text = ended(whole);
    let tag = text.get(..4).map(|tag| tag.to_ascii_lowercase()).unwrap_or_default();
    let yellow = [255, 255, 0];
    let (words, colour, kind, from) = match tag.as_slice() {
        b"tool" => (text.get(10..).unwrap_or_default().to_vec(), hex(text.get(4..10)).unwrap_or([0, 0, 0]), crate::said::ANNOUNCED, None),
        b"blue" => (text[4..].to_vec(), [0, 255, 255], crate::said::ANNOUNCED, None),
        b"micc" => {
            // The name is a field of 24 bytes with noughts after it: the rest is found past it.
            let from = name_at(whole, 4, 24);
            (ended(whole.get(0x22..).unwrap_or_default()).to_vec(), hex(whole.get(0x1c..0x22)).unwrap_or([0, 0, 0]), crate::said::GENERAL, Some(from))
        }
        b"ssss" => (text[4..].to_vec(), yellow, crate::said::SYSTEM, None),
        _ => (untagged(text), yellow, crate::said::ANNOUNCED, None),
    };
    let words = name_at(&words, 0, words.len());
    Some(Announcement { lines: wrapped(&words, LINE), words, colour, kind, from, letters: LETTERS })
}

/// ZC_BROADCAST2 (`ReceiveColorBroadcast`): the colour and the height of the letters are the server's, and
/// the words are one line. (A height of nought is taken for the usual 12: decided here, the client would
/// draw nothing.)
pub fn coloured(packet: &[u8]) -> Option<Announcement> {
    let text = untagged(ended(packet.get(16..)?));
    let words = name_at(&text, 0, text.len());
    let letters = word(packet, 10).filter(|size| (1..=64).contains(size)).unwrap_or(LETTERS);
    Some(Announcement { lines: vec![words.clone()], words, colour: [packet[6], packet[5], packet[4]], kind: crate::said::ANNOUNCED, from: None, letters })
}

/// Words cut into lines of so many letters (`WrapPlainText`): a line ends where the last blank before its
/// end was, or at its end when it has none; a new line in the words ends one too.
pub fn wrapped(words: &str, width: usize) -> Vec<String> {
    let letters: Vec<char> = words.chars().collect();
    let (mut lines, mut begin, mut last_blank, mut at) = (Vec::new(), 0, 0, 0);
    while at < letters.len() {
        at += 1;
        while letters.get(at) == Some(&' ') {
            at += 1;
            last_blank = at;
        }
        let new_line = letters.get(at) == Some(&'\n');
        if at - begin >= width || new_line {
            let end = if new_line || last_blank <= begin { at } else { last_blank };
            lines.push(letters[begin..end].iter().collect::<String>());
            begin = if new_line { at + 1 } else { end };
            at = at.max(begin);
        }
    }
    if begin < letters.len() {
        lines.push(letters[begin..].iter().collect());
    }
    lines.into_iter().map(|line: String| line.trim_end().to_owned()).filter(|line| !line.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn broadcast(words: &[u8]) -> Vec<u8> {
        let mut packet = vec![0x9a, 0x00, 0, 0];
        packet.extend(words);
        packet.push(0);
        packet
    }

    #[test]
    fn a_plain_announcement_is_yellow() {
        let said = plain(&broadcast(b"The server restarts soon.")).unwrap();
        assert_eq!((said.words.as_str(), said.colour, said.kind), ("The server restarts soon.", [255, 255, 0], crate::said::ANNOUNCED));
        assert_eq!(said.lines, vec!["The server restarts soon.".to_owned()]);
    }

    #[test]
    fn the_first_letters_name_a_colour() {
        let blue = plain(&broadcast(b"blueHello")).unwrap();
        assert_eq!((blue.words.as_str(), blue.colour), ("Hello", [0, 255, 255]));
        let tool = plain(&broadcast(b"toolFF8000Hello")).unwrap();
        assert_eq!((tool.words.as_str(), tool.colour), ("Hello", [255, 128, 0]));
        let system = plain(&broadcast(b"ssssHello")).unwrap();
        assert_eq!((system.words.as_str(), system.kind), ("Hello", crate::said::SYSTEM));
    }

    #[test]
    fn one_that_names_its_sender_is_read() {
        let mut words = b"micc".to_vec();
        words.extend(crate::net::bytes::name24("Somebody"));
        words.extend(b"00FF00Hello all");
        let said = plain(&broadcast(&words)).unwrap();
        assert_eq!((said.from.as_deref(), said.words.as_str(), said.colour, said.kind), (Some("Somebody"), "Hello all", [0, 255, 0], crate::said::GENERAL));
    }

    #[test]
    fn what_follows_a_name_is_cut() {
        assert_eq!(plain(&broadcast(b"Name#1234 : Hello")).unwrap().words, "Name: Hello");
        assert_eq!(plain(&broadcast(b"#first")).unwrap().words, "#first");
    }

    #[test]
    fn a_coloured_one_has_its_colour_and_size() {
        let mut packet = vec![0xc3, 0x01, 0, 0, 0x33, 0x22, 0x11, 0, 0x90, 0x01, 20, 0, 0, 0, 0, 0];
        packet.extend(b"Hello\0");
        let said = coloured(&packet).unwrap();
        assert_eq!((said.words.as_str(), said.colour, said.letters), ("Hello", [0x11, 0x22, 0x33], 20));
        packet[10] = 0;
        assert_eq!(coloured(&packet).unwrap().letters, LETTERS);
    }

    #[test]
    fn long_words_are_cut_at_blanks() {
        assert_eq!(wrapped("one two three four", 9), vec!["one two", "three", "four"]);
        assert_eq!(wrapped("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
        assert_eq!(wrapped("one\ntwo", 72), vec!["one", "two"]);
        assert_eq!(wrapped("", 72), Vec::<String>::new());
    }
}
