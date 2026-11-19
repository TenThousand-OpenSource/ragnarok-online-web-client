//! The marks the client's texts carry, told apart from the text here once: a colour for what follows
//! (`^RRGGBB`, six digits of sixteen) and the tags around a link (`<NAVI>…</NAVI>`, `<ITEM>`, `<INFO>`,
//! `<URL>`…). What a text does with them is its own: a quest's shows a link's words and drops where it leads
//! (`crate::quests`), the chat makes links of them (`crate::links`), a window writes the colours
//! (`hud/texts.rs`).
//!
//! A `^` that no six digits follow and a `<` that begins none of the tags asked for are text, as in the client.

/// A piece of a text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece<'a> {
    /// Words as they stand.
    Text(&'a str),
    /// The colour of what follows: its six digits, as written.
    Colour(&'a str),
    /// A tag, by its name as it was asked for; `closes` for the one with a stroke (`</NAVI>`).
    Tag { name: &'a str, closes: bool },
}

/// A text as its pieces, in order. `tags` are the names of the tags to tell from the text (`"NAVI"`, `"INFO"`),
/// in capitals as the client writes them; with none, only the colours are told apart. Where one name begins
/// another (`ITEM`, `ITEML`) the tag must end with its `>` to be it, so the order of the names does not matter.
pub fn pieces<'a>(text: &'a str, tags: &[&'a str]) -> Vec<Piece<'a>> {
    let mut pieces = Vec::new();
    // Where the words not yet given out begin, and how far the text was looked through.
    let (mut from, mut at) = (0, 0);
    while let Some(found) = text[at..].find(['^', '<']).map(|found| at + found) {
        let (piece, size) = match mark(&text[found..], tags) {
            Some(mark) => mark,
            None => {
                at = found + 1;
                continue;
            }
        };
        if from < found {
            pieces.push(Piece::Text(&text[from..found]));
        }
        pieces.push(piece);
        at = found + size;
        from = at;
    }
    if from < text.len() {
        pieces.push(Piece::Text(&text[from..]));
    }
    pieces
}

/// The mark a text begins with, if it begins with one, and how many bytes the mark takes.
fn mark<'a>(text: &'a str, tags: &[&'a str]) -> Option<(Piece<'a>, usize)> {
    if text.starts_with('^') {
        let code = text.get(1..7).filter(|code| code.bytes().all(|byte| byte.is_ascii_hexdigit()))?;
        return Some((Piece::Colour(code), 7));
    }
    let inside = text.strip_prefix('<')?;
    let (closes, named) = inside.strip_prefix('/').map_or((false, inside), |named| (true, named));
    let name = tags.iter().copied().find(|tag| named.strip_prefix(tag).is_some_and(|after| after.starts_with('>')))?;
    Some((Piece::Tag { name, closes }, name.len() + 2 + usize::from(closes)))
}

/// A text without the colours written into it, as a list shows a summary (the client's `StripColorCodes`).
pub fn uncoloured(text: &str) -> String {
    pieces(text, &[]).into_iter().filter_map(|piece| if let Piece::Text(words) = piece { Some(words) } else { None }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Piece::{Colour, Tag, Text};

    #[test]
    fn colours_are_told_from_the_text() {
        assert_eq!(pieces("^0000FFFather^000000 waits", &[]), [Colour("0000FF"), Text("Father"), Colour("000000"), Text(" waits")]);
        // No six digits of sixteen after it: a `^` like any other character.
        assert_eq!(pieces("2^3 and ^ff", &[]), [Text("2^3 and ^ff")]);
        assert_eq!(pieces("^00ff0g^", &[]), [Text("^00ff0g^")]);
        assert_eq!(pieces("^ff00ff", &[]), [Colour("ff00ff")]);
        assert_eq!(pieces("", &[]), []);
        // A character of several bytes after the `^` is no digit, and is not cut in half.
        assert_eq!(pieces("^가나다라", &[]), [Text("^가나다라")]);
    }

    #[test]
    fn the_tags_asked_for_are_told_from_the_text() {
        let tags = ["NAVI", "INFO", "ITEML", "ITEM"];
        assert_eq!(
            pieces("<NAVI>[Luanda]<INFO>com_d02_i,241</INFO></NAVI> will reward you.", &tags),
            [
                Tag { name: "NAVI", closes: false },
                Text("[Luanda]"),
                Tag { name: "INFO", closes: false },
                Text("com_d02_i,241"),
                Tag { name: "INFO", closes: true },
                Tag { name: "NAVI", closes: true },
                Text(" will reward you."),
            ]
        );
        // A name that begins another is not taken for it, whichever is asked for first.
        assert_eq!(pieces("<ITEML>x</ITEML>", &["ITEM", "ITEML"]), [Tag { name: "ITEML", closes: false }, Text("x"), Tag { name: "ITEML", closes: true }]);
        // What is no tag asked for is text, a `<` alone too.
        assert_eq!(pieces("1 < 2 <b> <URL>", &tags), [Text("1 < 2 <b> <URL>")]);
        assert_eq!(pieces("a<NAVI", &tags), [Text("a<NAVI")]);
        assert_eq!(pieces("<ITEM>^ff0000Apple", &tags), [Tag { name: "ITEM", closes: false }, Colour("ff0000"), Text("Apple")]);
    }

    #[test]
    fn a_text_is_shown_without_its_colours() {
        assert_eq!(uncoloured("^0000FFFather^000000 waits"), "Father waits");
        assert_eq!(uncoloured("2^3 and ^ff"), "2^3 and ^ff");
        assert_eq!(uncoloured("<NAVI>kept</NAVI>"), "<NAVI>kept</NAVI>");
    }
}
