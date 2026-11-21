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

/// A piece of a text that carries items' pictures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shown<'a> {
    /// Words as they stand, their other marks still in them.
    Words(&'a str),
    /// The small picture of the item of this number, in the line (`^i[501]`).
    Item(u32),
}

/// A text as its words and the items' pictures written into it: `^i[501]` is the picture of item 501, 25 by
/// 25 in the line, and what follows it stays text (the tips of `tipbox.lub` write `^i[537] [Pet Food]`). As
/// the client's `CCreateLineInfo::ParseItemImage` takes it: `^i` and every `[number]` of one to seven digits
/// right after it are the mark, and `LayoutItemImage` draws the item of the first; a `^i` that no such
/// number follows is taken out of the text all the same, with no picture. A colour is looked for before this
/// (`ParseTag`), which changes nothing: no colour begins with an `i`.
pub fn pictured(text: &str) -> Vec<Shown<'_>> {
    let mut shown = Vec::new();
    let mut rest = text;
    while let Some(found) = rest.find("^i") {
        if found > 0 {
            shown.push(Shown::Words(&rest[..found]));
        }
        rest = &rest[found + 2..];
        let mut first = None;
        while let Some((number, size)) = bracketed(rest) {
            first = first.or(Some(number));
            rest = &rest[size..];
        }
        shown.extend(first.map(Shown::Item));
    }
    if !rest.is_empty() {
        shown.push(Shown::Words(rest));
    }
    shown
}

/// The number in square brackets a text begins with, of one to seven digits, and how many bytes it takes.
fn bracketed(text: &str) -> Option<(u32, usize)> {
    let inside = text.strip_prefix('[')?;
    let digits = inside.bytes().take_while(u8::is_ascii_digit).count();
    ((1..=7).contains(&digits) && inside[digits..].starts_with(']')).then(|| (inside[..digits].parse().unwrap_or(0), digits + 2))
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

    #[test]
    fn the_pictures_of_items_are_told_from_the_words() {
        use Shown::{Item, Words};
        assert_eq!(pictured("Feed it ^i[537] [Pet Food] now"), [Words("Feed it "), Item(537), Words(" [Pet Food] now")]);
        assert_eq!(pictured("^i[9140] [Pet Blue Moon Loli Ruri]"), [Item(9140), Words(" [Pet Blue Moon Loli Ruri]")]);
        assert_eq!(pictured("^i[12580]^i[23368]"), [Item(12580), Item(23368)]);
        // Every number right after the mark belongs to it, and the first is the one drawn.
        assert_eq!(pictured("^i[501][502] a"), [Item(501), Words(" a")]);
        // No number of one to seven digits after it: the mark is taken out, and nothing is drawn.
        assert_eq!(pictured("a ^i{501} b"), [Words("a "), Words("{501} b")]);
        assert_eq!(pictured("^i[12345678]"), [Words("[12345678]")]);
        assert_eq!(pictured("^i[]x"), [Words("[]x")]);
        assert_eq!(pictured("^i"), []);
        // The other marks are left to whoever writes the words.
        assert_eq!(pictured("^ff0000red ^i[501]"), [Words("^ff0000red "), Item(501)]);
        assert_eq!(pictured("no marks"), [Words("no marks")]);
        assert_eq!(pictured(""), []);
    }
}
