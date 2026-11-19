//! The tabs of the chat window as the player has them: which kinds of line each lists, the tabs taken out
//! into windows of their own, and the text all of it is kept as. (The links to items in a line of chat are
//! `crate::links::items`'s, and are still named from here.)
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
    let tabs: Vec<Tab> = tabs.into_iter()
        .take(MOST)
        .map(|(tab, place, out)| Tab { own: out.then_some((place[0], place[1], place[2], place[3])), ..tab })
        .collect();
    (!tabs.is_empty()).then_some((tabs, locked))
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
}
