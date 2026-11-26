//! The windows of the game's screen as one list: each with its element, when it is open, where it stands, what
//! draws it, how it lies among the others and what Escape does to it. Everything that goes through the windows
//! goes through this list: the screen is written from it (`App::hud`), a window is drawn anew from it
//! (`App::redraw`), and the order the windows lie in and what Escape closes are read from it
//! (`app/windows.rs`). A new window is a new entry in its system's file of `windows/`, its name in `ALL`, and
//! its drawing in its system's file of `hud/`; nothing else has to know of it, the page's style sheet least of
//! all: how a window lies over the others is its entry's `layer`.
//!
//! A window's element is always on the page: closed, it is empty and hidden. So a window can open, close and
//! change without the whole screen being written again.
//!
//! Here are the type, what the entries are written with (`ordered`, `Window::usual` for a plain window, and
//! the kinds of window that are the same in every system: `Window::question`, `Window::count`,
//! `Window::menu`, `Window::pick`), and `ALL`. Where a window stands is written with `windows/places.rs`
//! (`framed`, `middle`, `staged`); the screen is written and a window drawn anew by `windows/redraw.rs`; the
//! entries themselves are in `windows/`, a file to a system.

use super::*;

/// Closing a window whose being open is one switch: the switch off, and the screen written again.
macro_rules! switched_off {
    ($($field:ident).+) => {
        |app: &mut App| {
            app.$($field).+ = false;
            app.render();
        }
    };
}

mod places;
mod redraw;
pub(in crate::web) use places::menu_style;
use places::*;
use redraw::{typed_into, typed_over};

mod basic;
mod choosing;
mod equipment;
mod signs;
mod statuses;
mod exchange;
mod fight;
mod friends;
mod guide;
mod guild;
mod homunculus;
mod mercenary;
mod inventory;
mod mail;
mod cart;
mod vending;
mod party;
mod pets;
mod player_menu;
mod quests;
mod settings;
mod shops;
mod skills;
mod status;
mod storage;
mod talk;
mod viewequip;
mod whispers;
mod broadcast;
mod unread;
mod scripted;
mod chat_rooms;
mod family;
mod chat;
mod emotions;
mod tips;
mod worldmap;
mod navigation;
mod books;
mod cashshop;
mod achievements;
mod reputation;
mod probability;
mod attendance;
mod bank;
mod battleground;
mod agency;
mod replay;
pub(in crate::web) use {exchange::*, friends::*, guide::*, guild::*, inventory::*, mail::*, party::*, pets::*, player_menu::*, quests::*};
pub(in crate::web) use choosing::*;
pub(in crate::web) use {homunculus::*, mercenary::*};
pub(in crate::web) use unread::*;
pub(in crate::web) use {chat_rooms::*, family::*, scripted::*};
pub(in crate::web) use broadcast::*;
pub(in crate::web) use {chat::*, emotions::*, tips::*, worldmap::*};
pub(in crate::web) use fight::*;
pub(in crate::web) use navigation::*;
pub(in crate::web) use books::*;
pub(in crate::web) use cashshop::*;
pub(in crate::web) use {achievements::*, probability::*, reputation::*};
pub(in crate::web) use attendance::*;
pub(in crate::web) use bank::*;
pub(in crate::web) use battleground::*;
pub(in crate::web) use agency::*;
pub(in crate::web) use replay::*;
pub(in crate::web) use {cart::*, vending::*};
pub(in crate::web) use {settings::*, shops::*, skills::*, status::*, storage::*, talk::*, viewequip::*, whispers::*};
pub(in crate::web) use {basic::*, equipment::*, signs::*, statuses::*};

/// A window of the game's screen, or a fixed piece of it (the hotkey bar, the chat).
pub(in crate::web) struct Window {
    /// The ID of its element.
    pub id: &'static str,
    /// Whether it is open.
    pub open: fn(&App) -> bool,
    /// What its element's tag says beside the ID: its class, where it stands on a stage of that size, and
    /// anything more (`framed` writes the usual).
    pub frame: fn(&App, (f64, f64)) -> String,
    /// What is in it.
    pub draw: fn(&App) -> String,
    /// Its part in the order the windows lie in, for one that has a part (`app/windows.rs`).
    pub order: Option<Order>,
    /// How it lies among the others while it has no part in the order.
    pub layer: Layer,
    /// What is done to its element after it was drawn anew: a field to type into is given the keyboard, a text
    /// is shown from its end.
    pub drawn: Option<fn(&web_sys::Element)>,
}

/// A window's part in the order: when it lies in it (most whenever they are open), and what closes it, which is
/// what Escape does to the one in front.
pub(in crate::web) struct Order {
    pub when: fn(&App) -> bool,
    pub close: fn(&mut App),
}

/// How a window lies among the others. The windows that lie in the order take their heights from it
/// (`app/windows.rs`: 11 and up); the rest lie as this says, and the page's style sheet has one rule to a
/// layer (`[data-layer=…]` in `index.html`) and none to a window.
#[derive(Clone, Copy, PartialEq)]
pub(in crate::web) enum Layer {
    /// As the list has it: a later window over an earlier one.
    Usual,
    /// Under the windows: the chat, the notice of mail.
    Under,
    /// A window that asks something (what an NPC says, a question to answer, how many of a thing): over the
    /// windows that lie in the order.
    Asks,
    /// A menu opened with the right button: over those too.
    Menu,
    /// Over everything of the screen: the slot machine of taming.
    Top,
}

impl Layer {
    /// What the window's tag says of its layer, for the page's style sheet.
    fn written(self) -> &'static str {
        match self {
            Layer::Usual => "",
            Layer::Under => r#" data-layer="under""#,
            Layer::Asks => r#" data-layer="asks""#,
            Layer::Menu => r#" data-layer="menu""#,
            Layer::Top => r#" data-layer="top""#,
        }
    }
}

/// A window that lies in the order whenever it is open, and is closed by setting what says it is open.
const fn ordered(open: fn(&App) -> bool, close: fn(&mut App)) -> Option<Order> {
    Some(Order { when: open, close })
}

impl Window {
    /// A question in the client's message box (`hud::message_box`): 280 wide, 200 of 480 down the middle of
    /// the screen, over the windows.
    const fn question(id: &'static str, open: fn(&App) -> bool, draw: fn(&App) -> String) -> Window {
        Window { id, open, frame: |_, view| middle(view, 280, 200), draw, order: None, layer: Layer::Asks, drawn: None }
    }

    /// The box that asks how many of a thing (`App::count_box`, the client's `UIItemDropCntWnd`): 182 wide in
    /// the same place, the number ready to be typed over.
    const fn count(id: &'static str, open: fn(&App) -> bool, draw: fn(&App) -> String) -> Window {
        Window { id, open, frame: |_, view| middle(view, 182, 200), draw, order: None, layer: Layer::Asks, drawn: Some(typed_over) }
    }

    /// A menu opened with the right button, where the pointer was (`at` tells where: `menu_frame`), its rows
    /// drawn with `hud::menu_row`.
    const fn menu(id: &'static str, open: fn(&App) -> bool, frame: fn(&App, (f64, f64)) -> String, draw: fn(&App) -> String) -> Window {
        Window { id, open, frame, draw, order: None, layer: Layer::Menu, drawn: None }
    }

    /// A usual window: dragged by its title, in the order whenever it is open, and closed by `close`, which is
    /// also what Escape does to it when it is the one in front.
    const fn usual(id: &'static str, open: fn(&App) -> bool, frame: fn(&App, (f64, f64)) -> String, draw: fn(&App) -> String, close: fn(&mut App)) -> Window {
        Window { id, open, frame, draw, order: ordered(open, close), layer: Layer::Usual, drawn: None }
    }

    /// A list to pick one thing of (`hud::pick_list`, the client's `UIItemIdentifyWnd` and those made of it):
    /// 200 wide, 200 of 480 down the middle of the screen, over the windows.
    const fn pick(id: &'static str, open: fn(&App) -> bool, draw: fn(&App) -> String) -> Window {
        Window { id, open, frame: |_, view| middle(view, 200, 200), draw, order: None, layer: Layer::Asks, drawn: None }
    }

    /// The name of the field of a window that has one to type into (the number of a box that asks how many).
    pub(in crate::web) fn field(&self) -> String {
        format!("{}-field", self.id)
    }
}

const ALWAYS: fn(&App) -> bool = |_| true;

/// Every window of the screen, as they are written to the page: a later one lies over an earlier one where
/// neither has a place in the order and both are of one layer.
pub(in crate::web) static ALL: &[&Window] = &[
    &STATUSES,
    &SIGNS, &CUTIN, &ROOM, &ROOM_MAKE, &ROOM_PASSWORD, &ROOM_MENU, &FAMILY_ASKS,
    &BASIC,
        &MENU,
        &HOTKEYS,
        &STATUS,
        &INVENTORY,
        &SKILLS,
        &STORAGE,
        &STORAGE_APART_0,
        &STORAGE_APART_1,
        &STORAGE_APART_2,
        &STORAGE_APART_3,
        &STORAGE_APART_4,
        &STORAGE_APART_5,
        &STORAGE_APART_6,
        &STORAGE_APART_7,
        &STORAGE_FOUND,
        &STORAGE_COUNT,
    &CART, &CART_LOOKS, &CART_COUNT, &VENDING_STALL, &VENDING_STOCK, &VENDING_MINE, &VENDOR, &VENDOR_BASKET, &VENDOR_CONFIRM, &VENDING_ASKS, &VENDING_COUNT,
    &EQUIPMENT,
        &DIALOG,
        &ESC,
        &TRADE,
        &SHOP,
        &SHOP_BASKET,
        &SHOP_COMPARE,
        &SHOP_COUNT,
        &ITEM_INFO,
        &CARD_ILLUST,
        &SKILL_INFO,
        &SKILL_WORDS,
        &OBTAINED,
        &THROW,
        &TAMING,
        &EGGS,
        &IDENTIFY,
        &PET,
        &PET_ASKS,
    &KEYS,
        &KEY_GUIDE,
        &SETTINGS,
        &GUIDE,
        &PET_MENU,
        &HOMUNCULUS,
        &HOMUNCULUS_ASKS,
        &HOMUNCULUS_SKILLS,
        &HOMUNCULUS_MENU,
        &MERCENARY,
        &MERCENARY_ASKS,
        &MERCENARY_SKILLS,
        &MERCENARY_MENU,
        &PARTY,
        &PARTY_SIDE,
        &PARTY_ASKS,
        &PARTY_MINI_0, &PARTY_MINI_1, &PARTY_MINI_2, &PARTY_MINI_3, &PARTY_MINI_4, &PARTY_MINI_5, &PARTY_MINI_6, &PARTY_MINI_7, &PARTY_MINI_8,
        &PARTY_MINI_9, &PARTY_MINI_10, &PARTY_MINI_11,
        &PARTY_MENU,
        &GUILD,
        &GUILD_ASKS,
        &GUILD_MENU,
        &PLAYER_MENU,
        &EXCHANGE,
        &EXCHANGE_ASKS,
        &EXCHANGE_COUNT,
        &MAIL_NOTICE,
        &MAIL,
        &MAIL_READ,
        &MAIL_WRITE,
        &MAIL_ASKS,
        &MAIL_COUNT,
        &QUESTS,
        &QUEST_PANE,
        &QUEST_NOTICE,
        &LEVEL_TOLD,
        &JOB_LEVEL_TOLD,
        &QUEST_LIST,
    &NAVIGATION, &NAVIGATION_ROUTE, &NAVIGATION_ICONS, &NAVIGATION_HELP, &NAVIGATION_ASKS,
    &FRIEND_SETUP,
        &FRIEND_ASKS,
        &FRIEND_MENU,
        &VIEW_EQUIP,
        &WHISPER_0,
        &WHISPER_1,
        &WHISPER_2,
        &WHISPER_3,
        &WHISPER_4,
        &WHISPER_5,
        &WHISPER_6,
        &WHISPER_7,
        &WHISPER_MENU,
        &AWAY,
        &UNREAD,
        &CHOOSE_PLACE,
        &CHOOSE_PICTURED,
        &CHOOSE_LINE,
        &BROADCAST,
        &CHAT,
    &CHAT_OWN_0,
        &CHAT_OWN_1,
        &CHAT_OWN_2,
        &CHAT_OWN_3,
        &CHAT_OWN_4,
        &CHAT_OWN_5,
        &CHAT_OWN_6,
        &CHAT_OWN_7,
        &CHAT_OWN_8,
        &CHAT_OWN_9,
        &CHAT_OPTIONS,
        &EMOTIONS,
        &SHORTCUTS,
        &TIPS,
        &LARGE_MAP,
        &WORLD_MAP,
    &SENSE_WINDOW, &STAR_ASKS,
    &BOOK, &BOOK_LINE,
    &ACHIEVEMENTS_TRACED, &ACHIEVEMENTS, &ACHIEVEMENT_TOLD, &REPUTATION, &PROBABILITY_ASKS,
    &BANK,
    &ATTENDANCE,
    &CASH_SHOP, &CASH_SHOP_ASKS,
    &BATTLEGROUND, &BATTLEGROUND_HELP, &BATTLEGROUND_STANDBY, &BATTLEGROUND_ENTRY, &BATTLEGROUND_ASKS,
    &AGENCY, &AGENCY_NOTICE, &AGENCY_READ, &AGENCY_ASKS,
    &REPLAY_RECORDER, &REPLAY_PLAYER,
];
