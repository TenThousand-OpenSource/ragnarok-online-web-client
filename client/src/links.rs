//! The links written into what an NPC says (the client's `UITagTextViewer`, whose lines `CCreateLineInfo`
//! builds: `ParseLink`, `SplitLinkInfo` and `BuildWindows` of `NavigationLink.cpp`): a tag, the words shown,
//! and between `<INFO>` and `</INFO>` what the link leads to, divided by commas —
//! `<ITEM>[Apple]<INFO>512</INFO></ITEM>`, `<NAVI>[Kafra]<INFO>prontera,150,180</INFO></NAVI>`,
//! `<URL>site<INFO>http://…</INFO></URL>`. The links to items in a line of chat, which are written otherwise,
//! are `items.rs`.

pub mod items;

/// The kinds of link the viewer shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// An address of the web.
    Url,
    /// An item, by its number: pressed, it opens what is known of the item.
    Item,
    /// A place on a map, for the navigation.
    Navi,
    /// A tip of the Tipbox, by its number.
    Tip,
    /// A quest, by its number.
    Quest,
    /// Not a link: a line of the client's message table, named by its number, written in its place
    /// (`<MSG>1294</MSG>`).
    Message,
}

/// A link of a text: its words, and what is between its `<INFO>` tags, divided at the commas.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub kind: Kind,
    pub words: String,
    pub info: Vec<String>,
}

impl Link {
    /// The number a link leads to: an item's, a tip's, a quest's. It is read as the client reads it
    /// (`ShowTextButtonItem`, `ShowTextButtonTip`: `atoi`), the digits it begins with: the Tipbox names its
    /// items by a number and the name after it (`<INFO>611 Magnifier</INFO>`).
    pub fn number(&self) -> Option<u32> {
        let info = self.info.first()?.trim_start();
        let digits = info.find(|letter: char| !letter.is_ascii_digit()).unwrap_or(info.len());
        info[..digits].parse().ok()
    }
}

/// What stands in a text where a link was, before and after its place among the links: two characters no
/// text of the game has.
pub const MARK: char = '\u{e000}';
pub const MARK_END: char = '\u{e001}';

/// The colours the client writes a link in, underlined (`CreateLinkWindow`: `0xE6A300` for an address and
/// `0xF4934A` for any other, blue first).
pub const URL_COLOUR: &str = "#00a3e6";
pub const LINK_COLOUR: &str = "#4a93f4";

/// The tags, each with its kind; `None` is a tag the viewer reads and shows nothing of (`<ITEML>`, `<NAVIL>`).
const TAGS: [(&str, Option<Kind>); 8] = [
    ("URL", Some(Kind::Url)),
    ("ITEML", None),
    ("ITEM", Some(Kind::Item)),
    ("NAVIL", None),
    ("NAVI", Some(Kind::Navi)),
    ("MSG", Some(Kind::Message)),
    ("QUEST", Some(Kind::Quest)),
    ("TIPBOX", Some(Kind::Tip)),
];

/// A text taken apart: the text with each link's place marked (`MARK`, the link's place among the links, then
/// `MARK_END`), and the links. A link without `<INFO>` shows nothing, as in the client; a tag that is not
/// closed is text.
pub fn parted(text: &str) -> (String, Vec<Link>) {
    let (mut plain, mut links) = (String::with_capacity(text.len()), Vec::new());
    let mut rest = text;
    while let Some(at) = rest.find('<') {
        plain.push_str(&rest[..at]);
        rest = &rest[at..];
        let tagged = TAGS.iter().find_map(|(tag, kind)| {
            let inside = rest.strip_prefix('<')?.strip_prefix(tag)?.strip_prefix('>')?;
            let close = format!("</{tag}>");
            let end = inside.find(&close)?;
            Some((*kind, &inside[..end], &inside[end + close.len()..]))
        });
        let Some((kind, inside, after)) = tagged else {
            plain.push('<');
            rest = &rest[1..];
            continue;
        };
        rest = after;
        let link = match kind {
            Some(Kind::Message) => Some(Link { kind: Kind::Message, words: String::new(), info: vec![inside.trim().to_owned()] }),
            Some(kind) => inside.split_once("<INFO>").map(|(words, info)| Link {
                kind,
                words: words.to_owned(),
                info: info.split("</INFO>").next().unwrap_or_default().split(',').map(str::to_owned).collect()
            }),
            None => None,
        };
        if let Some(link) = link {
            plain.push(MARK);
            plain.push_str(&links.len().to_string());
            plain.push(MARK_END);
            links.push(link);
        }
    }
    plain.push_str(rest);
    (plain, links)
}

/// Puts something back where each link was marked: `written` gives what stands for a link.
pub fn put_back(marked: &str, links: &[Link], written: impl Fn(&Link) -> String) -> String {
    let mut whole = String::with_capacity(marked.len());
    let mut rest = marked;
    while let Some(at) = rest.find(MARK) {
        whole.push_str(&rest[..at]);
        let after = &rest[at + MARK.len_utf8()..];
        let end = after.find(MARK_END).unwrap_or(after.len());
        if let Some(link) = after[..end].parse::<usize>().ok().and_then(|place| links.get(place)) {
            whole.push_str(&written(link));
        }
        rest = after.get(end + MARK_END.len_utf8()..).unwrap_or_default();
    }
    whole.push_str(rest);
    whole
}

/// A text with each `<MSG>…</MSG>` of it written out (`NavigationLinkParser::ExpandMessages` and
/// `FormatMessageArguments` of `NavigationLink.cpp`, which the mailbox passes a letter's sender, title and text
/// through: a server writes its own letters so, `<MSG>3456,1</MSG>`): the line of the message table of that
/// number, and what follows the number after commas written into the line's blanks in turn, an argument
/// `MsgStr(n)` being line `n` itself. A tag that is not closed leaves the text as it was, as in the client.
/// Not made: a line named by its name in the client's code (`MSI_…`) instead of its number.
pub fn expanded(text: &str, line: impl Fn(usize) -> Option<String>) -> String {
    let mut whole = text.to_owned();
    while let Some(begin) = whole.find("<MSG>") {
        let inside = &whole[begin + 5..];
        let Some(end) = inside.find("</MSG>") else { return text.to_owned() };
        let mut given = inside[..end].split(',');
        let of = |number: &str| number.trim().parse().ok().and_then(&line).unwrap_or_default();
        let message = of(given.next().unwrap_or_default());
        let arguments: Vec<String> =
            given.map(|argument| argument.strip_prefix("MsgStr(").and_then(|rest| rest.strip_suffix(')')).map_or_else(|| argument.to_owned(), of)).collect();
        let written = crate::numbers::filled(&message, &arguments.iter().map(String::as_str).collect::<Vec<_>>());
        whole = format!("{}{written}{}", &whole[..begin], &inside[end + 6..]);
    }
    whole
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_of_the_message_table_is_written_out_with_what_is_given_it() {
        let line = |number| match number {
            3455 => Some("Attendance".to_owned()),
            3456 => Some("Day %d reward".to_owned()),
            7 => Some("%s and %s".to_owned()),
            _ => None,
        };
        assert_eq!(expanded("<MSG>3455</MSG>", line), "Attendance");
        assert_eq!(expanded("[<MSG>3456,12</MSG>] <MSG>3455</MSG>!", line), "[Day 12 reward] Attendance!");
        assert_eq!(expanded("<MSG>7,MsgStr(3455),you</MSG>", line), "Attendance and you");
        // Not closed: as it was written. No tag: untouched.
        assert_eq!(expanded("<MSG>3455</MSG> <MSG>7", line), "<MSG>3455</MSG> <MSG>7");
        assert_eq!(expanded("Hello", line), "Hello");
    }

    #[test]
    fn a_links_number_is_the_digits_it_begins_with() {
        let (_, links) = parted("<ITEM>Magnifier<INFO>611 Magnifier</INFO></ITEM> <TIPBOX>Stats<INFO> 7</INFO></TIPBOX> <ITEM>x<INFO>none</INFO></ITEM>");
        assert_eq!(links.iter().map(Link::number).collect::<Vec<_>>(), vec![Some(611), Some(7), None]);
    }

    #[test]
    fn takes_the_links_out_of_a_text() {
        let (plain, links) = parted("Bring <ITEM>[Apple]<INFO>512</INFO></ITEM> to <NAVI>[Kafra]<INFO>prontera,150,180,0,100,0</INFO></NAVI>.");
        assert_eq!(plain, "Bring \u{e000}0\u{e001} to \u{e000}1\u{e001}.");
        assert_eq!(links[0], Link { kind: Kind::Item, words: "[Apple]".to_owned(), info: vec!["512".to_owned()] });
        assert_eq!(links[0].number(), Some(512));
        assert_eq!((links[1].kind, links[1].words.as_str(), links[1].info.len()), (Kind::Navi, "[Kafra]", 6));
        assert_eq!(put_back(&plain, &links, |link| link.words.clone()), "Bring [Apple] to [Kafra].");
    }

    #[test]
    fn shows_nothing_of_what_the_viewer_drops_and_keeps_what_is_no_tag() {
        assert_eq!(parted("a <ITEML>x<INFO>1</INFO></ITEML> b <ITEM>no info</ITEM> c").0, "a  b  c");
        assert_eq!(parted("1 < 2 and <NAVI>open").0, "1 < 2 and <NAVI>open");
        let (plain, links) = parted("<MSG>1294</MSG> <URL>site<INFO>http://example.com/a,b</INFO></URL>");
        assert_eq!((links[0].kind, links[0].number()), (Kind::Message, Some(1294)));
        assert_eq!((links[1].kind, links[1].info.len()), (Kind::Url, 2));
        assert_eq!(put_back(&plain, &links, |link| format!("[{}]", link.words)), "[] [site]");
    }
}
