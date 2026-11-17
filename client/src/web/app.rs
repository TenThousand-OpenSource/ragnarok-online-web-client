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

pub fn start() -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let ui = ui()?;
    ui.set_attribute("style", &format!("background-image:url('{}')", asset_url(WALLPAPER))).map_err(js_error)?;
    let on_submit = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        event.prevent_default();
        let form = event.target().and_then(|target| target.dyn_into::<Element>().ok());
        if let Some(action) = form.and_then(|form| form.get_attribute("data-submit")) {
            dispatch(Action::Submit(action));
        }
    });
    fn pressed(event: &Event, attribute: &str) -> Option<Element> {
        let target = event.target().and_then(|target| target.dyn_into::<Element>().ok())?;
        let selector = if attribute == "button" { attribute.to_owned() } else { format!("[{attribute}]") };
        target.closest(&selector).ok().flatten()
    }
    let on_click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        APP.with_borrow(App::tune);
        if let Some(pressed) = pressed(&event, "data-click") {
            dispatch(Action::Click(pressed.get_attribute("data-click").unwrap_or_default(), pressed.get_attribute("data-arg").unwrap_or_default()));
        }
    });
    let on_double_click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        // A double click on a server enters it.
        if pressed(&event, "data-open-server").is_some() {
            dispatch(Action::Click("server-ok".to_owned(), String::new()));
        }
        if let Some(slot) = pressed(&event, "data-open").and_then(|pressed| pressed.get_attribute("data-open")).and_then(|slot| slot.parse().ok()) {
            dispatch(Action::Open(slot));
        }
        if let Some(what) = pressed(&event, "data-use").and_then(|pressed| pressed.get_attribute("data-use")) {
            dispatch(Action::Use(what));
        }
    });
    // Icons are dragged from the inventory and the skill list onto the hotkey bar; a right click empties a slot.
    let on_drag = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        let dragged = pressed(&event, "data-drag").and_then(|dragged| dragged.get_attribute("data-drag"));
        if let (Some(dragged), Some(transfer)) = (dragged, event.data_transfer()) {
            let _ = transfer.set_data("text/plain", &dragged);
        }
    });
    let on_drag_over = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        if pressed(&event, "data-slot").is_some() {
            event.prevent_default();
        }
    });
    let on_drop = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
        let slot = pressed(&event, "data-slot").and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok());
        let dragged = event.data_transfer().and_then(|transfer| transfer.get_data("text/plain").ok());
        if let (Some(slot), Some(dragged)) = (slot, dragged) {
            event.prevent_default();
            dispatch(Action::Dropped(slot, dragged));
        }
    });
    let on_menu = Closure::<dyn FnMut(Event)>::new(|event: Event| {
        if let Some(slot) = pressed(&event, "data-slot").and_then(|slot| slot.get_attribute("data-slot")).and_then(|slot| slot.parse().ok()) {
            event.prevent_default();
            dispatch(Action::Dropped(slot, String::new()));
        }
        // The right button on an item shows what it is.
        if let Some(id) = pressed(&event, "data-info").and_then(|item| item.get_attribute("data-info")).and_then(|id| id.parse().ok()) {
            event.prevent_default();
            dispatch(Action::Info(id));
        }
        // The right button on a skill shows its description.
        if let Some(id) = pressed(&event, "data-skill").and_then(|skill| skill.get_attribute("data-skill")).and_then(|id| id.parse().ok()) {
            event.prevent_default();
            dispatch(Action::SkillInfo(id));
        }
    });
    let on_key = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
        let in_game = APP.with_borrow(|app| matches!(app.screen, Screen::Game) && app.message.is_none() && app.ask.is_none());
        if APP.with_borrow_mut(|app| app.key(&event.key(), &event.code(), event.alt_key())) || (in_game && browsers_own(&event)) {
            event.prevent_default();
        }
        APP.with_borrow(App::tune);
    });
    // A browser plays nothing before the first press on the page, wherever it lands: the music starts with it.
    let on_press = Closure::<dyn FnMut()>::new(|| {
        APP.with_borrow(App::tune);
        check_music_soon();
    });
    // A thing dragged out of the inventory and let go over the world is dropped there.
    if let Some(world) = document.get_element_by_id("game") {
        let over = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| event.prevent_default());
        let thrown = Closure::<dyn FnMut(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
            event.prevent_default();
            if let Some(dragged) = event.data_transfer().and_then(|transfer| transfer.get_data("text/plain").ok()) {
                dispatch(Action::Thrown(dragged));
            }
        });
        world.add_event_listener_with_callback("dragover", over.as_ref().unchecked_ref()).map_err(js_error)?;
        world.add_event_listener_with_callback("drop", thrown.as_ref().unchecked_ref()).map_err(js_error)?;
        over.forget();
        thrown.forget();
    }
    drag_windows(&document)?;
    let on_resize = Closure::<dyn FnMut()>::new(|| dispatch(Action::Resized));
    let on_visibility = Closure::<dyn FnMut()>::new(|| APP.with_borrow(App::tune));
    let on_timer = Closure::<dyn FnMut()>::new(|| dispatch(Action::KeepAlive));
    ui.add_event_listener_with_callback("submit", on_submit.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dblclick", on_double_click.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dragstart", on_drag.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("dragover", on_drag_over.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("drop", on_drop.as_ref().unchecked_ref()).map_err(js_error)?;
    ui.add_event_listener_with_callback("contextmenu", on_menu.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref()).map_err(js_error)?;
    for event in ["pointerdown", "keydown"] {
        document.add_event_listener_with_callback(event, on_press.as_ref().unchecked_ref()).map_err(js_error)?;
    }
    window.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("visibilitychange", on_visibility.as_ref().unchecked_ref()).map_err(js_error)?;
    window.set_interval_with_callback_and_timeout_and_arguments_0(on_timer.as_ref().unchecked_ref(), KEEP_ALIVE_MS).map_err(js_error)?;
    // These live as long as the page.
    on_submit.forget();
    on_click.forget();
    on_double_click.forget();
    on_drag.forget();
    on_drag_over.forget();
    on_drop.forget();
    on_menu.forget();
    on_key.forget();
    on_press.forget();
    on_resize.forget();
    on_visibility.forget();
    on_timer.forget();

    APP.with_borrow_mut(|app| {
        app.lang = stored("roweb.lang").and_then(|code| Lang::from_code(&code)).unwrap_or_default();
        crate::i18n::speak(app.lang);
        app.saved_id = stored("roweb.id").unwrap_or_default();
        app.save_id = !app.saved_id.is_empty();
        app.sound = stored("roweb.sound").as_deref() != Some("off");
        // A reset link from an e-mail: `/?reset=<token>`.
        let query = window.location().search().unwrap_or_default();
        // `/?mute` opens the page silent without changing what the player has chosen: for tests beside a player.
        if query.trim_start_matches('?').split('&').any(|pair| pair == "mute") {
            app.sound = false;
        }
        if let Some(token) = query.trim_start_matches('?').split('&').find_map(|pair| pair.strip_prefix("reset=")) {
            app.screen = Screen::Reset(token.to_owned());
        }
        app.render();
        // The music is asked for at once. Most browsers hold it back until the first press on a freshly loaded
        // page; some let it through.
        app.tune();
    });
    check_music_soon();
    // The game's tables arrive a moment later; figures painted without them are painted again.
    wasm_bindgen_futures::spawn_local(async {
        data::load_music().await;
        match data::load().await {
            Ok(()) => {
                figure::forget();
                dispatch(Action::Resized);
            }
            Err(error) => web_sys::console::warn_1(&error.into()),
        }
    });
    Ok(())
}

/// Looks, a moment from now, whether the music plays: starting it takes the browser a moment.
fn check_music_soon() {
    let check = Closure::once_into_js(|| dispatch(Action::CheckMusic));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(check.unchecked_ref(), 400);
    }
}

/// Now, in seconds since 1970.
pub(super) fn epoch() -> f64 {
    js_sys::Date::now() / 1000.0
}

thread_local! {
    /// How far each of the game's windows was dragged from its place, by its element's ID; and the drag under
    /// way: the window, where the pointer was, and how far the window had been moved then.
    static MOVED: RefCell<std::collections::HashMap<String, (f64, f64)>> = RefCell::new(std::collections::HashMap::new());
    static DRAG: RefCell<Option<(String, (f64, f64), (f64, f64))>> = const { RefCell::new(None) };
}

/// Puts every window that was dragged where it was dragged to: a page drawn anew has them at their own places.
fn place_moved() {
    MOVED.with_borrow(|moved| {
        for (id, (x, y)) in moved {
            if let Some(window) = element(id).and_then(|window| window.dyn_into::<web_sys::HtmlElement>().ok()) {
                let _ = window.style().set_property("transform", &format!("translate({x}px,{y}px)"));
            }
        }
    });
}

/// The windows of the game are dragged by their title bars (the top strip of each), as the original's are.
fn drag_windows(document: &web_sys::Document) -> Result<(), String> {
    let on_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        if event.button() != 0 {
            return;
        }
        let Some(target) = event.target().and_then(|target| target.dyn_into::<Element>().ok()) else { return };
        // Buttons, fields and the things in the windows keep their own presses.
        if target.closest("button, input, select, [data-click], [data-use], [data-drag], .scroll").ok().flatten().is_some() {
            return;
        }
        let Some(window) = target.closest(".hud").ok().flatten().filter(|window| !window.id().is_empty()) else { return };
        // Only the top strip moves a window; the chat moves by all of it, unless its lock is closed.
        let chat = window.id() == "chat";
        if chat && window.get_attribute("data-locked").as_deref() == Some("1") {
            return;
        }
        if !chat && f64::from(event.client_y()) - window.get_bounding_client_rect().top() > 20.0 {
            return;
        }
        let id = window.id();
        let was = MOVED.with_borrow(|moved| moved.get(&id).copied().unwrap_or((0.0, 0.0)));
        DRAG.set(Some((id, (f64::from(event.client_x()), f64::from(event.client_y())), was)));
        event.prevent_default();
    });
    let on_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(|event: web_sys::MouseEvent| {
        let Some((id, from, was)) = DRAG.with_borrow(Clone::clone) else { return };
        if event.buttons() & 1 == 0 {
            return DRAG.set(None);
        }
        // The stage may be enlarged: the window moves as far as the pointer does on the screen.
        let scale = web_sys::window().map_or(1.0, |window| {
            let size = |value: Result<JsValue, JsValue>, fallback: f64| value.ok().and_then(|value| value.as_f64()).unwrap_or(fallback);
            (size(window.inner_width(), 1024.0) / 1024.0).min(size(window.inner_height(), 768.0) / 768.0).max(1.0)
        });
        let to = (was.0 + (f64::from(event.client_x()) - from.0) / scale, was.1 + (f64::from(event.client_y()) - from.1) / scale);
        MOVED.with_borrow_mut(|moved| moved.insert(id.clone(), to));
        place_moved();
        // No window leaves the screen: one pushed past an edge is put back against it.
        if let (Some(window), Some(page)) = (element(&id), web_sys::window()) {
            let size = |value: Result<JsValue, JsValue>, fallback: f64| value.ok().and_then(|value| value.as_f64()).unwrap_or(fallback);
            let (wide, tall) = (size(page.inner_width(), 1024.0), size(page.inner_height(), 768.0));
            let rect = window.get_bounding_client_rect();
            let back = |low: f64, high: f64, most: f64| if low < 0.0 { -low } else if high > most { (most - high).max(-low) } else { 0.0 };
            let (dx, dy) = (back(rect.left(), rect.right(), wide), back(rect.top(), rect.bottom(), tall));
            if dx != 0.0 || dy != 0.0 {
                MOVED.with_borrow_mut(|moved| moved.insert(id, (to.0 + dx / scale, to.1 + dy / scale)));
                place_moved();
            }
        }
    });
    let on_up = Closure::<dyn FnMut()>::new(|| DRAG.set(None));
    document.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref()).map_err(js_error)?;
    document.add_event_listener_with_callback("mouseup", on_up.as_ref().unchecked_ref()).map_err(js_error)?;
    on_down.forget();
    on_move.forget();
    on_up.forget();
    Ok(())
}

/// The effect an item shows on the one who uses it, and its sound: the client's own tables
/// (`CGameMode::PlayItemUseEffect` for the effect of each item, `CEZ2STRScreenEffect::InitParticle` for its file).
fn item_effect(item: u32) -> Option<(&'static str, Option<&'static str>)> {
    let found = |table: &'static str, wanted: u32| table.lines().filter_map(|line| line.split_once('\t')).find(|(id, _)| id.parse() == Ok(wanted)).map(|(_, value)| value.trim());
    let effect: u32 = found(include_str!("item_effects.txt"), item)?.parse().ok()?;
    let file = found(include_str!("effect_files.txt"), effect)?;
    let sound = match effect {
        204..=207 | 210 => Some("_heal_effect.wav"),
        208 | 209 | 211 => Some("effect/흡기.wav"),
        _ => None,
    };
    Some((file, sound))
}

/// The client's fades: the screen goes dark over 255 milliseconds, evenly (`FadeOutMode`), and comes back from
/// dark over as long (`CMode::DrawFadeIn`).
const FADE_MS: i32 = 255;

fn fade(dark: bool) {
    let Some(cover) = element("fade").and_then(|cover| cover.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
    let style = cover.style();
    if !dark {
        // It comes back from full dark, whatever it was.
        let _ = style.set_property("transition", "none");
        let _ = style.set_property("opacity", "1");
        let _ = cover.offset_width();
    }
    let _ = style.set_property("transition", &format!("opacity {FADE_MS}ms linear"));
    let _ = style.set_property("opacity", if dark { "1" } else { "0" });
}

/// Runs an action once the screen has gone dark.
fn after_fade(action: Action) {
    let run = Closure::once_into_js(move || dispatch(action));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(run.unchecked_ref(), FADE_MS);
    }
}

/// Shows the client's loading screen with one of its pictures (`loading00.jpg` and on, in turn), or takes it away.
fn loading(picture: Option<usize>) {
    let Some(screen) = element("loading").and_then(|screen| screen.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
    let style = screen.style();
    match picture {
        Some(picture) => {
            let scale = web_sys::window().map_or(1.0, |window| {
                let size = |value: Result<JsValue, JsValue>| value.ok().and_then(|value| value.as_f64()).unwrap_or(0.0);
                (size(window.inner_width()) / 1024.0).min(size(window.inner_height()) / 768.0).max(1.0)
            });
            let _ = style.set_property("--s", &scale.to_string());
            let _ = style.set_property("background-image", &format!("url('{}')", super::image_url(&format!("data/texture/유저인터페이스/loading{:02}.jpg", picture % LOADING_PICTURES))));
            let _ = style.set_property("display", "block");
        }
        None => {
            let _ = style.set_property("display", "none");
        }
    }
}

/// Has the browser fetch a loading picture ahead of its screen, so that it is there when the screen shows.
fn loading_ahead(picture: usize) {
    if let Ok(image) = web_sys::HtmlImageElement::new() {
        image.set_src(&super::image_url(&format!("data/texture/유저인터페이스/loading{:02}.jpg", picture % LOADING_PICTURES)));
        // The browser keeps what it fetched; the element itself is not needed.
        std::mem::forget(image);
    }
}

/// How many loading pictures the client has (`loading00.jpg` to `loading07.jpg`).
const LOADING_PICTURES: usize = 8;

fn dispatch(action: Action) {
    APP.with_borrow_mut(|app| app.handle(action));
}

/// Runs an action after the current browser callback has returned.
fn later(action: Action) {
    wasm_bindgen_futures::spawn_local(async move { dispatch(action) });
}

fn ui() -> Result<Element, String> {
    element("ui").ok_or_else(|| "no #ui element".to_owned())
}

fn element(id: &str) -> Option<Element> {
    web_sys::window()?.document()?.get_element_by_id(id)
}

/// What is typed into (or picked in) the form field with that ID.
fn field(id: &str) -> String {
    let Some(element) = element(id) else { return String::new() };
    match element.dyn_into::<HtmlInputElement>() {
        Ok(input) => input.value(),
        Err(element) => element.dyn_into::<HtmlSelectElement>().map(|select| select.value()).unwrap_or_default(),
    }
}

fn set_field(id: &str, value: &str) {
    match element(id).map(|element| element.dyn_into::<HtmlInputElement>()) {
        Some(Ok(input)) => input.set_value(value),
        Some(Err(element)) => {
            if let Ok(select) = element.dyn_into::<HtmlSelectElement>() {
                select.set_value(value);
            }
        }
        None => {}
    }
}

/// Whether a key pressed in the game is one the browser would do something of its own with, in the game's way:
/// the keys the game uses or will use (function keys, Alt and a letter, Tab, the keys that scroll), and the
/// browser's page commands. Typing, copying and reloading stay the browser's.
fn browsers_own(event: &KeyboardEvent) -> bool {
    let (key, code) = (event.key(), event.code());
    let writing = event.target().and_then(|target| target.dyn_into::<web_sys::Element>().ok()).is_some_and(|target| matches!(target.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT"));
    // F11 is the full screen, which suits the game.
    if code.starts_with('F') && code.len() > 1 && code[1..].parse::<u8>().is_ok_and(|number| number != 11) {
        return true;
    }
    if event.alt_key() && !event.ctrl_key() && !event.meta_key() {
        return true;
    }
    if event.ctrl_key() || event.meta_key() {
        return matches!(code.as_str(), "KeyS" | "KeyF" | "KeyG" | "KeyP" | "KeyD" | "KeyO" | "KeyE" | "KeyU" | "KeyH" | "KeyJ" | "KeyL");
    }
    key == "Tab" || (!writing && matches!(key.as_str(), " " | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "PageUp" | "PageDown" | "Home" | "End" | "Backspace" | "/" | "'"))
}

/// What this browser remembers under a name. Storage may be switched off: then nothing is remembered.
fn stored(name: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(name).ok()?
}

fn store(name: &str, value: Option<&str>) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|window| window.local_storage()) {
        let _ = match value {
            Some(value) => storage.set_item(name, value),
            None => storage.remove_item(name),
        };
    }
}

/// The client's clock, in milliseconds, as the servers count it.
fn now() -> u32 {
    web_sys::window().and_then(|window| window.performance()).map_or(0, |performance| performance.now() as u32)
}

/// Text made safe to put into HTML: names come from the server, and so from other players.
pub(super) fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Sends a form to the account service. Gives back the status and the seconds to wait, if it names any.
async fn post_form(path: &'static str, fields: Vec<(&'static str, String)>) -> Result<(u16, Option<u64>), String> {
    let window = web_sys::window().ok_or("no window")?;
    let body = web_sys::UrlSearchParams::new().map_err(js_error)?;
    for (name, value) in &fields {
        body.append(name, value);
    }
    let options = web_sys::RequestInit::new();
    options.set_method("POST");
    options.set_body(&body);
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str_and_init(path, &options)).await.map_err(js_error)?.dyn_into().map_err(js_error)?;
    let wait = response.headers().get("retry-after").ok().flatten().and_then(|seconds| seconds.parse().ok());
    Ok((response.status(), wait))
}

fn valid_id(id: &str) -> bool {
    (4..=23).contains(&id.len()) && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_password(password: &str) -> bool {
    (8..=23).contains(&password.len()) && password.bytes().all(|byte| byte.is_ascii_graphic())
}

/// A rough check, to catch slips before the form is sent; the account service decides.
fn valid_email(email: &str) -> bool {
    email.len() <= 39
        && !email.contains(' ')
        && email.split_once('@').is_some_and(|(name, domain)| !name.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.'))
}

fn minutes(seconds: u64) -> String {
    seconds.div_ceil(60).max(1).to_string()
}

impl App {
    fn handle(&mut self, action: Action) {
        match action {
            Action::Submit(form) => match form.as_str() {
                "login" => {
                    self.chime();
                    self.login()
                }
                "register" => self.register(),
                "recover" => self.recover(),
                "reset" => self.reset(),
                "create" => self.create_character(),
                "throw" => {
                    if let (Some((index, most)), Some(socket)) = (self.throwing.take(), &self.socket) {
                        let count = field("throw-count").trim().parse::<u32>().unwrap_or(0).min(u32::from(most)) as u16;
                        if count > 0 {
                            socket.send(&items::throw(index, count));
                        }
                    }
                    self.refresh_throw();
                }
                "chat" => {
                    let words = field("chat-input");
                    let words = words.trim();
                    if let (false, Some(socket), Some(game)) = (words.is_empty(), &self.socket, &self.game) {
                        // The client's own commands; everything else is said (the server reads its @commands).
                        // A name in the small box at the left sends the words to that one alone.
                        let to = field("chat-to");
                        let to = to.trim();
                        match words {
                            "/sit" | "/stand" => socket.send(&units::sit(words == "/sit")),
                            _ if !to.is_empty() => {
                                socket.send(&npc::whisper(to, words));
                                let line = format!("( To {to} ) : {words}");
                                set_field("chat-input", "");
                                return self.chat_line(line, "#ffff00");
                            }
                            _ => socket.send(&packets::say(&game.name, words)),
                        }
                        set_field("chat-input", "");
                    }
                }
                _ => {}
            },
            Action::Click(what, arg) => self.click(&what, &arg),
            Action::Open(slot) => {
                self.chime();
                self.open(slot)
            }
            Action::Use(what) => match (what.split_once(':'), &self.socket) {
                // With the storage open, a double click moves the whole of a stack into it or out of it.
                // A double click in the shop's list puts a thing aside; one on what was put aside takes it back.
                (Some(("shop-add", row)), _) => {
                    let Ok(row) = row.parse::<usize>() else { return };
                    // How many: what the line under the list says, or one bought and all of a stack sold.
                    let typed = field("shop-amount").trim().parse::<u32>().ok().filter(|amount| *amount > 0);
                    let most = match &self.shop {
                        Some(Shop::Buy(offers)) if row < offers.len() => 30000,
                        Some(Shop::Sell(bids)) => bids.get(row).and_then(|(index, _)| self.inventory.iter().find(|item| item.index == *index)).map_or(0, |item| u32::from(item.amount)),
                        _ => 0,
                    };
                    let buying = matches!(self.shop, Some(Shop::Buy(_)));
                    let aside = self.basket.iter().find(|(kept, _)| *kept == row).map_or(0, |(_, amount)| u32::from(*amount));
                    let amount = typed.unwrap_or(if buying { 1 } else { most }).min(most - aside.min(most)) as u16;
                    if amount > 0 {
                        match self.basket.iter_mut().find(|(kept, _)| *kept == row) {
                            Some((_, kept)) => *kept += amount,
                            None => self.basket.push((row, amount)),
                        }
                    }
                    self.refresh_trade();
                }
                (Some(("shop-remove", row)), _) => {
                    let row = row.parse::<usize>().ok();
                    self.basket.retain(|(kept, _)| Some(*kept) != row);
                    self.refresh_trade();
                }
                (Some(("put", index)), Some(socket)) => {
                    if let Some(item) = index.parse().ok().and_then(|index: u16| self.inventory.iter().find(|item| item.index == index && item.worn == 0)) {
                        socket.send(&trade::to_storage(item.index, u32::from(item.amount)));
                    }
                }
                (Some(("take", index)), Some(socket)) => {
                    if let Some(item) = index.parse().ok().and_then(|index: u16| self.storage.as_ref()?.items.iter().find(|item| item.index == index)) {
                        socket.send(&trade::from_storage(item.index, u32::from(item.amount)));
                    }
                }
                _ => self.use_it(&what),
            },
            Action::Dropped(slot, dragged) => self.set_hotkey(slot, &dragged),
            Action::Thrown(dragged) => {
                // The first of that item carried without being worn. One is dropped at once; of more, the
                // original asks how many.
                let Some(id) = dragged.strip_prefix("item:").and_then(|id| id.parse::<u32>().ok()) else { return };
                let Some(item) = self.inventory.iter().find(|item| item.id == id && item.worn == 0) else { return };
                if item.amount <= 1 {
                    if let Some(socket) = &self.socket {
                        socket.send(&items::throw(item.index, 1));
                    }
                } else {
                    self.throwing = Some((item.index, item.amount));
                    self.refresh_throw();
                }
            }
            Action::SkillInfo(id) => {
                self.skill_info = Some(id);
                data::want_skill(id, || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::Info(id) => {
                self.item_info = Some(id);
                data::want_items(std::iter::once(id), || dispatch(Action::ItemsKnown));
                self.refresh_items();
            }
            Action::ItemsKnown => {
                self.tell_obtained();
                self.refresh_items();
                self.refresh_trade();
                self.paint_figures();
            }
            Action::Resized => self.render(),
            Action::CheckMusic => {
                let hidden = web_sys::window().and_then(|window| window.document()).is_some_and(|document| document.hidden());
                let held = self.sound && !hidden && !matches!(self.screen, Screen::Game) && !sound::music_playing();
                if held != self.music_held {
                    self.music_held = held;
                    self.render();
                }
            }
            Action::RetryLogin(generation) => {
                if let Some(credentials) = self.relogin.clone().filter(|_| generation == self.generation) {
                    self.login_tries += 1;
                    self.credentials = Some(credentials);
                    self.connect(Server::Login, LOGIN_PORT);
                }
            }
            Action::Socket(generation, _) if generation != self.generation => {}
            Action::Socket(generation, event) => match event {
                SocketEvent::Open => self.connected(),
                SocketEvent::Bytes(bytes) => self.received(generation, &bytes),
                SocketEvent::Closed(TOO_MANY_ATTEMPTS, reason) => {
                    let wait = minutes(reason.parse().unwrap_or(60));
                    self.leave(Some(fill(Key::TooManyLogins.text(self.lang), &[&wait])));
                }
                SocketEvent::Closed(..) => {
                    let text = if self.server == Server::Login && self.account.is_none() { Key::ConnectFailed } else { Key::Disconnected };
                    self.leave(Some(text.text(self.lang).to_owned()));
                }
            },
            Action::KeepAlive => match (&self.server, &self.socket, &self.account) {
                (Server::Char, Some(socket), Some(account)) => socket.send(&packets::ping(account.account_id)),
                (Server::Map, Some(socket), _) if self.game.is_some() => socket.send(&packets::tick(now())),
                _ => {}
            },
            // While a monster is being tamed nothing is struck: the slot machine comes first.
            Action::Pressed(_, true) if self.taming.is_some() => world::stand_down(),
            Action::Pressed(id, monster) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    // A monster is fought until it or the character is down; anyone else is talked to.
                    socket.send(&if monster { units::attack(id) } else { npc::contact(id) });
                }
            }
            // A monster was picked to tame: the slot machine opens for it (not for players, as in the client).
            Action::Aimed(Some(target), _) if self.aimed.is_some_and(|(skill, _)| skill == TAME) => {
                self.aimed = None;
                self.close_taming();
                // The fight, if there was one, stops for the taming.
                world::stand_down();
                if let Some(socket) = &self.socket {
                    // CZ_CANCEL_LOCKON (0x0118): the server stops the blows it was dealing on its own.
                    socket.send(&0x0118u16.to_le_bytes());
                }
                let tick = Closure::<dyn FnMut()>::new(|| dispatch(Action::TamingTick));
                let timer = web_sys::window().and_then(|window| window.set_interval_with_callback_and_timeout_and_arguments_0(tick.as_ref().unchecked_ref(), 16).ok()).unwrap_or(0);
                tick.forget();
                self.taming = Some(Taming { target, state: 0, tick: 0, delay: 1, longest: 10000, wanted: -1, started: false, rested: None, frames: [Vec::new(), Vec::new()], shown: (usize::MAX, 0), timer });
                wasm_bindgen_futures::spawn_local(async {
                    let mut read = [Vec::new(), Vec::new()];
                    for (action, frames) in read.iter_mut().enumerate() {
                        for frame in 0..64 {
                            if frame > 0 && figure::timing("slotmachine", action).is_some_and(|(count, _)| frame >= count) {
                                break;
                            }
                            match figure::frame_picture("slotmachine", action, frame).await {
                                Ok(picture) => frames.push(picture),
                                Err(_) => break,
                            }
                        }
                    }
                    let [waiting, turning] = read;
                    dispatch(Action::TamingRead(waiting, turning));
                });
                self.render();
            }
            Action::Aimed(None, _) if self.aimed.is_some_and(|(skill, _)| skill == TAME) => self.aimed = None,
            Action::TamingRead(waiting, turning) => {
                if let Some(taming) = &mut self.taming {
                    taming.frames = [waiting, turning];
                }
            }
            // The client's `Paint` and `OnProcess` of the window, once a moment.
            Action::TamingTick => {
                let Some(taming) = &mut self.taming else { return };
                let count = taming.frames[taming.state].len() as i32;
                if count == 0 {
                    return;
                }
                let frame = taming.tick / taming.delay.max(1);
                let shown = (taming.state, frame.clamp(0, count - 1) as usize);
                if taming.wanted != frame || taming.delay > taming.longest {
                    taming.tick += 1;
                }
                if frame >= count {
                    taming.tick = 0;
                    if taming.wanted != -1 {
                        taming.delay += 1;
                    }
                }
                let resting = taming.wanted == frame && taming.delay <= taming.longest;
                let over = match taming.rested {
                    None => {
                        if resting {
                            taming.rested = Some(js_sys::Date::now());
                        }
                        false
                    }
                    Some(since) => resting && js_sys::Date::now() - since >= 2000.0,
                };
                // (The page may have been written anew since, and the picture with it.)
                if shown != taming.shown || element("taming-picture").is_some_and(|picture| picture.get_attribute("src").is_none()) {
                    taming.shown = shown;
                    if let Some(picture) = element("taming-picture") {
                        let (url, _, origin) = &taming.frames[shown.0][shown.1];
                        let _ = picture.set_attribute("src", url);
                        let _ = picture.set_attribute("style", &format!("position:absolute;left:{}px;top:{}px", 98.0 - origin.0, 128.0 - origin.1));
                    }
                }
                if over {
                    self.close_taming();
                    self.render();
                }
            }
            Action::Aimed(whom, cell) => {
                if let (Server::Map, Some(socket), Some((skill, level))) = (self.server, &self.socket, self.aimed.take()) {
                    socket.send(&match whom {
                        Some(id) => items::use_skill(level, skill, id),
                        None => crate::net::skills::use_on_ground(level, skill, cell),
                    });
                }
            }
            Action::Take(id) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&units::pick_up(id));
                }
            }
            Action::WalkTo(x, y) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&packets::walk_to(x, y));
                }
            }
            Action::Moved(x, y) => {
                if let Some(game) = &mut self.game {
                    (game.x, game.y) = (x, y);
                    if let Some(position) = element("position") {
                        position.set_text_content(Some(&format!(" {x}  {y}")));
                    }
                    if let (Some(marker), Some(place)) = (element("marker"), super::view::minimap_place(game)) {
                        let _ = marker.set_attribute("style", &place);
                    }
                }
            }
            Action::WorldReady(outcome) => {
                if !matches!(self.screen, Screen::Game) {
                    return;
                }
                if let Err(error) = outcome {
                    web_sys::console::error_1(&format!("world: {error}").into());
                    self.message = Some(error);
                }
                if let Some(game) = &mut self.game {
                    game.map_size = world::map_size();
                }
                self.wait = None;
                self.render();
                // The loading screen goes, and the game comes out of the dark.
                loading(None);
                fade(false);
            }
            Action::Arrive(map, x, y) => {
                if !matches!(self.screen, Screen::Game) {
                    return fade(false);
                }
                let same = self.game.as_ref().is_some_and(|game| game.map == map) && world::map_size().is_some();
                if let Some(game) = &mut self.game {
                    (game.x, game.y) = (x, y);
                    if !same {
                        (game.map, game.map_size) = (map, None);
                    }
                }
                // A dead character arrives alive.
                self.dead = false;
                self.esc = 0;
                self.dialog = None;
                self.refresh_values();
                self.refresh_dialog();
                if same {
                    // The map is already drawn: the character is only put elsewhere on it.
                    world::jump((x, y));
                    fade(false);
                } else {
                    self.enter_world();
                }
                if let Some(socket) = &self.socket {
                    socket.send(&packets::map_loaded());
                }
            }
            Action::Answer(request, answer) => self.answered(request, answer),
        }
    }

    fn click(&mut self, what: &str, arg: &str) {
        match what {
            "lang" => {
                if let Some(lang) = Lang::from_code(arg) {
                    self.lang = lang;
                    crate::i18n::speak(lang);
                    store("roweb.lang", Some(lang.code()));
                    // The items and skills are asked for again, in the new language.
                    data::forget_texts();
                    let known: Vec<u32> = self.inventory.iter().map(|item| item.id).chain(self.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(self.item_info).collect();
                    data::want_items(known.into_iter(), || dispatch(Action::ItemsKnown));
                    if let Some(skill) = self.skill_info {
                        data::want_skill(skill, || dispatch(Action::ItemsKnown));
                    }
                    self.render();
                }
            }
            "ok" => {
                self.message = None;
                self.render();
            }
            "inventory" => {
                self.inventory_open = !self.inventory_open;
                self.render();
            }
            "skills" => {
                self.skills_open = !self.skills_open;
                self.render();
            }
            "inventory-tab" => {
                self.inventory_tab = [Tab::Usable, Tab::Equipment, Tab::Other][arg.parse::<usize>().unwrap_or(0).min(2)];
                self.refresh_items();
            }
            "raise-skill" => {
                if let (Ok(id), Some(socket)) = (arg.parse(), &self.socket) {
                    socket.send(&items::raise_skill(id));
                }
            }
            "menu" => {
                self.menu_closed = !self.menu_closed;
                self.render();
            }
            "dialog-next" | "dialog-close" | "dialog-ok" | "dialog-cancel" | "dialog-pick" => {
                let (Some(dialog), Some(socket)) = (&mut self.dialog, &self.socket) else { return };
                let npc = dialog.npc;
                match what {
                    "dialog-next" => {
                        dialog.button = None;
                        dialog.stale = true;
                        socket.send(&npc::next(npc));
                    }
                    "dialog-close" => {
                        socket.send(&npc::close(npc));
                        self.dialog = None;
                    }
                    "dialog-pick" => dialog.picked = arg.parse().unwrap_or(0),
                    "dialog-ok" if !dialog.menu.is_empty() => {
                        // The choices are counted from 1.
                        socket.send(&npc::choose(npc, dialog.picked as u8 + 1));
                        dialog.menu.clear();
                        dialog.stale = true;
                    }
                    "dialog-ok" => {
                        let typed = field("dialog-input");
                        socket.send(&if dialog.asks == Some(true) { npc::words(npc, &typed) } else { npc::number(npc, typed.trim().parse().unwrap_or(0)) });
                        dialog.asks = None;
                        dialog.stale = true;
                    }
                    _ => {
                        // Turning a menu down ends the talk.
                        socket.send(&npc::choose(npc, 255));
                        self.dialog = None;
                    }
                }
                self.refresh_dialog();
            }
            "shop-pick" => {
                self.shop_picked = arg.parse().ok();
                self.refresh_trade();
            }
            "shop-buy" | "shop-sell" | "shop-cancel" | "shop-ok" => {
                let (Some(shop), Some(socket)) = (self.shop.take(), &self.socket) else { return };
                let basket = std::mem::take(&mut self.basket);
                self.shop_picked = None;
                match (what, shop) {
                    ("shop-buy", Shop::Choose(npc)) => socket.send(&trade::deal(npc, false)),
                    ("shop-sell", Shop::Choose(npc)) => socket.send(&trade::deal(npc, true)),
                    (_, Shop::Choose(_)) => {}
                    ("shop-ok", Shop::Buy(offers)) => {
                        let wanted: Vec<(u32, u16)> = basket.iter().filter_map(|(row, amount)| Some((offers.get(*row)?.id, *amount))).collect();
                        socket.send(&trade::buy(&wanted));
                    }
                    ("shop-ok", Shop::Sell(bids)) => {
                        let sold: Vec<(u16, u16)> = basket.iter().filter_map(|(row, amount)| Some((bids.get(*row)?.0, *amount))).collect();
                        socket.send(&trade::sell(&sold));
                    }
                    // Leaving a list tells the server so with an empty one.
                    (_, Shop::Buy(_)) => socket.send(&trade::buy(&[])),
                    (_, Shop::Sell(_)) => socket.send(&trade::sell(&[])),
                }
                self.refresh_trade();
            }
            "throw-cancel" => {
                self.throwing = None;
                self.refresh_throw();
            }
            "equipment" => {
                self.equipment_open = !self.equipment_open;
                self.render();
            }
            "chat-tab" => {
                self.chat_tab = arg.parse().unwrap_or(0);
                self.render();
            }
            "chat-lock" => {
                self.chat_locked = !self.chat_locked;
                self.render();
            }
            "chat-size" => {
                self.chat_resize();
                self.render();
            }
            // Hides the log, leaving the input line, or brings it back as tall as it was.
            "chat-mini" => {
                if self.chat_rows > 0 {
                    self.chat_rows_kept = self.chat_rows;
                    self.chat_rows = 0;
                } else {
                    self.chat_rows = self.chat_rows_kept.max(1);
                }
                self.render();
            }
            // A press on the slot machine starts its reels and asks the server for the catch.
            "taming-shoot" => {
                if let (Some(taming), Some(socket)) = (&mut self.taming, &self.socket) {
                    if !taming.started {
                        (taming.state, taming.wanted, taming.started, taming.tick) = (1, 7, true, 0);
                        socket.send(&[&0x019fu16.to_le_bytes()[..], &taming.target.to_le_bytes()].concat());
                    }
                }
            }
            "egg-pick" => {
                self.egg_picked = arg.parse().unwrap_or(0);
                self.render();
            }
            // CZ_SELECT_PETEGG (0x01a7): `u16 index`. The picked egg is hatched.
            "egg-ok" | "egg-cancel" => {
                if let (true, Some(socket), Some(index)) = (what == "egg-ok", &self.socket, self.eggs.as_ref().and_then(|eggs| eggs.get(self.egg_picked))) {
                    socket.send(&[0x01a7u16.to_le_bytes(), index.to_le_bytes()].concat());
                }
                self.eggs = None;
                self.render();
            }
            "skill-info-close" => {
                self.skill_info = None;
                self.refresh_items();
            }
            "item-info-close" => {
                self.item_info = None;
                self.refresh_items();
            }
            "esc-close" | "esc-save" | "esc-no" | "esc-yes" => {
                self.esc = match what {
                    "esc-save" => 2,
                    "esc-no" => 1,
                    _ => 0,
                };
                if what == "esc-yes" {
                    if let Some(socket) = &self.socket {
                        socket.send(&npc::restart(false));
                    }
                }
                self.refresh_dialog();
            }
            "hotkey-rows" => {
                self.hotkey_rows = if arg == "+" { (self.hotkey_rows + 1).min(3) } else { self.hotkey_rows.saturating_sub(1) };
                self.render();
            }
            "skill-tab" => {
                self.skill_tab = arg.parse().unwrap_or(0);
                self.render();
            }
            "storage-tab" => {
                self.storage_tab = arg.parse().unwrap_or(0);
                self.refresh_items();
            }
            "storage-close" => {
                if let Some(socket) = &self.socket {
                    socket.send(&trade::close_storage());
                }
            }
            "basic-size" => {
                self.basic_small = !self.basic_small;
                self.render();
            }
            "status" => {
                self.status_open = !self.status_open;
                self.render();
            }
            "raise" => {
                if let (Ok(stat), Some(socket)) = (arg.parse::<u16>(), &self.socket) {
                    socket.send(&packets::raise_stat(packets::parameter::STR + stat.min(5), 1));
                }
            }
            "sound" => {
                self.sound = !self.sound;
                store("roweb.sound", Some(if self.sound { "on" } else { "off" }));
                self.tune();
                self.music_held = false;
                self.render();
                check_music_soon();
            }
            "yes" => self.answer(true),
            "no" => self.answer(false),
            "delete" | "delete-now" => {
                if let Some(character) = self.character_in(self.selected) {
                    self.ask = Some(if what == "delete" { Ask::Reserve(character.char_id) } else { Ask::Delete(character.char_id) });
                    self.render();
                }
            }
            "cancel-delete" => {
                if let (Some(character), Some(socket)) = (self.character_in(self.selected), &self.socket) {
                    socket.send(&packets::delete_cancel(character.char_id));
                    self.wait = Some(Key::PleaseWait);
                    self.render();
                }
            }
            "race" => {
                self.make.doram = arg == "doram";
                self.make.hair_style = self.make.hair_style.min(hair_styles(self.make.doram));
                self.render();
            }
            "cancel-wait" => {
                // Whatever was being waited for is given up; a form stays as it was filled in.
                match self.screen {
                    Screen::Register | Screen::Recover | Screen::Reset(_) => {
                        self.wait = None;
                        self.render();
                    }
                    _ => self.leave(None),
                }
            }
            "save-id" => {
                self.save_id = !self.save_id;
                self.render();
            }
            "to-register" => self.show(Screen::Register),
            "to-recover" => self.show(Screen::Recover),
            "to-login" => self.show(Screen::Login),
            "slot" => {
                let Ok(slot) = arg.parse::<u8>() else { return };
                // A press on the one already chosen changes nothing: the page stays as it is, and a second press
                // makes a double click of it.
                if self.selected == slot && self.character_in(slot).is_some() {
                    return;
                }
                self.selected = slot;
                // An empty slot opens the making of a character at once, as in the original client.
                if self.character_in(slot).is_none() {
                    return self.open(slot);
                }
                self.render();
            }
            "start" => {
                self.chime();
                self.open(self.selected)
            }
            "sex" => {
                self.make.sex = u8::from(arg == "1");
                self.render();
            }
            "hair" => {
                if let Some(style) = arg.parse().ok().filter(|style| (1..=hair_styles(self.make.doram)).contains(style)) {
                    self.make.hair_style = style;
                    self.render();
                }
            }
            "colour" => {
                if let Some(colour) = arg.parse().ok().filter(|colour| *colour < HAIR_COLOURS) {
                    self.make.hair_colour = colour;
                    self.render();
                }
            }
            "turn" => {
                let step = if arg == "-1" { 7 } else { 1 };
                self.make.direction = (self.make.direction + step) % 8;
                self.render();
            }
            "back" => self.show(Screen::CharSelect),
            "server" => {
                let picked = arg.parse().unwrap_or(0);
                // A press on the one already picked leaves the page as it is, so a second makes a double click.
                if picked == self.server_picked {
                    return;
                }
                self.server_picked = picked;
                self.render();
            }
            "server-ok" => {
                self.chime();
                self.enter_server()
            }
            "server-cancel" | "exit" => self.leave(None),
            // In the game, leaving first goes back to the characters: another one may be wanted. The server
            // says when the character is out of the game; from the characters, leaving goes out altogether.
            "logout" if matches!(self.screen, Screen::Game) => {
                self.esc = 0;
                self.refresh_dialog();
                if let Some(socket) = &self.socket {
                    socket.send(&npc::restart(true));
                }
            }
            "logout" => self.leave(None),
            _ => {}
        }
    }

    /// A key was pressed. Returns whether it was used, so the browser does nothing else with it.
    fn key(&mut self, key: &str, code: &str, alt: bool) -> bool {
        if self.message.is_some() {
            if matches!(key, "Enter" | "Escape" | " ") {
                self.message = None;
                self.render();
                return true;
            }
            return false;
        }
        if self.wait.is_some() {
            return false;
        }
        if self.ask.is_some() {
            return match key {
                "Enter" => {
                    self.answer(true);
                    true
                }
                "Escape" => {
                    self.answer(false);
                    true
                }
                _ => false,
            };
        }
        if matches!(self.screen, Screen::Game) {
            return self.game_key(key, code, alt);
        }
        match (&self.screen, key) {
            (Screen::CharSelect, "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown") => {
                let step: i16 = match key {
                    "ArrowLeft" => -1,
                    "ArrowRight" => 1,
                    "ArrowUp" => -5,
                    _ => 5,
                };
                let moved = i16::from(self.selected) + step;
                if (0..i16::from(SLOTS_PER_PAGE)).contains(&moved) {
                    self.selected = moved as u8;
                    self.render();
                }
                true
            }
            (Screen::ServerSelect, "Enter") => {
                self.enter_server();
                true
            }
            (Screen::ServerSelect, "Escape") => {
                self.leave(None);
                true
            }
            (Screen::ServerSelect, "ArrowUp" | "ArrowDown") => {
                let servers = self.account.as_ref().map_or(0, |account| account.char_servers.len());
                self.server_picked = if key == "ArrowUp" { self.server_picked.saturating_sub(1) } else { (self.server_picked + 1).min(servers.saturating_sub(1)) };
                self.render();
                true
            }
            (Screen::CharSelect, "Enter") => {
                self.open(self.selected);
                true
            }
            (Screen::CharSelect, "Escape") => {
                self.leave(None);
                true
            }
            (Screen::CharCreate(_), "Escape") => {
                self.show(Screen::CharSelect);
                true
            }
            (Screen::Register | Screen::Recover, "Escape") => {
                self.show(Screen::Login);
                true
            }
            _ => false,
        }
    }

    /// The game's own keys, as the original client has them: Alt+V and Alt+A for the Basic Information and
    /// Status windows, F12 for the hotkey bar, F10 for the chat's size, Enter to write.
    fn game_key(&mut self, key: &str, code: &str, alt: bool) -> bool {
        let chat = element("chat-input").and_then(|input| input.dyn_into::<web_sys::HtmlElement>().ok());
        let writing = chat.as_ref().is_some_and(|input| web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active == **input));
        match (key, code, alt) {
            (_, "KeyV", true) => self.basic_small = !self.basic_small,
            (_, "KeyA", true) => self.status_open = !self.status_open,
            (_, "KeyE", true) => self.inventory_open = !self.inventory_open,
            (_, "KeyQ", true) => self.equipment_open = !self.equipment_open,
            (_, "KeyS", true) => self.skills_open = !self.skills_open,
            // F1 to F9 use the first row of the hotkey bar.
            (_, "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9", false) => {
                let slot = code[1..].parse::<usize>().unwrap_or(1) - 1;
                self.use_it(&format!("slot:{slot}"));
                return true;
            }
            ("Escape", ..) if self.throwing.is_some() => {
                self.throwing = None;
                self.refresh_throw();
                return true;
            }
            // Insert sits down and stands up.
            ("Insert", ..) => {
                if let Some(socket) = &self.socket {
                    socket.send(&units::sit(!self.sitting));
                }
                return true;
            }
            // F12 runs the bar through its sizes, as the client does: a row more each time, then away, then one
            // row again.
            ("F12", ..) => {
                if self.hotkeys_hidden {
                    self.hotkeys_hidden = false;
                } else if self.hotkey_rows >= 3 {
                    (self.hotkey_rows, self.hotkeys_hidden) = (0, true);
                } else {
                    self.hotkey_rows += 1;
                }
            }
            ("F10", ..) => self.chat_resize(),
            // Enter goes to the chat's line; from there it sends (the form does that).
            // In a field of some window Enter belongs to that window.
            ("Enter", ..) if web_sys::window().and_then(|window| window.document()).and_then(|document| document.active_element()).is_some_and(|active| active.tag_name() == "INPUT" && active.id() != "chat-input") => {
                return false;
            }
            ("Enter", ..) if !writing => {
                if let Some(input) = chat {
                    let _ = input.focus();
                }
                return true;
            }
            // Escape lets go of a skill being aimed.
            ("Escape", ..) if world::aiming() => {
                world::aim(None, 0);
                self.aimed = None;
                return true;
            }
            // Escape opens the window of game options and closes it, unless an NPC is being talked to.
            ("Escape", ..) if !writing && self.dialog.is_none() => {
                self.esc = u8::from(self.esc == 0);
                self.refresh_dialog();
                return true;
            }
            ("Escape", ..) if writing => {
                if let Some(input) = chat {
                    let _ = input.blur();
                }
                return true;
            }
            _ => return false,
        }
        self.render();
        true
    }

    /// Uses what was double-clicked or what a key stands for: `item:<index>`, `skill:<id>` or `slot:<number>`.
    fn use_it(&mut self, what: &str) {
        let (Some(socket), Some(account)) = (&self.socket, &self.account) else { return };
        let Some((kind, number)) = what.split_once(':') else { return };
        let Ok(number) = number.parse::<u32>() else { return };
        // A slot of the bar stands for its item or its skill.
        let (kind, number, level) = match kind {
            "slot" => match self.hotkeys.get(number as usize).copied().filter(|hotkey| hotkey.id != 0) {
                Some(hotkey) if hotkey.skill => ("skill", hotkey.id, Some(hotkey.count)),
                // The first of that item the character carries without wearing it.
                Some(hotkey) => match self.inventory.iter().find(|item| item.id == hotkey.id && item.worn == 0) {
                    Some(item) => ("item", u32::from(item.index), None),
                    None => return,
                },
                None => return,
            },
            other => (other, number, None),
        };
        match kind {
            "item" => {
                let Some(item) = self.inventory.iter().find(|item| u32::from(item.index) == number) else { return };
                if item.usable() {
                    socket.send(&items::use_item(item.index, account.account_id));
                } else if item.worn != 0 {
                    socket.send(&items::take_off(item.index));
                } else if item.wearable != 0 {
                    socket.send(&items::wear(item.index, item.wearable));
                }
            }
            "skill" => {
                let Some(skill) = self.skills.iter().find(|skill| u32::from(skill.id) == number).cloned() else { return };
                if skill.level == 0 || skill.passive() {
                    return;
                }
                if skill.on_self() {
                    socket.send(&items::use_skill(level.unwrap_or(skill.level).min(skill.level).max(1), skill.id, account.account_id));
                } else {
                    // A skill for someone or for a place is aimed with the pointer: the next press on the map
                    // says on whom or where.
                    self.aimed = Some((skill.id, level.unwrap_or(skill.level).min(skill.level).max(1)));
                    world::aim(Some(skill.target == 2), skill.range);
                }
            }
            _ => {}
        }
    }

    /// Puts what was dragged on a slot of the hotkey bar, or empties the slot (for nothing dragged), and tells
    /// the server, which keeps the bar.
    fn set_hotkey(&mut self, slot: usize, dragged: &str) {
        let mut parts = dragged.split(':');
        let hotkey = match (parts.next(), parts.next().and_then(|id| id.parse::<u32>().ok()), parts.next().and_then(|level| level.parse().ok())) {
            (Some("item"), Some(id), _) => Hotkey { skill: false, id, count: 0 },
            (Some("skill"), Some(id), Some(level)) => Hotkey { skill: true, id, count: level },
            (Some(""), ..) => Hotkey::default(),
            _ => return,
        };
        if slot >= items::HOTKEYS {
            return;
        }
        self.hotkeys.resize(items::HOTKEYS, Hotkey::default());
        self.hotkeys[slot] = hotkey;
        if let Some(socket) = &self.socket {
            socket.send(&items::set_hotkey(slot as u16, hotkey));
        }
        self.refresh_items();
    }

    /// Asks what the items carried and on the bar are called, and redraws their windows.
    fn items_changed(&self) {
        let stored = self.storage.iter().flat_map(|storage| storage.items.iter().map(|item| item.id));
        let ids = self.inventory.iter().map(|item| item.id).chain(self.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id)).chain(stored);
        data::want_items(ids, || dispatch(Action::ItemsKnown));
        // The weapon in hand (bit 2 of where a thing is worn) is drawn on the character.
        world::own_weapon(self.inventory.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id));
        self.refresh_items();
        self.paint_figures();
    }

    /// The sound a blow of the weapon in hand makes when it lands (the client's `_hit_*.wav`): by the kind of
    /// weapon its number tells, bare fists if nothing is held.
    fn weapon_sound(&self) -> &'static str {
        // Bit 2 of where a thing is worn is the hand that holds the weapon.
        match self.inventory.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id) {
            1100..=1199 | 13400..=13499 => "_hit_sword.wav",
            1200..=1249 | 13000..=13099 => "_hit_dagger.wav",
            1300..=1399 => "_hit_axe.wav",
            1400..=1499 => "_hit_spear.wav",
            1500..=1549 | 16000..=16099 => "_hit_mace.wav",
            1550..=1699 | 2000..=2099 => "_hit_rod.wav",
            1700..=1749 | 18100..=18199 => "_hit_arrow.wav",
            _ => "_hit_fist1.wav",
        }
    }

    /// The chat's next size, as the client steps it (`UM_TOGGLEPUSH`): one row, then three more each time up to
    /// thirteen, then none.
    fn chat_resize(&mut self) {
        let rows = if self.chat_rows == 0 { 1 } else { self.chat_rows + 3 };
        self.chat_rows = if rows > 15 { 0 } else { rows / 3 * 3 + 1 };
    }

    /// Tells of the thing just picked up, once its name is known: a line in the chat, and a note on the screen.
    fn tell_obtained(&mut self) {
        let Some((id, amount)) = self.obtained.filter(|_| !self.told_obtained) else { return };
        let Some((name, _)) = data::item(id) else { return };
        self.told_obtained = true;
        self.chat_line(format!("{name} - {amount} {}", crate::i18n::tr("obtained.")), "#ffff00");
        if let Some(note) = element("obtained") {
            note.set_inner_html(&self.obtained_note());
        }
    }

    /// Adds a line to the chat.
    fn chat_line(&mut self, words: String, colour: &str) {
        self.chat.push((words, colour.to_owned()));
        if self.chat.len() > super::hud::CHAT_KEPT {
            self.chat.remove(0);
        }
        self.refresh_chat();
    }

    fn show(&mut self, screen: Screen) {
        self.screen = screen;
        self.wait = None;
        self.ask = None;
        self.tune();
        self.render();
    }

    /// The title music plays in every window before the game, if sound is on and the page is the one looked at:
    /// a tab left in the background falls silent.
    fn tune(&self) {
        let hidden = web_sys::window().and_then(|window| window.document()).is_some_and(|document| document.hidden());
        sound::music(self.sound && !hidden && !matches!(self.screen, Screen::Game));
        // In the game the map has its own music, and everything its sound.
        sound::game(self.sound && !hidden && matches!(self.screen, Screen::Game));
    }

    /// OK or Cancel on a question.
    fn answer(&mut self, yes: bool) {
        let Some(ask) = self.ask.take() else { return };
        let Some(socket) = self.socket.as_ref().filter(|_| yes) else { return self.render() };
        match ask {
            Ask::Reserve(char_id) => socket.send(&packets::delete_reserve(char_id)),
            // An account without a date of birth confirms with none; the server says if it wants one.
            Ask::Delete(char_id) => socket.send(&packets::delete_now(char_id, "")),
            Ask::Birthdate(char_id) => socket.send(&packets::delete_now(char_id, &field("birthdate"))),
        }
        self.sent_birthdate = matches!(ask, Ask::Birthdate(_));
        self.wait = Some(Key::PleaseWait);
        self.render();
    }

    fn say(&mut self, key: Key) {
        self.message = Some(key.text(self.lang).to_owned());
        self.wait = None;
        self.render();
    }

    /// Closes the connection and goes back to the login window, saying why if there is a reason.
    fn leave(&mut self, error: Option<String>) {
        self.socket = None;
        self.generation += 1;
        self.server = Server::None;
        self.credentials = None;
        self.relogin = None;
        self.account = None;
        self.characters.clear();
        self.deletions.clear();
        self.ask = None;
        self.zone = None;
        self.game = None;
        world::leave();
        self.screen = Screen::Login;
        self.wait = None;
        self.message = error;
        self.tune();
        self.render();
    }

    /// Enters the server picked in the list: its char server, where the characters are.
    fn enter_server(&mut self) {
        let Some(port) = self.account.as_ref().and_then(|account| account.char_servers.get(self.server_picked)).map(|server| server.port) else { return };
        self.connect(Server::Char, port);
    }

    /// Out of the game and back to the characters of the account, on the server it was played on.
    fn back_to_characters(&mut self) {
        world::leave();
        self.game = None;
        self.zone = None;
        self.dialog = None;
        self.shop = None;
        self.storage = None;
        self.throwing = None;
        self.item_info = None;
        self.obtained = None;
        self.dead = false;
        self.esc = 0;
        self.close_taming();
        self.sitting = false;
        self.inventory.clear();
        self.basket.clear();
        self.chat.clear();
        self.enter_server();
    }

    /// Opens a connection to another server, closing the current one.
    fn connect(&mut self, server: Server, port: u16) {
        self.generation += 1;
        self.framer = Framer::new();
        if server == Server::Char {
            // The char server answers with the bare account ID before its first packet.
            self.framer.expect_raw(4);
        }
        self.server = server;
        let generation = self.generation;
        match Socket::open(port, move |event| later(Action::Socket(generation, event))) {
            Ok(socket) => {
                self.socket = Some(socket);
                self.wait = Some(Key::PleaseWait);
                self.render();
            }
            Err(error) => {
                web_sys::console::error_1(&error.into());
                self.leave(Some(Key::ConnectFailed.text(self.lang).to_owned()));
            }
        }
    }

    /// The client's chime (`버튼소리.wav`). The original plays it in a few places only: on logging in, on choosing
    /// a server, on entering the game with a character (and for a notice, a whisper window, someone joining a
    /// chat room). Its buttons, tabs and items make no sound.
    fn chime(&self) {
        if self.sound {
            sound::button();
        }
    }

    /// Takes the slot machine of taming away, and stops its timer.
    fn close_taming(&mut self) {
        if let Some(taming) = self.taming.take() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(taming.timer);
            }
        }
    }

    fn login(&mut self) {
        let (user, password) = (field("user"), field("password"));
        for (text, least) in [(&user, LOGIN_ID_MIN), (&password, LOGIN_PASSWORD_MIN)] {
            if text.len() < least {
                self.message = Some(fill(Key::EnterAtLeast.text(self.lang), &[&least.to_string()]));
                return self.render();
            }
        }
        self.saved_id = if self.save_id { user.clone() } else { String::new() };
        self.relogin = Some((user.clone(), password.clone()));
        self.login_tries = 0;
        self.credentials = Some((user, password));
        self.connect(Server::Login, LOGIN_PORT);
    }

    fn request(&mut self, request: Request, path: &'static str, fields: Vec<(&'static str, String)>) {
        self.wait = Some(Key::PleaseWait);
        self.render();
        wasm_bindgen_futures::spawn_local(async move {
            let answer = post_form(path, fields).await;
            dispatch(Action::Answer(request, answer));
        });
    }

    fn register(&mut self) {
        let (id, password, email) = (field("reg-id"), field("reg-password"), field("reg-email"));
        let problem = if !valid_id(&id) {
            Some(Key::BadId)
        } else if !valid_password(&password) {
            Some(Key::BadPassword)
        } else if password != field("reg-repeat") {
            Some(Key::PasswordsDiffer)
        } else if !valid_email(&email) {
            Some(Key::BadEmail)
        } else {
            None
        };
        match problem {
            Some(problem) => self.say(problem),
            None => self.request(Request::Register, "/api/register", vec![("userid", id), ("password", password), ("email", email), ("sex", field("reg-sex"))]),
        }
    }

    fn recover(&mut self) {
        let (id, email) = (field("rec-id"), field("rec-email"));
        if !valid_id(&id) {
            return self.say(Key::BadId);
        }
        if !valid_email(&email) {
            return self.say(Key::BadEmail);
        }
        // The letter is written in the language the player reads the client in.
        self.request(Request::Recover, "/api/recover", vec![("userid", id), ("email", email), ("lang", self.lang.code().to_owned())]);
    }

    fn reset(&mut self) {
        let Screen::Reset(token) = &self.screen else { return };
        let (token, password) = (token.clone(), field("new-password"));
        if !valid_password(&password) {
            return self.say(Key::BadPassword);
        }
        if password != field("new-repeat") {
            return self.say(Key::PasswordsDiffer);
        }
        self.request(Request::Reset, "/api/reset", vec![("token", token), ("password", password)]);
    }

    fn answered(&mut self, request: Request, answer: Result<(u16, Option<u64>), String>) {
        // The wait was given up, or the player went elsewhere: the answer is for nobody.
        if self.wait.is_none() || !matches!(self.screen, Screen::Register | Screen::Recover | Screen::Reset(_)) {
            return;
        }
        let (status, wait) = match answer {
            Ok(answer) => answer,
            Err(error) => {
                web_sys::console::error_1(&error.into());
                return self.say(Key::ServiceUnavailable);
            }
        };
        let done = match (request, status) {
            (Request::Register, 201) => Some(Key::AccountCreated),
            (Request::Recover, 200) => Some(Key::RestoreSent),
            (Request::Reset, 200) => Some(Key::PasswordChanged),
            _ => None,
        };
        if let Some(done) = done {
            if matches!(request, Request::Register) {
                self.saved_id = field("reg-id");
            }
            // A used reset link is taken out of the address bar.
            if matches!(request, Request::Reset) {
                if let Some(Ok(history)) = web_sys::window().map(|window| window.history()) {
                    let _ = history.replace_state_with_url(&JsValue::NULL, "", Some("/"));
                }
            }
            self.screen = Screen::Login;
            return self.say(done);
        }
        match (request, status) {
            (_, 429) => {
                let wait = minutes(wait.unwrap_or(3600));
                self.message = Some(fill(Key::TooManyAttempts.text(self.lang), &[&wait]));
                self.wait = None;
                self.render();
            }
            (Request::Register, 409) => self.say(Key::IdTaken),
            (Request::Reset, 400) => self.say(Key::LinkInvalid),
            (_, 400) => self.say(Key::CheckForm),
            _ => self.say(Key::ServiceUnavailable),
        }
    }

    /// The socket opened: each server expects the client to speak first.
    fn connected(&mut self) {
        let Some(socket) = &self.socket else { return };
        match self.server {
            Server::Login => {
                if let Some((user, password)) = self.credentials.take() {
                    socket.send(&packets::login(&user, &password));
                }
            }
            Server::Char => {
                if let Some(account) = &self.account {
                    socket.send(&packets::char_enter(account));
                }
            }
            Server::Map => {
                if let (Some(account), Some(zone)) = (&self.account, &self.zone) {
                    socket.send(&packets::map_enter(account, zone.char_id, now()));
                }
            }
            Server::None => {}
        }
    }

    fn received(&mut self, generation: u32, bytes: &[u8]) {
        self.framer.push(bytes);
        // A packet may move the client to another server: what is left of this stream is then dropped with it.
        while generation == self.generation {
            match self.framer.next() {
                Ok(Some(Frame::Packet(id, packet))) => self.packet(id, &packet),
                Ok(Some(Frame::Raw(_))) => {}
                Ok(None) => {
                    // More held than any packet is long: the length table is wrong about this one.
                    if let Some((id, held)) = self.framer.waiting().filter(|(_, held)| *held > 600) {
                        web_sys::console::error_1(&format!("protocol: waiting on packet {id:#06x} with {held} bytes held").into());
                    }
                    break;
                }
                Err(error) => {
                    web_sys::console::error_1(&format!("protocol: {error}").into());
                    return self.leave(Some(Key::Disconnected.text(self.lang).to_owned()));
                }
            }
        }
    }

    fn refusal(&self, key: Key, value: &str) -> Option<String> {
        Some(fill(key.text(self.lang), &[value]))
    }

    fn packet(&mut self, id: u16, packet: &[u8]) {
        match (self.server, id) {
            // The account is in use elsewhere. The servers are throwing that session out and refuse this login
            // meanwhile: it is tried again in a moment, so the newer login is the one that gets in.
            (Server::Login | Server::Char, packets::SC_NOTIFY_BAN)
                if packets::ban_reason(packet) == Some(STILL_LOGGED_IN) && self.relogin.is_some() && self.login_tries < LOGIN_RETRIES =>
            {
                self.socket = None;
                self.generation += 1;
                self.server = Server::None;
                self.account = None;
                let generation = self.generation;
                let retry = Closure::once_into_js(move || dispatch(Action::RetryLogin(generation)));
                if let Some(window) = web_sys::window() {
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(retry.unchecked_ref(), LOGIN_RETRY_MS);
                }
            }
            (_, packets::SC_NOTIFY_BAN) => {
                let text = match packets::ban_reason(packet) {
                    Some(1) => self.refusal(Key::ServerClosed, ""),
                    Some(2) => self.refusal(Key::AlreadyLoggedIn, ""),
                    Some(8) => self.refusal(Key::StillRecognized, ""),
                    Some(code) => self.refusal(Key::DisconnectedCode, &code.to_string()),
                    None => self.refusal(Key::Disconnected, ""),
                };
                self.leave(text);
            }

            (Server::Login, packets::AC_ACCEPT_LOGIN) => match packets::login_accepted(packet) {
                // With one char server there is nothing to choose. A server list comes with the second server.
                // The servers are shown with how many play on each, as the original does, and the player enters
                // the one picked.
                Some(account) if !account.char_servers.is_empty() => {
                    store("roweb.id", (!self.saved_id.is_empty()).then_some(self.saved_id.as_str()));
                    self.account = Some(account);
                    self.server_picked = 0;
                    self.relogin = None;
                    // The login server has said all it had to.
                    self.socket = None;
                    self.generation += 1;
                    self.server = Server::None;
                    self.show(Screen::ServerSelect);
                }
                _ => self.leave(Some(Key::NoCharServer.text(self.lang).to_owned())),
            },
            (Server::Login, packets::AC_REFUSE_LOGIN) => {
                let text = match packets::login_refused(packet) {
                    Some((0, _)) => self.refusal(Key::UnregisteredId, ""),
                    Some((1, _)) => self.refusal(Key::WrongPassword, ""),
                    Some((2, _)) => self.refusal(Key::IdExpired, ""),
                    Some((4, _)) => self.refusal(Key::Blocked, ""),
                    Some((6, until)) => self.refusal(Key::BlockedUntil, &until),
                    Some((7, _)) => self.refusal(Key::Overpopulated, ""),
                    Some((3, _)) | None => self.refusal(Key::Rejected, ""),
                    Some((code, _)) => self.refusal(Key::RejectedCode, &code.to_string()),
                };
                self.leave(text);
            }

            (Server::Char, packets::HC_ACCEPT_ENTER2) => self.slots = packets::char_slots(packet).map_or(0, |(usable, _)| usable),
            (Server::Char, packets::HC_ACCEPT_ENTER) => {
                self.characters = packets::char_list(packet);
                let now = epoch();
                self.deletions = self.characters.iter().filter_map(|character| Some((character.char_id, now + f64::from(character.delete_in?)))).collect();
                // The first character is chosen, or the first slot to make one in.
                self.selected = self.characters.iter().map(|character| character.slot).min().unwrap_or(0);
                // In: the password is not needed any more.
                self.relogin = None;
                self.show(Screen::CharSelect);
            }
            (Server::Char, packets::HC_REFUSE_ENTER) => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            (Server::Char, packets::HC_ACCEPT_MAKECHAR) => {
                if let Some(character) = packets::char_made(packet) {
                    self.selected = character.slot;
                    self.characters.push(character);
                }
                self.show(Screen::CharSelect);
            }
            (Server::Char, packets::HC_REFUSE_MAKECHAR) => self.say(match packets::make_char_refused(packet) {
                0x00 => Key::NameExists,
                0x01 => Key::Underaged,
                0x02 => Key::SymbolsForbidden,
                0x03 => Key::SlotNotAllowed,
                _ => Key::CreationDenied,
            }),
            (Server::Char, packets::HC_DELETE_CHAR3_RESERVED) => match packets::delete_reserved(packet) {
                Some((char_id, 1, seconds)) => {
                    self.deletions.insert(char_id, epoch() + f64::from(seconds));
                    self.show(Screen::CharSelect);
                }
                Some((_, 4, _)) => self.say(Key::DeleteGuild),
                Some((_, 5, _)) => self.say(Key::DeleteParty),
                _ => self.say(Key::DeleteFailed),
            },
            (Server::Char, packets::HC_DELETE_CHAR3_CANCEL) => match packets::delete_result(packet) {
                Some((char_id, 1)) => {
                    self.deletions.remove(&char_id);
                    self.show(Screen::CharSelect);
                }
                _ => self.say(Key::DeleteFailed),
            },
            (Server::Char, packets::HC_DELETE_CHAR3) => match packets::delete_result(packet) {
                Some((char_id, 1)) => {
                    self.characters.retain(|character| character.char_id != char_id);
                    self.deletions.remove(&char_id);
                    self.say(Key::Deleted);
                }
                Some((_, 4)) => self.say(Key::DeleteNotYet),
                // A date of birth is wanted: asked for, unless the one just given was the wrong one.
                Some((char_id, 5)) if !self.sent_birthdate => {
                    self.wait = None;
                    self.ask = Some(Ask::Birthdate(char_id));
                    self.render();
                }
                Some((_, 5)) => self.say(Key::WrongBirthdate),
                _ => self.say(Key::DeleteFailed),
            },
            (Server::Char, packets::HC_NOTIFY_ZONESVR) => match packets::zone_server(packet) {
                Some(zone) => {
                    let port = zone.port;
                    self.zone = Some(zone);
                    self.connect(Server::Map, port);
                }
                None => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            },

            (Server::Map, packets::ZC_ACCEPT_ENTER) => self.entered_map(packet),
            (Server::Map, packets::ZC_REFUSE_ENTER) => self.leave(Some(Key::Rejected.text(self.lang).to_owned())),
            (Server::Map, packets::ZC_NOTIFY_PLAYERMOVE) => {
                if let Some((from, to)) = packets::player_move(packet) {
                    world::walk(from, to);
                }
            }
            (Server::Map, packets::ZC_STOPMOVE) => {
                let own = self.account.as_ref().map(|account| account.account_id);
                if let Some((id, x, y)) = packets::stop_move(packet) {
                    if Some(id) == own { world::stand((x, y)) } else { world::unit_stand(id, (x, y)) }
                }
            }
            // Everyone else on the map: coming into sight, walking, and leaving it.
            (Server::Map, units::ZC_NOTIFY_STANDENTRY | units::ZC_NOTIFY_NEWENTRY | units::ZC_NOTIFY_MOVEENTRY) => {
                if let Some(unit) = units::unit(id, packet) {
                    world::unit_shown(&unit);
                }
            }
            (Server::Map, units::ZC_HP_INFO) => {
                if let Some((id, left, most)) = units::life(packet) {
                    world::unit_life(id, left, most);
                }
            }
            (Server::Map, units::ZC_ITEM_FALL_ENTRY | units::ZC_ITEM_ENTRY) => {
                if let Some((drop, item, cell, within)) = units::dropped(id, packet) {
                    // Its picture is drawn once the item's name is known. One that falls now (not one that
                    // already lay there) is seen falling.
                    data::want_items(std::iter::once(item), || {});
                    world::drop_shown(drop, item, cell, within, id == units::ZC_ITEM_FALL_ENTRY);
                }
            }
            (Server::Map, units::ZC_ITEM_DISAPPEAR) => {
                if let Some(id) = npc::npc(id, packet) {
                    world::drop_gone(id);
                }
            }
            // 0x0147: an item used has the character cast a skill (a Fly Wing casts Teleport): the client casts it,
            // on the character itself, as the original does. `u16 skill, u32 kind, u16 level, …`.
            (Server::Map, 0x0147) => {
                if let (Some(skill), Some(level), Some(socket), Some(account)) = (packet.get(2..4), packet.get(8..10), &self.socket, &self.account) {
                    socket.send(&items::use_skill(u16::from_le_bytes([level[0], level[1]]), u16::from_le_bytes([skill[0], skill[1]]), account.account_id));
                }
            }
            // 0x013d and 0x0a27: the character recovers life or spirit (a potion, food): `u16 which, amount`.
            // The client shows the amount as a number over the character, green for life and blue for spirit.
            (Server::Map, 0x013d | 0x0a27) => {
                let which = packet.get(2..4).map_or(0, |which| u16::from_le_bytes([which[0], which[1]]));
                let amount = if id == 0x013d { packet.get(4..6).map_or(0, |amount| i32::from(i16::from_le_bytes([amount[0], amount[1]]))) } else { packet.get(4..8).map_or(0, |amount| i32::from_le_bytes([amount[0], amount[1], amount[2], amount[3]])) };
                if amount > 0 && matches!(which, packets::parameter::HP | packets::parameter::SP) {
                    world::recovered(amount, which == packets::parameter::SP);
                    sound::effect("_heal_effect.wav", 1.0);
                }
            }
            // 0x019b: someone gains a level (0 base, 1 job).
            (Server::Map, 0x019b) => {
                let own = self.account.as_ref().map(|account| account.account_id);
                if npc::npc(id, packet) == own {
                    sound::effect("levelup.wav", 1.0);
                }
            }
            (Server::Map, units::ZC_NOTIFY_MOVE) => {
                if let Some((id, from, to)) = units::moved(packet) {
                    world::unit_walk(id, from, to);
                }
            }
            (Server::Map, units::ZC_NOTIFY_ACT) => {
                let own = self.account.as_ref().map_or(0, |account| account.account_id);
                // Hits of every kind; sitting down, picking up and the like are not shown yet.
                match units::act(packet) {
                    Some(act) if matches!(act.kind, 0 | 4 | 8 | 9 | 10 | 11 | 13) => {
                        // The sound of the blow, heard when it lands: the weapon's when the character strikes,
                        // the common one when it is struck. A monster's own sounds are in its sprite.
                        let heard = if act.source == own {
                            Some(self.weapon_sound().to_owned())
                        } else if act.target == own {
                            Some(format!("_enemy_hit_normal{}.wav", 1 + packet.get(10).copied().unwrap_or(0) % 4))
                        } else {
                            None
                        };
                        world::struck(act.source, act.target, own, f64::from(act.speed), f64::from(act.delay), act.damage, heard);
                        // One who holds a bow sends an arrow, which arrives as the blow lands.
                        world::arrow(act.source, act.target, own, f64::from(act.delay));
                    }
                    // Someone bends down for a thing on the ground.
                    Some(act) if act.kind == 1 => world::acts(act.source, own, 24, 450.0),
                    Some(act) if matches!(act.kind, 2 | 3) => {
                        if act.source == own {
                            self.sitting = act.kind == 2;
                        }
                        world::sat(act.source, own, act.kind == 2);
                    }
                    _ => {}
                }
            }
            (Server::Map, npc::ZC_SAY_DIALOG | npc::ZC_SAY_DIALOG2 | npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2 | npc::ZC_CLOSE_DIALOG | npc::ZC_MENU_LIST | npc::ZC_CLEAR_DIALOG | npc::ZC_OPEN_EDITDLG | npc::ZC_OPEN_EDITDLGSTR) => {
                let Some(from) = npc::npc(id, packet) else { return };
                // A script that ends after its window was closed says so once more: there is nothing to show.
                if self.dialog.is_none() && matches!(id, npc::ZC_CLOSE_DIALOG | npc::ZC_CLEAR_DIALOG | npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2) {
                    if id == npc::ZC_CLOSE_DIALOG {
                        if let Some(socket) = &self.socket {
                            socket.send(&npc::close(from));
                        }
                    }
                    return;
                }
                let dialog = self.dialog.get_or_insert_with(Dialog::default);
                dialog.npc = from;
                match id {
                    npc::ZC_SAY_DIALOG | npc::ZC_SAY_DIALOG2 => {
                        if std::mem::take(&mut dialog.stale) {
                            dialog.lines.clear();
                        }
                        dialog.lines.extend(npc::said(id, packet));
                    }
                    npc::ZC_WAIT_DIALOG | npc::ZC_WAIT_DIALOG2 => dialog.button = Some(true),
                    // A window with nothing said in it has nothing to close: it is closed for the player.
                    npc::ZC_CLOSE_DIALOG if dialog.lines.is_empty() && dialog.menu.is_empty() && dialog.asks.is_none() => {
                        if let Some(socket) = &self.socket {
                            socket.send(&npc::close(from));
                        }
                        self.dialog = None;
                        self.refresh_dialog();
                        return;
                    }
                    npc::ZC_CLOSE_DIALOG => dialog.button = Some(false),
                    npc::ZC_CLEAR_DIALOG => dialog.lines.clear(),
                    npc::ZC_MENU_LIST => {
                        dialog.menu = npc::menu(packet).unwrap_or_default();
                        dialog.picked = 0;
                    }
                    asks => dialog.asks = Some(asks == npc::ZC_OPEN_EDITDLGSTR),
                }
                self.refresh_dialog();
            }
            (Server::Map, units::ZC_NOTIFY_VANISH) => {
                if let Some((id, died)) = units::vanished(packet) {
                    // The player's own character does not go away when it dies: it lies there until the player
                    // has it returned to its save point.
                    if Some(id) == self.account.as_ref().map(|account| account.account_id) {
                        if died {
                            self.dead = true;
                            // The client's data has one voice for dying, a man's (`die_male.wav`); where the
                            // client starts it was not found in its code.
                            if self.own_figure().sex == 1 {
                                sound::effect("die_male.wav", 1.0);
                            }
                            // The client opens its window of options (or closes it, if it was open).
                            self.esc = u8::from(self.esc == 0);
                            self.dialog = None;
                            world::own_dead(true);
                            self.refresh_dialog();
                            self.refresh_values();
                        }
                    } else {
                        world::unit_gone(id, died);
                    }
                }
            }
            (Server::Map, packets::ZC_PAR_CHANGE | packets::ZC_LONGPAR_CHANGE) => {
                if let Some((name, value)) = packets::parameter(packet) {
                    if name == packets::PARAMETER_SPEED {
                        world::speed(value);
                    }
                    self.params.insert(name, i64::from(value));
                    self.refresh_values();
                }
            }
            (Server::Map, packets::ZC_LONGLONGPAR_CHANGE) => {
                if let Some((name, value)) = packets::long_parameter(packet) {
                    self.params.insert(name, value);
                    self.refresh_values();
                }
            }
            (Server::Map, packets::ZC_STATUS) => {
                self.params.extend(packets::status(packet));
                self.refresh_values();
            }
            (Server::Map, packets::ZC_STATUS_CHANGE) => {
                if let Some((name, cost)) = packets::stat_cost(packet) {
                    self.params.insert(name, i64::from(cost));
                    self.refresh_values();
                }
            }
            (Server::Map, packets::ZC_COUPLESTATUS) => {
                if let Some((name, base, bonus)) = packets::stat(packet) {
                    self.params.insert(name, i64::from(base));
                    if let Some(slot) = name.checked_sub(packets::parameter::STR).and_then(|stat| self.stat_bonus.get_mut(usize::from(stat))) {
                        *slot = bonus;
                    }
                    self.refresh_values();
                }
            }
            // What the character carries. A list starts over with the packet that announces it.
            (Server::Map, items::ZC_INVENTORY_START) => {
                if packet.get(4) == Some(&0) {
                    self.inventory.clear();
                }
                if packet.get(4) == Some(&trade::STORAGE) {
                    self.storage = Some(Storage::default());
                }
            }
            (Server::Map, items::ZC_INVENTORY_ITEMLIST_NORMAL) => {
                self.inventory.extend(items::stackable(packet));
                if let Some(storage) = &mut self.storage {
                    storage.items.extend(items::stackable_of(packet, trade::STORAGE));
                }
                self.items_changed();
            }
            (Server::Map, items::ZC_INVENTORY_ITEMLIST_EQUIP) => {
                self.inventory.extend(items::equipment(packet));
                if let Some(storage) = &mut self.storage {
                    storage.items.extend(items::equipment_of(packet, trade::STORAGE));
                }
                self.items_changed();
            }
            // The Kafra's storage.
            (Server::Map, trade::ZC_NOTIFY_STOREITEM_COUNTINFO) => {
                if let Some(count) = trade::storage_count(packet) {
                    self.storage.get_or_insert_with(Storage::default).count = count;
                    // With the storage open the inventory is too, beside it.
                    self.inventory_open = true;
                    self.items_changed();
                    self.render();
                }
            }
            (Server::Map, trade::ZC_ADD_ITEM_TO_STORE) => {
                if let (Some(storage), Some(added)) = (&mut self.storage, trade::stored(packet)) {
                    match storage.items.iter_mut().find(|item| item.index == added.index) {
                        Some(item) => item.amount = item.amount.saturating_add(added.amount),
                        None => storage.items.push(added),
                    }
                    self.items_changed();
                }
            }
            (Server::Map, trade::ZC_DELETE_ITEM_FROM_STORE) => {
                if let (Some(storage), Some((index, amount))) = (&mut self.storage, trade::taken(packet)) {
                    if let Some(item) = storage.items.iter_mut().find(|item| item.index == index) {
                        item.amount = item.amount.saturating_sub(amount.min(u32::from(u16::MAX)) as u16);
                    }
                    storage.items.retain(|item| item.amount > 0);
                    self.items_changed();
                }
            }
            (Server::Map, trade::ZC_CLOSE_STORE) => {
                self.storage = None;
                self.render();
            }
            // An NPC's shop.
            (Server::Map, trade::ZC_SELECT_DEALTYPE) => {
                if let Some(npc) = npc::npc(id, packet) {
                    self.shop = Some(Shop::Choose(npc));
                    self.refresh_trade();
                }
            }
            (Server::Map, trade::ZC_PC_PURCHASE_ITEMLIST) => {
                let offers = trade::offers(packet);
                data::want_items(offers.iter().map(|offer| offer.id), || dispatch(Action::ItemsKnown));
                self.basket.clear();
                self.shop_picked = None;
                self.shop = Some(Shop::Buy(offers));
                self.refresh_trade();
            }
            (Server::Map, trade::ZC_PC_SELL_ITEMLIST) => {
                self.basket.clear();
                self.shop_picked = None;
                self.shop = Some(Shop::Sell(trade::bids(packet)));
                self.refresh_trade();
            }
            (Server::Map, trade::ZC_PC_PURCHASE_RESULT | trade::ZC_PC_SELL_RESULT) => {
                let words = match (id, packet.get(2).copied().unwrap_or(0)) {
                    (trade::ZC_PC_PURCHASE_RESULT, 0) => "The deal has successfully completed.",
                    (trade::ZC_PC_PURCHASE_RESULT, 1) => "You do not have enough zeny.",
                    (trade::ZC_PC_PURCHASE_RESULT, 2) => "You are over your weight limit.",
                    (trade::ZC_PC_PURCHASE_RESULT, _) => "You cannot carry any more items.",
                    (_, 0) => "The deal has successfully completed.",
                    _ => "The deal has failed.",
                };
                self.chat_line(words.to_owned(), "#ffff00");
            }
            (Server::Map, items::ZC_ITEM_PICKUP_ACK) => {
                if let Some(gained) = items::gained(packet) {
                    // What was picked up is told in the chat and shown for a moment, as the original does.
                    self.obtained = Some((gained.id, gained.amount));
                    self.obtained_at = js_sys::Date::now();
                    match self.inventory.iter_mut().find(|item| item.index == gained.index) {
                        Some(item) => item.amount += gained.amount,
                        None => self.inventory.push(gained),
                    }
                    self.items_changed();
                    self.told_obtained = false;
                    self.tell_obtained();
                }
            }
            (Server::Map, items::ZC_USE_ITEM_ACK) => {
                if let Some((index, item, who, left)) = items::used(packet) {
                    let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
                    // What the item looks and sounds like when used, by the client's own tables.
                    if let Some((file, sound)) = item_effect(item) {
                        world::effect(file, (!own).then_some(who));
                        if let Some(sound) = sound {
                            sound::effect(sound, 1.0);
                        }
                    }
                    if own {
                        if let Some(item) = self.inventory.iter_mut().find(|item| item.index == index) {
                            item.amount = left;
                        }
                        self.inventory.retain(|item| item.amount > 0);
                        self.refresh_items();
                    }
                }
            }
            (Server::Map, items::ZC_DELETE_ITEM_FROM_BODY | items::ZC_ITEM_THROW_ACK) => {
                if let Some((index, count)) = items::lost(id, packet) {
                    if let Some(item) = self.inventory.iter_mut().find(|item| item.index == index) {
                        item.amount = item.amount.saturating_sub(count);
                    }
                    self.inventory.retain(|item| item.amount > 0);
                    self.refresh_items();
                }
            }
            (Server::Map, items::ZC_REQ_WEAR_EQUIP_ACK) => {
                if let Some((index, place)) = items::put_on(packet) {
                    if let Some(item) = self.inventory.iter_mut().find(|item| item.index == index) {
                        item.worn = place;
                    }
                    self.items_changed();
                } else {
                    // The server would not have it worn (another job's thing, too low a level): the client says so.
                    self.chat_line(crate::i18n::tr("You can't put this item on.").to_owned(), "#ff8484");
                }
            }
            (Server::Map, items::ZC_REQ_TAKEOFF_EQUIP_ACK) => {
                if let Some(index) = items::taken_off(packet) {
                    if let Some(item) = self.inventory.iter_mut().find(|item| item.index == index) {
                        item.worn = 0;
                    }
                    self.items_changed();
                }
            }
            (Server::Map, items::ZC_SKILLINFO_LIST) => {
                self.skills = items::skills(packet);
                self.refresh_items();
            }
            (Server::Map, items::ZC_ADD_SKILL) => {
                if let Some(added) = items::skill_added(packet) {
                    self.skills.retain(|skill| skill.id != added.id);
                    self.skills.push(added);
                    self.refresh_items();
                }
            }
            (Server::Map, items::ZC_SKILLINFO_UPDATE) => {
                if let Some((id, level, sp, range, raisable)) = items::skill_raised(packet) {
                    if let Some(skill) = self.skills.iter_mut().find(|skill| skill.id == id) {
                        (skill.level, skill.sp, skill.range, skill.raisable) = (level, sp, range, raisable);
                    }
                    self.refresh_items();
                }
            }
            (Server::Map, items::ZC_SHORTCUT_KEY_LIST) => {
                self.hotkeys = items::hotkeys(packet);
                self.items_changed();
            }
            // What the character says comes back from the server in green; what others say is white; the
            // server's announcements are yellow, or the colour they name.
            (Server::Map, packets::ZC_NOTIFY_PLAYERCHAT | packets::ZC_NOTIFY_CHAT | packets::ZC_BROADCAST | packets::ZC_BROADCAST2) => {
                if let Some(words) = packets::chat_text(id, packet) {
                    // What is said near also hangs over the head of whoever says it.
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    match id {
                        packets::ZC_NOTIFY_PLAYERCHAT => world::said(own, own, &words),
                        packets::ZC_NOTIFY_CHAT => world::said(npc::npc(id, packet).unwrap_or(0), own, &words),
                        _ => {}
                    }
                    let colour = match id {
                        packets::ZC_NOTIFY_PLAYERCHAT => "#0f0".to_owned(),
                        packets::ZC_NOTIFY_CHAT => "#fff".to_owned(),
                        packets::ZC_BROADCAST2 => packets::broadcast_colour(packet).map_or("#ff0".to_owned(), |[red, green, blue]| format!("rgb({red},{green},{blue})")),
                        _ => "#ff0".to_owned(),
                    };
                    self.chat_line(words, &colour);
                }
            }
            // 0x00b3: the server lets the character out of the game, back to the characters (type 1).
            (Server::Map, 0x00b3) => {
                if packet.get(2) == Some(&1) {
                    self.back_to_characters();
                }
            }
            // 0x018b: leaving was refused (a fight just now, for one).
            (Server::Map, 0x018b) => {
                if packet.get(2..4) != Some(&[0, 0]) {
                    self.chat_line(crate::i18n::tr("You cannot leave the game right now.").to_owned(), "#ff8484");
                }
            }
            (Server::Map, npc::ZC_WHISPER) => {
                if let Some((name, words)) = npc::whispered(packet) {
                    self.chat_line(format!("( {} {name} ) : {words}", crate::i18n::tr("From")), "#ffff00");
                }
            }
            (Server::Map, npc::ZC_ACK_WHISPER) => {
                if packet.get(2).copied().unwrap_or(0) != 0 {
                    self.chat_line(crate::i18n::tr("There is no such character name or the user is offline.").to_owned(), "#ff8484");
                }
            }
            // Moved to another place, on this map or another: the map is loaded and the server told so.
            (Server::Map, packets::ZC_NPCACK_MAPMOVE) => {
                if let Some((map, x, y)) = packets::map_move(packet) {
                    // The screen goes dark, the character is moved, and the screen comes back, as in the
                    // original: no window is shown for it.
                    fade(true);
                    after_fade(Action::Arrive(map, x, y));
                }
            }
            // Someone starts to cast (the client's `NotifySkillCasting`). A skill with no casting time shows
            // nothing here. The rings are those of the cast's element (`GetSkillActionInfo`).
            (Server::Map, skills::ZC_USESKILL_ACK) => {
                if let Some(cast) = skills::casting(packet).filter(|cast| cast.milliseconds > 0) {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    let effect = match cast.element {
                        1 => 54,
                        2 => 56,
                        3 => 55,
                        4 => 57,
                        5 => 59,
                        6 => 58,
                        _ => 12,
                    };
                    world::cast(cast.source, own, cast.target, cast.cell, f64::from(cast.milliseconds), effect);
                }
            }
            // ZC_DISPEL (0x01b9): `u32 who`. A cast is broken off (the caster was struck, or moved).
            (Server::Map, 0x01b9) => {
                if packet.len() >= 6 {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    world::cast_over(u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]), own, true);
                }
            }
            // A skill lands: the caster strikes, and each of its hits shows its share of the damage, one after
            // another.
            (Server::Map, skills::ZC_NOTIFY_SKILL) => {
                if let Some(landed) = skills::landing(packet) {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    world::cast_over(landed.source, own, false);
                    let hits = i32::from(landed.hits.max(1));
                    let each = landed.damage / hits;
                    let delay = f64::from(landed.attack_motion.clamp(0, 1500));
                    // Each hit of a skill sounds as one of the client's four plain blows (its
                    // `GetWeaponHitWaveName(-1)` for the hits of skills).
                    let blow = || Some(format!("_enemy_hit_normal{}.wav", 1 + (js_sys::Math::random() * 4.0) as u32));
                    world::struck(landed.source, landed.target, own, f64::from(landed.attack_motion), delay, each, blow());
                    // The hits follow one another 200 milliseconds apart (the client's `g_multiHitInterval`).
                    for hit in 1..hits {
                        world::lands(landed.target, own, delay + f64::from(hit) * 200.0, each, blow());
                    }
                    // What the skill looks and sounds like on the one struck (the client's `PlaySkillEffect`
                    // for the bolts themselves, and the effect of each hit by the skill's element).
                    let on = (landed.target != own).then_some(landed.target);
                    let pick = |most: f64| 1 + (js_sys::Math::random() * most) as u32;
                    match landed.skill {
                        // Fire Bolt and Cold Bolt: a bolt for each hit.
                        19 => world::bolts(landed.target, hits as u32, true),
                        14 => world::bolts(landed.target, hits as u32, false),
                        // Lightning Bolt: the client's lightning, once.
                        // (Its sound is the client's `EF_LightBolt.wav`; where the client starts it was not
                        // found in its code.)
                        20 => world::effect_later("lightning.str", on, 0.0, Some("effect/ef_lightbolt.wav")),
                        _ => {}
                    }
                    for hit in 0..hits {
                        let when = delay + f64::from(hit) * 200.0;
                        match landed.skill {
                            19 => world::effect_later(&format!("firehit{}.str", pick(3.0)), on, when, Some(&format!("_enemy_hit_fire{}.wav", pick(2.0)))),
                            20 => world::effect_later(&format!("windhit{}.str", pick(3.0)), on, when, None),
                            _ => {}
                        }
                    }
                }
            }
            (Server::Map, skills::ZC_USE_SKILL) => {
                if let Some((skill, _, target, who, _)) = skills::used(packet) {
                    let own = self.account.as_ref().map_or(0, |account| account.account_id);
                    world::cast_over(who, own, false);
                    // Resurrection (skill 54) shows the client's effect on the one raised, with its sound
                    // (`EF_RESURRECTION`: `Resurrection.str` and `priest_resurrection.wav`).
                    if skill == 54 {
                        world::effect("resurrection.str", (target != own).then_some(target));
                        sound::effect("effect/priest_resurrection.wav", 1.0);
                    }
                }
            }
            (Server::Map, skills::ZC_ACK_TOUSESKILL) => self.chat_line(crate::i18n::tr("Skill has failed.").to_owned(), "#ff8484"),
            // ZC_EQUIP_ARROW (0x013c): `u16 index`. The ammunition in use (0: none).
            (Server::Map, 0x013c) => {
                if packet.len() >= 4 {
                    let index = u16::from_le_bytes([packet[2], packet[3]]);
                    for item in self.inventory.iter_mut().filter(|item| item.kind == 10) {
                        item.worn = if item.index == index { 0x8000 } else { 0 };
                    }
                    self.refresh_items();
                }
            }
            // ZC_PETEGG_LIST (0x01a6): `u16 length`, then the place in the inventory of each egg. An incubator was
            // used: the eggs are listed to pick the one to hatch (the client's `UIPetEggListWnd`).
            (Server::Map, 0x01a6) => {
                self.eggs = Some(packet.get(4..).unwrap_or_default().chunks_exact(2).map(|index| u16::from_le_bytes([index[0], index[1]])).collect());
                self.egg_picked = 0;
                self.render();
            }
            // ZC_START_CAPTURE (0x019e): a taming item was used. The monster to tame is picked with the pointer, as
            // a skill's target is (the client's `Zc_Start_Capture`: reach 5).
            (Server::Map, 0x019e) => {
                self.aimed = Some((TAME, 0));
                world::aim(Some(false), 5);
                // Taming and fighting do not go together: the fight stops as soon as the bait is in hand.
                world::stand_down();
                if let Some(socket) = &self.socket {
                    socket.send(&0x0118u16.to_le_bytes());
                }
            }
            // ZC_TRYCAPTURE_MONSTER (0x01a0): `u8 caught`. The reels slow down and stop on the picture of the
            // catch (the eighth) or of the miss (the fourth).
            (Server::Map, 0x01a0) => {
                if let (Some(taming), Some(caught)) = (&mut self.taming, packet.get(2)) {
                    (taming.wanted, taming.delay, taming.longest) = (if *caught != 0 { 7 } else { 3 }, 7, 7);
                }
            }
            // ZC_ATTACK_FAILURE_FOR_DISTANCE (0x0139): `u32 target, i16 x, i16 y, i16 own x, i16 own y, i16 range`.
            // The blow was asked for from too far: the character goes nearer and strikes from there.
            (Server::Map, 0x0139) => {
                if packet.len() >= 16 {
                    let target = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    world::too_far(target, i32::from(i16::from_le_bytes([packet[14], packet[15]])));
                }
            }
            // ZC_NPCSPRITE_CHANGE (0x01b0): `u32 who, u8 kind, u32 class`. A monster becomes another (the server has
            // a Fabre turn into a Pupa and a Pupa into a Creamy by itself, as a skill of theirs).
            (Server::Map, 0x01b0) => {
                if packet.len() >= 11 {
                    let who = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    let class = u32::from_le_bytes([packet[7], packet[8], packet[9], packet[10]]);
                    world::unit_becomes(who, class as u16);
                }
            }
            // ZC_ACK_REQNAMEALL_NPC (0x0adf): `u32 who, u32 group, char[24] name, char[24] title`.
            (Server::Map, 0x0adf) => {
                if packet.len() >= 34 {
                    let who = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    let name = &packet[10..34];
                    let end = name.iter().position(|byte| *byte == 0).unwrap_or(name.len());
                    world::unit_named(who, &String::from_utf8_lossy(&name[..end]));
                }
            }
            // ZC_SPRITE_CHANGE (0x01d7): `u32 who, u8 kind, u32 value, u32 second`. What someone wears or holds
            // changes: it shows on them at once.
            (Server::Map, 0x01d7) => {
                if packet.len() >= 11 {
                    let who = u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]]);
                    let (kind, value) = (packet[6], u32::from_le_bytes([packet[7], packet[8], packet[9], packet[10]]));
                    let own = Some(who) == self.account.as_ref().map(|account| account.account_id);
                    world::looked((!own).then_some(who), kind, value);
                    // The windows that show the character follow.
                    let char_id = self.zone.as_ref().map(|zone| zone.char_id);
                    if let Some(character) = self.characters.iter_mut().find(|character| own && Some(character.char_id) == char_id) {
                        match kind {
                            1 => character.hair_style = value as _,
                            3 => character.headgear[0] = value as u16,
                            4 => character.headgear[1] = value as u16,
                            5 => character.headgear[2] = value as u16,
                            6 => character.hair_color = value as _,
                            7 => character.body_palette = value as _,
                            _ => {}
                        }
                        self.refresh_items();
                    }
                }
            }
            // ZC_RESURRECTION (0x0148): `u32 who, u16 how`. The player's own character stands up again, and the
            // window that offered the save point goes.
            (Server::Map, 0x0148) => {
                if packet.len() >= 6 && Some(u32::from_le_bytes([packet[2], packet[3], packet[4], packet[5]])) == self.account.as_ref().map(|account| account.account_id) {
                    self.dead = false;
                    self.esc = 0;
                    world::own_dead(false);
                    self.refresh_dialog();
                    self.refresh_values();
                }
            }
            (Server::Map, _) => {
                if let Some(game) = &mut self.game {
                    game.packets += 1;
                    if let Some(counter) = element("packets") {
                        counter.set_text_content(Some(&game.packets.to_string()));
                    }
                }
            }
            _ => {}
        }
    }

    /// Enter or a double-click on a slot: into the game with its character, or to making one in it.
    fn open(&mut self, slot: u8) {
        if !matches!(self.screen, Screen::CharSelect) || self.wait.is_some() {
            return;
        }
        if let Some(character) = self.character_in(slot) {
            // A character waiting to be deleted stays out of the game.
            if self.deletions.contains_key(&character.char_id) {
                return;
            }
            if let Some(socket) = &self.socket {
                socket.send(&packets::select_char(slot));
                self.wait = Some(Key::PleaseWait);
                self.render();
            }
        } else if slot < self.slots {
            self.make = Make::default();
            self.show(Screen::CharCreate(slot));
        }
    }

    fn entered_map(&mut self, packet: &[u8]) {
        let (Some(accepted), Some(zone), Some(socket)) = (packets::map_accepted(packet), &self.zone, &self.socket) else {
            return self.leave(Some(Key::Rejected.text(self.lang).to_owned()));
        };
        let name = self.characters.iter().find(|character| character.char_id == zone.char_id).map(|character| character.name.clone()).unwrap_or_default();
        let map = zone.map.clone();
        // The map counts as loaded: the server now puts the character into the world, where other players see it.
        socket.send(&packets::map_loaded());
        // Until the map server sends the character's values, the ones of the character list stand in.
        let character = self.characters.iter().find(|character| character.char_id == zone.char_id);
        let job = character.map_or(0, |character| character.job);
        self.params.clear();
        self.inventory.clear();
        self.skills.clear();
        self.hotkeys.clear();
        self.stat_bonus = [0; 6];
        self.chat.clear();
        if let Some(character) = character {
            use packets::parameter::*;
            self.params.extend([
                (HP, character.hp as i64),
                (MAX_HP, character.max_hp as i64),
                (SP, character.sp as i64),
                (MAX_SP, character.max_sp as i64),
                (BASE_LEVEL, i64::from(character.base_level)),
                (JOB_LEVEL, i64::from(character.job_level)),
                (BASE_EXP, character.base_exp as i64),
                (ZENY, i64::from(character.zeny)),
            ]);
            for (stat, value) in character.stats.iter().enumerate() {
                self.params.insert(STR + stat as u16, i64::from(*value));
            }
        }
        // The chat starts four rows tall; hidden by the player, it stays hidden.
        if self.chat_rows == 0 && self.chat_rows_kept == 0 {
            self.chat_rows = 4;
        }
        self.game = Some(Game { name, job, map, x: accepted.x, y: accepted.y, packets: 0, map_size: None });
        self.screen = Screen::Game;
        self.tune();
        self.enter_world();
    }

    /// The player's own character as a figure, standing and facing the screen, with the weapon it holds.
    pub(super) fn own_figure(&self) -> Figure {
        let character = self.zone.as_ref().and_then(|zone| self.characters.iter().find(|character| character.char_id == zone.char_id));
        Figure {
            job: character.map_or(0, |character| character.job),
            sex: character.map_or(1, |character| character.sex),
            hair_style: character.map_or(1, |character| character.hair_style),
            hair_colour: character.map_or(0, |character| u16::from(character.hair_color)),
            headgear: character.map_or([0; 3], |character| character.headgear),
            body_palette: character.map_or(0, |character| character.body_palette),
            weapon: self.inventory.iter().find(|item| item.worn & 2 != 0).map_or(0, |item| item.id),
            action: 0,
            frame: 0,
        }
    }

    /// Draws the map the character is on. The wait box stays until it shows.
    fn enter_world(&mut self) {
        let (Some(game), Some(zone)) = (&self.game, &self.zone) else { return };
        let _ = zone;
        let figure = self.own_figure();
        let hooks = world::Hooks {
            walk_to: |x, y| dispatch(Action::WalkTo(x, y)),
            press: |id, monster| dispatch(Action::Pressed(id, monster)),
            take: |id| dispatch(Action::Take(id)),
            moved: |x, y| dispatch(Action::Moved(x, y)),
            ready: |outcome| dispatch(Action::WorldReady(outcome)),
            aimed: |whom, cell| dispatch(Action::Aimed(whom, cell)),
        };
        sound::map_track(data::music(&game.map));
        world::enter(game.map.clone(), (game.x, game.y), figure, hooks);
        // The client's loading screen stands over everything until the map shows.
        loading(Some(self.loadings));
        self.loadings += 1;
        loading_ahead(self.loadings);
        self.wait = None;
        self.ask = None;
        self.render();
    }

    fn create_character(&mut self) {
        let Screen::CharCreate(slot) = self.screen else { return };
        let name = field("char-name");
        let name = name.trim();
        if !NAME_LENGTH.contains(&name.len()) || !name.chars().all(|character| character.is_ascii_alphanumeric()) {
            return self.say(Key::BadName);
        }
        if let Some(socket) = &self.socket {
            // A human starts as a Novice (job 0), a Doram as a Summoner.
            let job = if self.make.doram { u32::from(data::SUMMONER) } else { 0 };
            socket.send(&packets::make_char(name, slot, self.make.hair_colour, self.make.hair_style, job, self.make.sex));
            self.wait = Some(Key::PleaseWait);
            self.render();
        }
    }

    pub(super) fn render(&self) {
        let Ok(ui) = ui() else { return };
        let view = web_sys::window().map_or((1024.0, 768.0), |window| {
            let size = |value: Result<JsValue, JsValue>, fallback: f64| value.ok().and_then(|value| value.as_f64()).unwrap_or(fallback);
            (size(window.inner_width(), 1024.0), size(window.inner_height(), 768.0))
        });
        // A window redrawn (a message shown, the view resized) keeps what was typed into it.
        let typed = KEPT_FIELDS.map(|id| (id, field(id)));
        // The windows are laid out for the original client's screen, 1024 by 768, and enlarged together on a
        // larger view, so they keep the share of the screen they have there. A smaller view shrinks each window
        // to fit instead (`view::window`).
        let scale = (view.0 / 1024.0).min(view.1 / 768.0).max(1.0);
        let stage = (view.0 / scale, view.1 / scale);
        // Too narrow a view is covered by a note saying why. The windows stay underneath as they are, so a wider
        // window brings everything back, what was typed included.
        // In the game the canvas shows through, and takes the mouse where no window is.
        let _ = ui.set_attribute("class", if matches!(self.screen, Screen::Game) { "ingame" } else { "" });
        let narrow = if view.0 < MIN_WIDTH {
            format!(r#"<div id="narrow"><p>{}</p></div>"#, fill(Key::TooNarrow.text(self.lang), &[&MIN_WIDTH.to_string()]))
        } else {
            String::new()
        };
        ui.set_inner_html(&format!(
            r#"<div class="stage" style="width:{}px;height:{}px;transform:scale({scale})">{}</div>{narrow}"#,
            stage.0,
            stage.1,
            self.html(stage)
        ));
        for (id, value) in typed {
            if !value.is_empty() {
                set_field(id, &value);
            }
        }
        // `autofocus` only works on page load; here the topmost window's marked element is focused.
        if let Ok(marked) = ui.query_selector_all("[autofocus]") {
            if let Some(last) = marked.length().checked_sub(1).and_then(|index| marked.item(index)) {
                let _ = last.unchecked_into::<web_sys::HtmlElement>().focus();
            }
        }
        self.paint_figures();
        place_moved();
    }

    pub(super) fn paint_figures(&self) {
        match self.screen {
            Screen::CharSelect => {
                for character in &self.characters {
                    let figure = Figure {
                        job: character.job,
                        sex: character.sex,
                        hair_style: character.hair_style,
                        hair_colour: u16::from(character.hair_color),
                        headgear: character.headgear,
                        body_palette: character.body_palette,
                        weapon: 0,
                        action: if self.deletions.contains_key(&character.char_id) { SITTING } else { 0 },
                        frame: 0,
                    };
                    figure::paint(&format!("pc{}", character.slot), figure);
                }
            }
            // The character in the Equipment window.
            Screen::Game if self.equipment_open || self.skills_open => {
                if self.equipment_open {
                    figure::paint("pc-equip", self.own_figure());
                }
                // The character in the skill window.
                if self.skills_open {
                    figure::paint("pc-skill", self.own_figure());
                }
            }
            Screen::CharCreate(_) => {
                let make = &self.make;
                let job = if make.doram { data::SUMMONER } else { 0 };
                let figure = Figure { job, sex: make.sex, hair_style: make.hair_style, hair_colour: make.hair_colour, headgear: [0; 3], body_palette: 0, weapon: 0, action: usize::from(make.direction), frame: 0 };
                figure::paint("pc-make", figure);
            }
            _ => {}
        }
    }
}
