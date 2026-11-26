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

/// The size a tab's own window opens with (`UIWindowMgr::OpenSubChatWindow`), the smallest and the largest it
/// can be made by its corner, and the step it grows by (`UISubChatHisWnd::SendMsg`, `UM_RESIZE`).
pub const OWN_SIZE: (i32, i32) = (300, 95);
pub const OWN_LEAST: (i32, i32) = (280, 74);
pub const OWN_MOST: (i32, i32) = (512, 384);
pub const OWN_STEP: i32 = 32;

/// How wide the chat window is, and how much wider than its name a tab is in the window's line of tabs
/// (`UITabControl::Configure` with 10 of room about a name).
pub const WINDOW: i32 = 600;
pub const TAB_ROOM: i32 = 10;

/// How wide each tab is in the window's line of tabs, the names of those in the window being so wide: all as
/// wide as the widest of them with its room (`UITabControl::RecalculateWidths`, the widths made the same).
pub fn tab_wide(names: impl Iterator<Item = i32>) -> i32 {
    names.max().unwrap_or(0) + TAB_ROOM
}

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

/// Where a tab's own window opens when there are already so many of them (`OpenSubChatWindow` with no place
/// given: by `retainedWindows.size()`).
pub fn own_place(windows: usize) -> (i32, i32, i32, i32) {
    ((windows as i32 + 10) * 10, (windows as i32 + 5) * 20, OWN_SIZE.0, OWN_SIZE.1)
}

/// Where a tab's own window is shown on a screen of that size, and how large. The client
/// (`OpenSubChatWindow`) takes 300 for a width of nought or wider than the screen and 95 for such a height,
/// puts a window whose left is past the screen's right against that side, and one whose top is past the
/// bottom at 100 down. Beyond that, and this client's own: a window kept anywhere else out of sight (left of
/// the screen, above it, hanging over its right or its bottom) is brought wholly into it, so that one put out
/// of reach by an older version, or by a smaller browser window, can be taken hold of again.
pub fn own_shown((x, y, width, height): (i32, i32, i32, i32), screen: (i32, i32)) -> (i32, i32, i32, i32) {
    let width = if width <= 0 || width > screen.0 { OWN_SIZE.0 } else { width };
    let height = if height <= 0 || height > screen.1 { OWN_SIZE.1 } else { height };
    let x = if x > screen.0 { screen.0 - width } else { x };
    let y = if y > screen.1 { 100 } else { y };
    (x.min(screen.0 - width).max(0), y.min(screen.1 - height).max(0), width, height)
}

/// The size a tab's own window takes when its corner is dragged to make it that large (`UM_RESIZE`): between
/// 280 by 74 and 512 by 384, in steps of 32.
pub fn own_resized(width: i32, height: i32) -> (i32, i32) {
    let stepped = |asked: i32, least: i32, most: i32| (asked.clamp(least, most) - least) / OWN_STEP * OWN_STEP + least;
    (stepped(width, OWN_LEAST.0, OWN_MOST.0), stepped(height, OWN_LEAST.1, OWN_MOST.1))
}

/// Whether a tab's own window let go there goes back among the tabs (`UISubChatHisWnd::OnLBtnUp`): its left
/// side within the chat window's width and its bottom within the chat window's height.
pub fn own_dropped_in(own: (i32, i32, i32, i32), chat: (i32, i32, i32, i32)) -> bool {
    let bottom = own.1 + own.3;
    chat.0 <= own.0 && own.0 <= chat.0 + chat.2 && chat.1 <= bottom && bottom <= chat.1 + chat.3
}

/// Whether a tab held by the pointer is torn out of the window, the pointer having gone that far from where
/// it was pressed. The client (`UITabControl::OnMouseMove`) tears a tab out at the first move of the pointer
/// with the button down, and after that first time only at a move more than 5 to the left of or above where
/// the last one was torn; here, on purpose, a tab is torn out once the pointer is more than 5 away either
/// way, every time, so that a press with a shaking hand tears nothing out.
pub fn torn(from: (f64, f64), to: (f64, f64)) -> bool {
    (to.0 - from.0).abs() > 5.0 || (to.1 - from.1).abs() > 5.0
}

/// How many of the tabs are in the window.
pub fn inside(tabs: &[Tab]) -> usize {
    tabs.iter().filter(|tab| tab.own.is_none()).count()
}

/// Whether the window's line of tabs, each tab so wide, has room for one more (`ID_ADD_CHAT_TAB`,
/// `UM_ATTACH_CHAT_TAB`: the tabs may take the window's width less 90).
pub fn room(tabs: &[Tab], wide: i32) -> bool {
    (inside(tabs) as i32 + 1) * wide <= WINDOW - 90
}

/// Whether a name so wide may be given to a tab in the window, with so many tabs there
/// (`UITabControl::SendMsg`, `UM_DEFPUSH`: the name with its room, times the tabs, must leave 90 of the
/// window's width), or to a tab in a window of its own that wide (`UISubChatHisWnd::SendMsg`: with its room
/// it must leave 50 of that window's).
pub fn name_fits(name: i32, tabs_inside: usize, own: Option<i32>) -> bool {
    match own {
        None => (name + TAB_ROOM) * (tabs_inside as i32) < WINDOW - 90,
        Some(window) => name + TAB_ROOM < window - 50,
    }
}

/// The tabs in the client's order: those in the window, then those out of it, each kind as it stood
/// (the client keeps the two apart, `m_noSubChatWnds` and `m_subChatWnds`).
pub fn ordered(tabs: Vec<Tab>) -> Vec<Tab> {
    let (inside, out): (Vec<Tab>, Vec<Tab>) = tabs.into_iter().partition(|tab| tab.own.is_none());
    inside.into_iter().chain(out).collect()
}

/// A tab goes out of the window into one of its own at that place (`UM_DETACH_CHAT_TAB`): it becomes the
/// last of those out, and the tab before it is the open one (the first stays the first). Nothing, and no
/// change, for a tab that is not in the window or is the only one there.
pub fn detached(tabs: &mut Vec<Tab>, tab: usize, place: (i32, i32, i32, i32)) -> Option<usize> {
    if tabs.get(tab)?.own.is_some() || inside(tabs) <= 1 {
        return None;
    }
    let mut out = tabs.remove(tab);
    out.own = Some(place);
    tabs.push(out);
    Some(tab.saturating_sub(1))
}

/// A tab's own window goes back among the tabs (`UM_ATTACH_CHAT_TAB`): it is the last of them, and the open
/// one; where it is in the list then.
pub fn attached(tabs: &mut Vec<Tab>, tab: usize) -> Option<usize> {
    tabs.get(tab)?.own?;
    let mut back = tabs.remove(tab);
    back.own = None;
    let last = inside(tabs);
    tabs.insert(last, back);
    Some(last)
}

/// The tabs as the client writes them to its file: those in the window first, then those out of it.
pub fn written(tabs: &[Tab], locked: bool) -> String {
    let mut text = format!("CanDragWnd = {}\n", u8::from(!locked));
    let in_order = tabs.iter().filter(|tab| tab.own.is_none()).chain(tabs.iter().filter(|tab| tab.own.is_some()));
    for (number, tab) in in_order.enumerate() {
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
    (!tabs.is_empty()).then(|| (ordered(tabs), locked))
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
        out.own = Some(own_place(0));
        out.shows[16] = true;
        let third = Tab { name: "Battle".to_owned(), shows: [false; KINDS], own: None };
        let text = written(&[first.clone(), out.clone(), third.clone()], true);
        assert!(text.starts_with("CanDragWnd = 0\nChatSubWnd_1 = {\n\tXPos = 0,"));
        assert!(text.contains("TabState = \"Off\",\n\tTabName = [[NewTab_2]]"));
        // Those in the window come first in the file, as the client writes them.
        assert_eq!(read(&text), Some((vec![first, third, out], true)));
        assert_eq!(read("CanDragWnd = 1\n"), None);
        assert_eq!(own_place(0), (100, 100, 300, 95));
        assert_eq!(own_place(2), (120, 140, 300, 95));
    }

    fn named(name: &str) -> Tab {
        Tab { name: name.to_owned(), shows: [false; KINDS], own: None }
    }

    fn names(tabs: &[Tab]) -> Vec<(&str, bool)> {
        tabs.iter().map(|tab| (tab.name.as_str(), tab.own.is_some())).collect()
    }

    #[test]
    fn a_tab_goes_out_and_comes_back_as_the_last() {
        let mut tabs = vec![named("Public"), named("Battle"), named("Third")];
        // The open tab is the one before it; the first stays the first.
        assert_eq!(detached(&mut tabs, 1, own_place(0)), Some(0));
        assert_eq!(names(&tabs), [("Public", false), ("Third", false), ("Battle", true)]);
        assert_eq!(detached(&mut tabs, 0, own_place(1)), Some(0));
        assert_eq!(names(&tabs), [("Third", false), ("Battle", true), ("Public", true)]);
        // The only tab in the window stays, and one out of it cannot go out again.
        assert_eq!(detached(&mut tabs, 0, own_place(2)), None);
        assert_eq!(detached(&mut tabs, 1, own_place(2)), None);
        // Back, a tab is the last in the window and the open one.
        assert_eq!(attached(&mut tabs, 2), Some(1));
        assert_eq!(names(&tabs), [("Third", false), ("Public", false), ("Battle", true)]);
        assert_eq!(attached(&mut tabs, 2), Some(2));
        assert_eq!(names(&tabs), [("Third", false), ("Public", false), ("Battle", false)]);
        assert_eq!(attached(&mut tabs, 2), None);
        assert_eq!(attached(&mut tabs, 7), None);
    }

    #[test]
    fn the_line_of_tabs_has_room_by_how_wide_its_tabs_are() {
        assert_eq!(tab_wide([37, 61, 40].into_iter()), 71);
        assert_eq!(tab_wide(std::iter::empty()), 10);
        let mut tabs: Vec<Tab> = (0..6).map(|_| named("x")).collect();
        assert!(room(&tabs, 71) && !room(&tabs, 80));
        tabs.push(named("x"));
        assert!(!room(&tabs, 71));
        tabs[0].own = Some(own_place(0));
        assert!(room(&tabs, 71));
    }

    #[test]
    fn a_name_too_wide_is_not_given() {
        // Two tabs in the window: each may be 254 wide with its room, not 255.
        assert!(name_fits(244, 2, None) && !name_fits(245, 2, None));
        // In a window of its own 300 wide: 249 at most with its room.
        assert!(name_fits(239, 2, Some(300)) && !name_fits(240, 2, Some(300)));
    }

    #[test]
    fn an_own_window_is_always_shown_on_the_screen() {
        let screen = (1024, 768);
        assert_eq!(own_shown((100, 100, 300, 95), screen), (100, 100, 300, 95));
        // The client's own: no size, or one larger than the screen; past the right side; past the bottom.
        assert_eq!(own_shown((10, 10, 0, 0), screen), (10, 10, 300, 95));
        assert_eq!(own_shown((10, 10, 2000, 900), screen), (10, 10, 300, 95));
        assert_eq!(own_shown((1500, 10, 300, 95), screen), (724, 10, 300, 95));
        assert_eq!(own_shown((10, 900, 300, 95), screen), (10, 100, 300, 95));
        // This client's: anywhere else out of sight.
        assert_eq!(own_shown((-400, -50, 300, 95), screen), (0, 0, 300, 95));
        assert_eq!(own_shown((900, 700, 300, 95), screen), (724, 673, 300, 95));
    }

    #[test]
    fn an_own_window_grows_by_steps_between_its_least_and_its_most() {
        assert_eq!(own_resized(0, 0), (280, 74));
        assert_eq!(own_resized(300, 95), (280, 74));
        assert_eq!(own_resized(312, 106), (312, 106));
        assert_eq!(own_resized(343, 137), (312, 106));
        assert_eq!(own_resized(9000, 9000), (504, 362));
    }

    #[test]
    fn an_own_window_let_go_over_the_chat_goes_back() {
        let chat = (0, 644, 600, 124);
        assert!(own_dropped_in((0, 644, 300, 95), chat));
        assert!(own_dropped_in((600, 600, 300, 95), chat));
        // Its bottom above the chat, or under it; its left side out of the chat's width.
        assert!(!own_dropped_in((100, 100, 300, 95), chat));
        assert!(!own_dropped_in((100, 700, 300, 95), chat));
        assert!(!own_dropped_in((601, 644, 300, 95), chat));
    }

    #[test]
    fn a_tab_is_torn_out_past_five_points() {
        assert!(!torn((10.0, 10.0), (15.0, 5.0)));
        assert!(torn((10.0, 10.0), (16.0, 10.0)));
        assert!(torn((10.0, 10.0), (10.0, 4.0)));
    }
}
