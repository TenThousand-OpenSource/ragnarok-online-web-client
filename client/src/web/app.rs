//! The app: the way into the game (login, accounts, character select and creation) and the game itself.
//!
//! One `App` holds the state, a group to each system of the game (`app/state.rs`). Everything that happens (a
//! form sent, a button pressed, a key, bytes from a server, an answer of the account service, a timer, a word
//! from the world) becomes an `Action` (`app/action.rs`) given to `dispatch`; `App::handle` (`app/actions.rs`)
//! says who sees to it, and that changes the state and draws the windows that show it anew.

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Element, Event, HtmlInputElement, HtmlSelectElement, KeyboardEvent};

use super::figure::{self, Figure};
use super::hud::windows as wnd;
use crate::effect_tables;
use super::socket::{Socket, SocketEvent};
use super::view::{HAIR_COLOURS, SLOTS_PER_PAGE, WALLPAPER, hair_styles};
use super::{asset_url, data, js_error, sound, world};
use crate::hotkeys::Chord;
use crate::press::things::{self, Drag, Drop, Form, Use};
use crate::said::{self, colour::*};
use crate::i18n::{Key, Lang, fill};
use crate::net::skills;
use crate::net::inventory::{Hotkey, Item, Skill};
use crate::inventory::Tab;
use crate::net::packets::{self, Character, LoginAccepted, ZoneServer};
use crate::net::{self, emotions};
use crate::net::{Frame, Framer};

mod action;
mod state;
mod wiring;
mod dragging;
mod page;
mod account;
mod connection;
mod actions;
mod ai;
mod clicks;
mod keys;
mod hotkeys;
mod settings;
mod commands;
mod incoming;
mod chat;
mod screen;
mod screenshot;
mod replay_files;
mod windows;

pub(in crate::web) use state::*;

pub use wiring::*;
pub(super) use page::*;
pub(super) use hotkeys::PAGE;
use account::*;
use connection::*;
pub(super) use action::*;

thread_local! {
    static APP: RefCell<App> = RefCell::new(App::default());
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

#[derive(Default)]
pub(super) struct App {
    /// The order the game's windows lie in (`windows.rs`). Drawing the windows settles it, and drawing only
    /// reads the App: hence the cell.
    pub windows: RefCell<crate::stack::Stack>,
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
    /// What the game remembers, one group for each of its systems.
    pub hero: Hero,
    pub bag: Bag,
    pub skills: Skills,
    pub chat: Chat,
    pub shops: Shops,
    pub storage: Storage,
    pub pets: Pets,
    pub quests: Quests,
    pub statuses: Statuses,
    pub cooldowns: Cooldowns,
    pub commands: Commands,
    pub keys: Hotkeys,
    pub settings: Settings,
    pub guide: Guide,
    pub party: Party,
    pub pvp: Pvp,
    pub guild: Guild,
    pub exchange: Exchange,
    pub mail: Mail,
    pub cart: Cart,
    pub vending: Vending,
    pub friends: Friends,
    pub whispers: Whispers,
    pub viewequip: ViewEquip,
    pub fight: Fight,
    pub unread: Unread,
    pub scripted: Scripted,
    pub chat_rooms: ChatRooms,
    pub family: Family,
    pub homunculus: Homunculus,
    pub mercenary: Mercenary,
    pub ai: Scripts,
    pub emotions: Emotions,
    pub worldmap: WorldMap,
    pub tips: Tips,
    pub navigation: Navigation,
    pub books: Books,
    pub cashshop: CashShop,
    pub achievements: Achievements,
    pub reputation: Reputation,
    pub probability: Probability,
    pub attendance: Attendance,
    pub bank: Bank,
    pub battleground: Battleground,
    pub agency: Agency,
    pub replay: Replay,
    pub choosing: Choosing,
    pub notices: Notices,
    pub link: Link,
    pub marks: Marks,
    pub broadcast: Broadcast,
    /// The menu on another player, while it is open.
    pub player_menu: Option<PlayerMenu>,
    /// Which server of the list is picked.
    pub server_picked: usize,
    /// The window of game options: 0 closed, 1 open, 2 asking whether to go back to the save point.
    pub esc: u8,
    /// How many loading screens were shown: each takes the next of the client's pictures.
    loadings: usize,
    /// How many play on the server, as it last said (ZC_USER_COUNT): asked as each map begins and at every
    /// keep-alive (`actions/link.rs`).
    pub online: Option<u32>,
    /// How near the minimap shows the map: 1 (all of it; also what 0 means) to 4.
    pub minimap_zoom: f32,
    /// The minimap's marks of who stands where in a town are hidden (its first button; the client shows them
    /// until it is pressed).
    pub minimap_marks_off: bool,
    /// The window of the NPC being talked to.
    pub dialog: Option<Dialog>,
    /// The strip of menu icons is folded away.
    pub menu_closed: bool,
    /// Whether the last request to delete carried a date of birth.
    sent_birthdate: bool,
}

/// The client's fades: the screen goes dark over 255 milliseconds, evenly (`FadeOutMode`), and comes back from
/// dark over as long (`CMode::DrawFadeIn`).
const FADE_MS: i32 = 255;

/// How many loading pictures the client has (`loading00.jpg` to `loading07.jpg`).
const LOADING_PICTURES: usize = 8;

