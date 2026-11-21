//! The words the client does not let be written (`data\manner.txt`, read by `ForbiddenWords::Load` of rodecomp's
//! `Source/Contents/Tables/ForbiddenWords.cpp`; the rule that uses them is the browser client's `crate::manner`).

use super::*;

/// The words of the file, in its order, as the client keeps them (`CCommonObject::LoadTokens` in its mode 1,
/// then `ForbiddenWords::Load`): a word ends at the end of its line or at a `#`; a line that begins with `//`
/// and a word of nothing but blanks are left out; the blanks a word begins and ends with are kept, and its
/// Latin capitals are made small. The client takes a line only at its carriage return, so a last line that
/// has none is no word.
pub fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        // What stands after the last line break ended with no carriage return, unless it ends with one.
        let Some(line) = line.strip_suffix('\r').or_else(|| lines.peek().is_some().then_some(line)) else { continue };
        if line.starts_with("//") {
            continue;
        }
        words.extend(line.split('#').filter(|word| !word.trim_matches([' ', '\t']).is_empty()).map(str::to_ascii_lowercase));
    }
    words
}

/// `["word", …]` out of the archives; an empty list, with a note, when the file is not there.
pub fn manner(grf: &Grf) -> serde_json::Value {
    match grf.read("data\\manner.txt") {
        Ok(bytes) => words(&korean(&bytes)).into(),
        Err(error) => {
            eprintln!("tables: manner.txt: {error}");
            serde_json::json!([])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_is_a_line_with_its_blanks_and_small_letters() {
        let read = words("Bad Word\r\n 1 8\r\n//note\r\n\r\n  \r\nonE#Two#\r\nlast");
        assert_eq!(read, ["bad word", " 1 8", "one", "two"]);
        assert_eq!(words("a\r\nb\r"), ["a", "b"]);
    }
}
