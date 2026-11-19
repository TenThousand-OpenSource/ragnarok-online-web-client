//! What the app keeps, in groups: one for each part of the game. Here are the groups the whole game shares (the
//! character, the bag, the skills, the chat, what an NPC says) and what the way in keeps; every other system's
//! group is in a file of its name in `state/`. `App` itself, which holds one of each, is in `app.rs`.

use super::*;

mod ai;
mod choosing;
mod broadcast;
mod commands;
mod cooldowns;
mod exchange;
mod fight;
mod friends;
mod guide;
mod guild;
mod homunculus;
mod link;
mod mail;
mod cart;
mod vending;
mod mercenary;
mod marks;
mod party;
mod notices;
mod pets;
mod player_menu;
mod pvp;
mod quests;
mod settings;
mod shops;
mod statuses;
mod storage;
mod unread;
mod scripted;
mod chat_rooms;
mod family;
mod viewequip;
mod whispers;
mod emotions;
mod tips;
mod worldmap;
pub(in crate::web) use mail::{MAIL_SAID, Mail, MailAsk, Writing};
pub(in crate::web) use choosing::{Choice, Choosing, Shape};
pub(in crate::web) use quests::Quests;
pub(in crate::web) use cart::{CHANGE_CART, Cart, Carting, Looks};
pub(in crate::web) use vending::{Crossing, Stall, Stage, Stalled, Vending, VendingAsk, Visit};
pub(in crate::web) use {ai::{Script, Scripts}, homunculus::{Homunculus, HomunculusAsk}, mercenary::Mercenary};
pub(in crate::web) use storage::{Storage, Stored, Stowing};
pub(in crate::web) use unread::Unread;
pub(in crate::web) use {chat_rooms::{ChatRooms, Making, Sitting}, family::Family, scripted::{Mark, Repeated, Scripted}};
pub(in crate::web) use notices::Notices;
pub(in crate::web) use link::Link;
pub(in crate::web) use marks::Marks;
pub(in crate::web) use broadcast::{Announced, Broadcast};
pub(in crate::web) use {commands::*, cooldowns::*, exchange::*, friends::*, guide::*, guild::*, party::*, pets::*, player_menu::*, pvp::*};
pub(in crate::web) use {settings::*, shops::*, statuses::*, viewequip::*, whispers::*};
pub(in crate::web) use fight::*;
pub(in crate::web) use {emotions::Emotions, tips::Tips, worldmap::WorldMap};

pub(in crate::web) struct Game {
    pub name: String,
    pub job: u16,
    pub map: String,
    pub x: u16,
    pub y: u16,
    /// The map's size in cells, once it is drawn: for the minimap.
    pub map_size: Option<(usize, usize)>,
}

/// A question over the window, waiting for OK or Cancel; each is about the character with that ID.
#[derive(Clone, Copy)]
pub(in crate::web) enum Ask {
    /// Start the wait for a deletion?
    Reserve(u32),
    /// Delete for good?
    Delete(u32),
    /// The same, for an account that has a date of birth to confirm it with.
    Birthdate(u32),
    /// A key set in the Shortcut Settings window is another entry's already: take it from that one? The entry
    /// it is set for, the one that has it (each a table and a place), and the key.
    Rebind((usize, usize), (usize, usize), Chord),
}

/// The character being made.
pub(in crate::web) struct Make {
    pub doram: bool,
    pub sex: u8,
    pub hair_style: u16,
    pub hair_colour: u16,
    /// Which way the preview looks: 0 is south, then clockwise.
    pub direction: u8,
}

impl Default for Make {
    fn default() -> Self {
        Self { doram: false, sex: 1, hair_style: 1, hair_colour: 0, direction: 0 }
    }
}

/// A talk with an NPC, as its window shows it.
#[derive(Default)]
pub(in crate::web) struct Dialog {
    pub npc: u32,
    pub lines: Vec<String>,
    /// The button under the text: "next" (true) or "close" (false).
    pub button: Option<bool>,
    /// What "next" was pressed after: the next line starts the text afresh.
    pub stale: bool,
    /// The choices of a menu, and the one picked.
    pub menu: Vec<String>,
    pub picked: usize,
    /// Something is asked for: words (true) or a number (false).
    pub asks: Option<bool>,
}

/// The character itself: its values, and the windows that show them.
#[derive(Default)]
pub(in crate::web) struct Hero {
    /// The character's values as the map server gives them, by parameter number (`packets::parameter`).
    pub params: HashMap<u16, i64>,
    /// What equipment and the like add to each stat: Str, Agi, Vit, Int, Dex, Luk.
    pub stat_bonus: [i32; 6],
    /// The character lies dead, until it is returned to its save point.
    pub dead: bool,
    pub sitting: bool,
    /// The bits of the state the character is in (the server's `option`): what it rides.
    pub state: u32,
    /// Basic Information is in its small form; the Status window is open.
    pub basic_small: bool,
    pub status_open: bool,
}

/// What the character carries, and the windows of it.
#[derive(Default)]
pub(in crate::web) struct Bag {
    pub items: Vec<Item>,
    /// The Inventory window is open, and which of its tabs.
    pub open: bool,
    pub tab: Tab,
    /// The places in the inventory of the things kept under the tab "Fav", and whether that tab's button
    /// "Align items" is on (the client's `m_sortPrivateTab`).
    pub favourites: Vec<u16>,
    pub aligned: bool,
    pub equipment_open: bool,
    /// The Equipment window shows its Costume page.
    pub costumes: bool,
    /// The item whose description is open, and whether it is identified.
    pub info: Option<(u32, bool)>,
    /// The card whose illustration is open (the client's `UICardItemIllustWnd`).
    pub illust: Option<u32>,
    /// The items a magnifier can identify (their places in the inventory) while its list is open, and the row
    /// picked in it (none at first, as in the client's `UIItemIdentifyWnd`).
    pub identify: Option<Vec<u16>>,
    pub identify_picked: Option<usize>,
    /// The thing being dropped while its count is asked for: where it is carried, and how many there are.
    pub throwing: Option<(u16, u16)>,
    /// What was picked up last, and whether the chat has told of it (its name may arrive a moment later).
    pub obtained: Option<(u32, u16, bool)>,
    /// When it was picked up (milliseconds since 1970): the note of it shows for five seconds from then.
    pub obtained_at: f64,
    pub told_obtained: bool,
}

/// What the character knows, and what is on its hotkey bar.
#[derive(Default)]
pub(in crate::web) struct Skills {
    pub known: Vec<Skill>,
    /// The Skill window is open, and which of its pages.
    pub open: bool,
    pub tab: usize,
    /// The skill whose description is shown.
    pub info: Option<u16>,
    /// The skill being aimed with the pointer, and its level.
    pub aimed: Option<(u16, u16)>,
    /// The level each skill is used at when it is less than all it has (the arrows of the skill window).
    pub use_level: HashMap<u16, u16>,
    pub hotkeys: Vec<Hotkey>,
    /// The hotkey bar is put away; how many of its rows show beyond the first (none to three).
    pub hotkeys_hidden: bool,
    pub hotkey_rows: u8,
}

/// The chat window.
#[derive(Default)]
pub(in crate::web) struct Chat {
    /// Its lines, oldest first, each with its colour and its kind (`said`).
    pub lines: Vec<(String, String, u8)>,
    /// How many rows it shows (0 leaves only its input line), and how many it had before its log was hidden.
    pub rows: usize,
    pub rows_kept: usize,
    /// Which of its tabs is open, and whether the window is locked where it is.
    pub tab: usize,
    pub locked: bool,
    /// Its tabs as the player has them (`crate::chat_tabs`), those taken out into windows of their own among
    /// them: kept in the browser. Empty until the player changes one: then they are as the client starts them.
    pub tabs: Vec<crate::chat_tabs::Tab>,
    /// A line with a link to an item came in whose name may not be known yet.
    pub linked: bool,
    /// The window is put away (Alt+F10 shows and hides it).
    pub hidden: bool,
    /// The tab whose window of what it lists is open (`UIBattleMsgOptionWnd`).
    pub options: Option<usize>,
}

