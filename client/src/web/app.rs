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
use super::hud::windows as wnd;
use crate::effect_tables;
use super::socket::{Socket, SocketEvent};
use super::view::{HAIR_COLOURS, SLOTS_PER_PAGE, WALLPAPER, hair_styles};
use super::{asset_url, data, js_error, sound, world};
use crate::hotkeys::{self, Chord};
use crate::press::things::{self, Drag, Drop, Form, Use};
use crate::said::{self, colour::*};
use crate::i18n::{Key, Lang, fill};
use crate::net::skills;
use crate::net::items::{self, Hotkey, Item, Skill, Tab};
use crate::net::packets::{self, Character, LoginAccepted, ZoneServer};
use crate::net::{emotions, npc, pets, trade, units};
use crate::net::{Frame, Framer};

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
mod hotkeys_set;
mod settings;
mod commands;
mod incoming;
mod game;
mod screen;
mod screenshot;
mod windows;

pub(in crate::web) use state::*;

pub use wiring::*;
pub(super) use page::*;
pub(super) use hotkeys_set::PAGE;
use account::*;

const LOGIN_PORT: u16 = 6900;
/// How often the open connection is told the client is still there, and the map server asked its time: the
/// client's twelve seconds (its `g_gameTickInterval`, and `nextKeepaliveTime` on the character server). The
/// servers drop a silent connection after a minute.
const KEEP_ALIVE_MS: i32 = 12_000;
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
    /// A form was sent.
    Submit(Form),
    /// Something with `data-click` was pressed: that, and its `data-arg`.
    Click(String, String),
    /// A character slot was double-clicked.
    Open(u8),
    /// Something of the game was double-clicked.
    Use(Use),
    /// What was dragged was dropped on a slot of the hotkey bar; nothing dragged empties the slot.
    Dropped(usize, Option<Drag>),
    /// A window of the game was pressed, anywhere on it: its element's ID.
    WindowPressed(String),
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
    /// The moment the watch on the connection looks at it, and what is shown for a time is looked over.
    Watch,
    /// The player pressed a cell of the map to walk to.
    WalkTo(u16, u16),
    /// The character looked towards a place: how its head is turned, and the direction it faces.
    Face(u8, u8),
    /// Something was dragged out of its window and let go over the world.
    Thrown(Drag),
    /// Something was dragged out of its window and let go over the window of a deal between two players.
    Offered(Drag),
    /// Something was dragged out of its window and let go over the window of a thing being forged.
    Forged(Drag),
    /// Something was dragged out of its window and let go over a window that takes things (`data-drop`).
    LetGo(Drag, Drop),
    /// A thing was pressed with the right button while Alt was held, with the storage open: all of it goes
    /// into the storage, or comes out of it.
    Stowed(Use),
    /// A thing of the storage (its place there) was dragged out and let go over the inventory.
    Unstored(u16),
    /// Something was dragged out of its window and let go over the letter being written.
    Mailed(Drag),
    /// A thing of the cart (its place there) was dragged out and let go over the inventory.
    Uncarted(u16),
    /// The first picture of the sprite of a look offered for the cart was read: the kind, the picture, and
    /// where in it the cart stands.
    CartLookRead(u8, String, (f32, f32)),
    /// A thing of the letter being written (where it is carried) was dragged out and let go over the inventory.
    Unmailed(u16),
    /// A skill was pressed with the right button: show its description.
    SkillInfo(u16),
    /// An item was pressed with the right button: show what is known of it.
    Info(u32, bool),
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
    /// The wheel was turned while a skill is aimed: a level up (1) or down (-1).
    AimLevel(i32),
    /// One blow at that monster.
    StrikeOnce(u32),
    /// The fight is called off.
    StopFight,
    /// A slot of the hotkey bar is used by the mouse.
    Quick(usize),
    /// A slider (`data-slide`) was moved: its name and where it stands, 0 to 100.
    Slid(String, u8),
    /// The right button was pressed on someone on the map: their ID, and where on the page.
    Menu(u32, (f32, f32)),
    /// The right button was pressed on a thing in a window that has a menu: the thing, and where on the page.
    MenuOn(things::Menu, (f32, f32)),
    /// The slot machine's pictures were read.
    /// A monster's picture for the Adventure Guide was made: the monster's number, and the picture.
    GuideRead(u32, (String, (usize, usize), (f32, f32))),
    TamingRead(Vec<(String, (usize, usize), (f32, f32))>, Vec<(String, (usize, usize), (f32, f32))>),
    /// The digits of the standing on a PVP map were read (the pictures of `RankFont`, action by action).
    PvpRead(Vec<(String, (usize, usize), (f32, f32))>),
    /// The pictures of the emotions for their window were read: each emotion's picture and its size.
    EmotionsRead(Vec<(u16, (String, (usize, usize)))>),
    /// A picture of the screen was made (or could not be): the name of its file.
    Pictured(Result<String, String>),
    /// A moment of the slot machine has passed.
    TamingTick,
    /// A while after the pet's trick: it may show how it feels standing about.
    PetIdle,
    /// A second has passed while a state of the character has a time running: the times are written anew.
    StatusTick,
    /// A time something of an NPC's script runs by has come: a bar is full, a sound repeats, a mark goes.
    Scripted,
    /// Something of the books has come back: a book's text, a button's picture, the next line read aloud.
    Book(BookNews),
    /// A moment has passed while the player has a homunculus or a mercenary: its script may be due.
    AiTick,
    /// The mouse commanded the homunculus, or the mercenary (true): to fight someone or to walk to a cell, at
    /// once or (true) after its other commands.
    Commanded(bool, Option<u32>, (u16, u16), bool),
    /// The screen has gone dark for a move: the character is put at its new place (a map and a cell).
    Arrive(String, u16, u16),
    /// Something the guild waited for was read: a member's head, a file picked as the emblem.
    Guild(GuildEvent),
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
    pub deal: Deal,
    pub storage: Storage,
    pub pets: Pets,
    pub quests: Quests,
    pub statuses: Statuses,
    pub cooldowns: Cooldowns,
    pub commands: Commands,
    pub keys: Keys,
    pub settings: Settings,
    pub guide: GuideWnd,
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
    pub books: Books,
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
    /// How many play on the server, as it last said (ZC_USER_COUNT); and the keep-alive ticks counted, every
    /// third of which asks again.
    pub online: Option<u32>,
    ticks: u32,
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

