//! What the app keeps of the guild.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::net::guild::{Info, Investment, Member, Position, Skill};

/// What the guild asks before it acts.
#[derive(Clone, PartialEq)]
pub(in crate::web) enum GuildAsk {
    /// A guild invites the player: its number and its name (the client's `UIJoinGuildAcceptWnd`).
    Invited(u32, String),
    /// Why the player leaves (`UIGuildLeaveReasonDescWnd`).
    Leave,
    /// Why a member is expelled: the member's account and character (`UIGuildBanReasonDescWnd`).
    Expel(u32, u32),
    /// Another guild's master asks for an alliance: that master's account and the guild's name
    /// (`UIAllyGuildAcceptWnd`).
    Ally(u32, String),
    /// Whether the guild is handed to a member: the member's account, character and name (the client's
    /// question of line 2924).
    Hand(u32, u32, String),
    /// Whether the character is moved to a castle of the guild's, for the zeny it costs: the castle's number
    /// (the client's `UIAgitMessageBox`).
    Castle(u8),
    /// What the server told of a castle's economy and defence, to be read and put away (the client's message
    /// box of `CGameMode::ReceiveAgitNotice`).
    Invested(Investment),
}

/// Something the guild waited for, read.
pub(in crate::web) enum GuildEvent {
    /// A head's sprite as a picture for the page with its size; nothing when there is none.
    Head(String, Option<(String, (usize, usize))>),
    /// The file the master picked to be the emblem.
    EmblemPicked(Vec<u8>),
}

/// What the right button opened a menu on in the guild window.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum GuildMenu {
    /// A member, by its place in the list as it stands.
    Member(usize),
    /// A guild allied or opposed, by its place among those the server told.
    Related(usize),
}

/// The guild: what is known of the player's own, its window, and what is known of the guilds of others in sight
/// (the client's `GuildInfo`, `GuildWndInfo` and `CGuildMgr`).
#[derive(Default)]
pub(in crate::web) struct Guild {
    /// The player's guild (0 for none), its name and the version of its emblem; what the player may do in it;
    /// the master's character.
    pub id: u32,
    pub name: String,
    pub emblem: u32,
    pub invites: bool,
    pub expels: bool,
    pub master: bool,
    pub master_character: u32,
    /// Which tabs the server lets the player open, once it has said (ZC_ACK_GUILD_MENUINTERFACE).
    pub tabs: Option<u32>,
    pub info: Option<Info>,
    /// The guilds allied and opposed: each guild, its name, and whether it is opposed.
    pub related: Vec<(u32, String, bool)>,
    /// The row of those lists the right button was last pressed on: whether of the opposed, and which (the
    /// client's `m_hostileRow` and `m_allyRow`). It stays marked while the tab is open.
    pub related_row: Option<(bool, usize)>,
    pub members: Vec<Member>,
    pub positions: Vec<Position>,
    /// The positions whose rights the server has told (ZC_POSITION_INFO, ZC_ACK_CHANGE_GUILD_POSITIONINFO).
    pub rights: HashSet<u32>,
    pub skills: Vec<Skill>,
    pub skill_points: u16,
    /// Those expelled: the name and the reason.
    pub expelled: Vec<(String, String)>,
    pub subject: String,
    pub notice: String,
    /// The castles the guild holds, by the numbers the client's table has for them (the client's
    /// `GuildInfo::agitIds`); the line of the list that is picked, and the line the menu is open on (the
    /// client's `m_agitList->selected`, `m_agitMenu->m_agitIndex`).
    pub castles: Vec<u8>,
    pub castle: Option<usize>,
    pub castle_menu: Option<usize>,
    /// The window is open, on which tab; the member picked in it; members in the game are listed first.
    pub open: bool,
    pub tab: usize,
    pub picked: Option<usize>,
    pub online_first: bool,
    /// The column the members are ordered by, and whether from the greatest down (the client's `m_sortColumn`
    /// and `m_sortOrders`).
    pub sort: Option<(usize, bool)>,
    /// The skill picked in the list of skills (the client's `m_hoverIndex`).
    pub skill: Option<usize>,
    /// The menu the right button opened: where on the stage, and on what.
    pub menu: Option<((i32, i32), GuildMenu)>,
    pub asks: Option<GuildAsk>,
    /// The member (by its place in the list) whose list of positions to pick from is open.
    pub combo: Option<usize>,
    /// What the master changed of the positions and has not sent yet (the client's `m_positionMap`).
    pub position_edits: BTreeMap<u32, Position>,
    /// The members the master gave another position and has not sent yet: by character, the account and the
    /// position (the client's `membersByCharacterId`).
    pub member_moves: BTreeMap<u32, (u32, u32)>,
    /// The emblems read so far, as pictures for the page: by guild and version.
    pub emblems: HashMap<(u32, u32), String>,
    /// The heads of members read so far, as pictures for the page with their sizes: by the head's sprite. One
    /// asked for and not read yet has no picture.
    pub heads: HashMap<String, Option<(String, (usize, usize))>>,
    /// Those in sight whose names (with party, guild and position) were asked for.
    pub asked: HashSet<u32>,
}

impl Guild {
    /// The guilds allied to the player's.
    pub fn allies(&self) -> Vec<u32> {
        self.related.iter().filter(|(_, _, opposed)| !opposed).map(|(guild, ..)| *guild).collect()
    }

    /// No guild any more: nothing is kept but the pictures read (the client's `GuildInfo::Init`).
    pub fn left(&mut self) {
        *self = Guild { emblems: std::mem::take(&mut self.emblems), heads: std::mem::take(&mut self.heads), ..Guild::default() };
    }

    /// Another guild than before, or the first: what was known of the old one is dropped, the window stays as
    /// it is.
    pub fn renewed(&mut self) {
        let (open, tab) = (self.open, self.tab);
        self.left();
        (self.open, self.tab) = (open, tab);
    }

    /// A position as the window shows it: with what the master changed and has not sent yet.
    pub fn position_shown(&self, position: &Position) -> Position {
        self.position_edits.get(&position.id).unwrap_or(position).clone()
    }

    /// What a position is called.
    pub fn position_name(&self, id: u32) -> String {
        self.positions.iter().find(|position| position.id == id).map(|position| position.name.clone()).unwrap_or_default()
    }

    /// The position a member is shown in: the one the master picked and has not sent yet, or its own.
    pub fn position_of(&self, member: &Member) -> u32 {
        self.member_moves.get(&member.character).map_or(member.position, |(_, position)| *position)
    }
}
