//! The server's announcements as the client shows them across the top of the screen (`UIBroadcastBalloon`):
//! how their words are cut into lines. The packets are `net/broadcast.rs`.

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

    #[test]
    fn long_words_are_cut_at_blanks() {
        assert_eq!(wrapped("one two three four", 9), vec!["one two", "three", "four"]);
        assert_eq!(wrapped("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
        assert_eq!(wrapped("one\ntwo", 72), vec!["one", "two"]);
        assert_eq!(wrapped("", 72), Vec::<String>::new());
    }
}
