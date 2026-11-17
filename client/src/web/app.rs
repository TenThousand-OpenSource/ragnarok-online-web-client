//! The way into the game: login, accounts, character select and creation, and entering the map.
//!
//! One `App` holds the state. Everything that happens (a form sent, a button pressed, a key, bytes from a server,
//! an answer of the account service, the timer) becomes an `Action` given to `dispatch`, which changes the state
//! and redraws the window (`view`).

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Element, Event, HtmlInputElement, HtmlSelectElement, KeyboardEvent};

use super::figure::{self, Figure};
use super::socket::{Socket, SocketEvent};
use super::view::{HAIR_COLOURS, SLOTS_PER_PAGE, WALLPAPER, hair_styles};
use super::{asset_url, data, js_error, sound, world};
use crate::i18n::{Key, Lang, fill};
use crate::net::skills;
use crate::net::items::{self, Hotkey, Item, Skill, Tab};
use crate::net::packets::{self, Character, LoginAccepted, ZoneServer};
use crate::net::{npc, trade, units};
use crate::net::{Frame, Framer};

mod wiring;
mod page;
mod account;
mod connection;
mod actions;
mod clicks;
mod keys;
mod incoming;
mod game;
mod screen;

pub use wiring::*;
pub(super) use page::*;
use account::*;
use game::*;

const LOGIN_PORT: u16 = 6900;
/// How often the open connection is told the client is still there. The servers drop a silent one after a minute.
const KEEP_ALIVE_MS: i32 = 10_000;
/// The WebSocket close code the gateway turns a login away with; the reason is the seconds to wait.
const TOO_MANY_ATTEMPTS: u16 = 4029;
/// The original client's least lengths for a login, and rAthena's for a character name.
const LOGIN_ID_MIN: usize = 4;
const LOGIN_PASSWORD_MIN: usize = 6;
const NAME_LENGTH: std::ops::RangeInclusive<usize> = 4..=23;
/// The form fields whose content survives a redraw of the same window.
const KEPT_FIELDS: [&str; 13] =
    ["user", "password", "reg-id", "reg-password", "reg-repeat", "reg-email", "reg-sex", "rec-id", "rec-email", "new-password", "new-repeat", "char-name", "birthdate"];
/// The action of a sitting character, facing south: how one waiting to be deleted is shown.
const SITTING: usize = 16;
/// The narrowest view the client is used in, for now: the width of the original client's smallest screen.
const MIN_WIDTH: f64 = 1024.0;
/// A login on an account that is in use elsewhere throws the other session out, which takes the servers a
/// moment: the login is tried again this often, this far apart, before the player is told to wait.
const LOGIN_RETRIES: u8 = 6;
const LOGIN_RETRY_MS: i32 = 1500;
/// The servers' "the last login is not forgotten yet".
const STILL_LOGGED_IN: u8 = 8;

thread_local! {
    static APP: RefCell<App> = RefCell::new(App::default());
}

pub(super) enum Action {
    /// A form was sent: its `data-submit`.
    Submit(String),
    /// Something with `data-click` was pressed: that, and its `data-arg`.
    Click(String, String),
    /// A character slot was double-clicked.
    Open(u8),
    /// Something of the game was double-clicked: `item:<index>`, `skill:<id>` or `slot:<number>`.
    Use(String),
    /// What was dragged (`item:<id>` or `skill:<id>:<level>`) was dropped on a slot of the hotkey bar.
    Dropped(usize, String),
    /// The gateway told what some items are called.
    ItemsKnown,
    Resized,
    /// A moment after the music was asked for: see whether the browser let it play.
    CheckMusic,
    /// The moment to try a login again, if nothing else has happened since (the generation is the same).
    RetryLogin(u32),
    /// From the socket of that generation; older sockets' events are dropped.
    Socket(u32, SocketEvent),
    KeepAlive,
    /// The player pressed a cell of the map to walk to.
    WalkTo(u16, u16),
    /// An item was dragged out of the inventory onto the ground: what was dragged (`item:<id>`).
    Thrown(String),
    /// A skill was pressed with the right button: show its description.
    SkillInfo(u16),
    /// An item was pressed with the right button: show what is known of it.
    Info(u32),
    /// The character stands by a thing on the ground it was sent to: pick it up.
    Take(u32),
    /// The player pressed someone on the map: their ID, and whether it is a monster.
    Pressed(u32, bool),
    /// The character is on another cell now.
    Moved(u16, u16),
    /// The map is drawn, or could not be.
    WorldReady(Result<(), String>),
    /// The skill being aimed was pointed at someone (their ID) or at a cell.
    Aimed(Option<u32>, (u16, u16)),
    /// The slot machine's pictures were read.
    TamingRead(Vec<(String, (usize, usize), (f32, f32))>, Vec<(String, (usize, usize), (f32, f32))>),
    /// A moment of the slot machine has passed.
    TamingTick,
    /// The screen has gone dark for a move: the character is put at its new place (a map and a cell).
    Arrive(String, u16, u16),
    /// The account service answered: the HTTP status and, when it says to wait, for how many seconds.
    Answer(Request, Result<(u16, Option<u64>), String>),
}

#[derive(Clone, Copy)]
pub(super) enum Request {
    Register,
    Recover,
    Reset,
}

#[derive(Default)]
pub(super) enum Screen {
    #[default]
    Login,
    Register,
    Recover,
    /// With the token of a reset link.
    Reset(String),
    /// The servers of the account, with how many play on each.
    ServerSelect,
    CharSelect,
    CharCreate(u8),
    Game,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Server {
    #[default]
    None,
    Login,
    Char,
    Map,
}

pub(super) struct Game {
    pub name: String,
    pub job: u16,
    pub map: String,
    pub x: u16,
    pub y: u16,
    /// Packets received since entering the map.
    pub packets: u32,
    /// The map's size in cells, once it is drawn: for the minimap.
    pub map_size: Option<(usize, usize)>,
}

/// The slot machine shown when a monster is being tamed (the client's `UIPetTamingDeceiveWnd`): the pictures
/// of `slotmachine.spr` run in turn; a press starts the reels and asks the server, whose answer has them slow
/// down and stop on the picture of the catch or of the miss.
pub struct Taming {
    /// The monster being tamed.
    target: u32,
    /// Which action of the sprite plays: 0 waiting, 1 the reels turning.
    state: usize,
    /// The client's counters: the picture shown is `tick / delay`.
    tick: i32,
    delay: i32,
    longest: i32,
    /// The picture to stop on (-1: none yet).
    wanted: i32,
    started: bool,
    /// When it came to rest: it closes two seconds later.
    rested: Option<f64>,
    /// The pictures of both actions, once read: each a data URL, its size and where its origin is.
    pub frames: [Vec<(String, (usize, usize), (f32, f32))>; 2],
    /// The picture on the page now.
    pub shown: (usize, usize),
    /// The page's timer that drives it.
    timer: i32,
}

/// The pseudo-skill the client aims with when a monster is to be tamed (`Zc_Start_Capture`).
const TAME: u16 = 10000;

/// A question over the window, waiting for OK or Cancel; each is about the character with that ID.
#[derive(Clone, Copy)]
pub(super) enum Ask {
    /// Start the wait for a deletion?
    Reserve(u32),
    /// Delete for good?
    Delete(u32),
    /// The same, for an account that has a date of birth to confirm it with.
    Birthdate(u32),
}

/// The character being made.
pub(super) struct Make {
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

/// An NPC's shop, as far as the dealing has come.
pub(super) enum Shop {
    /// The shop asks whether to buy or to sell.
    Choose(u32),
    Buy(Vec<trade::Offer>),
    /// What the shop takes of what is carried: `(index, price)`.
    Sell(Vec<(u16, u32)>),
}

/// The Kafra's storage while it is open.
#[derive(Default)]
pub(super) struct Storage {
    pub items: Vec<items::Item>,
    /// How many kinds of things are in it, and how many fit.
    pub count: (u16, u16),
}

/// A talk with an NPC, as its window shows it.
#[derive(Default)]
pub(super) struct Dialog {
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

#[derive(Default)]
pub(super) struct App {
    pub lang: Lang,
    pub screen: Screen,
    /// A message to confirm, shown over the window.
    pub message: Option<String>,
    /// Waiting for a server: shown over the window until it answers.
    pub wait: Option<Key>,
    pub ask: Option<Ask>,
    /// Whether music and button sounds play.
    pub sound: bool,
    /// The music is wanted, and the browser holds it back until the player presses something on the page.
    pub music_held: bool,
    pub save_id: bool,
    pub saved_id: String,
    socket: Option<Socket>,
    generation: u32,
    server: Server,
    framer: Framer,
    /// The typed name and password, kept only until the login server's socket opens.
    credentials: Option<(String, String)>,
    /// The same, kept until the character list is there, for logging in again while another session of the
    /// account is being thrown out; and how often that was tried.
    relogin: Option<(String, String)>,
    login_tries: u8,
    pub account: Option<LoginAccepted>,
    /// How many slots the account may use.
    pub slots: u8,
    pub characters: Vec<Character>,
    /// When each character waiting to be deleted can be: by character ID, in seconds since 1970.
    pub deletions: HashMap<u32, f64>,
    /// The slot chosen in the character list.
    pub selected: u8,
    pub make: Make,
    zone: Option<ZoneServer>,
    pub game: Option<Game>,
    /// The character's values as the map server gives them, by parameter number (`packets::parameter`).
    pub params: HashMap<u16, i64>,
    /// What equipment and the like add to each stat: Str, Agi, Vit, Int, Dex, Luk.
    pub stat_bonus: [i32; 6],
    /// The chat's lines, oldest first, each with its colour.
    pub chat: Vec<(String, String)>,
    /// Basic Information in its small form; the Status window open; the hotkey bar put away; and how many
    /// rows the chat shows (0 leaves only its input line).
    pub basic_small: bool,
    pub status_open: bool,
    pub hotkeys_hidden: bool,
    /// How many rows of the hotkey bar show beyond the first (none to three).
    pub hotkey_rows: u8,
    /// The thing being dropped while its count is asked for: where it is carried, and how many there are.
    pub throwing: Option<(u16, u16)>,
    /// Which server of the list is picked.
    pub server_picked: usize,
    /// The Equipment window is open.
    pub equipment_open: bool,
    /// The item whose description is open.
    pub item_info: Option<u32>,
    /// The skill whose description is shown.
    pub skill_info: Option<u16>,
    /// What was picked up last, and whether the chat has told of it (its name may arrive a moment later).
    pub obtained: Option<(u32, u16)>,
    /// When it was picked up (milliseconds since 1970): the note of it shows for five seconds from then.
    pub obtained_at: f64,
    pub told_obtained: bool,
    /// The character lies dead, until it is returned to its save point.
    pub dead: bool,
    /// The pet eggs offered for hatching (their places in the inventory), while the incubator's list is open,
    /// and which of them is picked.
    pub eggs: Option<Vec<u16>>,
    pub egg_picked: usize,
    /// The slot machine of taming a monster, while it is open.
    pub taming: Option<Taming>,
    /// Which page of the skill window is open.
    pub skill_tab: usize,
    /// The skill being aimed with the pointer, and its level.
    aimed: Option<(u16, u16)>,
    /// The window of game options: 0 closed, 1 open, 2 asking whether to go back to the save point.
    pub esc: u8,
    /// How many loading screens were shown: each takes the next of the client's pictures.
    loadings: usize,
    /// The character sits.
    pub sitting: bool,
    pub shop: Option<Shop>,
    /// The row picked in the shop's list, and what was put aside to buy or sell: `(row, amount)`.
    pub shop_picked: Option<usize>,
    pub basket: Vec<(usize, u16)>,
    pub storage: Option<Storage>,
    /// Which of the storage's tabs is open.
    pub storage_tab: usize,
    /// The window of the NPC being talked to.
    pub dialog: Option<Dialog>,
    /// The strip of menu icons is folded away.
    pub menu_closed: bool,
    pub chat_rows: usize,
    /// The rows the chat had before its log was hidden.
    pub chat_rows_kept: usize,
    /// Which of the chat's tabs is open, and whether the window is locked where it is.
    pub chat_tab: usize,
    pub chat_locked: bool,
    /// What the character carries, what it knows, and what is on its hotkey bar.
    pub inventory: Vec<Item>,
    pub skills: Vec<Skill>,
    pub hotkeys: Vec<Hotkey>,
    pub inventory_open: bool,
    pub inventory_tab: Tab,
    pub skills_open: bool,
    /// Whether the last request to delete carried a date of birth.
    sent_birthdate: bool,
}

/// The client's fades: the screen goes dark over 255 milliseconds, evenly (`FadeOutMode`), and comes back from
/// dark over as long (`CMode::DrawFadeIn`).
const FADE_MS: i32 = 255;

/// How many loading pictures the client has (`loading00.jpg` to `loading07.jpg`).
const LOADING_PICTURES: usize = 8;

