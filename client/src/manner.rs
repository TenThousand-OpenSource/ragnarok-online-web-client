//! The words the client does not let be written (`data\manner.txt`), and what it does with a text that has
//! one: refuses it (`ForbiddenWords::Allows`, `Contains`) or puts stars in the word's place
//! (`ForbiddenWords::Mask`), rodecomp `Source/Contents/Tables/ForbiddenWords.cpp`. Which of the two a place
//! does, and what it then says, is that place's own: this is only the rule.
//!
//! The client goes through a text a character at a time and takes a word that the text has from there on,
//! whatever stands before or after it: a word inside a longer word is found too. Latin capitals count as
//! their small letters (in the table and in the text); a link to an item (`<ITEML>…</ITEML>`) is passed
//! over, and what follows it is looked at.

use std::collections::HashMap;

/// A link to an item, as the client looks for it in a text made small: where one begins, and where it ends.
const LINK: (&str, &str) = ("<iteml>", "</iteml>");

/// The words, as the gateway gives them out of the client's file (`tables/manner.rs`: small letters, the
/// file's order).
#[derive(Default)]
pub struct Words {
    /// By the character a word begins with, the words in the table's order: of the words a text has at a
    /// place the client takes the first of the table (`std::find_if`), and all of them begin alike.
    by_first: HashMap<char, Vec<Vec<char>>>,
}

impl Words {
    pub fn new<Word: AsRef<str>>(words: impl IntoIterator<Item = Word>) -> Self {
        let mut by_first: HashMap<char, Vec<Vec<char>>> = HashMap::new();
        for word in words {
            let word: Vec<char> = word.as_ref().chars().map(|letter| letter.to_ascii_lowercase()).collect();
            if let Some(first) = word.first() {
                by_first.entry(*first).or_default().push(word);
            }
        }
        Self { by_first }
    }

    /// The first word of the table that the text has at a place: how many characters it is.
    fn word_at(&self, text: &[char], at: usize) -> Option<usize> {
        let rest = text.get(at..)?;
        self.by_first.get(rest.first()?)?.iter().find(|word| rest.starts_with(word)).map(Vec::len)
    }

    /// Whether a text has none of the words (`Allows`; the client's `Contains` is its opposite).
    pub fn allows(&self, text: &str) -> bool {
        let small = small(text);
        let mut at = 0;
        while at < small.len() {
            at = past_link(&small, at);
            if self.word_at(&small, at).is_some() {
                return false;
            }
            at += 1;
        }
        true
    }

    /// A text with every word it has written as stars, one for each character of the word (`Mask`). What
    /// follows a word is looked at from the word's end.
    pub fn masked(&self, text: &str) -> String {
        let small = small(text);
        let mut written: Vec<char> = text.chars().collect();
        let mut at = 0;
        while at < small.len() {
            at = past_link(&small, at);
            match self.word_at(&small, at) {
                Some(length) => {
                    written[at..at + length].fill('*');
                    at += length;
                }
                None => at += 1,
            }
        }
        written.into_iter().collect()
    }
}

/// A text as the client compares it: its Latin capitals made small, nothing else changed (`Lowercase`).
fn small(text: &str) -> Vec<char> {
    text.chars().map(|letter| letter.to_ascii_lowercase()).collect()
}

/// Where the client goes on from a place of a text made small: past a link to an item that begins there and
/// is closed somewhere after, and otherwise from the place itself. (The client counts the link in bytes of its
/// own encoding and then steps so many characters: a link with Korean in it would be passed by more than its
/// length. A character that is not Latin is counted as the two bytes it is there.)
fn past_link(small: &[char], at: usize) -> usize {
    let starts = |from: usize, word: &str| small[from..].iter().copied().take(word.len()).eq(word.chars());
    if at >= small.len() || !starts(at, LINK.0) {
        return at;
    }
    let Some(close) = (at..small.len()).find(|from| starts(*from, LINK.1)) else { return at };
    let bytes: usize = small[at..close].iter().map(|letter| if letter.is_ascii() { 1 } else { 2 }).sum::<usize>() + LINK.1.len();
    (at + bytes).min(small.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Words {
        Words::new(["bad", "bad word", " 1 8", "나쁜말", "x"])
    }

    #[test]
    fn a_text_with_a_word_anywhere_in_it_is_not_allowed() {
        let words = words();
        assert!(words.allows("a good line"));
        assert!(!words.allows("so BAD"));
        // Inside a longer word too, and a word with blanks only as it is written.
        assert!(!words.allows("badminton"));
        assert!(!words.allows("room 1 8"));
        assert!(words.allows("room 18"));
        assert!(!words.allows("이건 나쁜말이다"));
        assert!(Words::default().allows("bad"));
        assert!(words.allows(""));
    }

    #[test]
    fn a_word_is_written_as_a_star_for_each_of_its_characters() {
        let words = words();
        // The first word of the table that fits is taken: "bad", not "bad word".
        assert_eq!(words.masked("A Bad Word, xbad"), "A *** Word, ****");
        assert_eq!(words.masked("이건 나쁜말이다"), "이건 ***이다");
        assert_eq!(words.masked("nothing here"), "nothing here");
        assert_eq!(words.masked("Room 1 8"), "Room****");
    }

    #[test]
    fn a_link_to_an_item_is_passed_over() {
        let words = words();
        assert!(words.allows("<ITEML>bad</ITEML> good"));
        assert!(!words.allows("<ITEML>000</ITEML>bad"));
        assert_eq!(words.masked("see <ITEML>badx</ITEML>bad"), "see <ITEML>badx</ITEML>***");
        // A link that is not closed is a text like any other.
        assert_eq!(words.masked("<ITEML>bad"), "<ITEML>***");
        // The client looks at the place after a link and then steps on by one: a link that follows another
        // at once is not seen to begin, and is gone through as any text.
        assert_eq!(words.masked("<ITEML>1</ITEML><ITEML>x</ITEML>"), "<ITEML>1</ITEML><ITEML>*</ITEML>");
        assert_eq!(words.masked("<ITEML>1</ITEML> <ITEML>x</ITEML>"), "<ITEML>1</ITEML> <ITEML>x</ITEML>");
    }
}
