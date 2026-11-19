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
//! Here are the type, what the entries are written with (`framed`, `middle`, `staged`, `ordered`, and the three
//! kinds of window that are the same in every system: `Window::question`, `Window::count`, `Window::menu`),
//! and `ALL`; the entries themselves are in `windows/`, a file to a system.

use super::*;
use wasm_bindgen::JsCast;

/// Closing a window whose being open is one switch: the switch off, and the screen written again.
macro_rules! switched_off {
    ($($field:ident).+) => {
        |app: &mut App| {
            app.$($field).+ = false;
            app.render();
        }
    };
}

mod choosing;
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
pub(in crate::web) use {exchange::*, friends::*, guide::*, guild::*, inventory::*, mail::*, party::*, pets::*, player_menu::*, quests::*};
pub(in crate::web) use choosing::*;
pub(in crate::web) use {homunculus::*, mercenary::*};
pub(in crate::web) use unread::*;
pub(in crate::web) use {chat_rooms::*, family::*, scripted::*};
pub(in crate::web) use broadcast::*;
pub(in crate::web) use {chat::*, emotions::*, tips::*, worldmap::*};
pub(in crate::web) use fight::*;
pub(in crate::web) use {cart::*, vending::*};
pub(in crate::web) use {settings::*, shops::*, skills::*, status::*, storage::*, talk::*, viewequip::*, whispers::*};

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

/// The tag of a usual window: one that is dragged by its title and comes to the front when pressed.
fn framed(style: String) -> String {
    format!(r#"class="a hud" style="{style}""#)
}

/// Where a window `width` wide stands that the client puts in the middle, so far down a screen 480 high.
fn middle(view: (f64, f64), width: i32, down: i32) -> String {
    framed(format!("left:{}px;top:{}px;width:{width}px", (view.0 as i32 - width) / 2, view.1 as i32 * down / 480))
}

/// Where a window of that size stands in the very middle of the screen.
fn centred(view: (f64, f64), width: i32, height: i32) -> String {
    format!("left:{}px;top:{}px;width:{width}px", ((view.0 as i32 - width) / 2).max(0), ((view.1 as i32 - height) / 2).max(0))
}

/// Where the client opens a window so far across its 640-wide stage and so far of 480 down
/// (`CenterOnViewportX`, `ScaleByScreenHeight`).
fn staged(view: (f64, f64), across: i32, down: i32, width: i32, height: i32) -> String {
    framed(format!("left:{}px;top:{}px;width:{width}px;height:{height}px", (view.0 as i32 - 640) / 2 + across, view.1 as i32 * down / 480))
}

/// Where a menu opened with the right button stands, as a style: a white box with a grey rim, its corner at
/// that place of the stage. `App::open_at` is given this when a menu opens without the screen being written.
pub(in crate::web) fn menu_style((x, y): (i32, i32)) -> String {
    format!("left:{x}px;top:{y}px;background:#fff;border:1px solid #9a9a9a")
}

/// The tag of a menu that was opened at that place.
fn menu_frame(at: Option<(i32, i32)>) -> String {
    framed(menu_style(at.unwrap_or_default()))
}

/// A window that lies in the order whenever it is open, and is closed by setting what says it is open.
const fn ordered(open: fn(&App) -> bool, close: fn(&mut App)) -> Option<Order> {
    Some(Order { when: open, close })
}

/// The number in a box that asks how many is ready to be typed over: its field has the keyboard, and what is
/// in it is marked.
fn typed_over(window: &web_sys::Element) {
    if let Some(count) = window.query_selector("input").ok().flatten().and_then(|count| count.dyn_into::<web_sys::HtmlInputElement>().ok()) {
        let _ = count.focus();
        count.select();
    }
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

    /// The name of the field of a window that has one to type into (the number of a box that asks how many).
    pub(in crate::web) fn field(&self) -> String {
        format!("{}-field", self.id)
    }
}

const ALWAYS: fn(&App) -> bool = |_| true;

/// Every window of the screen, as they are written to the page: a later one lies over an earlier one where
/// neither has a place in the order and both are of one layer.
pub(in crate::web) static ALL: [&Window; 122] = [
    &STATUSES,
    &SIGNS, &CUTIN, &ROOM, &ROOM_MAKE, &ROOM_PASSWORD, &ROOM_MENU, &FAMILY_ASKS,
    &BASIC, &MENU, &HOTKEYS, &STATUS, &INVENTORY, &SKILLS, &STORAGE, &STORAGE_APART_0, &STORAGE_APART_1, &STORAGE_APART_2, &STORAGE_APART_3, &STORAGE_APART_4, &STORAGE_APART_5, &STORAGE_APART_6, &STORAGE_APART_7, &STORAGE_FOUND, &STORAGE_COUNT,
    &CART, &CART_LOOKS, &CART_COUNT, &VENDING_STALL, &VENDING_STOCK, &VENDING_MINE, &VENDOR, &VENDOR_BASKET, &VENDOR_CONFIRM, &VENDING_ASKS, &VENDING_COUNT,
    &EQUIPMENT, &DIALOG, &TRADE, &SHOP, &SHOP_BASKET, &SHOP_COUNT, &ITEM_INFO, &CARD_ILLUST, &SKILL_INFO, &OBTAINED, &THROW, &TAMING, &EGGS, &IDENTIFY, &PET, &PET_ASKS,
    &KEYS, &SETTINGS, &GUIDE, &PET_MENU, &HOMUNCULUS, &HOMUNCULUS_ASKS, &HOMUNCULUS_SKILLS, &HOMUNCULUS_MENU, &MERCENARY, &MERCENARY_ASKS, &MERCENARY_SKILLS, &MERCENARY_MENU, &PARTY, &PARTY_SIDE, &PARTY_ASKS, &PARTY_MENU, &GUILD, &GUILD_ASKS, &GUILD_MENU, &PLAYER_MENU, &EXCHANGE, &EXCHANGE_ASKS, &EXCHANGE_COUNT, &MAIL_NOTICE, &MAIL, &MAIL_READ, &MAIL_WRITE, &MAIL_ASKS, &MAIL_COUNT, &QUESTS, &QUEST_PANE, &QUEST_NOTICE, &QUEST_LIST,
    &FRIEND_SETUP, &FRIEND_ASKS, &FRIEND_MENU, &VIEW_EQUIP, &WHISPER_0, &WHISPER_1, &WHISPER_2, &WHISPER_3, &WHISPER_4, &WHISPER_5, &WHISPER_6, &WHISPER_7, &WHISPER_MENU, &UNREAD, &CHOOSE_PLACE, &CHOOSE_PICTURED, &CHOOSE_LINE, &BROADCAST, &CHAT,
    &CHAT_OWN_0, &CHAT_OWN_1, &CHAT_OWN_2, &CHAT_OWN_3, &CHAT_OWN_4, &CHAT_OWN_5, &CHAT_OWN_6, &CHAT_OWN_7, &CHAT_OWN_8, &CHAT_OWN_9, &CHAT_OPTIONS, &EMOTIONS, &SHORTCUTS, &TIPS, &LARGE_MAP, &WORLD_MAP,
    &SENSE_WINDOW, &STAR_ASKS,
];

impl Window {
    /// What is in it now: nothing while it is closed.
    fn inside(&self, app: &App) -> String {
        if (self.open)(app) { (self.draw)(app) } else { String::new() }
    }
}

impl App {
    /// Every window of the game, each in its own element. `view` is the stage's size.
    pub(in crate::web) fn hud(&self, view: (f64, f64)) -> String {
        ALL.iter()
            .map(|window| {
                let hidden = if (window.open)(self) { "" } else { " hidden" };
                format!(r#"<div id="{}" {}{}{hidden}>{}</div>"#, window.id, (window.frame)(self, view), window.layer.written(), window.inside(self))
            })
            .collect()
    }

    /// Draws windows anew where they are on the page: each open one with what is in it now, each closed one
    /// emptied and hidden. A list keeps its place and a field being typed into keeps its text and the keyboard.
    pub(in crate::web) fn redraw(&self, windows: &[&Window]) {
        let mut figures = false;
        for window in windows {
            let Some(element) = crate::web::by_id(window.id) else { continue };
            let inside = window.inside(self);
            figures |= inside.contains("<canvas");
            let scrolled = element.query_selector(".scroll, .viewer").ok().flatten().map(|list| list.scroll_top());
            let typed = crate::web::document()
                .and_then(|document| document.active_element())
                .filter(|active| !active.id().is_empty() && element.contains(Some(active)))
                .and_then(|active| active.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| (input.id(), input.value()));
            element.set_inner_html(&inside);
            let _ = if inside.is_empty() && !(window.open)(self) { element.set_attribute("hidden", "") } else { element.remove_attribute("hidden") };
            if let (Some(scrolled), Ok(Some(list))) = (scrolled, element.query_selector(".scroll, .viewer")) {
                list.set_scroll_top(scrolled);
            }
            if let Some(drawn) = window.drawn {
                drawn(&element);
            }
            if let Some((id, text)) = typed {
                if let Some(input) = crate::web::by_id(&id).and_then(|input| input.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                    input.set_value(&text);
                    let _ = input.focus();
                }
            }
        }
        // A window written anew has its canvases anew: the character is drawn on them again.
        if figures {
            self.paint_figures();
        }
        // One opened this way goes over the other windows.
        self.order_windows();
    }

    /// Redraws the windows that show the character's values.
    pub(in crate::web) fn refresh_values(&self) {
        crate::web::world::own_life(self.value(parameter::HP), self.value(parameter::MAX_HP), self.value(parameter::SP), self.value(parameter::MAX_SP));
        crate::web::world::own_level(self.value(parameter::BASE_LEVEL) as u16);
        self.redraw(&[&BASIC, &STATUS]);
    }

    /// Redraws the windows that show items and skills.
    pub(in crate::web) fn refresh_items(&self) {
        self.redraw(&[&INVENTORY, &SKILLS, &HOTKEYS, &ITEM_INFO, &SKILL_INFO, &EQUIPMENT]);
        self.redraw(&STORAGE_WINDOWS);
    }
}
