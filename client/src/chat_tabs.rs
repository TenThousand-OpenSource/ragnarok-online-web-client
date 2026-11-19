//! The tabs of the chat window as the player has them: which kinds of line each lists, the tabs taken out
//! into windows of their own, and the text all of it is kept as; and the links to items in a line of chat.
//!
//! The client keeps the tabs in `SaveData\ChatWndInfo_U.lua` (`UINewChatWnd::SaveChatWndInfo`): `CanDragWnd`,
//! then a table `ChatSubWnd_N` to a tab, with its place and size, whether it is in the window (`TabState`
//! "On") or out of it ("Off"), its name and `option1` to `option25`. The same text is kept here, in the browser.

/// How many kinds of line a tab can be set to list (`option1`…`option25`).
pub const KINDS: usize = 25;
/// How many tabs there can be, in the window and out of it together (`UINewChatWnd`'s `m_maxTabs`).
pub const MOST: usize = 10;

/// The rows of the window that sets what a tab lists (`UIBattleMsgOptionWnd`), in its order: the kind of line
/// (the row's place) and the line of the client's message table that names it. Kind 15 has a name and is never
/// drawn there.
pub const ROWS: [(usize, usize); 24] = [
    (0, 1291), (1, 1277), (2, 1278), (3, 1279), (4, 1280), (5, 4031), (6, 1281), (7, 1282), (8, 1283), (9, 1284), (10, 1285), (11, 1286),
    (12, 1287), (13, 1288), (14, 1331), (16, 1618), (17, 1619), (18, 1620), (19, 1622), (20, 1623), (21, 2365), (22, 2980), (23, 4028), (24, 4030),
];

/// The smallest and the largest a tab's own window can be, and the step it grows by (`UISubChatHisWnd::Resize`);
/// the size it opens with, and where (`UIWindowMgr::OpenSubChatWindow`: by its number).
pub const OWN_SIZE: (i32, i32) = (300, 95);

/// A tab of the chat.
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub name: String,
    /// Whether it lists a line of each kind.
    pub shows: [bool; KINDS],
    /// Out of the window, in one of its own: where that stands and how large it is.
    pub own: Option<(i32, i32, i32, i32)>,
}

impl Tab {
    /// A tab the player adds (`ID_ADD_CHAT_TAB`): named by how many there are with it, listing nothing yet.
    pub fn added(total: usize) -> Tab {
        Tab { name: format!("NewTab_{}", total + 1), shows: [false; KINDS], own: None }
    }

    /// Whether it lists a line (`UINewChatWnd`, on `UM_ADDCHAT`): one of a kind it is set to list, and besides,
    /// in every tab, a line of kind 25 and one of no particular kind written in pure red.
    pub fn lists(&self, kind: u8, red: bool) -> bool {
        kind == 25 || (kind == 0 && red) || self.shows.get(usize::from(kind)).copied().unwrap_or(false)
    }
}

/// Where the window of the tab with that number opens (`OpenSubChatWindow`).
pub fn own_place(number: usize) -> (i32, i32, i32, i32) {
    ((number as i32 + 10) * 10, (number as i32 + 5) * 20, OWN_SIZE.0, OWN_SIZE.1)
}

/// The tabs as the client writes them to its file: those in the window first, then those out of it.
pub fn written(tabs: &[Tab], locked: bool) -> String {
    let mut text = format!("CanDragWnd = {}\n", u8::from(!locked));
    let ordered = tabs.iter().filter(|tab| tab.own.is_none()).chain(tabs.iter().filter(|tab| tab.own.is_some()));
    for (number, tab) in ordered.enumerate() {
        let (x, y, width, height) = tab.own.unwrap_or_default();
        text.push_str(&format!("ChatSubWnd_{} = {{\n\tXPos = {x},\n\tYPos = {y},\n\tWidth = {width},\n\tHeight = {height},\n", number + 1));
        text.push_str(&format!("\tTabState = \"{}\",\n\tTabName = [[{}]],\n", if tab.own.is_some() { "Off" } else { "On" }, tab.name.replace("]]", "")));
        for (kind, shown) in tab.shows.iter().enumerate() {
            text.push_str(&format!("\toption{} = {},\n", kind + 1, u8::from(*shown)));
        }
        text.push_str("}\n");
    }
    text
}

/// The tabs read back from that text, and whether the window is locked; nothing for a text with no tab in it.
pub fn read(text: &str) -> Option<(Vec<Tab>, bool)> {
    let mut tabs: Vec<(Tab, [i32; 4], bool)> = Vec::new();
    let mut locked = false;
    for line in text.lines() {
        let line = line.trim().trim_end_matches(',');
        if line.starts_with("ChatSubWnd_") {
            tabs.push((Tab { name: String::new(), shows: [false; KINDS], own: None }, [0; 4], false));
            continue;
        }
        let Some((field, value)) = line.split_once('=').map(|(field, value)| (field.trim(), value.trim())) else { continue };
        if field == "CanDragWnd" {
            locked = value == "0";
            continue;
        }
        let Some((tab, place, out)) = tabs.last_mut() else { continue };
        let number = value.parse::<i32>().unwrap_or(0);
        match field {
            "XPos" => place[0] = number,
            "YPos" => place[1] = number,
            "Width" => place[2] = number,
            "Height" => place[3] = number,
            "TabState" => *out = value.trim_matches('"') == "Off",
            "TabName" => tab.name = value.trim_start_matches("[[").trim_end_matches("]]").trim_matches('"').to_owned(),
            _ => {
                if let Some(kind) = field.strip_prefix("option").and_then(|kind| kind.parse::<usize>().ok()).filter(|kind| (1..=KINDS).contains(kind)) {
                    tab.shows[kind - 1] = number != 0;
                }
            }
        }
    }
    let tabs: Vec<Tab> = tabs.into_iter().take(MOST).map(|(tab, place, out)| Tab { own: out.then_some((place[0], place[1], place[2], place[3])), ..tab }).collect();
    (!tabs.is_empty()).then_some((tabs, locked))
}

/// A piece of a line of chat: plain words, or a link to an item (`<ITEML>…</ITEML>`), which the client shows as
/// the item's name between `<` and `>` (`UIChatHisBox::RenderItemLinks`).
#[derive(Debug, PartialEq)]
pub enum Piece<'line> {
    Words(&'line str),
    Item(Link),
}

/// What a link to an item says of it (`UIChatHisBox::DecodeItemLinkFields`): the item, and how far it is refined.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Link {
    pub item: u32,
    pub refine: u32,
}

/// The signs that part the fields of a link, as the client's table has them.
const PARTING: &str = "!#$%&'()*+,-/";

/// A number written with the digits, the small and the large letters, sixty-two in all.
fn base62(digits: &str) -> Option<u32> {
    digits.chars().try_fold(0u32, |number, digit| {
        let value = match digit {
            '0'..='9' => digit as u32 - '0' as u32,
            'a'..='z' => digit as u32 - 'a' as u32 + 10,
            'A'..='Z' => digit as u32 - 'A' as u32 + 36,
            _ => return None,
        };
        number.checked_mul(62)?.checked_add(value)
    })
}

/// The same number written so.
fn to_base62(mut number: u32) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut written = Vec::new();
    loop {
        written.push(DIGITS[(number % 62) as usize]);
        number /= 62;
        if number == 0 {
            break;
        }
    }
    written.reverse();
    String::from_utf8(written).unwrap_or_default()
}

/// Reads what is between `<ITEML>` and `</ITEML>`: five signs for where the thing is worn, one for whether it
/// is worn at all, the item's number up to the first parting sign, and after a `%` how far it is refined.
pub fn link(data: &str) -> Option<Link> {
    let rest = data.get(6..)?;
    let end = rest.find(|sign| PARTING.contains(sign)).unwrap_or(rest.len());
    let item = base62(&rest[..end]).filter(|item| *item != 0)?;
    let refine = rest[end..].strip_prefix('%').map(|after| &after[..after.find(|sign| PARTING.contains(sign)).unwrap_or(after.len())]).and_then(base62).unwrap_or(0);
    Some(Link { item, refine })
}

/// A link to an item as the client writes it into what is said (`UIItemTagCtrl::AddItemTag`): where it is worn,
/// whether it is equipment, the item, how far it is refined when it is, and what is set into its four slots.
pub fn link_to(item: u32, wearable: u32, equipment: bool, refine: u8, cards: [u32; 4]) -> String {
    let mut data = format!("{:0>5}{}{}", to_base62(wearable), u8::from(equipment), to_base62(item));
    if refine > 0 {
        data.push_str(&format!("%{:0>2}", to_base62(u32::from(refine))));
    }
    for card in cards {
        data.push_str(&format!("){:0>2}", to_base62(card)));
    }
    format!("<ITEML>{data}</ITEML>")
}

/// A line of chat cut into its words and its links. A link that cannot be read stays as the words it is.
pub fn pieces(line: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("<ITEML>") {
        let Some(length) = rest[start..].find("</ITEML>") else { break };
        let Some(found) = link(&rest[start + 7..start + length]) else { break };
        if start > 0 {
            pieces.push(Piece::Words(&rest[..start]));
        }
        pieces.push(Piece::Item(found));
        rest = &rest[start + length + 8..];
    }
    if !rest.is_empty() {
        pieces.push(Piece::Words(rest));
    }
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tab_lists_what_it_is_set_to_and_what_every_tab_lists() {
        let mut tab = Tab::added(2);
        assert_eq!(tab.name, "NewTab_3");
        assert!(!tab.lists(1, false) && tab.lists(25, false) && tab.lists(0, true) && !tab.lists(0, false));
        tab.shows[1] = true;
        assert!(tab.lists(1, false) && !tab.lists(2, false) && !tab.lists(40, false));
    }

    #[test]
    fn tabs_are_read_back_from_what_they_are_written_as() {
        let mut first = Tab { name: "Public".to_owned(), shows: [false; KINDS], own: None };
        first.shows[..14].fill(true);
        let mut out = Tab::added(1);
        out.own = Some(own_place(2));
        out.shows[16] = true;
        let third = Tab { name: "Battle".to_owned(), shows: [false; KINDS], own: None };
        let text = written(&[first.clone(), out.clone(), third.clone()], true);
        assert!(text.starts_with("CanDragWnd = 0\nChatSubWnd_1 = {\n\tXPos = 0,"));
        assert!(text.contains("TabState = \"Off\",\n\tTabName = [[NewTab_2]]"));
        // Those in the window come first in the file, as the client writes them.
        assert_eq!(read(&text), Some((vec![first, third, out], true)));
        assert_eq!(read("CanDragWnd = 1\n"), None);
        assert_eq!(own_place(2), (120, 140, 300, 95));
    }

    #[test]
    fn a_link_is_read_as_the_server_writes_it() {
        // rAthena's `create_item_link` for a Knife [3] (1202) refined to 7 with a card 4001 in its first slot.
        let knife = "<ITEML>000021jo%07&01'00)12x)00)00)00</ITEML>";
        assert_eq!(pieces(&format!("Look: {knife} here")), vec![Piece::Words("Look: "), Piece::Item(Link { item: 1202, refine: 7 }), Piece::Words(" here")]);
        assert_eq!(link("000000b9"), Some(Link { item: 691, refine: 0 }));
        assert_eq!(link("0000"), None);
        assert_eq!(pieces("<ITEML>broken"), vec![Piece::Words("<ITEML>broken")]);
        assert_eq!(pieces("plain"), vec![Piece::Words("plain")]);
    }

    #[test]
    fn a_link_is_written_as_it_is_read() {
        let written = link_to(1202, 2, true, 7, [4001, 0, 0, 0]);
        assert_eq!(written, "<ITEML>000021jo%07)12x)00)00)00</ITEML>");
        assert_eq!(pieces(&written), vec![Piece::Item(Link { item: 1202, refine: 7 })]);
        assert_eq!(to_base62(0), "0");
        assert_eq!(base62("jo"), Some(1202));
    }
}
