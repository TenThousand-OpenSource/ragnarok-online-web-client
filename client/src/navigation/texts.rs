//! What the navigation writes and reads (the client's `ShowRouteResult`, `BuildTargetRouteLines`,
//! `LoadTargetPreview`, the lists of `UINavigationV4Wnd`, the commands `/navi` and `/navi2`, the links
//! `<NAVI>` of a text and `<NAVIL>` of the chat, and the packet that starts a navigation).

use super::guide::{Guide, Mode};
use super::search::atoi;
use super::{Atlas, Target, What};

/// The line of the client's message table that says how an attempt ended (`ShowRouteResult`), and the same
/// in this client's words until the table is read. None for a number the client has no line for ("NoMsg.").
pub fn result_line(result: i32) -> Option<(usize, &'static str)> {
    Some(match result {
        -100 => (2294, "The purpose is unclear"),
        -99 => (2295, "Does not meet the map requirement"),
        -98 => (2296, "Information Failure | Change settings"),
        -97 => (2297, "Failed to set info for location!"),
        -96 => (2298, "Failed to find a path"),
        -95 => (2299, "Failed to find players"),
        -94 => (2300, "No Information"),
        -93 => (2301, "Map doesn't support directions"),
        -92 => (2302, "Please specify target goals."),
        -90 => (4692, "Navigation is not possible from inside this place."),
        0 => (2303, "Found"),
        1 => (2304, "Directions were started"),
        2 => (2305, "Is the map that you're looking for mob"),
        3 => (2306, "Map appears on the guide you are looking for"),
        4 => (2307, "Please navigate using the item"),
        5 => (2308, "To find the location, follow the arrows"),
        6 => (2309, "Arrived at the target map"),
        7 => (2310, "Arrived on the map that has the Npc you're looking for. Follow the arrows to go to that NPC."),
        8 => (2311, "You have arrived at the mob you were looking for"),
        9 => (2313, "Please go to indicated direction."),
        10 => (2312, "You have reached your goal"),
        11 => (2314, "The goal has been reached"),
        _ => return None,
    })
}

/// The line of the message table for how many of a monster a map has (`UINavigationV4Wnd::MobName`): over 99
/// "Very High", over 49 "High", from 20 "Average", from 5 "Low", else "Very Low"; none for no count.
pub fn density_line(many: i32) -> Option<(usize, &'static str)> {
    Some(match many {
        ..=0 => return None,
        100.. => (2449, "Very High"),
        50..=99 => (2450, "High"),
        20..=49 => (2451, "Average"),
        5..=19 => (2452, "Low"),
        _ => (2453, "Very Low"),
    })
}

/// What a monster is, out of the number its table keeps (`BuildTargetRouteLines`): the lines of the message
/// table for its element (with the element's level to fill in), its race and its size.
pub fn monster_lines(what: i32) -> ((usize, i32), usize, usize) {
    let element = (what >> 16) & 0xff;
    let race = what & 0xff;
    let element_line = if (1..=9).contains(&(element % 10)) { 2262 + (element % 10) as usize } else { 2272 };
    let race_line = if (1..=9).contains(&race) { 2275 + race as usize } else { 2285 };
    let size_line = match what & 0xff00 {
        0x100 => 2273,
        0x200 => 2274,
        _ => 2275,
    };
    ((element_line, element / 20), race_line, size_line)
}

/// A name cut into the lines the window writes beside a target's picture (`AddWrappedRouteLines`): in square
/// brackets, eleven letters to a line, one line more for every twelve letters.
pub fn wrapped(name: &str) -> Vec<String> {
    let letters: Vec<char> = std::iter::once('[').chain(name.chars()).collect();
    let mut lines: Vec<String> = (0..=letters.len() / 12).map(|line| letters.iter().skip(11 * line).take(11).collect()).collect();
    if let Some(last) = lines.last_mut() {
        last.push(']');
    }
    lines
}

/// The row of the window's first list for a group of what was found (`FillRouteGroups`): how many it has and
/// the name of its first, with the client's number of that one's kind for the picture before it.
pub fn group_row(atlas: &Atlas, found: &[Target], group: &[usize]) -> Option<(i32, String)> {
    let first = found.get(*group.first()?)?;
    let name = if matches!(first.what, What::Place(_)) && !first.name.is_empty() { first.name.as_str() } else { atlas.name_of(first.what) };
    Some((first.what.kind(), format!("[{}]{name}", group.len())))
}

/// The row of the window's second list for a member of the group picked (`SelectRoute`): a monster listed
/// under its map by its name, anything else by where it is; `density` writes how many of a monster there are.
pub fn member_row(atlas: &Atlas, target: &Target, density: impl Fn(i32) -> Option<String>) -> (i32, String) {
    let many = |mob: usize| density(atlas.mobs[mob].many()).map(|words| format!(" ({words})")).unwrap_or_default();
    let map = &atlas.maps[atlas.map_of(target.what)];
    let text = match target.what {
        What::Mob(mob) if target.under_its_map() => format!("{}{}", atlas.mobs[mob].name, many(mob)),
        What::Mob(mob) => format!("{}{}", map.name, many(mob)),
        What::Npc(npc) => format!("{} [{},{}]", map.name, atlas.npcs[npc].x, atlas.npcs[npc].y),
        What::Map(_) => map.title.clone(),
        What::Place(_) => format!("{} [{},{}]", map.title, target.x, target.y),
        What::Node(_) => String::new(),
    };
    (target.what.kind(), text)
}

/// A line the window writes over a target's picture (`NaviRouteLine`): where, the words, and whether it is
/// the red word for a boss.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub boss: bool,
}

/// What the window writes of the target looked at (`BuildTargetRouteLines`): its name cut into lines from
/// (141, 46), 14 apart; under an NPC and a place the cell, at (142, 149); under a monster its level, its
/// element with the element's level, its race and its size, 16 apart from (142, 77), and "Boss" for one
/// (kind 301). `words` gives a line of the client's message table.
pub fn labels(atlas: &Atlas, target: &Target, words: impl Fn(usize, &str) -> String) -> Vec<Label> {
    let line = |x: i32, y: i32, text: String| Label { x, y, text, boss: false };
    let named = |name: &str| wrapped(name).into_iter().enumerate().map(|(row, text)| line(141, 46 + 14 * row as i32, text)).collect::<Vec<_>>();
    match target.what {
        What::Map(map) => named(&atlas.maps[map].title),
        What::Place(map) => {
            let mut lines = named(&atlas.maps[map].name);
            lines.push(line(142, 149, format!("( {} {} )", target.x, target.y)));
            lines
        }
        What::Npc(npc) => {
            let mut lines = named(&atlas.npcs[npc].name);
            lines.push(line(142, 149, format!("( {} {} )", atlas.npcs[npc].x, atlas.npcs[npc].y)));
            lines
        }
        What::Mob(mob) => {
            let mob = &atlas.mobs[mob];
            let ((element, level), race, size) = monster_lines(mob.y);
            let mut lines = named(&mob.name);
            lines.push(line(142, 77, format!("Lv. {}", mob.x)));
            lines.push(line(142, 93, crate::numbers::filled(&words(element, "%d"), &[&level.to_string()])));
            lines.push(line(142, 109, words(race, "")));
            lines.push(line(142, 125, words(size, "")));
            if mob.kind == 301 {
                lines.push(Label { x: 142, y: 149, text: words(2230, "Boss"), boss: true });
            }
            lines
        }
        What::Node(_) => Vec::new(),
    }
}

/// The picture a target is shown with (`LoadTargetPreview`): the small map of a map with one (kinds 5001 and
/// 5005), else the client's picture for a place indoors.
#[derive(Debug, Clone, PartialEq)]
pub enum Picture {
    Map(String),
    Indoors,
}

/// The picture for the target looked at, or none with nothing looked at.
pub fn picture(atlas: &Atlas, target: Option<&Target>) -> Option<Picture> {
    let map = &atlas.maps[atlas.map_of(target?.what)];
    Some(if map.kind == 5001 || map.kind == 5005 { Picture::Map(map.name.clone()) } else { Picture::Indoors })
}

/// How much smaller a map is drawn to fit a square, and how far in from the square's left and bottom it then
/// begins (`FitToSquare`): the longer side fills the square.
pub fn fitted(width: i32, height: i32, size: f32) -> (f32, i32, i32) {
    let longer = width.max(height).max(1) as f32;
    let scale = size / longer;
    match width.cmp(&height) {
        std::cmp::Ordering::Greater => (scale, 0, ((width - height) as f32 * 0.5 * scale) as i32),
        std::cmp::Ordering::Less => (scale, ((height - width) as f32 * 0.5 * scale) as i32, 0),
        std::cmp::Ordering::Equal => (scale, 0, 0),
    }
}

/// Where a cell of a map is in a square picture of it, from the picture's left and top (`MapCellToView`).
pub fn on_view(width: i32, height: i32, cell: (i32, i32), size: f32) -> (i32, i32) {
    let (scale, left, bottom) = fitted(width, height, size);
    ((cell.0 as f32 * scale + left as f32) as i32, (size as i32 as f32 - (cell.1 as f32 * scale + bottom as f32)) as i32)
}

/// The cell under a place pressed on the map of the window's third face (`UINavigationV4Wnd::OnLBtnUp`): the
/// place counted from the map's left and top, of 266 each way; none outside the map.
pub fn pressed_cell(width: i32, height: i32, (x, y): (i32, i32)) -> Option<(i32, i32)> {
    if !(0..=266).contains(&x) || !(0..=266).contains(&y) {
        return None;
    }
    let size = width.max(height);
    let scale = size as f32 / 265.0;
    let difference = width - height;
    let (left, bottom) = if difference > 0 { (0, (difference as f32 * 0.5).abs() as i32) } else { ((difference as f32 * 0.5).abs() as i32, 0) };
    let cell = ((x as f32 * scale - left as f32) as i32, (size as f32 - y as f32 * scale - bottom as f32) as i32);
    (cell.0 >= 0 && cell.1 >= 0 && width >= cell.0 && height >= cell.1).then_some(cell)
}

impl Guide {
    /// The three lines over the map of the way (`DestinationTitle`, `DestinationText`, `RouteText`): what the
    /// client calls the map of the way out gone to (or of the place sought), that map with the cell, and
    /// where the navigation ends; `destination` is the word of line 3585 of the message table.
    pub fn way_lines(&self, atlas: &Atlas, destination: &str) -> [String; 3] {
        let node = self.node.and_then(|node| self.nodes.get(node)).map(|link| &atlas.links[*link]);
        let goal = self.goal.map(|(map, cell)| (&atlas.maps[map], cell));
        let (title, at) = match (node, goal, self.mode) {
            (Some(link), _, _) => (atlas.maps[link.map].title.clone(), format!("{} {} {}", atlas.maps[link.map].name, link.x, link.y)),
            (None, Some((map, cell)), Mode::Going) => (map.title.clone(), format!("{} {} {}", map.name, cell.0, cell.1)),
            _ => (String::new(), String::new()),
        };
        let end = match goal {
            Some((map, cell)) if cell.0 > 0 => format!("[{destination}] -> {} {} {}", map.name, cell.0, cell.1),
            Some((map, _)) => format!("[{destination}] {}", map.name),
            None => String::new(),
        };
        [title, at, end]
    }

    /// The two lines of the small window (`SelectedNodeText`, `SubDestinationText`): where to go on this map,
    /// and where the navigation ends.
    pub fn small_lines(&self, atlas: &Atlas) -> [String; 2] {
        let [_, at, _] = self.way_lines(atlas, "");
        let end = match self.goal.map(|(map, cell)| (&atlas.maps[map], cell)) {
            Some((map, cell)) if cell.0 > 0 => format!(">.{} {} {}", map.name, cell.0, cell.1),
            Some((map, _)) => format!(">.{}", map.name),
            None => String::new(),
        };
        [if at.is_empty() { at } else { format!("{at}...") }, end]
    }
}

/// What the World Map marks the maps of what was found with (`CWorldViewMgr::AddLevel`, `LevelAreaAt`): each
/// map once, with the name of its picture among `worldview_interface\wv_Sreach….bmp` — `Map` for the map
/// itself, `Mob1` to `Mob5` by how many of the monster found it has, `Npc` and `Npc_Mob1` to `Npc_Mob5` where
/// an NPC was found there too. `found` is `Guide::found_maps`.
pub fn map_marks(found: &[(&str, i32)]) -> Vec<(String, String)> {
    let mut levels: Vec<(&str, i32)> = Vec::new();
    for (map, level) in found {
        match levels.iter_mut().find(|(known, _)| known == map) {
            None => levels.push((map, *level)),
            Some((_, current)) if *level == 10000 => *current += if *current < 10000 { 10000 } else { 0 },
            Some((_, current)) if *level != 0 && *current < 10000 => *current = (*current).max(*level),
            Some((_, current)) if *level != 0 && *current - 10000 < *level => *current = level + 10000,
            Some(_) => {}
        }
    }
    let named = |level: i32| {
        let many = match level % 10000 {
            100.. => 5,
            50..=99 => 4,
            20..=49 => 3,
            5..=19 => 2,
            1..=4 => 1,
            _ => 0,
        };
        match (level >= 10000, many) {
            (false, 0) => "Map".to_owned(),
            (false, many) => format!("Mob{many}"),
            (true, 0) => "Npc".to_owned(),
            (true, many) => format!("Npc_Mob{many}"),
        }
    };
    levels.into_iter().map(|(map, level)| (map.to_owned(), named(level))).collect()
}

/// A cell written as `x/y` (`CNavigationManager::ParseCell`): both from 0 to 500.
pub fn cell(text: &str) -> Option<(i32, i32)> {
    let (x, y) = text.split_once('/')?;
    if x.is_empty() || y.is_empty() {
        return None;
    }
    let (x, y) = (atoi(x), atoi(y));
    ((0..=500).contains(&x) && (0..=500).contains(&y)).then_some((x, y))
}

/// The three switches of `/navi2`, written as `1/0/1` (`ParseFlags`), as the ways out that may be taken
/// (`mode` of `Guide::start`): the first for scrolls (2), the second for zeny (1), the third for the airship (4).
pub fn switches(text: &str) -> Option<u8> {
    let mut parts = text.splitn(3, '/');
    let (scrolls, zeny, airship) = (parts.next()?, parts.next()?, parts.next()?);
    if scrolls.is_empty() || zeny.is_empty() || airship.is_empty() {
        return None;
    }
    let (scrolls, zeny, airship) = (atoi(scrolls), atoi(zeny), atoi(airship));
    [scrolls, zeny, airship].iter().all(|switch| (0..=1).contains(switch)).then_some((zeny + scrolls * 2 + airship * 4) as u8)
}

/// What the server's number for the ways a navigation may take says (`CGameMode::ReceiveNavigation`, the
/// same in a link): hundreds for those that cost zeny (1), tens for scrolls (2), units for the airship (4).
pub fn mode_of(mut code: i32) -> u8 {
    let mut mode = 0;
    if code >= 100 {
        mode = 1;
        code -= 100;
    }
    if code >= 10 {
        mode += 2;
        code -= 10;
    }
    if code >= 1 { mode + 4 } else { mode }
}

/// What a link `<NAVI>` of a text asks for when pressed (`OpenTextButtonNavigation`).
#[derive(Debug, PartialEq)]
pub enum Asked {
    /// A navigation (`Guide::start`): the map, the kind of target, the ways out that may be taken, whether the
    /// window stays away, the cell and the monster's job.
    Start { map: String, kind: i32, mode: u8, hide: bool, x: i32, y: i32, id: i32 },
    /// A search (`Guide::look_for`) for a text, narrowed to a kind.
    Search { text: String, kind: i32 },
}

/// Reads the fields of a link `<NAVI>` (what stands in its `<INFO>`, cut at the commas): a map and a cell
/// alone lead to the NPC there; with more, the kind of target, the ways that may be taken (or -222 for a
/// search), whether the window is shown, and a monster's job.
pub fn linked(info: &[String]) -> Option<Asked> {
    let map = info.first()?.clone();
    let number = |place: usize| info.get(place).map_or(0, |field| atoi(field));
    let (x, y) = (number(1), number(2));
    if info.len() <= 3 {
        return Some(Asked::Start { map, kind: 2, mode: 5, hide: false, x, y, id: 1002 });
    }
    let (kind, code) = (number(3), number(4));
    if code == -222 {
        return Some(Asked::Search { text: if kind == 3 { format!("$$DB {map}") } else { map }, kind });
    }
    Some(Asked::Start { map, kind, mode: mode_of(code), hide: number(5) == 0, x, y, id: number(6) })
}

const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// A number as a link of the chat writes it (`EncodeNavigationNumber`): in sixty-two digits, two at least.
fn encoded(mut number: i32) -> String {
    let mut digits = Vec::new();
    loop {
        digits.push(DIGITS[(number % 62) as usize]);
        number /= 62;
        if number < 62 {
            break;
        }
    }
    digits.push(DIGITS[number as usize]);
    digits.reverse();
    String::from_utf8(digits).unwrap_or_default()
}

/// A place as a link of the chat (`UINavigationV4Wnd::InsertNavigationLink`): `<NAVIL>`, the map, the cell's
/// two numbers of two digits each (1000 for a cell that is none), `</NAVIL>`.
pub fn chat_link(map: &str, x: i32, y: i32) -> String {
    let number = |value: i32| encoded(if value >= 0 { value.min(3843) } else { 1000 });
    format!("<NAVIL>{map}{}{}</NAVIL>", number(x), number(y))
}

/// Reads what stands between `<NAVIL>` and `</NAVIL>` (`UINaviLinkButton::OnLBtnDown`): the map, and the cell
/// out of its last four letters.
pub fn chat_linked(data: &str) -> Option<(&str, i32, i32)> {
    let at = data.len().checked_sub(4).filter(|at| data.is_char_boundary(*at) && *at > 0)?;
    let number = |digits: &str| digits.bytes().try_fold(0, |number, digit| Some(number * 62 + DIGITS.iter().position(|known| *known == digit)? as i32));
    Some((&data[..at], number(&data[at..at + 2])?, number(&data[at + 2..])?))
}

/// A line of chat cut at its links to places: the words between them, and for each link what stands between
/// its tags (`Err`). A link that is not closed is words.
pub fn chat_parts(line: &str) -> Vec<Result<&str, &str>> {
    let mut parts = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find("<NAVIL>") {
        let Some(length) = rest[start..].find("</NAVIL>") else { break };
        if start > 0 {
            parts.push(Ok(&rest[..start]));
        }
        parts.push(Err(&rest[start + 7..start + length]));
        rest = &rest[start + length + 8..];
    }
    if !rest.is_empty() {
        parts.push(Ok(rest));
    }
    parts
}

/// How a link of the chat is shown (`FormatNavigationLink`): the map with its cell, or the map alone.
pub fn chat_link_words(map: &str, x: i32, y: i32) -> String {
    if x > 999 && y > 999 { format!("<{map}>") } else { format!("<{map} {x},{y}>") }
}
