//! The Adventurer's Agency: a party's notice, the forms the client posts to the game's web service and what
//! its answers mean, the levels typed into its windows, and what the server's answer to asking to join says.
//! The client's `CAdventurerAgencyMgr` with its works (`Source/Contents/Party/`: `AgencyRequests.cpp`,
//! `LoadPartyListAsyncWork.cpp`, `RegisterPartyAsyncWork.cpp`, the `…Completion.cpp` files,
//! `PartyJoinResult.cpp`) and its windows (`Source/UI/Party/UIAdvenPartyBoardWnd.cpp`,
//! `UIRegisterPartyWnd.cpp`). The answers are JSON, read by `json` here.

pub mod json;

use json::Value;

/// A party's notice (`CAdvenPartyInfo`): which roles it looks for, the levels it takes, its purpose (0 all, 1
/// quest, 2 field, 3 dungeon, 4 memorial dungeon, 5 paradise, 6 other), its note, and whose it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    pub tanker: bool,
    pub healer: bool,
    pub dealer: bool,
    pub assist: bool,
    pub least: i32,
    pub most: i32,
    pub purpose: i32,
    pub memo: String,
    pub account: u32,
    pub character: u32,
    pub name: String,
}

impl Default for Notice {
    /// As the client's `CAdvenPartyInfo` begins: every role wanted, every level.
    fn default() -> Self {
        Self {
            tanker: true,
            healer: true,
            dealer: true,
            assist: true,
            least: 0,
            most: 999,
            purpose: 0,
            memo: String::new(),
            account: 0,
            character: 0,
            name: String::new(),
        }
    }
}

impl Notice {
    /// The roles in the order the windows show them: tank, healer, dealer, support.
    pub fn roles(&self) -> [bool; 4] {
        [self.tanker, self.healer, self.dealer, self.assist]
    }

    /// The levels as a row writes them (`UIPartyInfoWnd::SetInfo`: `%d~%d` in ten bytes, so nine letters).
    pub fn levels(&self) -> String {
        format!("{}~{}", self.least, self.most).chars().take(9).collect()
    }

    /// A notice out of the service's answer (`CAdventurerAgencyMgr::LoadList`): what is not said is as
    /// `default` has it, but for the purpose, which is 1.
    fn read(entry: &Value) -> Self {
        let flag = |name: &str| entry.get(name).and_then(Value::number).is_none_or(|flag| flag as i64 != 0);
        let whole = |name: &str, otherwise: i32| entry.get(name).and_then(Value::number).map_or(otherwise, |number| number as i32);
        let id = |name: &str| {
            let number = entry.get(name).and_then(Value::number).unwrap_or(0.0);
            if (0.0..4_294_967_296.0).contains(&number) { number as u32 } else { u32::MAX }
        };
        let words = |name: &str| entry.get(name).and_then(Value::text).unwrap_or_default().to_owned();
        Self {
            tanker: flag("Tanker"),
            healer: flag("Healer"),
            dealer: flag("Dealer"),
            assist: flag("Assist"),
            least: whole("MinLV", 0),
            most: whole("MaxLV", 999),
            purpose: whole("Type", 1),
            memo: words("Memo"),
            account: id("AID"),
            character: id("GID"),
            name: words("CharName"),
        }
    }
}

/// The lines of the message table that name the purposes, 0 to 6 (`UIPartyInfoWnd::TypeName`).
pub const PURPOSES: [usize; 7] = [3377, 3360, 3361, 3362, 3363, 3364, 3365];
/// The lines that name the four roles (the tooltips of their buttons), and the pictures of each, on and off,
/// in the folder `UIadventurerAgency`.
pub const ROLES: [usize; 4] = [3420, 3491, 3444, 3508];
pub const ROLE_PICTURES: [[&str; 2]; 4] = [
    ["btn_job_def_on.bmp", "btn_job_def_off.bmp"],
    ["btn_job_heal_on.bmp", "btn_job_heal_off.bmp"],
    ["btn_job_atk_on.bmp", "btn_job_atk_off.bmp"],
    ["btn_job_buf_on.bmp", "btn_job_buf_off.bmp"],
];
/// The fields of the board's search and of a notice being written, as the page names them: the least level,
/// the most, the note.
pub const SEARCH_FIELDS: [&str; 3] = ["agency-least", "agency-most", "agency-memo"];
pub const NOTICE_FIELDS: [&str; 3] = ["agency-notice-least", "agency-notice-most", "agency-notice-memo"];
/// How many notices a page of the list shows.
pub const ROWS: usize = 10;

/// What the client asks of the service, each with the end of its address (`CAdventurerAgencyMgr::ServiceUrl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// The notices, a page of them.
    List,
    /// Those that fit what was searched for.
    Search,
    /// The player's own notice.
    Mine,
    /// One party's notice, before asking to join it.
    Info,
    /// To put up a notice, or to change it.
    Add,
    /// To take a notice down.
    Stop,
}

impl Request {
    pub fn path(self) -> &'static str {
        match self {
            Request::List => "/party/list",
            Request::Search => "/party/search",
            Request::Mine => "/party/get",
            Request::Info => "/party/info",
            Request::Add => "/party/add",
            Request::Stop => "/party/del",
        }
    }
}

/// Who asks: the account, the character, the name of the world (the char server picked) and the token the
/// login server gave for the web service.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Asker {
    pub account: u32,
    pub character: u32,
    pub world: String,
    pub token: String,
}

/// The fields of the form a request is posted as, in the client's order.
pub type Fields = Vec<(&'static str, String)>;

impl Asker {
    /// The four fields most requests begin with (`CLoadPartyListAsyncWork::Execute`).
    fn fields(&self) -> Fields {
        vec![("GID", self.character.to_string()), ("AID", self.account.to_string()), ("WorldName", self.world.clone()), ("AuthToken", self.token.clone())]
    }

    /// A page of the list.
    pub fn list(&self, page: i32) -> Fields {
        let mut fields = self.fields();
        fields.push(("page", page.to_string()));
        fields
    }

    /// A page of what fits a search: the levels always, a role only when it is asked for, the purpose unless
    /// it is "all", the note unless it is empty (`CLoadPartyListAsyncWork::Execute`, its search).
    pub fn search(&self, filter: &Notice, page: i32) -> Fields {
        let mut fields = self.fields();
        fields.push(("MinLV", filter.least.to_string()));
        fields.push(("MaxLV", filter.most.to_string()));
        for (name, asked) in [("Tanker", filter.tanker), ("Dealer", filter.dealer), ("Healer", filter.healer), ("Assist", filter.assist)] {
            if asked {
                fields.push((name, "1".to_owned()));
            }
        }
        if filter.purpose != 0 {
            fields.push(("Type", filter.purpose.to_string()));
        }
        if !filter.memo.is_empty() {
            fields.push(("Memo", filter.memo.clone()));
        }
        fields.push(("page", page.to_string()));
        fields
    }

    /// The player's own notice (`CLoadMyPartyAsyncWork`), and the taking down of the notice of that account
    /// (`CStopRecruitingAsyncWork`: the player's own, or a party's that turned out to be full).
    pub fn mine(&self) -> Fields {
        vec![("AID", self.account.to_string()), ("GID", self.character.to_string()), ("WorldName", self.world.clone()), ("AuthToken", self.token.clone())]
    }

    pub fn stop(&self, master: u32) -> Fields {
        vec![("AID", self.account.to_string()), ("WorldName", self.world.clone()), ("AuthToken", self.token.clone()), ("MasterAID", master.to_string())]
    }

    /// The notice of that account (`CLoadPartyInfoAsyncWork`).
    pub fn info(&self, of: u32) -> Fields {
        vec![
            ("AID", self.account.to_string()),
            ("GID", self.character.to_string()),
            ("AuthToken", self.token.clone()),
            ("WorldName", self.world.clone()),
            ("QueryAID", of.to_string()),
        ]
    }

    /// A notice put up under the player's own name (`CRegisterPartyAsyncWork::Execute`).
    pub fn add(&self, notice: &Notice, name: &str) -> Fields {
        let flag = |on: bool| if on { "1" } else { "0" }.to_owned();
        vec![
            ("AID", self.account.to_string()),
            ("GID", self.character.to_string()),
            ("AuthToken", self.token.clone()),
            ("WorldName", self.world.clone()),
            ("CharName", name.to_owned()),
            ("MinLV", notice.least.to_string()),
            ("MaxLV", notice.most.to_string()),
            ("Tanker", flag(notice.tanker)),
            ("Healer", flag(notice.healer)),
            ("Dealer", flag(notice.dealer)),
            ("Assist", flag(notice.assist)),
            ("Type", notice.purpose.to_string()),
            ("Memo", notice.memo.clone()),
        ]
    }
}

/// What parts the fields of a posted form (`multipart/form-data`): a line no field of the agency can hold.
pub const BOUNDARY: &str = "----roweb-agency-7f3a9c1e";

/// The fields as the body of a `multipart/form-data` form, which is what the web service reads.
pub fn form(fields: &Fields) -> String {
    let mut body = String::new();
    for (name, value) in fields {
        body.push_str(&format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"));
    }
    body.push_str(&format!("--{BOUNDARY}--\r\n"));
    body
}

/// An answer the client goes by: none when it is no JSON, or says with its `Type` of 2 or 3 that the request
/// failed.
fn accepted(answer: &str) -> Option<Value> {
    let root = json::parse(answer)?;
    let kind = root.get("Type").and_then(Value::number).map_or(0, |kind| kind as i64);
    (kind != 2 && kind != 3).then_some(root)
}

/// The notices of a page and how many pages there are, if the answer says (`CompleteList`).
pub fn listed(answer: &str) -> Option<(Vec<Notice>, Option<i32>)> {
    let root = accepted(answer)?;
    let notices = root.get("data").map(|data| data.items().iter().map(Notice::read).collect()).unwrap_or_default();
    let pages = root.get("totalPage").and_then(Value::number).map(|pages| pages as i32).filter(|pages| *pages != 0);
    Some((notices, pages))
}

/// The player's own notice, or none when there is none (`CompleteMyParty`); the outer none is an answer the
/// client does nothing at.
pub fn mine(answer: &str) -> Option<Option<Notice>> {
    let root = accepted(answer)?;
    Some(root.get("data").filter(|data| !matches!(data, Value::Null)).map(Notice::read))
}

/// One party's notice, or none when it has none any more (`CompletePartyInfo`: the first of `data`).
pub fn info(answer: &str) -> Option<Option<Notice>> {
    let root = accepted(answer)?;
    Some(root.get("data").and_then(|data| data.items().first()).map(Notice::read))
}

/// Whether a notice was put up, or taken down: `Some(false)` for an answer that says no, none for one that
/// is no JSON (`CompleteRegistration`, `CompleteStopRecruiting`).
pub fn done(answer: &str) -> Option<bool> {
    let root = json::parse(answer)?;
    let kind = root.get("Type").and_then(Value::number).map_or(0, |kind| kind as i64);
    Some(kind != 2 && kind != 3)
}

/// The levels the windows begin with, for a character of that level: five under it, but 1 at least, to five
/// over it (`UIAdvenPartyBoardWnd::OnCreate`, `UIRegisterPartyWnd::OnCreate`).
pub fn levels_around(level: i32) -> (i32, i32) {
    ((level - 5).max(1), level + 5)
}

/// A level as it was typed: its number when it is digits and nothing else, or the one the window began with
/// (`UIRegisterPartyWnd::NormalizeLevels`, `UIAdvenPartyBoardWnd::Search`).
pub fn level_typed(typed: &str, otherwise: i32) -> i32 {
    if !typed.is_empty() && typed.bytes().all(|digit| digit.is_ascii_digit()) { typed.parse().unwrap_or(i32::MAX) } else { otherwise }
}

/// Whether the arrows of the list's pages are live, before and after, on that page of so many
/// (`UIAdvenPartyBoardWnd::UpdatePagination`).
pub fn arrows(page: i32, pages: i32) -> (bool, bool) {
    (page.min(pages) != 1, page < pages)
}

/// Which name a line about joining takes into its blank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Named {
    Nobody,
    /// The first name of the packet (what the client reads as the party's).
    First,
    /// The second (what it reads as the player's).
    Second,
}

/// What ZC_PARTY_JOIN_REQ_ACK_FROM_MASTER says in the chat, by its status and by whether the player leads a
/// party: the line of the message table, the name it takes, whether it is good news (the client's 0xFFFF64;
/// the others are red), and whether the notice of that party is to be taken down, the party being full
/// (`ReceivePartyJoinResult`).
pub fn join_result(status: u32, leader: bool) -> Option<(usize, Named, bool, bool)> {
    let said = |line, named| Some((line, named, false, false));
    match status {
        0 if leader => Some((3501, Named::First, true, false)),
        0 => Some((3498, Named::Second, true, false)),
        1 if leader => said(3502, Named::First),
        1 => said(3499, Named::Second),
        2 => said(3497, Named::Nobody),
        3 => said(3442, Named::Nobody),
        4 => said(3496, Named::Nobody),
        5 | 6 => said(3492, Named::Nobody),
        7 => said(3500, Named::First),
        8 => said(3503, Named::First),
        9 => Some((3494, Named::Nobody, false, true)),
        10 => said(3443, Named::Nobody),
        11 => said(3495, Named::Nobody),
        12 => said(608, Named::First),
        13 => said(1816, Named::Nobody),
        _ => None,
    }
}

/// Why the client does not ask to join a party whose notice it has just read anew, as the line it says in
/// red; none when it asks (`CompletePartyInfo`).
pub fn join_refused(notice: &Notice, level: i32, in_party: bool) -> Option<usize> {
    if notice.least > level || notice.most < level {
        Some(3416)
    } else if in_party {
        Some(79)
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
