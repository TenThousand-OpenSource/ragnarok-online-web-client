//! The interface's own tables: the client's messages, keys, options and emotions.

use super::*;

thread_local! {
    /// The client's keys: its four tables, each entry's name and the keys it starts with.
    static HOTKEYS: RefCell<Vec<Vec<(String, crate::hotkeys::Chord)>>> = const { RefCell::new(Vec::new()) };
    /// How the client writes each key, by the key's number.
    static KEY_NAMES: RefCell<HashMap<u16, String>> = RefCell::new(HashMap::new());
    /// The options of the client's Game Settings window, in its order.
    static OPTIONS: RefCell<Vec<GameOption>> = const { RefCell::new(Vec::new()) };
    /// The client's message table, by line.
    static MESSAGES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// The client's emotions by their number: the sprite each is in, and which of its actions it is.
    static EMOTIONS: RefCell<HashMap<u16, (String, usize)>> = RefCell::new(HashMap::new());
    /// What is typed after a slash to show an emotion, in small letters, and the emotion.
    static EMOTION_NAMES: RefCell<HashMap<String, u16>> = RefCell::new(HashMap::new());
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

/// Takes its tables out of what the gateway gave: the client's messages, the options of its Game Settings
/// window, its keys, its emotions.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    MESSAGES.set(field(tables, "messages").dyn_into::<js_sys::Array>().map(|lines| lines.iter().map(|line| line.as_string().unwrap_or_default()).collect()).unwrap_or_default());
    OPTIONS.set(
        field(tables, "options")
            .dyn_into::<js_sys::Array>()
            .map(|list| list.to_vec())
            .unwrap_or_default()
            .into_iter()
            .map(|option| {
                let flag = |name: &str| js_sys::Reflect::get(&option, &name.into()).ok().and_then(|flag| flag.as_bool()).unwrap_or(false);
                GameOption {
                    id: text_of(&option, "id").unwrap_or_default(),
                    tab: js_sys::Reflect::get(&option, &"tab".into()).ok().and_then(|tab| tab.as_f64()).unwrap_or(0.0) as u8,
                    run: flag("run"),
                    title: text_of(&option, "title").unwrap_or_default(),
                    description: text_of(&option, "description").unwrap_or_default(),
                    on: flag("on"),
                }
            })
            .collect(),
    );
    let keys = field(tables, "hotkeys");
    let part = |of: &wasm_bindgen::JsValue, name: &str| js_sys::Reflect::get(of, &name.into()).unwrap_or(wasm_bindgen::JsValue::UNDEFINED);
    let listed = |of: wasm_bindgen::JsValue| of.dyn_into::<js_sys::Array>().map(|list| list.to_vec()).unwrap_or_default();
    HOTKEYS.set(
        listed(part(&keys, "tables"))
            .into_iter()
            .map(|entries| {
                listed(entries)
                    .into_iter()
                    .map(|entry| {
                        let chord = listed(part(&entry, "keys"));
                        let number = |at: usize| chord.get(at).and_then(|number| number.as_f64()).unwrap_or(0.0) as u16;
                        (text_of(&entry, "name").unwrap_or_default(), crate::hotkeys::Chord { key: number(0), with: number(1) })
                    })
                    .collect()
            })
            .collect(),
    );
    KEY_NAMES.set(entries::<u16>(&part(&keys, "names")).into_iter().filter_map(|(key, name)| Some((key, name.as_string()?))).collect());
    let emotions = entries::<u16>(&field(tables, "emotions"));
    EMOTIONS.set(emotions.iter().filter_map(|(id, about)| Some((*id, (text_of(about, "sprite")?, js_sys::Reflect::get(about, &"action".into()).ok()?.as_f64()? as usize)))).collect());
    EMOTION_NAMES.set(
        emotions
            .iter()
            .flat_map(|(id, about)| {
                let names = js_sys::Reflect::get(about, &"names".into()).ok().and_then(|names| names.dyn_into::<js_sys::Array>().ok()).map(|names| names.to_vec()).unwrap_or_default();
                names.into_iter().filter_map(|name| name.as_string()).map(|name| (name.to_lowercase(), *id)).collect::<Vec<_>>()
            })
            .collect(),
    );
}
