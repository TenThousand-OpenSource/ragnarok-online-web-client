//! What the app keeps, in groups: one for each part of the game (the bag, the skills, the chat, a deal, the pets…).
//! `App` itself, which holds one of each, is in `app.rs`.

use super::*;

pub(in crate::web) struct Game {
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
pub(in crate::web) struct Taming {
    /// The monster being tamed.
    pub(super) target: u32,
    /// Which action of the sprite plays: 0 waiting, 1 the reels turning.
    pub(super) state: usize,
    /// The client's counters: the picture shown is `tick / delay`.
    pub(super) tick: i32,
    pub(super) delay: i32,
    pub(super) longest: i32,
    /// The picture to stop on (-1: none yet).
    pub(super) wanted: i32,
    pub(super) started: bool,
    /// When it came to rest: it closes two seconds later.
    pub(super) rested: Option<f64>,
    /// The pictures of both actions, once read: each a data URL, its size and where its origin is.
    pub frames: [Vec<(String, (usize, usize), (f32, f32))>; 2],
    /// The picture on the page now.
    pub shown: (usize, usize),
    /// The page's timer that drives it.
    pub(super) timer: i32,
}

/// The pseudo-skill the client aims with when a monster is to be tamed (`Zc_Start_Capture`).
pub(super) const TAME: u16 = 10000;

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

/// An NPC's shop, as far as the dealing has come.
pub(in crate::web) enum Shop {
    /// The shop asks whether to buy or to sell.
    Choose(u32),
    Buy(Vec<trade::Offer>),
    /// What the shop takes of what is carried: `(index, price)`.
    Sell(Vec<(u16, u32)>),
}

/// The Kafra's storage while it is open.
#[derive(Default)]
pub(in crate::web) struct Storage {
    pub items: Vec<items::Item>,
    /// How many kinds of things are in it, and how many fit.
    pub count: (u16, u16),
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
    pub equipment_open: bool,
    /// The item whose description is open, and whether it is identified.
    pub info: Option<(u32, bool)>,
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
}

/// The kinds of line the chat window keeps, by the numbers the client's code gives them (the last but two of
/// what it sends with `UIM_PUSHINTOCHATHISTORY`): each tab of the window lists some kinds and not others.
pub(super) mod said {
    /// Anything of no particular kind: most of what the game itself says.
    pub const GENERAL: u8 = 0;
    /// What someone says aloud.
    pub const CHAT: u8 = 1;
    pub const WHISPER: u8 = 2;
    /// A thing obtained.
    pub const ITEM: u8 = 6;
    /// What is worn: put on, taken off, refused.
    pub const EQUIPMENT: u8 = 7;
    /// A skill that failed.
    pub const SKILL_FAILED: u8 = 11;
}

/// Dealing with NPCs: a shop and the Kafra's storage.
#[derive(Default)]
pub(in crate::web) struct Deal {
    pub shop: Option<Shop>,
    /// The row picked in the shop's list, and what was put aside to buy or sell: `(row, amount)`.
    pub picked: Option<usize>,
    pub basket: Vec<(usize, u16)>,
    pub storage: Option<Storage>,
    /// Which of the storage's tabs is open.
    pub storage_tab: usize,
}

/// The Adventure Guide's window: whether it is open, which of its tabs, the monster picked in its list, and the
/// monsters' pictures made so far (and those asked for), by the monster's number.
#[derive(Default)]
pub(in crate::web) struct GuideWnd {
    pub open: bool,
    pub tab: usize,
    /// The kind picked under the tab (the tab's own smaller tabs), where it has kinds: its number.
    pub kind: Option<u32>,
    /// The open tab's kinds are folded away: a press on the tab that is open already does that, and another
    /// brings them back (the client's `CUITab::SetSelected`).
    pub folded: bool,
    pub picked: Option<String>,
    pub pictures: HashMap<u32, (String, (usize, usize), (f32, f32))>,
    pub asked: std::collections::HashSet<u32>,
}

/// The Game Settings window, and what the player has set in it.
#[derive(Default)]
pub(in crate::web) struct Settings {
    /// The window is open, and which of its five pages.
    pub open: bool,
    pub tab: usize,
    /// The options the player has switched otherwise than they start, by the client's name: kept in the browser.
    pub set: HashMap<String, bool>,
    /// How loud the music and the effects are (0 to 100), and whether each is off: kept in the browser.
    pub music: u8,
    pub effects: u8,
    pub music_off: bool,
    pub effects_off: bool,
}

/// The keys: what the player has bound otherwise than the client does, and the window that shows and sets them.
#[derive(Default)]
pub(in crate::web) struct Keys {
    /// The Shortcut Settings window is open; which of its tabs, which page of it, and the row picked in it.
    pub open: bool,
    pub tab: usize,
    pub page: usize,
    pub row: Option<usize>,
    /// What the player has set, by table and place: kept in the browser.
    pub own: HashMap<(usize, usize), Chord>,
    /// What was set in the window since it was opened, until OK keeps it or Cancel drops it.
    pub edited: HashMap<(usize, usize), Chord>,
    /// The keyboard is a Mac's: some keys are missing on it, and Alt is called Option.
    pub mac: bool,
    /// What the window says of the last key pressed in it, when it would not take it.
    pub notice: Option<String>,
}

/// The player's pet: catching one, hatching it, and its window.
#[derive(Default)]
pub(in crate::web) struct Pets {
    /// The slot machine of taming a monster, while it is open.
    pub taming: Option<Taming>,
    /// The pet eggs offered for hatching (their places in the inventory), while the incubator's list is open,
    /// and which of them is picked.
    pub eggs: Option<Vec<u16>>,
    pub egg_picked: usize,
    /// The pet, while one is out of its egg, and the unit on the map that is it.
    pub pet: Option<pets::Pet>,
    pub unit: u32,
    /// The pet window is open; its menu of commands is; what it asks before acting (1 whether to feed, 2
    /// whether to give the name typed, kept in `named`).
    pub open: bool,
    pub menu: bool,
    pub asks: u8,
    pub named: String,
    /// The answer to feeding the pet (whether it ate, and its food), until the chat has told of it: the food's
    /// name may arrive a moment later.
    pub fed: Option<(bool, u32)>,
    /// The pet window was opened by itself for this pet's low intimacy: it is, once for a pet.
    pub alerted: bool,
    /// The menu of the pet, opened with the right button on it: where on the stage.
    pub menu_at: Option<(i32, i32)>,
    /// How hungry the pet was before its hunger last changed: what it feels on being fed goes by that.
    pub hunger_before: i32,
    /// When the pet last told of its master's danger (milliseconds since 1970): at most once in five seconds.
    pub warned_at: f64,
}
