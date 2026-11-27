//! The old party booking as rules (the client's `PartyBookingMgr`, `UISeekPartyMBWnd`, `UISeekPartyListWnd`,
//! `UISeekPartyWnd`, `UIJobListWnd` and the `ReceivePartyBooking*` of `GameModeHandlers.cpp`): which jobs an ad
//! may name and how they are grouped, the number a place is told by, what the window of registering and the
//! list's search refuse, the list of ads as the client keeps it, and what the window of one's own ad shows.
//!
//! A job of an ad is not a job of the game: it is a number of the booking's own, 1 to 63, whose name is a
//! line of the message table (`job_line`). A place is a region and a map of it, as the client's
//! `party_booking_config.lub` lists them (`Region`).

use std::collections::BTreeMap;

use crate::net::booking::{Ad, JOBS};

mod own;
#[cfg(test)]
mod tests;

pub use own::*;

/// The groups the jobs are picked in (`UISeekPartyMBWnd::CreateJobChecks`: `starts` and `counts`): the first
/// job of each and how many it has. The four of the first are roles.
pub const GROUPS: [(i16, i16); 8] = [(1, 4), (5, 6), (11, 13), (24, 6), (30, 7), (37, 6), (43, 13), (56, 8)];

/// The line of the message table that names the first group (`MSI_PARTY_BOOKING_TYPE`); the others follow.
pub const GROUP_LINE: usize = 1755;

/// The names on the page of the fields the level is typed into, of the list and of registering.
pub const LEVEL: &str = "booking-level";
pub const OWN_LEVEL: &str = "booking-own-level";

/// How many ads a search asks for at once, and how many rows the list shows.
pub const PAGE: i16 = 10;
pub const ROWS: i32 = 5;

/// The lines of the message table the booking says (their numbers as the client's code has them).
pub mod line {
    /// "Party Recruitment", "Party Booking List": the titles of the two windows.
    pub const REGISTER: usize = 1688;
    pub const LIST: usize = 1689;
    /// "Notice": the box that is ticked to be told of new ads.
    pub const ALARM: usize = 1694;
    /// The heads of the list's columns: the time, the name, the level, the map.
    pub const TIME: usize = 1697;
    pub const NAME: usize = 407;
    pub const LEVEL: usize = 408;
    pub const MAP: usize = 1698;
    /// The title of the help.
    pub const HELP: usize = 1716;
    /// "Party ad has been added."
    pub const ADDED: usize = 1753;
    /// "Recruit party members": the title of the window of one's own ad.
    pub const OWN: usize = 1754;
    /// "Recruit", and what it says when pointed at.
    pub const RECRUIT: usize = 1763;
    pub const RECRUIT_TIP: usize = 1764;
    /// "Searching - ": the list's title after a search.
    pub const SEARCHING: usize = 1765;
    /// "Select All": any job, in the list's two boxes of jobs.
    pub const ANY: usize = 1766;
    /// What is refused: no job left sought, no job ticked, more than six, a level that is no number.
    pub const NONE_LEFT: usize = 1767;
    pub const NO_JOB: usize = 1768;
    pub const TOO_MANY: usize = 1769;
    pub const NOT_NUMBER: usize = 1770;
    /// The server's refusals: registering failed, nothing found, the ad not taken away, no ad there.
    pub const REGISTER_FAILED: usize = 1826;
    pub const NOTHING_FOUND: usize = 1827;
    pub const DELETE_FAILED: usize = 1828;
    pub const NO_AD: usize = 1829;
    /// "Chat Dialog": beside the button that opens a talk with the leader.
    pub const TALK: usize = 1834;
    /// "Advertising is pending registration."
    pub const PENDING: usize = 1941;
    /// What the field of the level says when pointed at.
    pub const LEVEL_TIP: usize = 1949;
    /// "Level, enter a number between 1 and %d."
    pub const LEVEL_RANGE: usize = 2601;
}

/// The line of the message table that names a job of an ad (`PartyBookingMgr::MakeEntry`: `job + 0x6AF` for
/// the roles, `job + 0x658` for the others); none for a job nobody is sought for.
pub fn job_line(job: i16) -> Option<usize> {
    match job {
        0..=4 => Some(job as usize + 0x6AF),
        5.. => Some(job as usize + 0x658),
        _ => None,
    }
}

/// The jobs of a group, in their order.
pub fn jobs_of(group: usize) -> impl Iterator<Item = i16> {
    let (first, count) = GROUPS.get(group).copied().unwrap_or((0, 0));
    first..first + count
}

/// A region of the client's list of places with its maps, each with the colour its name is written in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Region {
    pub name: String,
    pub maps: Vec<(String, [u8; 3])>,
}

/// The number a place is told by: the region's place in the client's list, whose first is "all maps", in the
/// high byte and the map's place in the region in the low one (`UISeekPartyMBWnd::SendMsg`: `(m_type + 1) *
/// 256 | m_subType`, its regions counted without the first; `UISeekPartyListWnd::SendMsg`: `m_region << 8 |
/// m_mapIndex`, and 0 for any place).
pub fn place_code(region: usize, map: usize) -> i16 {
    if region == 0 { 0 } else { ((region << 8) | (map & 0xff)) as i16 }
}

/// What an ad's place is called: the region and the map (`PartyBookingMgr::MakeEntry`). None for a place the
/// list does not have, whose ad the client does not show. (Nor is one of region 0 shown here: the client's
/// table of maps by region is filled by code that was not found, and no ad is registered for region 0.)
pub fn location(regions: &[Region], place: i16) -> Option<String> {
    let (region, map) = ((place >> 8) as usize, (place & 0xff) as usize);
    let region = regions.get(region).filter(|_| region != 0)?;
    Some(format!("{} {}", region.name, region.maps.get(map)?.0))
}

/// The levels an ad is for as the list writes them (`MakeEntry`: "%02d~%02d", fifteen levels from the one
/// asked, the highest level of the game at most).
pub fn levels(level: i16, highest: i32) -> String {
    format!("{:02}~{:02}", level, (i32::from(level) + 15).min(highest))
}

/// The time of an ad as the list writes it (`MakeEntry`: " %02d : %02d ").
pub fn clock(hour: u32, minute: u32) -> String {
    format!(" {hour:02} : {minute:02} ")
}

/// A number typed as C's `atoi` reads it: the digits it begins with, after blanks and a sign; 0 for none.
fn atoi(text: &str) -> i32 {
    let text = text.trim_start();
    let (sign, digits) = match text.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, text.strip_prefix('+').unwrap_or(text)),
    };
    let value = digits.bytes().take_while(u8::is_ascii_digit).fold(0i64, |value, digit| (value * 10 + i64::from(digit - b'0')).min(i64::from(i32::MAX)));
    sign * value as i32
}

/// What the two windows refuse, each said in a box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// No job is ticked.
    NoJob,
    /// More than six are: how many.
    TooMany(usize),
    /// The level typed has something that is no figure.
    NotNumber,
    /// The level is not one of the game: its highest.
    Level(i32),
}

impl Refused {
    /// The line of the message table that says it, and the number written into the line's blank.
    pub fn said(self) -> (usize, Option<i64>) {
        match self {
            Refused::NoJob => (line::NO_JOB, None),
            Refused::TooMany(count) => (line::TOO_MANY, Some(count as i64)),
            Refused::NotNumber => (line::NOT_NUMBER, None),
            Refused::Level(highest) => (line::LEVEL_RANGE, Some(highest.into())),
        }
    }
}

/// The level typed into either window, as both read it: a letter that is not between '/' and '9' refuses it
/// (the client's own test, which lets a slash by), and what is left is read as `atoi` reads.
fn level_typed(typed: &str) -> Result<i32, Refused> {
    if typed.bytes().any(|letter| !(b'/'..=b'9').contains(&letter)) {
        return Err(Refused::NotNumber);
    }
    Ok(atoi(typed))
}

/// An ad as the window of registering makes it of what is ticked and typed (`UISeekPartyMBWnd::SendMsg`,
/// `ID_OK`): the level, the place and the six jobs; or why not. `ticked` are the jobs ticked, in their order;
/// `region` and `map` are what the two boxes of places show, the regions counted without "all maps".
pub fn registered(ticked: &[i16], level: &str, highest: i32, region: usize, map: usize) -> Result<(i16, i16, [i16; JOBS]), Refused> {
    if ticked.is_empty() {
        return Err(Refused::NoJob);
    }
    if ticked.len() > JOBS {
        return Err(Refused::TooMany(ticked.len()));
    }
    let level = level_typed(level)? as i16;
    if level < 1 || i32::from(level) > highest {
        return Err(Refused::Level(highest));
    }
    let mut jobs = [-1; JOBS];
    jobs[..ticked.len()].copy_from_slice(ticked);
    Ok((level, place_code(region + 1, map), jobs))
}

/// What a search looks for: the level (0 for any), the place (0 for any) and the job (-1 for any).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sought {
    pub level: i16,
    pub place: i16,
    pub job: i16,
}

impl Sought {
    /// What the list's window looks for before its first search (`UISeekPartyListWnd`'s constructor: -5).
    pub const NOTHING_YET: Sought = Sought { level: -5, place: -5, job: -5 };
    /// What the manager looks for before the first search (`PartyBookingMgr`'s constructor: -1).
    pub const ANYTHING: Sought = Sought { level: -1, place: -1, job: -1 };

    /// Whether an ad is one of those looked for (`UISeekPartyListWnd::MatchesFilter`, and the same test in
    /// `PartyBookingMgr::OnAdvertisement`): its level within fifteen over the one sought, its place the one
    /// sought, the job sought among its six.
    pub fn takes(&self, ad: &Ad) -> bool {
        let level = self.level <= 0 || (self.level <= ad.level && i32::from(self.level) + 15 >= i32::from(ad.level));
        let place = self.place <= 0 || self.place == ad.place;
        level && place && (self.job <= 0 || ad.jobs.contains(&self.job))
    }
}

/// What the list's search looks for, of what is typed and picked (`UISeekPartyListWnd::SendMsg`,
/// `ID_PARTY_SEARCH`): `region` is the box of regions with "all maps" its first, `group` the box of the jobs'
/// groups with "all" its first, `job` the place picked in the box of that group's jobs.
pub fn sought(level: &str, highest: i32, (region, map): (usize, usize), (group, job): (usize, usize)) -> Result<Sought, Refused> {
    let level = level_typed(level)? as i16;
    if level < 0 || i32::from(level) > highest {
        return Err(Refused::Level(highest));
    }
    let job = match group.checked_sub(1).and_then(|group| GROUPS.get(group)) {
        Some((first, _)) => first + job as i16,
        None => -1,
    };
    Ok(Sought { level, place: place_code(region, map), job })
}

/// The list's title after a search (`UISeekPartyListWnd::SendMsg`): `searching` is the table's "Searching - ",
/// then what was sought, each named only when it is not "any".
pub fn title(searching: &str, level: i16, place: Option<(&str, &str)>, job: Option<&str>) -> String {
    let mut title = searching.to_owned();
    if level > 0 {
        title.push_str(&format!(" Level: {level}"));
    }
    if let Some((region, map)) = place {
        title.push_str(&format!(" Map: {region} {map}"));
    }
    if let Some(job) = job {
        title.push_str(&format!(" Job: {job}"));
    }
    title
}

/// An ad as the list shows it (the client's `PartyBookingEntry`, whose texts are written as it is drawn
/// here): who registered it, when, from which level, where and which jobs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub time: u32,
    pub level: i16,
    pub location: String,
    pub jobs: [i16; JOBS],
}

impl Entry {
    /// An ad as the list shows it; none for one whose place the client's list of places does not have.
    pub fn of(ad: &Ad, regions: &[Region]) -> Option<Entry> {
        Some(Entry { name: ad.name.clone(), time: ad.time, level: ad.level, location: location(regions, ad.place)?, jobs: ad.jobs })
    }
}

/// A row of the list: an ad, or the row after the last ad of a page that asks for the next page (the client's
/// `isEnd`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    Ad(Entry),
    More,
}

/// The ads as the client keeps them (`PartyBookingMgr`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    /// The rows of the list, by the number of their ad (`m_entries`).
    pub rows: BTreeMap<u32, Row>,
    /// Ads told of while the list had more pages to ask for: they wait for the last page (`m_pending`).
    pub waiting: BTreeMap<u32, Entry>,
    /// The server has more ads than it gave (the client's `m_lastPage`, true while it has more).
    pub more: bool,
    /// The last ad the server gave (`m_lastIndex`).
    pub last: i32,
    /// A search was answered once, and what was last sought: by these an ad told of while the list is closed
    /// is one to tell the player of (`m_filterEnabled`, `m_filterLevel`…).
    pub searched: bool,
    pub sought: Sought,
    /// The box "Notice" as it stood when the list was last closed by its OK (`m_alarm`).
    pub alarm: bool,
}

impl Default for Board {
    fn default() -> Self {
        Self { rows: BTreeMap::new(), waiting: BTreeMap::new(), more: true, last: -1, searched: false, sought: Sought::ANYTHING, alarm: false }
    }
}

impl Board {
    /// `PartyBookingMgr::Reset`: as the character is chosen anew.
    pub fn reset(&mut self) {
        (self.more, self.last) = (true, -1);
        self.clear();
    }

    /// `Clear`: a new search begins.
    pub fn clear(&mut self) {
        self.rows.clear();
        self.waiting.clear();
    }

    /// `Remove`: an ad is gone.
    pub fn remove(&mut self, index: u32) {
        self.rows.remove(&index);
        self.waiting.remove(&index);
    }

    /// `SetJobs`: an ad looks for other jobs now.
    pub fn set_jobs(&mut self, index: u32, jobs: [i16; JOBS]) {
        match (self.rows.get_mut(&index), self.waiting.get_mut(&index)) {
            (Some(Row::Ad(entry)), _) | (None, Some(entry)) => entry.jobs = jobs,
            _ => {}
        }
    }

    /// `FlushPending`: up to eleven of the ads that waited go into the list; whether some still wait.
    pub fn flush(&mut self) -> bool {
        for _ in 0..=10 {
            let Some((index, entry)) = self.waiting.pop_first() else { return false };
            self.rows.entry(index).or_insert(Row::Ad(entry));
            if self.waiting.is_empty() {
                return false;
            }
        }
        true
    }

    /// `AddEntries`: ads the server gave, each as the list shows it (none for one it does not show), and
    /// whether the server has more: with more to come, a row that asks for the next page follows the last.
    pub fn add(&mut self, ads: impl IntoIterator<Item = (u32, Option<Entry>)>, mut more: bool) {
        self.more = more;
        for (index, entry) in ads {
            let Some(entry) = entry else { continue };
            self.rows.entry(index).or_insert(Row::Ad(entry));
            self.last = index as i32;
            if !more {
                more = self.flush();
            }
        }
        if more {
            self.rows.entry((self.last + 1) as u32).or_insert(Row::More);
        }
    }

    /// `OnAdvertisement`: someone registered an ad. `list` is the list's window if it is open: what it last
    /// sought, and whether its box "Notice" is ticked. Whether the player is to be told (a sound and a line
    /// of the chat).
    pub fn advertised(&mut self, ad: &Ad, entry: Option<Entry>, list: Option<(Sought, bool)>) -> bool {
        let Some((sought, alarm)) = list else {
            return (!self.searched || self.sought.takes(ad)) && self.alarm;
        };
        if !sought.takes(ad) {
            return false;
        }
        if self.more {
            if let Some(entry) = entry {
                self.waiting.entry(ad.index).or_insert(entry);
            }
        } else {
            self.add([(ad.index, entry)], false);
        }
        alarm
    }
}

/// What the list does when its search is asked for (`UISeekPartyListWnd::SendMsg`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    /// Nothing: the same was sought last.
    Same,
    /// The ads that waited go into the list, and nothing is asked: the server has no further page.
    Flushed,
    /// The server is asked, for the ads after that one.
    Asked(i32),
}

/// The button Search: what was sought last is not asked again; anything else empties the list and asks from
/// the first ad.
pub fn search_pressed(board: &mut Board, last: Sought, now: Sought) -> Search {
    if last == now {
        return Search::Same;
    }
    board.clear();
    Search::Asked(0)
}

/// The row that asks for the next page: with more to come the server is asked for the ads after the last it
/// gave, otherwise the ads that waited are shown.
pub fn next_page(board: &mut Board) -> Search {
    if !board.more {
        board.flush();
        return Search::Flushed;
    }
    Search::Asked(board.last)
}
