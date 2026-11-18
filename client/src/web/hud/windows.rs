//! The windows of the game's screen as one list: each with its element, when it is open, where it stands, what
//! draws it and what Escape does to it. Everything that goes through the windows goes through this list: the
//! screen is written from it (`App::hud`), a window is drawn anew from it (`App::redraw`), and the order the
//! windows lie in and what Escape closes are read from it (`app/windows.rs`). A new window is a new entry here
//! and its drawing in its system's file; nothing else has to know of it.
//!
//! A window's element is always on the page: closed, it is empty and hidden. So a window can open, close and
//! change without the whole screen being written again.

use super::*;
use wasm_bindgen::JsCast;

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

/// A window that lies in the order whenever it is open, and is closed by setting what says it is open.
const fn ordered(open: fn(&App) -> bool, close: fn(&mut App)) -> Option<Order> {
    Some(Order { when: open, close })
}

/// Closing a window whose being open is one switch: the switch off, and the screen written again.
macro_rules! switched_off {
    ($($field:ident).+) => {
        |app: &mut App| {
            app.$($field).+ = false;
            app.render();
        }
    };
}

const ALWAYS: fn(&App) -> bool = |_| true;

pub(in crate::web) static BASIC: Window =
    Window { id: "basic", open: ALWAYS, frame: |_, _| framed("left:0;top:0;width:220px".to_owned()), draw: App::basic_info, order: None, drawn: None };

/// The menu hangs under the basic window, a little into it.
pub(in crate::web) static MENU: Window =
    Window { id: "menu", open: ALWAYS, frame: |app, _| framed(format!("left:0;top:{}px;width:220px", app.basic_height() - 4)), draw: App::menu_icons, order: None, drawn: None };

pub(in crate::web) static HOTKEYS: Window = Window {
    id: "hotkeys",
    open: |app| !app.skills.hotkeys_hidden,
    frame: |app, _| framed(at(230, 0, 290, (i32::from(app.skills.hotkey_rows) + 1) * 33 + 1)),
    draw: App::hotkeys,
    order: None,
    drawn: None,
};

pub(in crate::web) static STATUS: Window = Window {
    id: "status",
    open: |app| app.hero.status_open,
    frame: |_, _| framed(at(225, 285, 280, 141)),
    draw: App::status_window,
    order: ordered(|app| app.hero.status_open, switched_off!(hero.status_open)),
    drawn: None,
};

/// The inventory stands aside while the storage is open beside it.
pub(in crate::web) static INVENTORY: Window = Window {
    id: "inventory",
    open: |app| app.bag.open,
    frame: |app, _| framed(at(if app.deal.storage.is_some() { 515 } else { 225 }, 80, 40 + 32 * ITEM_COLUMNS + 16, 17 + 32 * ITEM_ROWS + 21)),
    draw: App::inventory_window,
    order: ordered(|app| app.bag.open, switched_off!(bag.open)),
    drawn: None,
};

pub(in crate::web) static SKILLS: Window = Window {
    id: "skills",
    open: |app| app.skills.open,
    frame: |_, _| framed(at(230, 60, 560, 400)),
    draw: App::skill_window,
    order: ordered(|app| app.skills.open, switched_off!(skills.open)),
    drawn: None,
};

pub(in crate::web) static STORAGE: Window = Window {
    id: "storage",
    open: |app| app.deal.storage.is_some(),
    frame: |_, _| framed("left:225px;top:80px;width:280px".to_owned()),
    draw: App::storage_window,
    order: ordered(|app| app.deal.storage.is_some(), |app| app.pressed(crate::press::shops::Press::StorageClose)),
    drawn: None,
};

pub(in crate::web) static EQUIPMENT: Window = Window {
    id: "equipment",
    open: |app| app.bag.equipment_open,
    frame: |_, _| framed("left:560px;top:290px;width:280px".to_owned()),
    draw: App::equipment_window,
    order: ordered(|app| app.bag.equipment_open, switched_off!(bag.equipment_open)),
    drawn: None,
};

/// What an NPC says, and in the same place the game's menu (the client's `UIEscOptionWnd`). The original puts
/// the NPC's window a fifth of the way down the screen, in the middle, and what it asks under it. Only the menu
/// lies in the order: a window it opens (Game Settings, the keys) goes over it, as `EnsureWindow` puts it in
/// front there, and the menu stays open behind; what an NPC says is over the windows, by the page's own rules.
pub(in crate::web) static DIALOG: Window = Window {
    id: "dialog",
    open: |app| app.esc != 0 || app.dialog.is_some(),
    frame: |_, view| middle(view, 280, 100),
    draw: App::dialog_window,
    order: ordered(|app| app.esc == 1, |app| app.pressed(crate::press::general::Press::EscClose)),
    // What was said last is in sight.
    drawn: Some(|dialog| {
        if let Ok(Some(text)) = dialog.query_selector(".said") {
            text.set_scroll_top(text.scroll_height());
        }
    }),
};

pub(in crate::web) static TRADE: Window =
    Window { id: "trade", open: |app| app.deal.shop.is_some(), frame: |_, view| middle(view, 280, 100), draw: App::trade_window, order: None, drawn: None };

pub(in crate::web) static ITEM_INFO: Window = Window {
    id: "iteminfo",
    open: |app| app.bag.info.is_some(),
    frame: |_, _| framed("left:300px;top:230px;width:280px".to_owned()),
    draw: App::item_info_window,
    order: ordered(|app| app.bag.info.is_some(), |app| app.pressed(crate::press::inventory::Press::InfoClose)),
    drawn: None,
};

pub(in crate::web) static SKILL_INFO: Window = Window {
    id: "skillinfo",
    open: |app| app.skills.info.is_some(),
    frame: |_, _| framed("left:330px;top:110px;width:304px".to_owned()),
    draw: App::skill_info_window,
    order: ordered(|app| app.skills.info.is_some(), |app| app.pressed(crate::press::skills::Press::InfoClose)),
    drawn: None,
};

/// The note of what was just picked up: not a window, nothing on it is pressed.
pub(in crate::web) static OBTAINED: Window = Window {
    id: "obtained",
    open: ALWAYS,
    frame: |_, view| format!(r#"class="a" style="left:{}px;top:75px;width:260px;pointer-events:none""#, (view.0 as i32 - 260) / 2),
    draw: App::obtained_note,
    order: None,
    drawn: None,
};

/// How many of a stack to drop: the number is ready to be typed over.
pub(in crate::web) static THROW: Window = Window {
    id: "throw",
    open: |app| app.bag.throwing.is_some(),
    frame: |_, view| middle(view, 182, 200),
    draw: App::throw_window,
    order: None,
    drawn: Some(|window| {
        if let Some(count) = window.query_selector("input").ok().flatten().and_then(|count| count.dyn_into::<web_sys::HtmlInputElement>().ok()) {
            let _ = count.focus();
            count.select();
        }
    }),
};

/// The slot machine of taming: 196 by 256 in the middle of the screen; a press on it starts the reels.
pub(in crate::web) static TAMING: Window = Window {
    id: "taming",
    open: |app| app.pets.taming.is_some(),
    frame: |_, view| {
        format!(
            r#"class="a" style="{};cursor:pointer;z-index:30;pointer-events:auto" {}"#,
            at((view.0 as i32 - 196) / 2, (view.1 as i32 - 256) / 2, 196, 256),
            pressed(crate::press::pets::Press::TamingShoot)
        )
    },
    draw: |_| r#"<img id="taming-picture" alt="" draggable="false" style="position:absolute;left:0;top:0">"#.to_owned(),
    order: None,
    drawn: None,
};

/// The list of eggs to hatch: 200 by 200, where the client opens it.
pub(in crate::web) static EGGS: Window =
    Window { id: "eggs", open: |app| app.pets.eggs.is_some(), frame: |_, view| middle(view, 200, 200), draw: App::egg_window, order: None, drawn: None };

/// The list of items to identify: the same window, in the same place.
pub(in crate::web) static IDENTIFY: Window =
    Window { id: "identify", open: |app| app.bag.identify.is_some(), frame: |_, view| middle(view, 200, 200), draw: App::identify_window, order: None, drawn: None };

/// The pet window, where the client opens it: 185 across its 640-wide stage, 60 above 400 of 480 down.
pub(in crate::web) static PET: Window = Window {
    id: "pet",
    open: |app| app.pets.open && app.pets.pet.is_some(),
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px", (view.0 as i32 - 640) / 2 + 185, view.1 as i32 * 400 / 480 - 60)),
    draw: App::pet_window,
    order: ordered(|app| app.pets.open && app.pets.pet.is_some(), switched_off!(pets.open)),
    drawn: None,
};

/// What the pet window asks before it acts, in the middle of the screen.
pub(in crate::web) static PET_ASKS: Window = Window {
    id: "pet-asks",
    open: |app| app.pets.open && app.pets.pet.is_some(),
    frame: |_, view| framed(format!("left:{}px;top:{}px;width:280px", (view.0 as i32 - 280) / 2, (view.1 as i32 - 120) / 2)),
    draw: App::pet_question,
    order: None,
    drawn: None,
};

/// The Shortcut Settings window, in the middle of the view.
pub(in crate::web) static KEYS: Window = Window {
    id: "keys",
    open: |app| app.keys.open,
    frame: |_, view| framed(centred(view, 436, 432)),
    draw: App::key_window,
    order: ordered(|app| app.keys.open, |app| app.pressed(crate::press::hotkeys::Press::Close)),
    drawn: None,
};

/// The Game Settings window, in the middle of the view, as the client puts it.
pub(in crate::web) static SETTINGS: Window = Window {
    id: "settings",
    open: |app| app.settings.open,
    frame: |_, view| framed(centred(view, 400, 350)),
    draw: App::settings_window,
    order: ordered(|app| app.settings.open, switched_off!(settings.open)),
    drawn: None,
};

/// The Adventure Guide, 750 by 468, in the middle.
pub(in crate::web) static GUIDE: Window = Window {
    id: "guide",
    open: |app| app.guide.open,
    frame: |_, view| framed(format!("{};height:468px", centred(view, 750, 468))),
    draw: App::guide_window,
    order: ordered(|app| app.guide.open, switched_off!(guide.open)),
    drawn: None,
};

/// The menu of the pet, opened with the right button on it, where the pointer was.
pub(in crate::web) static PET_MENU: Window = Window {
    id: "pet-menu",
    open: |app| app.pets.menu_at.is_some() && app.pets.pet.is_some(),
    frame: |app, _| {
        let (x, y) = app.pets.menu_at.unwrap_or_default();
        framed(format!("left:{x}px;top:{y}px;background:#fff;border:1px solid #9a9a9a;z-index:26"))
    },
    draw: App::pet_menu,
    order: None,
    drawn: None,
};

/// The chat, along the bottom of the screen: as tall as its rows make it.
pub(in crate::web) static CHAT: Window = Window {
    id: "chat",
    open: ALWAYS,
    frame: |app, view| format!(r#"class="a hud" data-locked="{}" style="{}""#, u8::from(app.chat.locked), at(0, view.1 as i32 - app.chat_height(), 600, app.chat_height())),
    draw: |app| app.chat_window(app.chat.rows, app.chat_height()),
    order: None,
    drawn: None,
};

/// Every window of the screen, as they are written to the page: a later one lies over an earlier one where
/// neither has a place in the order.
pub(in crate::web) static ALL: [&Window; 24] = [
    &BASIC, &MENU, &HOTKEYS, &STATUS, &INVENTORY, &SKILLS, &STORAGE, &EQUIPMENT, &DIALOG, &TRADE, &ITEM_INFO, &SKILL_INFO, &OBTAINED, &THROW, &TAMING, &EGGS, &IDENTIFY, &PET, &PET_ASKS,
    &KEYS, &SETTINGS, &GUIDE, &PET_MENU, &CHAT,
];

impl Window {
    /// What is in it now: nothing while it is closed.
    fn inside(&self, app: &App) -> String {
        if (self.open)(app) { (self.draw)(app) } else { String::new() }
    }
}

impl App {
    /// The chat's height: the client's own sum (`UINewChatWnd::ResizeRows`), the input line, the rows and three
    /// more, the tabs.
    fn chat_height(&self) -> i32 {
        if self.chat.rows == 0 { 24 } else { 24 + (self.chat.rows as i32 + 3) * 14 + 16 }
    }

    /// Every window of the game, each in its own element. `view` is the stage's size.
    pub(in crate::web) fn hud(&self, view: (f64, f64)) -> String {
        ALL.iter()
            .map(|window| {
                let hidden = if (window.open)(self) { "" } else { " hidden" };
                format!(r#"<div id="{}" {}{hidden}>{}</div>"#, window.id, (window.frame)(self, view), window.inside(self))
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
        self.redraw(&[&BASIC, &STATUS]);
    }

    /// Redraws the windows that show items and skills.
    pub(in crate::web) fn refresh_items(&self) {
        self.redraw(&[&INVENTORY, &SKILLS, &HOTKEYS, &STORAGE, &ITEM_INFO, &SKILL_INFO, &EQUIPMENT]);
    }
}
