//! The interface's own tables: the client's messages, keys, options and emotions.

use super::*;

thread_local! {
    /// The chat window's tabs as the client starts with them: for each, whether it lists a line of each kind.
    static CHAT_TABS: RefCell<Vec<Vec<bool>>> = const { RefCell::new(Vec::new()) };
    /// The names of those tabs, as the client's own names of lines of its message table (`MSI_ST_CHAT`).
    static CHAT_TAB_NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// The client's keys: its four tables, each entry's name and the keys it starts with.
    static HOTKEYS: RefCell<Vec<Vec<(String, crate::hotkeys::Chord)>>> = const { RefCell::new(Vec::new()) };
    /// How the client writes each key, by the key's number.
    static KEY_NAMES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The options of the client's Game Settings window, in its order.
    static OPTIONS: RefCell<Vec<GameOption>> = const { RefCell::new(Vec::new()) };
    /// Whether the server plays by Renewal's rules, not the classic game's.
    static RENEWAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The client's message table, by line.
    static MESSAGES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// The client's emotions by their number: the sprite each is in, and which of its actions it is.
    static EMOTIONS: RefCell<HashMap<u16, (String, usize)>> = RefCell::new(HashMap::new());
    /// What is typed after a slash to show an emotion, in small letters, and the emotion.
    static EMOTION_NAMES: RefCell<HashMap<String, u16>> = RefCell::new(HashMap::new());
    /// The first name of each emotion, as the client's list gives it.
    static EMOTION_FIRST_NAMES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
}

/// The entries of one of the client's four tables of keys (`crate::hotkeys::table`): each one's name, and the
/// keys the client gives it. An entry without a name is a place the client leaves empty.
pub fn hotkeys(table: usize) -> Vec<(String, crate::hotkeys::Chord)> {
    HOTKEYS.with_borrow(|tables| tables.get(table).cloned().unwrap_or_default())
}

/// How the client writes a key.
pub fn key_name(key: u16) -> Option<String> {
    KEY_NAMES.with_borrow(|names| names.get(&key).cloned())
}

/// One option of the client's Game Settings window (its `GameSettings.lub`).
#[derive(Clone, Default)]
pub struct GameOption {
    /// The client's name for it (`TT_EFFECT_ON_OFF`).
    pub id: String,
    /// The page it stands on: 1 effects, 2 controls, 4 the rest.
    pub tab: u8,
    /// A thing done at a press, not switched on and off.
    pub run: bool,
    pub title: String,
    pub description: String,
    /// How it starts.
    pub on: bool,
}

/// The options of one page of the Game Settings window.
pub fn options(tab: u8) -> Vec<GameOption> {
    OPTIONS.with_borrow(|options| options.iter().filter(|option| option.tab == tab).cloned().collect())
}

/// How an option starts, by the client's name for it; off for one the client's table does not have.
pub fn option_starts(id: &str) -> bool {
    OPTIONS.with_borrow(|options| options.iter().find(|option| option.id == id).is_some_and(|option| option.on))
}

/// Whether the server plays by Renewal's rules; by the classic game's until the tables are read.
pub fn renewal() -> bool {
    RENEWAL.get()
}

/// A line of the client's message table, by the number the client's code knows it by.
pub fn message(line: usize) -> Option<String> {
    MESSAGES.with_borrow(|messages| messages.get(line).cloned())
}

/// The sprite an emotion is in and which of its actions it is (the client's `CashEmotionList`).
pub fn emotion(id: u16) -> Option<(String, usize)> {
    EMOTIONS.with_borrow(|emotions| emotions.get(&id).cloned())
}

/// The emotion a chat command shows (`/ho`, `/delight`), if it is one.
pub fn emotion_named(command: &str) -> Option<u16> {
    EMOTION_NAMES.with_borrow(|names| names.get(&command.to_lowercase()).copied())
}

/// The chat window's tabs as the client starts with them (its `ChatWndInfo.lub`): the name of each, and whether
/// it lists a line of each kind. The file names a tab by a line of the message table: `MSI_ST_CHAT` is line
/// 1291, `MSI_BT_CHAT` line 1292 (the client's `MSGSTRINGID`). Until the table is read there are the two tabs
/// the client starts with, the first listing what its file has it list.
pub fn chat_tabs_start() -> Vec<crate::chat_tabs::Tab> {
    let named = |name: &str| match name {
        "MSI_ST_CHAT" => message(1291).unwrap_or_else(|| "Public".to_owned()),
        "MSI_BT_CHAT" => message(1292).unwrap_or_else(|| "Battle".to_owned()),
        other => other.to_owned(),
    };
    let tab = |name: String, lists: &[bool]| {
        let mut shows = [false; crate::chat_tabs::KINDS];
        shows.iter_mut().zip(lists).for_each(|(shown, listed)| *shown = *listed);
        crate::chat_tabs::Tab { name, shows, own: None }
    };
    let read: Vec<crate::chat_tabs::Tab> = CHAT_TABS.with_borrow(
        |tabs| CHAT_TAB_NAMES.with_borrow(|names| tabs.iter().zip(names).map(|(lists, name)| tab(named(name), lists)).collect())
    );
    if read.is_empty() { vec![tab(named("MSI_ST_CHAT"), &[true; 14]), tab(named("MSI_BT_CHAT"), &[])] } else { read }
}

/// Every emotion of the client's basic pack, by its number, in the order of the numbers.
pub fn emotions() -> Vec<u16> {
    let mut all: Vec<u16> = EMOTIONS.with_borrow(|emotions| emotions.keys().copied().collect());
    all.sort_unstable();
    all
}

/// What is typed after a slash to show an emotion: the first of its names (`/!`).
pub fn emotion_name(id: u16) -> Option<String> {
    EMOTION_FIRST_NAMES.with_borrow(|names| names.get(&id).cloned())
}

/// Takes its tables out of what the gateway gave: the client's messages, the options of its Game Settings
/// window, its keys, its emotions.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    CHAT_TABS.set(
        list(&field(tables, "chat_tabs"))
            .iter()
            .map(|tab| list(&field(tab, "shows")).iter().map(|shown| shown.as_bool().unwrap_or(false)).collect())
            .collect(),
    );
    CHAT_TAB_NAMES.set(list(&field(tables, "chat_tabs")).iter().map(|tab| text_of(tab, "name").unwrap_or_default()).collect());
    RENEWAL.set(field(tables, "renewal").as_bool().unwrap_or(false));
    MESSAGES.set(list(&field(tables, "messages")).iter().map(|line| line.as_string().unwrap_or_default()).collect());
    OPTIONS.set(
        list(&field(tables, "options"))
            .into_iter()
            .map(|option| GameOption {
                id: text_of(&option, "id").unwrap_or_default(),
                tab: number(&option, "tab").unwrap_or(0.0) as u8,
                run: flag(&option, "run"),
                title: text_of(&option, "title").unwrap_or_default(),
                description: text_of(&option, "description").unwrap_or_default(),
                on: flag(&option, "on"),
            })
            .collect(),
    );
    let keys = field(tables, "hotkeys");
    HOTKEYS.set(
        list(&field(&keys, "tables"))
            .iter()
            .map(|entries| {
                list(entries)
                    .iter()
                    .map(|entry| {
                        let chord = list(&field(entry, "keys"));
                        let number = |at: usize| chord.get(at).and_then(|number| number.as_f64()).unwrap_or(0.0) as u16;
                        (text_of(entry, "name").unwrap_or_default(), crate::hotkeys::Chord { key: number(0), with: number(1) })
                    })
                    .collect()
            })
            .collect(),
    );
    KEY_NAMES.set(names::<u16>(&field(&keys, "names")).into_iter().collect());
    let emotions = entries::<u16>(&field(tables, "emotions"));
    // What is typed after a slash to show an emotion, as the gateway lists it; none when it lists nothing.
    let names_of = |about: &wasm_bindgen::JsValue| field(about, "names").dyn_into::<js_sys::Array>().ok();
    EMOTIONS.set(emotions.iter().filter_map(|(id, about)| Some((*id, (text_of(about, "sprite")?, number(about, "action")? as usize)))).collect());
    EMOTION_FIRST_NAMES.set(
        emotions
            .iter()
            .filter_map(|(id, about)| {
                Some((*id, names_of(about)?.get(0).as_string()?))
            })
            .collect(),
    );
    EMOTION_NAMES.set(
        emotions
            .iter()
            .flat_map(|(id, about)| {
                let names = names_of(about).map(|names| names.to_vec()).unwrap_or_default();
                names.into_iter().filter_map(|name| name.as_string()).map(|name| (name.to_lowercase(), *id)).collect::<Vec<_>>()
            })
            .collect(),
    );
}
