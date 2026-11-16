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
use crate::net::items::{self, Hotkey, Item, Skill, Tab};
use crate::net::packets::{self, Character, LoginAccepted, ZoneServer};
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
    /// The character is on another cell now.
    Moved(u16, u16),
    /// The map is drawn, or could not be.
    WorldReady(Result<(), String>),
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
    account: Option<LoginAccepted>,
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
    /// lines the chat shows (an index into `hud::CHAT_STEPS`).
    pub basic_small: bool,
    pub status_open: bool,
    pub hotkeys_hidden: bool,
    pub chat_step: usize,
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
    // The frame around a chosen character runs through eight pictures.
    if let Some(frames) = element("frames") {
        let steps: String = (0..8).map(|frame| format!("{}%{{background-image:url('{}')}}", frame as f32 * 12.5, super::view::chosen_frame(frame))).collect();
        frames.set_text_content(Some(&format!("@keyframes chosen{{{steps}}}")));
    }

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
        // Every button clicks, as in the original client.
        if pressed(&event, "button").is_some() && APP.with_borrow(|app| app.sound) {
            sound::button();
        }
        APP.with_borrow(App::tune);
        if let Some(pressed) = pressed(&event, "data-click") {
            dispatch(Action::Click(pressed.get_attribute("data-click").unwrap_or_default(), pressed.get_attribute("data-arg").unwrap_or_default()));
        }
    });
    let on_double_click = Closure::<dyn FnMut(Event)>::new(|event: Event| {
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
    });
    let on_key = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
        if APP.with_borrow_mut(|app| app.key(&event.key(), &event.code(), event.alt_key())) {
            event.prevent_default();
        }
        APP.with_borrow(App::tune);
    });
    // A browser plays nothing before the first press on the page, wherever it lands: the music starts with it.
    let on_press = Closure::<dyn FnMut()>::new(|| {
        APP.with_borrow(App::tune);
        check_music_soon();
    });
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
                "login" => self.login(),
                "register" => self.register(),
                "recover" => self.recover(),
                "reset" => self.reset(),
                "create" => self.create_character(),
                "chat" => {
                    let words = field("chat-input");
                    let words = words.trim();
                    if let (false, Some(socket), Some(game)) = (words.is_empty(), &self.socket, &self.game) {
                        socket.send(&packets::say(&game.name, words));
                        set_field("chat-input", "");
                    }
                }
                _ => {}
            },
            Action::Click(what, arg) => self.click(&what, &arg),
            Action::Open(slot) => self.open(slot),
            Action::Use(what) => self.use_it(&what),
            Action::Dropped(slot, dragged) => self.set_hotkey(slot, &dragged),
            Action::ItemsKnown => self.refresh_items(),
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
            Action::WalkTo(x, y) => {
                if let (Server::Map, Some(socket)) = (self.server, &self.socket) {
                    socket.send(&packets::walk_to(x, y));
                }
            }
            Action::Moved(x, y) => {
                if let Some(game) = &mut self.game {
                    (game.x, game.y) = (x, y);
                    if let Some(position) = element("position") {
                        position.set_text_content(Some(&format!("{x}, {y}")));
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
            }
            Action::Answer(request, answer) => self.answered(request, answer),
        }
    }

    fn click(&mut self, what: &str, arg: &str) {
        match what {
            "lang" => {
                if let Some(lang) = Lang::from_code(arg) {
                    self.lang = lang;
                    store("roweb.lang", Some(lang.code()));
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
                self.selected = slot;
                // An empty slot opens the making of a character at once, as in the original client.
                if self.character_in(slot).is_none() {
                    return self.open(slot);
                }
                self.render();
            }
            "start" => self.open(self.selected),
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
            (_, "KeyS", true) => self.skills_open = !self.skills_open,
            // F1 to F9 use the first row of the hotkey bar.
            (_, "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9", false) => {
                let slot = code[1..].parse::<usize>().unwrap_or(1) - 1;
                self.use_it(&format!("slot:{slot}"));
                return true;
            }
            ("F12", ..) => self.hotkeys_hidden = !self.hotkeys_hidden,
            ("F10", ..) => self.chat_step = (self.chat_step + 1) % super::hud::CHAT_STEPS.len(),
            // Enter goes to the chat's line; from there it sends (the form does that).
            ("Enter", ..) if !writing => {
                if let Some(input) = chat {
                    let _ = input.focus();
                }
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
                    // Skills aimed at someone or somewhere wait for there to be someone to aim at.
                    self.chat_line("This skill needs a target, and nobody else is drawn on the map yet.".to_owned(), "#f00");
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
        let ids = self.inventory.iter().map(|item| item.id).chain(self.hotkeys.iter().filter(|hotkey| !hotkey.skill).map(|hotkey| hotkey.id));
        data::want_items(ids, || dispatch(Action::ItemsKnown));
        self.refresh_items();
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
                Ok(None) => break,
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
                Some(account) if !account.char_servers.is_empty() => {
                    store("roweb.id", (!self.saved_id.is_empty()).then_some(self.saved_id.as_str()));
                    let port = account.char_servers[0].port;
                    self.account = Some(account);
                    self.connect(Server::Char, port);
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
                // Everyone's stops come by; only the player's own character is drawn so far.
                let own = self.account.as_ref().map(|account| account.account_id);
                if let Some((_, x, y)) = packets::stop_move(packet).filter(|(id, ..)| Some(*id) == own) {
                    world::stand((x, y));
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
            }
            (Server::Map, items::ZC_INVENTORY_ITEMLIST_NORMAL) => {
                self.inventory.extend(items::stackable(packet));
                self.items_changed();
            }
            (Server::Map, items::ZC_INVENTORY_ITEMLIST_EQUIP) => {
                self.inventory.extend(items::equipment(packet));
                self.items_changed();
            }
            (Server::Map, items::ZC_ITEM_PICKUP_ACK) => {
                if let Some(gained) = items::gained(packet) {
                    match self.inventory.iter_mut().find(|item| item.index == gained.index) {
                        Some(item) => item.amount += gained.amount,
                        None => self.inventory.push(gained),
                    }
                    self.items_changed();
                }
            }
            (Server::Map, items::ZC_USE_ITEM_ACK) => {
                if let Some((index, left)) = items::used(packet) {
                    if let Some(item) = self.inventory.iter_mut().find(|item| item.index == index) {
                        item.amount = left;
                    }
                    self.inventory.retain(|item| item.amount > 0);
                    self.refresh_items();
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
                    self.refresh_items();
                }
            }
            (Server::Map, items::ZC_REQ_TAKEOFF_EQUIP_ACK) => {
                if let Some(index) = items::taken_off(packet) {
                    if let Some(item) = self.inventory.iter_mut().find(|item| item.index == index) {
                        item.worn = 0;
                    }
                    self.refresh_items();
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
                    let colour = match id {
                        packets::ZC_NOTIFY_PLAYERCHAT => "#0f0".to_owned(),
                        packets::ZC_NOTIFY_CHAT => "#fff".to_owned(),
                        packets::ZC_BROADCAST2 => packets::broadcast_colour(packet).map_or("#ff0".to_owned(), |[red, green, blue]| format!("rgb({red},{green},{blue})")),
                        _ => "#ff0".to_owned(),
                    };
                    self.chat_line(words, &colour);
                }
            }
            // Moved to another place, on this map or another: the map is loaded and the server told so.
            (Server::Map, packets::ZC_NPCACK_MAPMOVE) => {
                if let Some((map, x, y)) = packets::map_move(packet) {
                    if let Some(game) = &mut self.game {
                        (game.map, game.x, game.y, game.map_size) = (map, x, y, None);
                    }
                    self.enter_world();
                    if let Some(socket) = &self.socket {
                        socket.send(&packets::map_loaded());
                    }
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
        self.game = Some(Game { name, job, map, x: accepted.x, y: accepted.y, packets: 0, map_size: None });
        self.screen = Screen::Game;
        self.tune();
        self.enter_world();
    }

    /// Draws the map the character is on. The wait box stays until it shows.
    fn enter_world(&mut self) {
        let (Some(game), Some(zone)) = (&self.game, &self.zone) else { return };
        let character = self.characters.iter().find(|character| character.char_id == zone.char_id);
        let figure = Figure {
            job: character.map_or(0, |character| character.job),
            sex: character.map_or(1, |character| character.sex),
            hair_style: character.map_or(1, |character| character.hair_style),
            hair_colour: character.map_or(0, |character| u16::from(character.hair_color)),
            headgear: character.map_or([0; 3], |character| character.headgear),
            body_palette: character.map_or(0, |character| character.body_palette),
            action: 0,
            frame: 0,
        };
        let hooks = world::Hooks {
            walk_to: |x, y| dispatch(Action::WalkTo(x, y)),
            moved: |x, y| dispatch(Action::Moved(x, y)),
            ready: |outcome| dispatch(Action::WorldReady(outcome)),
        };
        world::enter(game.map.clone(), (game.x, game.y), figure, hooks);
        self.wait = Some(Key::PleaseWait);
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
    }

    fn paint_figures(&self) {
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
                        action: if self.deletions.contains_key(&character.char_id) { SITTING } else { 0 },
                        frame: 0,
                    };
                    figure::paint(&format!("pc{}", character.slot), figure);
                }
            }
            Screen::CharCreate(_) => {
                let make = &self.make;
                let job = if make.doram { data::SUMMONER } else { 0 };
                let figure = Figure { job, sex: make.sex, hair_style: make.hair_style, hair_colour: make.hair_colour, headgear: [0; 3], body_palette: 0, action: usize::from(make.direction), frame: 0 };
                figure::paint("pc-make", figure);
            }
            _ => {}
        }
    }
}
